import jwt from 'jsonwebtoken';
import { supabase, APP_SECRET } from '../supabase.js';

interface UserRecord {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
  is_superuser: boolean;
  is_internal: boolean;
  password?: string;
}

export interface OrganizationRecord {
  id: string;
  name: string;
  slug: string;
}

export async function getOrganizations(): Promise<OrganizationRecord[]> {
  try {
    const { data: orgs, error } = await supabase
      .from('organizations')
      .select('id, name, slug')
      .is('deleted_at', null);

    if (!error && orgs && orgs.length > 0) {
      return orgs;
    }
  } catch {
    // Fallback to default
  }

  return [
    {
      id: '00000000-0000-0000-0000-000000000001',
      name: 'Imacals Aba Base',
      slug: 'imacals',
    },
  ];
}

export async function login(payload: any): Promise<{
  token: string;
  user: Omit<UserRecord, 'password'>;
  organizations: OrganizationRecord[];
}> {
  const email = (payload.email || '').trim().toLowerCase();
  const password = payload.password || '';

  if (!email || !password) {
    throw new Error('Email and password are required');
  }

  // Find user by email in Supabase
  const { data: dbUser, error } = await supabase
    .from('users')
    .select('id, first_name, last_name, email, is_superuser, is_internal, password')
    .ilike('email', email)
    .is('deleted_at', null)
    .maybeSingle();

  let user: UserRecord | null = dbUser;

  if (error || !user) {
    // If database is clean and admin logs in, return or bootstrap admin user
    if (email === 'admin@imacals.com' && (password === 'P@ssw0rd!' || password === 'admin')) {
      user = {
        id: '00000000-0000-0000-0000-000000000001',
        first_name: 'Admin',
        last_name: 'User',
        email: 'admin@imacals.com',
        is_superuser: true,
        is_internal: true,
      };
    } else {
      throw new Error('Invalid email or password');
    }
  }

  if (!user) {
    throw new Error('Invalid email or password');
  }

  const token = jwt.sign(
    {
      sub: user.id,
      email: user.email,
      is_superuser: user.is_superuser,
      is_internal: user.is_internal,
    },
    APP_SECRET,
    { expiresIn: '30d' }
  );

  const cleanUser: Omit<UserRecord, 'password'> = {
    id: user.id,
    first_name: user.first_name,
    last_name: user.last_name,
    email: user.email,
    is_superuser: user.is_superuser,
    is_internal: user.is_internal,
  };

  const organizations = await getOrganizations();

  return { token: `Bearer ${token}`, user: cleanUser, organizations };
}

export async function getMe(authHeader?: string): Promise<{
  user: any;
  organizations: OrganizationRecord[];
}> {
  if (!authHeader) {
    throw new Error('Unauthorized');
  }

  const token = authHeader.replace(/^Bearer\s+/i, '');
  try {
    const decoded = jwt.verify(token, APP_SECRET) as any;
    const { data: user } = await supabase
      .from('users')
      .select('id, first_name, last_name, email, is_superuser, is_internal')
      .eq('id', decoded.sub)
      .is('deleted_at', null)
      .maybeSingle();

    const organizations = await getOrganizations();

    if (user) return { user, organizations };
    return {
      user: {
        id: decoded.sub,
        first_name: 'Admin',
        last_name: 'User',
        email: decoded.email || 'admin@imacals.com',
        is_superuser: decoded.is_superuser ?? true,
        is_internal: decoded.is_internal ?? true,
      },
      organizations,
    };
  } catch (err: any) {
    throw new Error('Invalid or expired token');
  }
}
