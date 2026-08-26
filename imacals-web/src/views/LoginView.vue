<script setup lang="ts">
import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import { useAuth, ApiException } from '@/composables/useAuth';
import { useCart } from '@/composables/useCart';

const route  = useRoute();
const router = useRouter();
const { login } = useAuth();
const { lines } = useCart();

const form = ref({
  email: '',
  password: '',
});

const showPassword: Ref<boolean> = ref(false);
const submitting: Ref<boolean>   = ref(false);
const error: Ref<string | null>  = ref(null);

const canSubmit: ComputedRef<boolean> = computed(() =>
  form.value.email.trim().length > 0
  && form.value.password.length > 0
  && !submitting.value,
);

async function onSubmit(): Promise<void> {
  if (!canSubmit.value) return;
  error.value      = null;
  submitting.value = true;

  try {
    await login({
      email: form.value.email.trim(),
      password: form.value.password,
    });

    const redirect = (route.query.redirect as string) || (lines.value.length > 0 ? '/checkout' : '/account');
    router.push(redirect);
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Invalid email or password.';
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div class="auth-card">
      <header class="auth-head">
        <p class="eyebrow">Customer Sign In</p>
        <h1 class="auth-title">Welcome back</h1>
        <p class="auth-copy">
          Sign in to access your orders, track deliveries, and manage your account.
        </p>
      </header>

      <form class="auth-form" @submit.prevent="onSubmit" novalidate>
        <div class="field">
          <label class="field-label" for="email">Email address</label>
          <input
            id="email"
            v-model="form.email"
            class="field-input"
            type="email"
            required
            placeholder="you@example.com"
            autocomplete="email"
          />
        </div>

        <div class="field">
          <div class="field-label-row">
            <label class="field-label" for="password">Password</label>
            <button
              class="field-toggle-btn"
              type="button"
              @click="showPassword = !showPassword"
            >
              {{ showPassword ? 'Hide' : 'Show' }}
            </button>
          </div>
          <input
            id="password"
            v-model="form.password"
            class="field-input"
            :type="showPassword ? 'text' : 'password'"
            required
            placeholder="Your password"
            autocomplete="current-password"
          />
        </div>

        <p v-if="error" class="form-error" role="alert">{{ error }}</p>

        <!-- The single Tertiary action on this screen. -->
        <button class="btn-primary auth-submit" type="submit" :disabled="!canSubmit">
          {{ submitting ? 'Signing in…' : 'Sign in' }}
        </button>

        <p class="auth-foot">
          New to Imacals?
          <RouterLink
            class="inline-link"
            :to="{ path: '/register', query: route.query }"
          >
            Create an account
          </RouterLink>
        </p>
      </form>
    </div>
  </div>
</template>

<style scoped>
.page {
  display: flex;
  justify-content: center;
  align-items: flex-start;
  min-height: calc(100vh - var(--header-height) - 80px);
  padding: var(--spacing-lg) var(--spacing-md);
}

.auth-card {
  width: 100%;
  max-width: 440px;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.auth-head {
  margin-bottom: var(--spacing-lg);
}

.auth-title {
  font-family: var(--font-display);
  font-size: 1.85rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
  margin: var(--spacing-sm) 0;
}

.auth-copy {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--color-secondary);
  line-height: 1.45;
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.field {
  display: flex;
  flex-direction: column;
}

.field-label-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.field-toggle-btn {
  background: none;
  border: none;
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  cursor: pointer;
  padding: 0;
}

.field-toggle-btn:hover {
  color: var(--color-primary);
}

.form-error {
  color: var(--color-tertiary);
  font-size: 0.875rem;
}

.auth-submit {
  width: 100%;
  margin-top: var(--spacing-sm);
  padding: 12px 20px;
}

.auth-foot {
  text-align: center;
  font-family: var(--font-body);
  font-size: 0.85rem;
  color: var(--color-secondary);
  margin-top: var(--spacing-sm);
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: 500;
  border-bottom: 1px solid var(--color-border);
  transition: border-color 0.15s;
}

.inline-link:hover {
  border-bottom-color: var(--color-primary);
}
</style>
