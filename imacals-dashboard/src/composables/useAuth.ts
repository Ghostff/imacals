import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { api, ApiException } from '@/services/api';

interface User {
  id: string;
  first_name: string;
  last_name: string;
  email: string;
  is_superuser: boolean;
}

interface LoginResponse {
  user: User;
  token: string;
}

const token = ref<string | null>(localStorage.getItem('token'));
const user  = ref<User | null>(null);

export function useAuth(): {
  isAuthenticated: ComputedRef<boolean>;
  user: Ref<User | null>;
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
  }

  function logout(): void {
    token.value = null;
    user.value  = null;
    localStorage.removeItem('token');
  }

  async function fetchMe(): Promise<void> {
    try {
      const data = await api.get<{ user: User }>('/auth/me');
      user.value = data.user;
    } catch {
      logout();
    }
  }

  return { isAuthenticated, user, login, logout, fetchMe };
}

export { ApiException };
