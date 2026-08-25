import { supabase } from '../supabase.js';

export interface CatalogProduct {
  id: string;
  slug: string;
  name: string;
  description: string;
  category_slug: string;
  category_name: string;
  unit: string;
  unit_price_kobo: number;
  min_order_quantity: number;
  in_stock: boolean;
  image_url: string | null;
}

const FALLBACK_CATALOG: CatalogProduct[] = [
  {
    id: 'prev-1',
    slug: 'rice-50kg',
    name: 'Long Grain Rice — 50kg Bag',
    description: 'Parboiled long grain rice, 50kg bag. Sold by the bag, minimum five bags.',
    category_slug: 'foodstuff',
    category_name: 'Foodstuff',
    unit: 'bag (50kg)',
    unit_price_kobo: 8_950_000,
    min_order_quantity: 5,
    in_stock: true,
    image_url: null,
  },
  {
    id: 'prev-2',
    slug: 'vegetable-oil-25l',
    name: 'Vegetable Oil — 25L Keg',
    description: 'Refined vegetable oil in a 25 litre keg. Sold by the keg.',
    category_slug: 'foodstuff',
    category_name: 'Foodstuff',
    unit: 'keg (25L)',
    unit_price_kobo: 5_400_000,
    min_order_quantity: 2,
    in_stock: true,
    image_url: null,
  },
  {
    id: 'prev-3',
    slug: 'detergent-carton',
    name: 'Detergent Powder — Carton of 24',
    description: 'Carton of 24 × 900g detergent sachets.',
    category_slug: 'household',
    category_name: 'Household',
    unit: 'carton (24)',
    unit_price_kobo: 3_120_000,
    min_order_quantity: 1,
    in_stock: true,
    image_url: null,
  },
  {
    id: 'prev-4',
    slug: 'bar-soap-carton',
    name: 'Bar Soap — Carton of 48',
    description: 'Carton of 48 multipurpose bar soaps.',
    category_slug: 'household',
    category_name: 'Household',
    unit: 'carton (48)',
    unit_price_kobo: 2_760_000,
    min_order_quantity: 1,
    in_stock: false,
    image_url: null,
  },
  {
    id: 'prev-5',
    slug: 'sachet-water-bag',
    name: 'Sachet Water — Bag of 20',
    description: 'Bag of 20 sachets, 50cl each. Sold by the bag.',
    category_slug: 'beverages',
    category_name: 'Beverages',
    unit: 'bag (20)',
    unit_price_kobo: 30_000,
    min_order_quantity: 20,
    in_stock: true,
    image_url: null,
  },
  {
    id: 'prev-6',
    slug: 'malt-crate',
    name: 'Malt Drink — Crate of 24',
    description: 'Crate of 24 × 33cl bottles. Empties returnable at the Aba depot.',
    category_slug: 'beverages',
    category_name: 'Beverages',
    unit: 'crate (24)',
    unit_price_kobo: 1_080_000,
    min_order_quantity: 2,
    in_stock: true,
    image_url: null,
  },
];

export async function getCatalogProducts(categorySlug?: string): Promise<CatalogProduct[]> {
  try {
    let query = supabase
      .from('products')
      .select(`
        id,
        slug,
        name,
        description,
        unit,
        unit_price_kobo,
        min_order_quantity,
        in_stock,
        created_at,
        categories!inner (
          id,
          name,
          slug
        )
      `)
      .is('deleted_at', null)
      .order('created_at', { ascending: false });

    if (categorySlug) {
      query = query.eq('categories.slug', categorySlug);
    }

    const { data: prods, error } = await query;
    if (error || !prods || prods.length === 0) {
      return categorySlug
        ? FALLBACK_CATALOG.filter((p) => p.category_slug === categorySlug)
        : FALLBACK_CATALOG;
    }

    // Fetch images for products
    const productIds = prods.map((p: any) => p.id);
    const { data: files } = await supabase
      .from('files')
      .select('fileable_id, absolute_path, created_at')
      .eq('fileable_type', 'products')
      .in('fileable_id', productIds)
      .is('deleted_at', null)
      .order('created_at', { ascending: false });

    const imageMap = new Map<string, string>();
    if (files) {
      for (const f of files) {
        if (!imageMap.has(f.fileable_id)) {
          imageMap.set(f.fileable_id, f.absolute_path);
        }
      }
    }

    return prods.map((p: any) => ({
      id: p.id,
      slug: p.slug,
      name: p.name,
      description: p.description || '',
      category_slug: p.categories?.slug || '',
      category_name: p.categories?.name || '',
      unit: p.unit,
      unit_price_kobo: Number(p.unit_price_kobo),
      min_order_quantity: Number(p.min_order_quantity) || 1,
      in_stock: Boolean(p.in_stock),
      image_url: imageMap.get(p.id) || null,
    }));
  } catch {
    return categorySlug
      ? FALLBACK_CATALOG.filter((p) => p.category_slug === categorySlug)
      : FALLBACK_CATALOG;
  }
}

export async function getCatalogProductBySlug(slug: string): Promise<CatalogProduct | null> {
  try {
    const { data: p, error } = await supabase
      .from('products')
      .select(`
        id,
        slug,
        name,
        description,
        unit,
        unit_price_kobo,
        min_order_quantity,
        in_stock,
        categories!inner (
          id,
          name,
          slug
        )
      `)
      .eq('slug', slug)
      .is('deleted_at', null)
      .maybeSingle();

    if (error || !p) {
      return FALLBACK_CATALOG.find((prod) => prod.slug === slug) ?? null;
    }

    const { data: file } = await supabase
      .from('files')
      .select('absolute_path')
      .eq('fileable_type', 'products')
      .eq('fileable_id', p.id)
      .is('deleted_at', null)
      .order('created_at', { ascending: false })
      .limit(1)
      .maybeSingle();

    return {
      id: p.id,
      slug: p.slug,
      name: p.name,
      description: p.description || '',
      category_slug: (p as any).categories?.slug || '',
      category_name: (p as any).categories?.name || '',
      unit: p.unit,
      unit_price_kobo: Number(p.unit_price_kobo),
      min_order_quantity: Number(p.min_order_quantity) || 1,
      in_stock: Boolean(p.in_stock),
      image_url: file?.absolute_path || null,
    };
  } catch {
    return FALLBACK_CATALOG.find((prod) => prod.slug === slug) ?? null;
  }
}

export async function getCatalogCategories(): Promise<{ slug: string; name: string }[]> {
  try {
    const { data, error } = await supabase
      .from('categories')
      .select('slug, name')
      .is('deleted_at', null)
      .order('name', { ascending: true });

    if (error || !data || data.length === 0) {
      return [
        { slug: 'foodstuff', name: 'Foodstuff' },
        { slug: 'household', name: 'Household' },
        { slug: 'beverages', name: 'Beverages' },
      ];
    }
    return data;
  } catch {
    return [
      { slug: 'foodstuff', name: 'Foodstuff' },
      { slug: 'household', name: 'Household' },
      { slug: 'beverages', name: 'Beverages' },
    ];
  }
}
