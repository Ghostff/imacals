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

export async function getCatalogProducts(categorySlug?: string): Promise<CatalogProduct[]> {
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
  if (error) throw new Error(error.message);
  if (!prods) return [];

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
}

export async function getCatalogProductBySlug(slug: string): Promise<CatalogProduct | null> {
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

  if (error) throw new Error(error.message);
  if (!p) return null;

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
}

export async function getCatalogCategories(): Promise<{ slug: string; name: string }[]> {
  const { data, error } = await supabase
    .from('categories')
    .select('slug, name')
    .is('deleted_at', null)
    .order('name', { ascending: true });

  if (error) throw new Error(error.message);
  return data || [];
}
