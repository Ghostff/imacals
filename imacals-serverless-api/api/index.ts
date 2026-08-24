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
import { login, getMe } from '../src/routes/auth.js';

// Helpers
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

function sendSuccess(res: ServerResponse, data: any, status = 200): void {
  sendJson(res, status, { success: 'true', data });
}

function sendError(res: ServerResponse, status: number, message: string, code = 'Error'): void {
  sendJson(res, status, {
    success: 'false',
    code,
    error: { message },
  });
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

  const url = new URL(req.url || '/', `http://${req.headers.host || 'localhost'}`);
  let pathname = url.pathname.replace(/\/+/g, '/').replace(/\/+$/, '');
  const method = (req.method || 'GET').toUpperCase();

  // Ensure leading /api prefix
  if (!pathname.startsWith('/api') && pathname) {
    pathname = `/api${pathname}`;
  } else if (!pathname) {
    pathname = '/api';
  }

  try {
    // ── Health ─────────────────────────────────────────────────────────────
    if (pathname === '/api/health' || pathname === '/api') {
      sendSuccess(res, { status: 'healthy', timestamp: new Date().toISOString() });
      return;
    }

    // ── Catalog (Public) ───────────────────────────────────────────────────
    if (pathname === '/api/catalog/products' && method === 'GET') {
      const category = url.searchParams.get('category') || undefined;
      const products = await getCatalogProducts(category);
      sendSuccess(res, products);
      return;
    }

    const catalogProductMatch = pathname.match(/^\/api\/catalog\/products\/([^/]+)$/);
    if (catalogProductMatch && method === 'GET') {
      const slug = decodeURIComponent(catalogProductMatch[1]);
      const product = await getCatalogProductBySlug(slug);
      if (!product) return sendError(res, 404, 'Product not found', 'NotFound');
      sendSuccess(res, product);
      return;
    }

    if (pathname === '/api/catalog/categories' && method === 'GET') {
      const categories = await getCatalogCategories();
      sendSuccess(res, categories);
      return;
    }

    // ── Auth ───────────────────────────────────────────────────────────────
    if (pathname === '/api/auth/login' && method === 'POST') {
      const body = await parseJsonBody(req);
      const data = await login(body);
      sendSuccess(res, data);
      return;
    }

    if (pathname === '/api/auth/me' && method === 'GET') {
      const authHeader = req.headers['authorization'] as string | undefined;
      const data = await getMe(authHeader);
      sendSuccess(res, data);
      return;
    }

    // ── Categories (Admin) ─────────────────────────────────────────────────
    if (pathname === '/api/categories' && method === 'GET') {
      const categories = await listCategories();
      sendSuccess(res, categories);
      return;
    }

    if (pathname === '/api/categories' && method === 'POST') {
      const body = await parseJsonBody(req);
      const created = await createCategory(body);
      sendSuccess(res, created, 201);
      return;
    }

    const categoryIdMatch = pathname.match(/^\/api\/categories\/([^/]+)$/);
    if (categoryIdMatch) {
      const id = categoryIdMatch[1];
      if (method === 'PUT') {
        const body = await parseJsonBody(req);
        const updated = await updateCategory(id, body);
        sendSuccess(res, updated);
        return;
      }
      if (method === 'DELETE') {
        await deleteCategory(id);
        sendSuccess(res, { message: 'Category deleted successfully' });
        return;
      }
    }

    // ── Products (Admin) ───────────────────────────────────────────────────
    if (pathname === '/api/products' && method === 'GET') {
      const products = await listAdminProducts();
      sendSuccess(res, products);
      return;
    }

    if (pathname === '/api/products' && method === 'POST') {
      const body = await parseJsonBody(req);
      const created = await createProduct(body);
      sendSuccess(res, created, 201);
      return;
    }

    // Product Image Upload
    const productImageMatch = pathname.match(/^\/api\/products\/([^/]+)\/image$/);
    if (productImageMatch && method === 'POST') {
      const id = productImageMatch[1];
      const { file } = await parseMultipart(req);
      if (!file) return sendError(res, 400, 'No image file uploaded', 'Validation');
      const updated = await uploadProductImage(id, file.buffer, file.filename, file.mimeType);
      sendSuccess(res, updated);
      return;
    }

    const productIdMatch = pathname.match(/^\/api\/products\/([^/]+)$/);
    if (productIdMatch) {
      const id = productIdMatch[1];
      if (method === 'GET') {
        const product = await getAdminProductById(id);
        if (!product) return sendError(res, 404, 'Product not found', 'NotFound');
        sendSuccess(res, product);
        return;
      }
      if (method === 'PUT') {
        const body = await parseJsonBody(req);
        const updated = await updateProduct(id, body);
        sendSuccess(res, updated);
        return;
      }
      if (method === 'DELETE') {
        await deleteProduct(id);
        sendSuccess(res, { message: 'Product deleted successfully' });
        return;
      }
    }

    // ── 404 Not Found ──────────────────────────────────────────────────────
    sendError(res, 404, `Route ${method} ${pathname} not found`, 'NotFound');
  } catch (err: any) {
    sendError(res, 500, err.message || 'Internal Server Error', 'InternalServerError');
  }
}
