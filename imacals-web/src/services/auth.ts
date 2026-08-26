import { api } from '@/services/api';

export interface User {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
  phone?: string | null;
  date_of_birth?: string | null;
  created_at?: string;
  updated_at?: string;
}

export interface RegisterPayload {
  first_name: string;
  last_name: string;
  email: string;
  password: string;
  phone?: string;
}

export interface LoginPayload {
  email: string;
  password: string;
}

export interface AuthResponse {
  user: User;
  token: string;
}

export interface UpdateProfilePayload {
  first_name: string;
  last_name: string;
  email: string;
  phone?: string;
}

export const authService = {
  register: (payload: RegisterPayload): Promise<AuthResponse> =>
    api.post<AuthResponse>('/auth/register', payload),

  login: (payload: LoginPayload): Promise<AuthResponse> =>
    api.post<AuthResponse>('/auth/login', payload),

  me: (): Promise<{ user: User }> =>
    api.get<{ user: User }>('/auth/me'),

  updateProfile: (userId: string, payload: UpdateProfilePayload): Promise<void> =>
    api.put<void>(`/users/${userId}`, payload),
};
