import { createClient, SupabaseClient } from '@supabase/supabase-js';

const supabaseUrl = process.env.SUPABASE_URL || 'https://njcfsvdjbcshmsejwmrt.supabase.co';
const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY || process.env.SUPABASE_ANON_KEY || 'public-anon-key';

export const supabase: SupabaseClient = createClient(supabaseUrl, supabaseKey, {
  auth: {
    persistSession: false,
    autoRefreshToken: false,
  },
});

export const STORAGE_BUCKET = process.env.SUPABASE_STORAGE_BUCKET || 'products';
export const APP_SECRET = process.env.APP_SECRET || 'imacals-fallback-secret-for-dev';
