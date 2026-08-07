import { api, ApiException } from '@/services/api';

const BASE: string = (import.meta.env.VITE_API_BASE as string | undefined) ?? '/api';

type ApiResponse<T> = { success: string; data: T };
type ApiError = { success: string; code: string; error: { message: string } };

// XHR upload with progress; onProgress receives 0–100.
export function uploadDocumentXhr(
  userId: string,
  documentType: string,
  file: File,
  onProgress: (pct: number) => void,
): Promise<UserDocument> {
  return new Promise<UserDocument>((resolve, reject) => {
    const form = new FormData();
    form.append('document_type', documentType);
    form.append('file', file);

    const xhr = new XMLHttpRequest();
    xhr.open('POST', `${BASE}/users/${userId}/documents`);

    const token = localStorage.getItem('token');
    const orgId = localStorage.getItem('organization_id');
    if (token) xhr.setRequestHeader('Authorization', token);
    if (orgId) xhr.setRequestHeader('X-Organization-Id', orgId);

    xhr.upload.onprogress = (e: ProgressEvent): void => {
      if (e.lengthComputable) onProgress(Math.round((e.loaded / e.total) * 100));
    };

    xhr.onload = (): void => {
      let json: unknown;
      try { json = JSON.parse(xhr.responseText); } catch { json = null; }

      if (xhr.status >= 200 && xhr.status < 300) {
        resolve((json as ApiResponse<UserDocument>).data);
      } else {
        reject(new ApiException(xhr.status, json as ApiError));
      }
    };

    xhr.onerror = (): void => {
      reject(new Error('Network error during upload'));
    };

    xhr.send(form);
  });
}

export interface UserProfile {
  id: string;
  user_id: string;
  bio: string | null;
  date_of_birth: string | null;
  avatar_url: string | null;
  website: string | null;
  address_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface UpsertUserProfilePayload {
  bio?: string;
  date_of_birth?: string;
  avatar_url?: string;
  website?: string;
  address_id?: string;
}

// Matches the `File` struct returned by the API (polymorphic files table).
export interface UserDocument {
  id: string;
  created_by: string;
  fileable_type: string;
  fileable_id: string;
  // Kebab-case enum: "user-signature" | "user-initials" | "user-proof-of-funds"
  file_type: string;
  name: string;
  absolute_path: string;
  relative_path: string;
  size: number;
  mime_type: string;
  created_at: string;
  updated_at: string;
  deleted_at: string | null;
}

export interface UserBankAccount {
  id: string;
  user_id: string;
  bank_name: string;
  account_holder_name: string;
  account_type: string;
  account_number: string;
  routing_number: string;
  is_primary: boolean;
  created_at: string;
}

export interface CreateBankAccountPayload {
  bank_name: string;
  account_holder_name: string;
  account_type?: string;
  account_number: string;
  routing_number: string;
  is_primary?: boolean;
}

export const userProfileService = {
  getProfile:      (userId: string): Promise<UserProfile | null>  => api.get<UserProfile | null>(`/users/${userId}/profile`),
  upsertProfile:   (userId: string, payload: UpsertUserProfilePayload): Promise<UserProfile> =>
    api.put<UserProfile>(`/users/${userId}/profile`, payload),

  getDocuments:    (userId: string): Promise<UserDocument[]>      => api.get<UserDocument[]>(`/users/${userId}/documents`),
  deleteDocument:  (userId: string, docId: string): Promise<void> =>
    api.delete<void>(`/users/${userId}/documents/${docId}`),

  getBankAccounts: (userId: string): Promise<UserBankAccount[]>   => api.get<UserBankAccount[]>(`/users/${userId}/bank-accounts`),
  createBankAccount: (userId: string, payload: CreateBankAccountPayload): Promise<UserBankAccount> =>
    api.post<UserBankAccount>(`/users/${userId}/bank-accounts`, payload),
  updateBankAccount: (userId: string, accountId: string, payload: CreateBankAccountPayload): Promise<void> =>
    api.put<void>(`/users/${userId}/bank-accounts/${accountId}`, payload),
  deleteBankAccount: (userId: string, accountId: string): Promise<void> =>
    api.delete<void>(`/users/${userId}/bank-accounts/${accountId}`),
};
