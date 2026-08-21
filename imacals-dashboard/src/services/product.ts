import { api } from '@/services/api';

export interface Product {
  id: string;
  organization_id: string;
  domain_id: string;
  category_id: string;
  category_name: string;
  category_slug: string;
  created_by: string;
  name: string;
  slug: string;
  description: string | null;
  unit: string;
  unit_price_kobo: number;
  min_order_quantity: number;
  in_stock: boolean;
  image_url: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateProductPayload {
  name: string;
  slug: string;
  category_id: string;
  unit: string;
  unit_price_kobo: number;
  min_order_quantity?: number;
  in_stock?: boolean;
  description?: string;
  domain_id?: string;
}

export interface UpdateProductPayload {
  name?: string;
  slug?: string;
  category_id?: string;
  unit?: string;
  unit_price_kobo?: number;
  min_order_quantity?: number;
  in_stock?: boolean;
  description?: string;
  domain_id?: string;
}

export function formatNaira(kobo: number): string {
  return new Intl.NumberFormat('en-NG', {
    style: 'currency',
    currency: 'NGN',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(Math.round(kobo / 100));
}

export const productService = {
  async index(): Promise<Product[]> {
    return api.get<Product[]>('/products');
  },

  async get(id: string): Promise<Product> {
    return api.get<Product>(`/products/${id}`);
  },

  async create(payload: CreateProductPayload): Promise<Product> {
    return api.post<Product>('/products', payload);
  },

  async update(id: string, payload: UpdateProductPayload): Promise<Product> {
    return api.put<Product>(`/products/${id}`, payload);
  },

  async delete(id: string): Promise<{ message: string }> {
    return api.delete<{ message: string }>(`/products/${id}`);
  },

  async uploadImage(id: string, file: File): Promise<Product> {
    const formData = new FormData();
    formData.append('file', file);
    return api.upload<Product>(`/products/${id}/image`, formData);
  },
};
