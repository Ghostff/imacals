import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { authService, type User, type RegisterPayload, type LoginPayload, type UpdateProfilePayload } from '@/services/auth';
import { ApiException } from '@/services/api';

const token = ref<string | null>(localStorage.getItem('token'));
const user  = ref<User | null>(null);
const initialized = ref<boolean>(false);

export function useAuth(): {
  isAuthenticated: ComputedRef<boolean>;
  user: Ref<User | null>;
  token: Ref<string | null>;
  displayName: ComputedRef<string>;
  initials: ComputedRef<string>;
  login: (payload: LoginPayload) => Promise<void>;
  register: (payload: RegisterPayload) => Promise<void>;
  updateProfile: (payload: UpdateProfilePayload) => Promise<void>;
  logout: () => void;
  fetchMe: () => Promise<void>;
  initAuth: () => Promise<void>;
} {
  const isAuthenticated = computed<boolean>(() => token.value !== null);

  const displayName = computed<string>(() => {
    const full = `${user.value?.first_name ?? ''} ${user.value?.last_name ?? ''}`.trim();
    return full || user.value?.email || 'Account';
  });

  const initials = computed<string>(() => {
    const first = user.value?.first_name?.[0] ?? '';
    const last  = user.value?.last_name?.[0] ?? '';
    const fromName = `${first}${last}`.trim();
    if (fromName) return fromName.toUpperCase();
    return (user.value?.email?.[0] ?? '?').toUpperCase();
  });

  async function login(payload: LoginPayload): Promise<void> {
    const data = await authService.login(payload);
    token.value = data.token;
    user.value  = data.user;
    localStorage.setItem('token', data.token);
  }

  async function register(payload: RegisterPayload): Promise<void> {
    const data = await authService.register(payload);
    token.value = data.token;
    user.value  = data.user;
    localStorage.setItem('token', data.token);
  }

  async function updateProfile(payload: UpdateProfilePayload): Promise<void> {
    if (!user.value) throw new Error('Not logged in');
    await authService.updateProfile(user.value.id, payload);
    user.value = {
      ...user.value,
      first_name: payload.first_name,
      last_name:  payload.last_name,
      email:      payload.email,
      phone:      payload.phone ?? user.value.phone,
    };
  }

  function logout(): void {
    token.value = null;
    user.value  = null;
    localStorage.removeItem('token');
  }

  async function fetchMe(): Promise<void> {
    if (!token.value) return;
    try {
      const data = await authService.me();
      user.value = data.user;
    } catch {
      logout();
    }
  }

  async function initAuth(): Promise<void> {
    if (initialized.value) return;
    initialized.value = true;
    if (token.value) {
      await fetchMe();
    }
  }

  return {
    isAuthenticated,
    user,
    token,
    displayName,
    initials,
    login,
    register,
    updateProfile,
    logout,
    fetchMe,
    initAuth,
  };
}

export { ApiException };
