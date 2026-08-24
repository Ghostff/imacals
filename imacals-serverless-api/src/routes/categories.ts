import { supabase } from '../supabase.js';

export interface Category {
  id: string;
  domain_id: string;
  created_by: string | null;
  name: string;
  slug: string;
  description: string | null;
  created_at: string;
  updated_at: string;
}

export async function listCategories(): Promise<Category[]> {
  const { data, error } = await supabase
    .from('categories')
    .select('*')
    .is('deleted_at', null)
    .order('name', { ascending: true });

  if (error) throw new Error(error.message);
  return data || [];
}

export async function createCategory(payload: any, userId?: string): Promise<Category> {
  let domainId = payload.domain_id;
  if (!domainId) {
    const { data: domain } = await supabase.from('domains').select('id').limit(1).maybeSingle();
    domainId = domain?.id || '00000000-0000-0000-0000-000000000001';
  }

  const { data, error } = await supabase
    .from('categories')
    .insert({
      domain_id: domainId,
      created_by: userId || null,
      name: payload.name,
      slug: payload.slug,
      description: payload.description || null,
    })
    .select('*')
    .single();

  if (error) throw new Error(error.message);
  return data;
}

export async function updateCategory(id: string, payload: any): Promise<Category> {
  const updates: any = { updated_at: new Date().toISOString() };
  if (payload.name !== undefined) updates.name = payload.name;
  if (payload.slug !== undefined) updates.slug = payload.slug;
  if (payload.description !== undefined) updates.description = payload.description;

  const { data, error } = await supabase
    .from('categories')
    .update(updates)
    .eq('id', id)
    .is('deleted_at', null)
    .select('*')
    .single();

  if (error) throw new Error(error.message);
  return data;
}

export async function deleteCategory(id: string): Promise<void> {
  const { error } = await supabase
    .from('categories')
    .update({ deleted_at: new Date().toISOString() })
    .eq('id', id)
    .is('deleted_at', null);

  if (error) throw new Error(error.message);
}
