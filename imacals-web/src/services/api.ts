const BASE: string = import.meta.env.VITE_API_BASE ?? '/api';

type ApiResponse<T> = { success: string; data: T };
type ApiError = { success: string; code: string; error: { message: string } };

export class ApiException extends Error {
  constructor(
    public readonly status: number,
    public readonly body: ApiError,
  ) {
    super(body.error?.message ?? 'Request failed');
  }
}

// The storefront is public: most reads carry no token. A token is sent when one exists so a
// signed-in customer's own orders resolve without a second code path.
async function request<T>(method: string, path: string, payload?: unknown): Promise<T> {
  const token = localStorage.getItem('token');
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization'] = token;

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: payload !== undefined ? JSON.stringify(payload) : undefined,
  });

  const text: string = await res.text();

  if (!text) {
    throw new ApiException(res.status, {
      success: 'false',
      code: 'EmptyResponse',
      error: { message: `Empty response from server (${res.status})` },
    });
  }

  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new ApiException(res.status, {
      success: 'false',
      code: 'ParseError',
      error: { message: `Non-JSON response: ${text.slice(0, 200)}` },
    });
  }

  if (!res.ok) throw new ApiException(res.status, json as ApiError);
  return (json as ApiResponse<T>).data;
}

export const api = {
  get: <T>(path: string): Promise<T> => request<T>('GET', path),
  post: <T>(path: string, body: unknown): Promise<T> => request<T>('POST', path, body),
  put: <T>(path: string, body: unknown): Promise<T> => request<T>('PUT', path, body),
  delete: <T>(path: string): Promise<T> => request<T>('DELETE', path),
};
