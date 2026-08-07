import { api } from '@/services/api';

// Money crosses the wire as an integer count of kobo (₦1 = 100 kobo). Cart and order totals are
// summed as integers so no rounding drift can creep into a price the customer was quoted.
export interface Product {
  id: string;
  slug: string;
  name: string;
  description: string;
  category_slug: string;
  category_name: string;
  // The unit a single quantity buys — "carton", "bag (50kg)", "piece".
  unit: string;
  unit_price_kobo: number;
  // Minimum order quantity. Wholesale lines often cannot be bought as singles.
  min_order_quantity: number;
  in_stock: boolean;
  image_url: string | null;
}

export interface Category {
  slug: string;
  name: string;
}

export function formatNaira(kobo: number): string {
  return new Intl.NumberFormat('en-NG', {
    style: 'currency',
    currency: 'NGN',
    minimumFractionDigits: 0,
    maximumFractionDigits: 0,
  }).format(Math.round(kobo / 100));
}

// ── Preview catalogue ─────────────────────────────────────────────────────
// The products API does not exist yet (see docs/business_logic.md §3). Until it lands, the
// storefront renders this sample so the layout, cart and checkout flow can be exercised end to
// end. It is gated on VITE_USE_PREVIEW_CATALOG and the UI shows a banner whenever it is active,
// so preview rows are never mistaken for real stock. Delete this block once /catalog ships.
const PREVIEW_CATALOG: Product[] = [
  {
    id: 'prev-1', slug: 'rice-50kg', name: 'Long Grain Rice — 50kg Bag',
    description: 'Parboiled long grain rice, 50kg bag. Sold by the bag, minimum five bags.',
    category_slug: 'foodstuff', category_name: 'Foodstuff',
    unit: 'bag (50kg)', unit_price_kobo: 8_950_000, min_order_quantity: 5,
    in_stock: true, image_url: null,
  },
  {
    id: 'prev-2', slug: 'vegetable-oil-25l', name: 'Vegetable Oil — 25L Keg',
    description: 'Refined vegetable oil in a 25 litre keg. Sold by the keg.',
    category_slug: 'foodstuff', category_name: 'Foodstuff',
    unit: 'keg (25L)', unit_price_kobo: 5_400_000, min_order_quantity: 2,
    in_stock: true, image_url: null,
  },
  {
    id: 'prev-3', slug: 'detergent-carton', name: 'Detergent Powder — Carton of 24',
    description: 'Carton of 24 × 900g detergent sachets.',
    category_slug: 'household', category_name: 'Household',
    unit: 'carton (24)', unit_price_kobo: 3_120_000, min_order_quantity: 1,
    in_stock: true, image_url: null,
  },
  {
    id: 'prev-4', slug: 'bar-soap-carton', name: 'Bar Soap — Carton of 48',
    description: 'Carton of 48 multipurpose bar soaps.',
    category_slug: 'household', category_name: 'Household',
    unit: 'carton (48)', unit_price_kobo: 2_760_000, min_order_quantity: 1,
    in_stock: false, image_url: null,
  },
  {
    id: 'prev-5', slug: 'sachet-water-bag', name: 'Sachet Water — Bag of 20',
    description: 'Bag of 20 sachets, 50cl each. Sold by the bag.',
    category_slug: 'beverages', category_name: 'Beverages',
    unit: 'bag (20)', unit_price_kobo: 30_000, min_order_quantity: 20,
    in_stock: true, image_url: null,
  },
  {
    id: 'prev-6', slug: 'malt-crate', name: 'Malt Drink — Crate of 24',
    description: 'Crate of 24 × 33cl bottles. Empties returnable at the Aba depot.',
    category_slug: 'beverages', category_name: 'Beverages',
    unit: 'crate (24)', unit_price_kobo: 1_080_000, min_order_quantity: 2,
    in_stock: true, image_url: null,
  },
];

// True while the storefront is running against sample data instead of the live catalogue.
export const usingPreviewCatalog: boolean =
  (import.meta.env.VITE_USE_PREVIEW_CATALOG as string | undefined) === 'true';

export const catalogService = {
  async listProducts(categorySlug?: string): Promise<Product[]> {
    if (usingPreviewCatalog) {
      return categorySlug
        ? PREVIEW_CATALOG.filter((p) => p.category_slug === categorySlug)
        : PREVIEW_CATALOG;
    }
    const query = categorySlug ? `?category=${encodeURIComponent(categorySlug)}` : '';
    return api.get<Product[]>(`/catalog/products${query}`);
  },

  async findProduct(slug: string): Promise<Product> {
    if (usingPreviewCatalog) {
      const found = PREVIEW_CATALOG.find((p) => p.slug === slug);
      if (!found) throw new Error('Product not found');
      return found;
    }
    return api.get<Product>(`/catalog/products/${slug}`);
  },

  async listCategories(): Promise<Category[]> {
    if (usingPreviewCatalog) {
      const seen = new Map<string, Category>();
      for (const p of PREVIEW_CATALOG) {
        if (!seen.has(p.category_slug)) {
          seen.set(p.category_slug, { slug: p.category_slug, name: p.category_name });
        }
      }
      return [...seen.values()];
    }
    return api.get<Category[]>('/catalog/categories');
  },
};
