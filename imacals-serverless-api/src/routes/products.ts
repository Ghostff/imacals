import { supabase, STORAGE_BUCKET } from '../supabase.js';

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
  created_at: string;
  updated_at: string;
}

export async function listAdminProducts(): Promise<AdminProduct[]> {
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

  if (error) throw new Error(error.message);
  if (!prods) return [];

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
    created_at: p.created_at,
    updated_at: p.updated_at,
  }));
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
    image_url: file?.absolute_path || null,
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
  userId?: string,
): Promise<AdminProduct> {
  const ext = fileName.split('.').pop() || 'jpg';
  const filePath = `products/${productId}/${Date.now()}.${ext}`;

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

  // Mark old images as soft-deleted
  await supabase
    .from('files')
    .update({ deleted_at: new Date().toISOString() })
    .eq('fileable_type', 'products')
    .eq('fileable_id', productId)
    .is('deleted_at', null);

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
    type: 'product-image',
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
