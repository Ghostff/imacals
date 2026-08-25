import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { api, ApiException } from '@/services/api';

interface User {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
  is_superuser: boolean;
}

interface Organization {
  id: string;
  name: string;
  slug: string;
}

interface LoginResponse {
  user: User;
  organizations?: Organization[];
  token: string;
}

const token              = ref<string | null>(localStorage.getItem('token'));
const user               = ref<User | null>(null);
const organizationId     = ref<string | null>(localStorage.getItem('organization_id'));

function storeOrganization(orgs?: Organization[] | null): void {
  // Keep whichever org is already active if it's still in the list; otherwise use the first.
  const safeOrgs = Array.isArray(orgs) ? orgs : [];
  const current  = organizationId.value;
  const match    = safeOrgs.find((o) => o.id === current) ?? safeOrgs[0] ?? null;
  organizationId.value = match?.id ?? null;
  if (match) {
    localStorage.setItem('organization_id', match.id);
  } else {
    localStorage.removeItem('organization_id');
  }
}

export function useAuth(): {
  isAuthenticated: ComputedRef<boolean>;
  user: Ref<User | null>;
  organizationId: Ref<string | null>;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  fetchMe: () => Promise<void>;
} {
  const isAuthenticated = computed<boolean>(() => token.value !== null);

  async function login(email: string, password: string): Promise<void> {
    const data = await api.post<LoginResponse>('/auth/login', { email, password });
    token.value = data.token;
    user.value  = data.user;
    localStorage.setItem('token', data.token);
    storeOrganization(data.organizations);
  }

  function logout(): void {
    token.value          = null;
    user.value           = null;
    organizationId.value = null;
    localStorage.removeItem('token');
    localStorage.removeItem('organization_id');
  }

  async function fetchMe(): Promise<void> {
    try {
      const data = await api.get<{ user: User; organizations?: Organization[] }>('/auth/me');
      user.value = data.user;
      storeOrganization(data.organizations);
    } catch {
      logout();
    }
  }

  return { isAuthenticated, user, organizationId, login, logout, fetchMe };
}

export { ApiException };
