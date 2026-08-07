import { api } from '@/services/api';

export interface UserOrganization {
  id: string;
  name: string;
  slug: string;
}

export interface UserRole {
  id: string;
  name: string;
  title: string;
}

export interface User {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
  phone: string | null;
  date_of_birth: string | null;
  is_superuser: boolean;
  is_internal: boolean;
  last_logged_in_at: string | null;
  current_logged_in_at: string | null;
  created_at: string;
  updated_at: string;
  organizations: UserOrganization[];
  role: UserRole | null;
  user_role: UserRole | null;
}

export interface CreateUserPayload {
  first_name: string;
  last_name: string;
  email: string;
  password?: string;
  organization_ids: string[];
  user_role_id: string;
}

export interface UpdateUserPayload {
  first_name: string;
  last_name: string;
  email: string;
  phone?: string;
  date_of_birth?: string;
}

export const userService = {
  index:  (): Promise<User[]>                                    => api.get<User[]>('/users'),
  create: (payload: CreateUserPayload): Promise<{ user: User }> => api.post<{ user: User }>('/users', payload),
  update: (id: string, payload: UpdateUserPayload): Promise<void> => api.put<void>(`/users/${id}`, payload),
  delete: (id: string): Promise<void>                            => api.delete<void>(`/users/${id}`),
};
