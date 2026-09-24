import { supabase, STORAGE_BUCKET } from '../supabase.js';

export interface AdminProductImage {
  id: string;
  url: string;
  is_default: boolean;
  name?: string;
}

export interface AdminProduct {
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
  images?: AdminProductImage[];
  created_at: string;
  updated_at: string;
}

export const FALLBACK_ADMIN_PRODUCTS: AdminProduct[] = [
  {
    id: '00000000-0000-0000-0000-000000000001',
    organization_id: '00000000-0000-0000-0000-000000000001',
    domain_id: '00000000-0000-0000-0000-000000000001',
    category_id: '00000000-0000-0000-0000-000000000001',
    category_name: 'Foodstuff',
    category_slug: 'foodstuff',
    created_by: '00000000-0000-0000-0000-000000000001',
    name: 'Long Grain Rice — 50kg Bag',
    slug: 'rice-50kg',
    description: 'Parboiled long grain rice, 50kg bag. Sold by the bag, minimum five bags.',
    unit: 'bag (50kg)',
    unit_price_kobo: 8_950_000,
    min_order_quantity: 5,
    in_stock: true,
    image_url: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '00000000-0000-0000-0000-000000000002',
    organization_id: '00000000-0000-0000-0000-000000000001',
    domain_id: '00000000-0000-0000-0000-000000000001',
    category_id: '00000000-0000-0000-0000-000000000001',
    category_name: 'Foodstuff',
    category_slug: 'foodstuff',
    created_by: '00000000-0000-0000-0000-000000000001',
    name: 'Vegetable Oil — 25L Keg',
    slug: 'vegetable-oil-25l',
    description: 'Refined vegetable oil in a 25 litre keg. Sold by the keg.',
    unit: 'keg (25L)',
    unit_price_kobo: 5_400_000,
    min_order_quantity: 2,
    in_stock: true,
    image_url: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '00000000-0000-0000-0000-000000000003',
    organization_id: '00000000-0000-0000-0000-000000000001',
    domain_id: '00000000-0000-0000-0000-000000000001',
    category_id: '00000000-0000-0000-0000-000000000002',
    category_name: 'Household',
    category_slug: 'household',
    created_by: '00000000-0000-0000-0000-000000000001',
    name: 'Detergent Powder — Carton of 24',
    slug: 'detergent-carton',
    description: 'Carton of 24 × 900g detergent sachets.',
    unit: 'carton (24)',
    unit_price_kobo: 3_120_000,
    min_order_quantity: 1,
    in_stock: true,
    image_url: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '00000000-0000-0000-0000-000000000004',
    organization_id: '00000000-0000-0000-0000-000000000001',
    domain_id: '00000000-0000-0000-0000-000000000001',
    category_id: '00000000-0000-0000-0000-000000000002',
    category_name: 'Household',
    category_slug: 'household',
    created_by: '00000000-0000-0000-0000-000000000001',
    name: 'Bar Soap — Carton of 48',
    slug: 'bar-soap-carton',
    description: 'Carton of 48 multipurpose bar soaps.',
    unit: 'carton (48)',
    unit_price_kobo: 2_760_000,
    min_order_quantity: 1,
    in_stock: false,
    image_url: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '00000000-0000-0000-0000-000000000005',
    organization_id: '00000000-0000-0000-0000-000000000001',
    domain_id: '00000000-0000-0000-0000-000000000001',
    category_id: '00000000-0000-0000-0000-000000000003',
    category_name: 'Beverages',
    category_slug: 'beverages',
    created_by: '00000000-0000-0000-0000-000000000001',
    name: 'Sachet Water — Bag of 20',
    slug: 'sachet-water-bag',
    description: 'Bag of 20 sachets, 50cl each. Sold by the bag.',
    unit: 'bag (20)',
    unit_price_kobo: 30_000,
    min_order_quantity: 20,
    in_stock: true,
    image_url: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: '00000000-0000-0000-0000-000000000006',
    organization_id: '00000000-0000-0000-0000-000000000001',
    domain_id: '00000000-0000-0000-0000-000000000001',
    category_id: '00000000-0000-0000-0000-000000000003',
    category_name: 'Beverages',
    category_slug: 'beverages',
    created_by: '00000000-0000-0000-0000-000000000001',
    name: 'Malt Drink — Crate of 24',
    slug: 'malt-crate',
    description: 'Crate of 24 × 33cl bottles. Empties returnable at the Aba depot.',
    unit: 'crate (24)',
    unit_price_kobo: 1_080_000,
    min_order_quantity: 2,
    in_stock: true,
    image_url: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

export async function listAdminProducts(): Promise<AdminProduct[]> {
  try {
    const { data: prods, error } = await supabase
      .from('products')
      .select(`
        id,
        organization_id,
        domain_id,
        category_id,
        created_by,
        name,
        slug,
        description,
        unit,
        unit_price_kobo,
        min_order_quantity,
        in_stock,
        created_at,
        updated_at,
        categories (
          id,
          name,
          slug
        )
      `)
      .is('deleted_at', null)
      .order('created_at', { ascending: false });

    if (error || !prods || prods.length === 0) {
      return FALLBACK_ADMIN_PRODUCTS;
    }

    const productIds = prods.map((p: any) => p.id);
    const { data: files } = await supabase
      .from('files')
      .select('id, fileable_id, absolute_path, type, name, created_at')
      .eq('fileable_type', 'products')
      .in('fileable_id', productIds)
      .is('deleted_at', null)
      .order('created_at', { ascending: true });

    const imageMap = new Map<string, string>();
    const imagesMap = new Map<string, AdminProductImage[]>();

    if (files) {
      for (const f of files) {
        if (!imagesMap.has(f.fileable_id)) {
          imagesMap.set(f.fileable_id, []);
        }
        const isDefault = f.type === 'product-image-default';
        imagesMap.get(f.fileable_id)!.push({
          id: f.id,
          url: f.absolute_path,
          is_default: isDefault,
          name: f.name,
        });

        if (isDefault || !imageMap.has(f.fileable_id)) {
          imageMap.set(f.fileable_id, f.absolute_path);
        }
      }
    }

    return prods.map((p: any) => ({
      id: p.id,
      organization_id: p.organization_id,
      domain_id: p.domain_id,
      category_id: p.category_id,
      category_name: p.categories?.name || '',
      category_slug: p.categories?.slug || '',
      created_by: p.created_by,
      name: p.name,
      slug: p.slug,
      description: p.description,
      unit: p.unit,
      unit_price_kobo: Number(p.unit_price_kobo),
      min_order_quantity: Number(p.min_order_quantity) || 1,
      in_stock: Boolean(p.in_stock),
      image_url: imageMap.get(p.id) || null,
      images: imagesMap.get(p.id) || [],
      created_at: p.created_at,
      updated_at: p.updated_at,
    }));
  } catch {
    return FALLBACK_ADMIN_PRODUCTS;
  }
}

export async function getAdminProductById(id: string): Promise<AdminProduct | null> {
  const { data: p, error } = await supabase
    .from('products')
    .select(`
      id,
      organization_id,
      domain_id,
      category_id,
      created_by,
      name,
      slug,
      description,
      unit,
      unit_price_kobo,
      min_order_quantity,
      in_stock,
      created_at,
      updated_at,
      categories (
        id,
        name,
        slug
      )
    `)
    .eq('id', id)
    .is('deleted_at', null)
    .maybeSingle();

  if (error) throw new Error(error.message);
  if (!p) return null;

  const { data: files } = await supabase
    .from('files')
    .select('id, absolute_path, type, name, created_at')
    .eq('fileable_type', 'products')
    .eq('fileable_id', p.id)
    .is('deleted_at', null)
    .order('created_at', { ascending: true });

  let defaultUrl: string | null = null;
  const images: AdminProductImage[] = [];

  if (files) {
    for (const f of files) {
      const isDefault = f.type === 'product-image-default';
      images.push({
        id: f.id,
        url: f.absolute_path,
        is_default: isDefault,
        name: f.name,
      });
      if (isDefault || !defaultUrl) {
        defaultUrl = f.absolute_path;
      }
    }
  }

  return {
    id: p.id,
    organization_id: p.organization_id,
    domain_id: p.domain_id,
    category_id: p.category_id,
    category_name: (p as any).categories?.name || '',
    category_slug: (p as any).categories?.slug || '',
    created_by: p.created_by,
    name: p.name,
    slug: p.slug,
    description: p.description,
    unit: p.unit,
    unit_price_kobo: Number(p.unit_price_kobo),
    min_order_quantity: Number(p.min_order_quantity) || 1,
    in_stock: Boolean(p.in_stock),
    image_url: defaultUrl || null,
    images,
    created_at: p.created_at,
    updated_at: p.updated_at,
  };
}

export async function createProduct(payload: any, userId?: string): Promise<AdminProduct> {
  // Resolve organization_id if not present
  let orgId = payload.organization_id;
  if (!orgId) {
    const { data: org } = await supabase.from('organizations').select('id').limit(1).maybeSingle();
    orgId = org?.id || '00000000-0000-0000-0000-000000000001';
  }

  // Resolve domain_id if not present
  let domainId = payload.domain_id;
  if (!domainId) {
    const { data: domain } = await supabase.from('domains').select('id').limit(1).maybeSingle();
    domainId = domain?.id || '00000000-0000-0000-0000-000000000001';
  }

  // Resolve created_by if not present
  let createdBy = userId || payload.created_by;
  if (!createdBy) {
    const { data: user } = await supabase.from('users').select('id').limit(1).maybeSingle();
    createdBy = user?.id || '00000000-0000-0000-0000-000000000001';
  }

  const { data, error } = await supabase
    .from('products')
    .insert({
      organization_id: orgId,
      domain_id: domainId,
      category_id: payload.category_id,
      created_by: createdBy,
      name: payload.name,
      slug: payload.slug,
      description: payload.description || null,
      unit: payload.unit,
      unit_price_kobo: Number(payload.unit_price_kobo),
      min_order_quantity: Number(payload.min_order_quantity) || 1,
      in_stock: payload.in_stock !== undefined ? Boolean(payload.in_stock) : true,
    })
    .select('id')
    .single();

  if (error) throw new Error(error.message);

  const created = await getAdminProductById(data.id);
  if (!created) throw new Error('Product created but could not be retrieved');
  return created;
}

export async function updateProduct(id: string, payload: any): Promise<AdminProduct> {
  const updates: any = { updated_at: new Date().toISOString() };
  if (payload.name !== undefined) updates.name = payload.name;
  if (payload.slug !== undefined) updates.slug = payload.slug;
  if (payload.category_id !== undefined) updates.category_id = payload.category_id;
  if (payload.description !== undefined) updates.description = payload.description;
  if (payload.unit !== undefined) updates.unit = payload.unit;
  if (payload.unit_price_kobo !== undefined) updates.unit_price_kobo = Number(payload.unit_price_kobo);
  if (payload.min_order_quantity !== undefined) updates.min_order_quantity = Number(payload.min_order_quantity);
  if (payload.in_stock !== undefined) updates.in_stock = Boolean(payload.in_stock);

  const { error } = await supabase
    .from('products')
    .update(updates)
    .eq('id', id)
    .is('deleted_at', null);

  if (error) throw new Error(error.message);

  const updated = await getAdminProductById(id);
  if (!updated) throw new Error('Product updated but could not be retrieved');
  return updated;
}

export async function deleteProduct(id: string): Promise<void> {
  const { error } = await supabase
    .from('products')
    .update({ deleted_at: new Date().toISOString() })
    .eq('id', id)
    .is('deleted_at', null);

  if (error) throw new Error(error.message);
}

export async function uploadProductImage(
  productId: string,
  buffer: Buffer,
  fileName: string,
  mimeType: string,
  isDefault: boolean = false,
  userId?: string,
): Promise<AdminProduct> {
  const ext = fileName.split('.').pop() || 'jpg';
  const filePath = `products/${productId}/${Date.now()}_${Math.random().toString(36).substring(2, 7)}.${ext}`;

  // Ensure storage bucket exists
  await supabase.storage.createBucket(STORAGE_BUCKET, { public: true }).catch(() => {});

  // Upload to Supabase Storage
  const { error: uploadError } = await supabase.storage
    .from(STORAGE_BUCKET)
    .upload(filePath, buffer, {
      contentType: mimeType,
      upsert: true,
    });

  if (uploadError) throw new Error(`Supabase Storage upload failed: ${uploadError.message}`);

  const { data: publicUrlData } = supabase.storage.from(STORAGE_BUCKET).getPublicUrl(filePath);
  const absolutePath = publicUrlData.publicUrl;

  // Check existing images
  const { data: existing } = await supabase
    .from('files')
    .select('id')
    .eq('fileable_type', 'products')
    .eq('fileable_id', productId)
    .is('deleted_at', null);

  const shouldBeDefault = isDefault || !existing || existing.length === 0;

  if (shouldBeDefault && existing && existing.length > 0) {
    // Demote any existing default
    await supabase
      .from('files')
      .update({ type: 'product-image' })
      .eq('fileable_type', 'products')
      .eq('fileable_id', productId)
      .eq('type', 'product-image-default')
      .is('deleted_at', null);
  }

  // Resolve user id
  let createdBy = userId;
  if (!createdBy) {
    const { data: user } = await supabase.from('users').select('id').limit(1).maybeSingle();
    createdBy = user?.id || '00000000-0000-0000-0000-000000000001';
  }

  // Insert files record
  await supabase.from('files').insert({
    created_by: createdBy,
    fileable_type: 'products',
    fileable_id: productId,
    type: shouldBeDefault ? 'product-image-default' : 'product-image',
    name: fileName,
    absolute_path: absolutePath,
    relative_path: filePath,
    size: buffer.length,
    mime_type: mimeType,
  });

  const updated = await getAdminProductById(productId);
  if (!updated) throw new Error('Product not found after image upload');
  return updated;
}

export async function uploadProductImages(
  productId: string,
  files: { buffer: Buffer; filename: string; mimeType: string }[],
  defaultIndex?: number,
  userId?: string,
): Promise<AdminProduct> {
  let lastProduct: AdminProduct | null = null;
  for (let i = 0; i < files.length; i++) {
    const f = files[i];
    const isDefault = defaultIndex !== undefined ? i === defaultIndex : false;
    lastProduct = await uploadProductImage(productId, f.buffer, f.filename, f.mimeType, isDefault, userId);
  }

  const updated = await getAdminProductById(productId);
  if (!updated) throw new Error('Product not found after uploading images');
  return updated;
}

export async function setDefaultProductImage(
  productId: string,
  fileId: string,
): Promise<AdminProduct> {
  // Demote existing default
  await supabase
    .from('files')
    .update({ type: 'product-image' })
    .eq('fileable_type', 'products')
    .eq('fileable_id', productId)
    .is('deleted_at', null);

  // Set new default
  const { error } = await supabase
    .from('files')
    .update({ type: 'product-image-default' })
    .eq('id', fileId)
    .eq('fileable_type', 'products')
    .eq('fileable_id', productId)
    .is('deleted_at', null);

  if (error) throw new Error(`Could not set default image: ${error.message}`);

  const updated = await getAdminProductById(productId);
  if (!updated) throw new Error('Product not found after setting default image');
  return updated;
}

export async function deleteProductImage(
  productId: string,
  fileId: string,
): Promise<AdminProduct> {
  const { data: targetFile } = await supabase
    .from('files')
    .select('type')
    .eq('id', fileId)
    .maybeSingle();

  // Soft-delete
  const { error } = await supabase
    .from('files')
    .update({ deleted_at: new Date().toISOString() })
    .eq('id', fileId)
    .eq('fileable_type', 'products')
    .eq('fileable_id', productId)
    .is('deleted_at', null);

  if (error) throw new Error(`Could not delete image: ${error.message}`);

  // If deleted was default, promote first remaining
  if (targetFile?.type === 'product-image-default') {
    const { data: remaining } = await supabase
      .from('files')
      .select('id')
      .eq('fileable_type', 'products')
      .eq('fileable_id', productId)
      .is('deleted_at', null)
      .order('created_at', { ascending: true })
      .limit(1)
      .maybeSingle();

    if (remaining) {
      await supabase
        .from('files')
        .update({ type: 'product-image-default' })
        .eq('id', remaining.id);
    }
  }

  const updated = await getAdminProductById(productId);
  if (!updated) throw new Error('Product not found after deleting image');
  return updated;
}
