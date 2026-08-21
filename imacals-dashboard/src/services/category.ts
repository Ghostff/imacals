import { api } from '@/services/api';

export interface Category {
  id: string;
  domain_id: string;
  created_by?: string | null;
  name: string;
  slug: string;
  description: string | null;
  created_at?: string;
  updated_at?: string;
}

export interface CreateCategoryPayload {
  name: string;
  slug: string;
  description?: string;
  domain_id?: string;
}

export interface UpdateCategoryPayload {
  name?: string;
  slug?: string;
  description?: string;
}

export const categoryService = {
  async index(): Promise<Category[]> {
    return api.get<Category[]>('/categories');
  },

  async get(id: string): Promise<Category> {
    return api.get<Category>(`/categories/${id}`);
  },

  async create(payload: CreateCategoryPayload): Promise<Category> {
    return api.post<Category>('/categories', payload);
  },

  async update(id: string, payload: UpdateCategoryPayload): Promise<Category> {
    return api.put<Category>(`/categories/${id}`, payload);
  },

  async delete(id: string): Promise<{ message: string }> {
    return api.delete<{ message: string }>(`/categories/${id}`);
  },
};
