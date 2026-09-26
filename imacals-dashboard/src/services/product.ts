import { api } from '@/services/api';

export interface ProductImageItem {
  id: string;
  url: string;
  is_default: boolean;
  name?: string;
}

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
  discount_price_kobo?: number | null;
  discount_percent?: number | null;
  min_order_quantity: number;
  in_stock: boolean;
  is_tax_exempt?: boolean;
  tax_rate_basis_points?: number;
  image_url: string | null;
  images?: ProductImageItem[];
  created_at: string;
  updated_at: string;
}

export interface CreateProductPayload {
  name: string;
  slug: string;
  category_id: string;
  unit: string;
  unit_price_kobo: number;
  discount_price_kobo?: number | null;
  min_order_quantity?: number;
  in_stock?: boolean;
  is_tax_exempt?: boolean;
  tax_rate_basis_points?: number;
  description?: string;
  domain_id?: string;
}

export interface UpdateProductPayload {
  name?: string;
  slug?: string;
  category_id?: string;
  unit?: string;
  unit_price_kobo?: number;
  discount_price_kobo?: number | null;
  min_order_quantity?: number;
  in_stock?: boolean;
  is_tax_exempt?: boolean;
  tax_rate_basis_points?: number;
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

export function getEffectivePriceKobo(prod: { unit_price_kobo: number; discount_price_kobo?: number | null }): number {
  return prod.discount_price_kobo && prod.discount_price_kobo > 0 && prod.discount_price_kobo < prod.unit_price_kobo
    ? prod.discount_price_kobo
    : prod.unit_price_kobo;
}

export function getDiscountPercent(prod: { unit_price_kobo: number; discount_price_kobo?: number | null }): number | null {
  if (!prod.discount_price_kobo || prod.discount_price_kobo <= 0 || prod.discount_price_kobo >= prod.unit_price_kobo) {
    return null;
  }
  return Math.round(((prod.unit_price_kobo - prod.discount_price_kobo) / prod.unit_price_kobo) * 100);
}

export function getTaxRatePercent(prod: { is_tax_exempt?: boolean; tax_rate_basis_points?: number }): number {
  if (prod.is_tax_exempt) return 0;
  return ((prod.tax_rate_basis_points ?? 750) / 100);
}

export function getTaxStatusLabel(prod: { is_tax_exempt?: boolean; tax_rate_basis_points?: number }): string {
  if (prod.is_tax_exempt) return '0% (Exempt)';
  const rate = getTaxRatePercent(prod);
  return `${rate}% VAT`;
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

  async uploadImages(id: string, files: File[], defaultIndex?: number): Promise<Product> {
    const formData = new FormData();
    files.forEach((f) => formData.append('files[]', f));
    if (defaultIndex !== undefined) {
      formData.append('default_index', defaultIndex.toString());
    }
    return api.upload<Product>(`/products/${id}/images`, formData);
  },

  async setDefaultImage(productId: string, fileId: string): Promise<Product> {
    return api.put<Product>(`/products/${productId}/images/${fileId}/default`, {});
  },

  async deleteImage(productId: string, fileId: string): Promise<Product> {
    return api.delete<Product>(`/products/${productId}/images/${fileId}`);
  },
};
