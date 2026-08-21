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

async function request<T>(method: string, path: string, payload?: unknown): Promise<T> {
  const token  = localStorage.getItem('token');
  const orgId  = localStorage.getItem('organization_id');
  const headers: Record<string, string> = { 'Content-Type': 'application/json' };
  if (token) headers['Authorization']    = token;
  if (orgId) headers['X-Organization-Id'] = orgId;

  const res = await fetch(`${BASE}${path}`, {
    method,
    headers,
    body: payload !== undefined ? JSON.stringify(payload) : undefined,
  });

  const text: string = await res.text();

  if (!text) {
    throw new ApiException(res.status, { success: 'false', code: 'EmptyResponse', error: { message: `Empty response from server (${res.status})` } });
  }

  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new ApiException(res.status, { success: 'false', code: 'ParseError', error: { message: `Non-JSON response: ${text.slice(0, 200)}` } });
  }

  if (!res.ok) throw new ApiException(res.status, json as ApiError);
  return (json as ApiResponse<T>).data;
}

async function uploadRequest<T>(path: string, formData: FormData): Promise<T> {
  const token  = localStorage.getItem('token');
  const orgId  = localStorage.getItem('organization_id');
  const headers: Record<string, string> = {};
  if (token) headers['Authorization']    = token;
  if (orgId) headers['X-Organization-Id'] = orgId;

  const res = await fetch(`${BASE}${path}`, {
    method: 'POST',
    headers,
    body: formData,
  });

  const text: string = await res.text();

  if (!text) {
    throw new ApiException(res.status, { success: 'false', code: 'EmptyResponse', error: { message: `Empty response from server (${res.status})` } });
  }

  let json: unknown;
  try {
    json = JSON.parse(text);
  } catch {
    throw new ApiException(res.status, { success: 'false', code: 'ParseError', error: { message: `Non-JSON response: ${text.slice(0, 200)}` } });
  }

  if (!res.ok) throw new ApiException(res.status, json as ApiError);
  return (json as ApiResponse<T>).data;
}

export const api = {
  get:    <T>(path: string)                    => request<T>('GET',    path),
  post:   <T>(path: string, body: unknown)     => request<T>('POST',   path, body),
  put:    <T>(path: string, body: unknown)     => request<T>('PUT',    path, body),
  delete: <T>(path: string)                    => request<T>('DELETE', path),
  upload: <T>(path: string, formData: FormData)=> uploadRequest<T>(path, formData),
};
