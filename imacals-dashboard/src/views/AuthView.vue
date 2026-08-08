<script setup lang="ts">
import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import { useAuth, ApiException } from '@/composables/useAuth';

const router = useRouter();
const route  = useRoute();
const { login } = useAuth();

const email: Ref<string>    = ref('');
const password: Ref<string> = ref('');
const error: Ref<string>    = ref('');
const loading: Ref<boolean> = ref(false);
const showPassword: Ref<boolean> = ref(false);

const canSubmit: ComputedRef<boolean> = computed<boolean>(
  () => email.value.trim().length > 0 && password.value.length > 0 && !loading.value,
);

async function submit(): Promise<void> {
  if (!canSubmit.value) return;
  error.value   = '';
  loading.value = true;
  try {
    await login(email.value.trim(), password.value);
    // Land on Users — the only thing in the dashboard until the ecommerce screens exist.
    // `redirect` wins when the guard bounced someone here from a deep link.
    const redirect = typeof route.query.redirect === 'string' ? route.query.redirect : null;
    if (redirect) {
      await router.replace(redirect);
    } else {
      await router.replace({ name: 'users-all' });
    }
  } catch (e: unknown) {
    if (e instanceof ApiException || e instanceof Error) {
      error.value = e.message;
    } else {
      error.value = 'Something went wrong. Please try again.';
    }
  } finally {
    loading.value = false;
  }
}
</script>

<template>
  <div class="auth-page">
    <main class="auth-card">
      <header class="auth-header">
        <p class="auth-eyebrow">imacals</p>
        <h1 class="auth-title">Sign in</h1>
        <p class="auth-subtitle">Orders, catalogue and dispatch for the Aba warehouse.</p>
      </header>

      <form class="auth-form" @submit.prevent="submit" novalidate>
        <div class="field-group">
          <label class="field-label" for="email">Email</label>
          <input
            id="email"
            v-model="email"
            class="field-input"
            type="email"
            autocomplete="email"
            placeholder="you@example.com"
            :disabled="loading"
            required
          />
        </div>

        <div class="field-group">
          <label class="field-label" for="password">Password</label>
          <div class="password-wrap">
            <input
              id="password"
              v-model="password"
              class="field-input"
              :type="showPassword ? 'text' : 'password'"
              autocomplete="current-password"
              placeholder="••••••••"
              :disabled="loading"
              required
            />
            <button
              class="password-toggle"
              type="button"
              :aria-label="showPassword ? 'Hide password' : 'Show password'"
              @click="showPassword = !showPassword"
            >
              {{ showPassword ? 'Hide' : 'Show' }}
            </button>
          </div>
        </div>

        <p v-if="error" class="auth-error" role="alert">{{ error }}</p>

        <button class="btn-primary" type="submit" :disabled="!canSubmit">
          {{ loading ? 'Signing in…' : 'Sign in' }}
        </button>
      </form>

      <footer class="auth-footer">
        <p class="auth-note">Accounts are created by an administrator.</p>
      </footer>
    </main>
  </div>
</template>

<style scoped>
.auth-page {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background-color: var(--color-neutral);
  padding: var(--spacing-lg);
}

.auth-card {
  width: 100%;
  max-width: 420px;
  background-color: var(--color-surface);
  border-radius: var(--rounded-lg);
  padding: 40px;
  box-shadow: 0 1px 4px color-mix(in srgb, var(--color-primary) 8%, transparent);
}

.auth-header {
  margin-bottom: var(--spacing-lg);
}

.auth-eyebrow {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-sm);
}

.auth-title {
  font-family: var(--font-display);
  font-size: 2rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
  line-height: 1.2;
}

.auth-subtitle {
  font-family: var(--font-body);
  font-size: 0.9375rem;
  color: var(--color-secondary);
  margin-top: var(--spacing-sm);
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.field-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.field-input {
  width: 100%;
  font-family: var(--font-body);
  font-size: 1rem;
  color: var(--color-primary);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-sm);
  padding: 10px 12px;
  outline: none;
  transition: border-color 0.15s;
}

.field-input:focus {
  border-color: var(--color-primary);
}

.field-input:disabled {
  opacity: 0.6;
}

.field-input::placeholder {
  color: color-mix(in srgb, var(--color-secondary) 65%, transparent);
}

.password-wrap {
  position: relative;
  display: flex;
  align-items: center;
}

.password-toggle {
  position: absolute;
  right: 8px;
  font-family: var(--font-label);
  font-size: 0.6875rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-secondary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 4px 6px;
}

.password-toggle:hover {
  color: var(--color-primary);
}

.auth-error {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-tertiary);
  background-color: color-mix(in srgb, var(--color-tertiary) 10%, var(--color-surface));
  border-radius: var(--rounded-sm);
  padding: 10px 12px;
}

.btn-primary {
  font-family: var(--font-body);
  font-size: 1rem;
  font-weight: 500;
  color: var(--color-on-primary);
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 12px 20px;
  cursor: pointer;
  transition: opacity 0.15s;
  margin-top: var(--spacing-sm);
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.auth-footer {
  margin-top: var(--spacing-lg);
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--color-border);
}

.auth-note {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
}
</style>
