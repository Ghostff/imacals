import type { IncomingMessage, ServerResponse } from 'http';
import Busboy from 'busboy';
import {
  getCatalogProducts,
  getCatalogProductBySlug,
  getCatalogCategories,
} from '../src/routes/catalog.js';
import {
  listAdminProducts,
  getAdminProductById,
  createProduct,
  updateProduct,
  deleteProduct,
  uploadProductImage,
} from '../src/routes/products.js';
import {
  listCategories,
  createCategory,
  updateCategory,
  deleteCategory,
} from '../src/routes/categories.js';
import { login, getMe, getOrganizations } from '../src/routes/auth.js';
import {
  recordLog,
  getLogs,
  getMetrics,
  clearLogs,
  checkSupabaseHealth,
} from '../src/monitoring.js';

// Response helpers
function sendJson(res: ServerResponse, status: number, data: any): void {
  res.writeHead(status, {
    'Content-Type': 'application/json',
    'Access-Control-Allow-Origin': '*',
    'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS, PATCH',
    'Access-Control-Allow-Headers':
      'X-CSRF-Token, X-Requested-With, Accept, Accept-Version, Content-Length, Content-MD5, Content-Type, Date, X-Api-Version, Authorization, X-Organization-Id',
  });
  res.end(JSON.stringify(data));
}

function parseJsonBody(req: IncomingMessage): Promise<any> {
  return new Promise((resolve, reject) => {
    let body = '';
    req.on('data', (chunk) => {
      body += chunk.toString();
    });
    req.on('end', () => {
      if (!body) return resolve({});
      try {
        resolve(JSON.parse(body));
      } catch (err) {
        reject(new Error('Invalid JSON'));
      }
    });
    req.on('error', reject);
  });
}

function parseMultipart(
  req: IncomingMessage
): Promise<{ fields: Record<string, string>; file?: { buffer: Buffer; filename: string; mimeType: string } }> {
  return new Promise((resolve, reject) => {
    const busboy = Busboy({ headers: req.headers });
    const fields: Record<string, string> = {};
    let fileResult: { buffer: Buffer; filename: string; mimeType: string } | undefined;

    busboy.on('field', (fieldname, val) => {
      fields[fieldname] = val;
    });

    busboy.on('file', (fieldname, file, info) => {
      const { filename, mimeType } = info;
      const chunks: Buffer[] = [];
      file.on('data', (data: Buffer) => chunks.push(data));
      file.on('end', () => {
        fileResult = {
          buffer: Buffer.concat(chunks),
          filename: filename || 'upload.jpg',
          mimeType: mimeType || 'image/jpeg',
        };
      });
    });

    busboy.on('finish', () => {
      resolve({ fields, file: fileResult });
    });

    busboy.on('error', reject);
    req.pipe(busboy);
  });
}

export default async function handler(req: IncomingMessage, res: ServerResponse): Promise<void> {
  // CORS Preflight
  if (req.method === 'OPTIONS') {
    res.writeHead(200, {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS, PATCH',
      'Access-Control-Allow-Headers':
        'X-CSRF-Token, X-Requested-With, Accept, Accept-Version, Content-Length, Content-MD5, Content-Type, Date, X-Api-Version, Authorization, X-Organization-Id',
    });
    res.end();
    return;
  }

  const startMs = Date.now();
  const url = new URL(req.url || '/', `http://${req.headers.host || 'localhost'}`);
  let pathname = url.pathname.replace(/\/+/g, '/').replace(/\/+$/, '');
  const method = (req.method || 'GET').toUpperCase();

  const ip =
    (req.headers['x-forwarded-for'] as string)?.split(',')[0]?.trim() ||
    req.socket?.remoteAddress ||
    '127.0.0.1';
  const userAgent = (req.headers['user-agent'] as string) || undefined;

  let requestBody: any = undefined;

  // Ensure leading /api prefix
  if (!pathname.startsWith('/api') && pathname) {
    pathname = `/api${pathname}`;
  } else if (!pathname) {
    pathname = '/api';
  }

  function logAndSendSuccess(data: any, status = 200): void {
    const durationMs = Date.now() - startMs;
    // Don't flood logs with internal monitoring poll requests
    if (!pathname.startsWith('/api/monitoring/logs')) {
      recordLog({
        timestamp: new Date().toISOString(),
        method,
        pathname,
        statusCode: status,
        durationMs,
        ip,
        userAgent,
        requestBody,
        responseSummary: Array.isArray(data) ? `Array(${data.length})` : typeof data === 'object' ? Object.keys(data) : data,
      });
    }
    sendJson(res, status, { success: 'true', data });
  }

  function logAndSendError(status: number, message: string, code = 'Error', stackTrace?: string): void {
    const durationMs = Date.now() - startMs;
    recordLog({
      timestamp: new Date().toISOString(),
      method,
      pathname,
      statusCode: status,
      durationMs,
      ip,
      userAgent,
      errorMessage: message,
      errorCode: code,
      stackTrace,
      requestBody,
    });
    sendJson(res, status, {
      success: 'false',
      code,
      error: { message },
    });
  }

  try {
    // ── Monitoring & Health ────────────────────────────────────────────────
    if (pathname === '/api/monitoring/stats' && method === 'GET') {
      const stats = await getMetrics();
      logAndSendSuccess(stats);
      return;
    }

    if (pathname === '/api/monitoring/logs' && method === 'GET') {
      const status = url.searchParams.get('status') || undefined;
      const reqMethod = url.searchParams.get('method') || undefined;
      const search = url.searchParams.get('search') || undefined;
      const limit = Number(url.searchParams.get('limit')) || 100;
      const logsResult = getLogs({ status, method: reqMethod, search, limit });
      sendJson(res, 200, { success: 'true', data: logsResult });
      return;
    }

    if (pathname === '/api/monitoring/logs/clear' && method === 'POST') {
      clearLogs();
      logAndSendSuccess({ message: 'Logs cleared successfully' });
      return;
    }

    if (pathname === '/api/monitoring/test-error' && method === 'POST') {
      requestBody = await parseJsonBody(req);
      const type = requestBody?.type || '500';
      const msg = requestBody?.message || 'Simulated test error generated for monitoring verification';

      if (type === '400') {
        logAndSendError(400, msg, 'ValidationError');
        return;
      }
      if (type === '404') {
        logAndSendError(404, msg, 'NotFound');
        return;
      }
      logAndSendError(500, msg, 'DatabaseConnectionError', new Error(msg).stack);
      return;
    }

    if (pathname === '/api/monitoring/health' && method === 'GET') {
      const dbHealth = await checkSupabaseHealth();
      logAndSendSuccess(dbHealth);
      return;
    }

    // ── Health ─────────────────────────────────────────────────────────────
    if (pathname === '/api/health' || pathname === '/api') {
      logAndSendSuccess({ status: 'healthy', timestamp: new Date().toISOString() });
      return;
    }

    // ── Catalog (Public) ───────────────────────────────────────────────────
    if (pathname === '/api/catalog/products' && method === 'GET') {
      const category = url.searchParams.get('category') || undefined;
      const products = await getCatalogProducts(category);
      logAndSendSuccess(products);
      return;
    }

    const catalogProductMatch = pathname.match(/^\/api\/catalog\/products\/([^/]+)$/);
    if (catalogProductMatch && method === 'GET') {
      const slug = decodeURIComponent(catalogProductMatch[1]);
      const product = await getCatalogProductBySlug(slug);
      if (!product) return logAndSendError(404, 'Product not found', 'NotFound');
      logAndSendSuccess(product);
      return;
    }

    if (pathname === '/api/catalog/categories' && method === 'GET') {
      const categories = await getCatalogCategories();
      logAndSendSuccess(categories);
      return;
    }

    // ── Auth ───────────────────────────────────────────────────────────────
    if (pathname === '/api/auth/login' && method === 'POST') {
      requestBody = await parseJsonBody(req);
      const data = await login(requestBody);
      logAndSendSuccess(data);
      return;
    }

    if (pathname === '/api/auth/me' && method === 'GET') {
      const authHeader = req.headers['authorization'] as string | undefined;
      const data = await getMe(authHeader);
      logAndSendSuccess(data);
      return;
    }

    // ── Organizations (Admin) ──────────────────────────────────────────────
    if (pathname === '/api/organizations' && method === 'GET') {
      const orgs = await getOrganizations();
      logAndSendSuccess(orgs);
      return;
    }

    // ── Categories (Admin) ─────────────────────────────────────────────────
    if (pathname === '/api/categories' && method === 'GET') {
      const categories = await listCategories();
      logAndSendSuccess(categories);
      return;
    }

    if (pathname === '/api/categories' && method === 'POST') {
      requestBody = await parseJsonBody(req);
      const created = await createCategory(requestBody);
      logAndSendSuccess(created, 201);
      return;
    }

    const categoryIdMatch = pathname.match(/^\/api\/categories\/([^/]+)$/);
    if (categoryIdMatch) {
      const id = categoryIdMatch[1];
      if (method === 'PUT') {
        requestBody = await parseJsonBody(req);
        const updated = await updateCategory(id, requestBody);
        logAndSendSuccess(updated);
        return;
      }
      if (method === 'DELETE') {
        await deleteCategory(id);
        logAndSendSuccess({ message: 'Category deleted successfully' });
        return;
      }
    }

    // ── Products (Admin) ───────────────────────────────────────────────────
    if (pathname === '/api/products' && method === 'GET') {
      const products = await listAdminProducts();
      logAndSendSuccess(products);
      return;
    }

    if (pathname === '/api/products' && method === 'POST') {
      requestBody = await parseJsonBody(req);
      const created = await createProduct(requestBody);
      logAndSendSuccess(created, 201);
      return;
    }

    // Product Image Upload
    const productImageMatch = pathname.match(/^\/api\/products\/([^/]+)\/image$/);
    if (productImageMatch && method === 'POST') {
      const id = productImageMatch[1];
      const { file, fields } = await parseMultipart(req);
      requestBody = { filename: file?.filename, mimeType: file?.mimeType, ...fields };
      if (!file) return logAndSendError(400, 'No image file uploaded', 'Validation');
      const updated = await uploadProductImage(id, file.buffer, file.filename, file.mimeType);
      logAndSendSuccess(updated);
      return;
    }

    const productIdMatch = pathname.match(/^\/api\/products\/([^/]+)$/);
    if (productIdMatch) {
      const id = productIdMatch[1];
      if (method === 'GET') {
        const product = await getAdminProductById(id);
        if (!product) return logAndSendError(404, 'Product not found', 'NotFound');
        logAndSendSuccess(product);
        return;
      }
      if (method === 'PUT') {
        requestBody = await parseJsonBody(req);
        const updated = await updateProduct(id, requestBody);
        logAndSendSuccess(updated);
        return;
      }
      if (method === 'DELETE') {
        await deleteProduct(id);
        logAndSendSuccess({ message: 'Product deleted successfully' });
        return;
      }
    }

    // ── 404 Not Found ──────────────────────────────────────────────────────
    logAndSendError(404, `Route ${method} ${pathname} not found`, 'NotFound');
  } catch (err: any) {
    logAndSendError(
      500,
      err.message || 'Internal Server Error',
      'InternalServerError',
      err.stack
    );
  }
}
