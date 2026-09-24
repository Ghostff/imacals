import { api } from '@/services/api';
import type { Product } from '@/services/catalog';

// A wishlist as returned by the index endpoint — basic fields plus the live item count.
export interface WishlistSummary {
  id: string;
  organization_id: string;
  customer_id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
  item_count: number;
}

// A single wishlist item with the joined product snapshot — the shape the storefront renders.
export interface WishlistItem {
  id: string;
  wishlist_id: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
  product: Product;
}

// A wishlist detail — basic fields flattened together with its items.
export interface WishlistDetail {
  id: string;
  organization_id: string;
  customer_id: string;
  name: string;
  description: string | null;
  created_at: string;
  updated_at: string;
  items: WishlistItem[];
}

export const wishlistService = {
  list: (): Promise<WishlistSummary[]> =>
    api.get<WishlistSummary[]>('/wishlists'),

  show: (id: string): Promise<WishlistDetail> =>
    api.get<WishlistDetail>(`/wishlists/${id}`),

  create: (name: string, description?: string): Promise<WishlistSummary> =>
    api.post<WishlistSummary>('/wishlists', {
      name,
      description: description ?? null,
    }),

  update: (id: string, payload: { name?: string; description?: string | null }): Promise<WishlistSummary> =>
    api.put<WishlistSummary>(`/wishlists/${id}`, payload),

  remove: (id: string): Promise<void> =>
    api.delete<void>(`/wishlists/${id}`),

  addItem: (wishlistId: string, productId: string, notes?: string): Promise<WishlistDetail> =>
    api.post<WishlistDetail>(`/wishlists/${wishlistId}/items`, {
      product_id: productId,
      notes: notes ?? null,
    }),

  removeItem: (wishlistId: string, itemId: string): Promise<void> =>
    api.delete<void>(`/wishlists/${wishlistId}/items/${itemId}`),
};
