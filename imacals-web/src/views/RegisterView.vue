<script setup lang="ts">
import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import { useAuth, ApiException } from '@/composables/useAuth';
import { useCart } from '@/composables/useCart';

const route  = useRoute();
const router = useRouter();
const { register } = useAuth();
const { lines } = useCart();

const form = ref({
  first_name: '',
  last_name: '',
  email: '',
  phone: '',
  password: '',
  confirm_password: '',
});

const showPassword: Ref<boolean> = ref(false);
const submitting: Ref<boolean>   = ref(false);
const error: Ref<string | null>  = ref(null);

const canSubmit: ComputedRef<boolean> = computed(() =>
  form.value.first_name.trim().length > 0
  && form.value.last_name.trim().length > 0
  && form.value.email.trim().length > 0
  && form.value.password.length >= 6
  && form.value.password === form.value.confirm_password
  && !submitting.value,
);

const passwordMismatch: ComputedRef<boolean> = computed(() =>
  form.value.confirm_password.length > 0 && form.value.password !== form.value.confirm_password,
);

async function onSubmit(): Promise<void> {
  if (!canSubmit.value) return;
  error.value      = null;
  submitting.value = true;

  try {
    await register({
      first_name: form.value.first_name.trim(),
      last_name:  form.value.last_name.trim(),
      email:      form.value.email.trim(),
      phone:      form.value.phone.trim() || undefined,
      password:   form.value.password,
    });

    const redirect = (route.query.redirect as string) || (lines.value.length > 0 ? '/checkout' : '/account');
    router.push(redirect);
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Registration failed. Please try again.';
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div class="auth-card">
      <header class="auth-head">
        <p class="eyebrow">Customer Registration</p>
        <h1 class="auth-title">Create your account</h1>
        <p class="auth-copy">
          Order faster, save delivery addresses, and track your wholesale shipments from Aba.
        </p>
      </header>

      <form class="auth-form" @submit.prevent="onSubmit" novalidate>
        <div class="field-grid">
          <div class="field">
            <label class="field-label" for="first_name">First name</label>
            <input
              id="first_name"
              v-model="form.first_name"
              class="field-input"
              type="text"
              required
              placeholder="e.g. Chukwudi"
              autocomplete="given-name"
            />
          </div>

          <div class="field">
            <label class="field-label" for="last_name">Last name</label>
            <input
              id="last_name"
              v-model="form.last_name"
              class="field-input"
              type="text"
              required
              placeholder="e.g. Okonkwo"
              autocomplete="family-name"
            />
          </div>
        </div>

        <div class="field">
          <label class="field-label" for="email">Email address</label>
          <input
            id="email"
            v-model="form.email"
            class="field-input"
            type="email"
            required
            placeholder="chukwudi@example.com"
            autocomplete="email"
          />
        </div>

        <div class="field">
          <label class="field-label" for="phone">Phone number</label>
          <input
            id="phone"
            v-model="form.phone"
            class="field-input"
            type="tel"
            placeholder="0800 000 0000"
            autocomplete="tel"
          />
          <span class="field-hint">Used by dispatch to confirm deliveries.</span>
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
            placeholder="At least 6 characters"
            autocomplete="new-password"
          />
        </div>

        <div class="field">
          <label class="field-label" for="confirm_password">Confirm password</label>
          <input
            id="confirm_password"
            v-model="form.confirm_password"
            class="field-input"
            :type="showPassword ? 'text' : 'password'"
            required
            placeholder="Re-enter your password"
            autocomplete="new-password"
          />
          <span v-if="passwordMismatch" class="field-error">Passwords do not match</span>
        </div>

        <p v-if="error" class="form-error" role="alert">{{ error }}</p>

        <!-- The single Tertiary action on this screen. -->
        <button class="btn-primary auth-submit" type="submit" :disabled="!canSubmit">
          {{ submitting ? 'Creating account…' : 'Create account' }}
        </button>

        <p class="auth-foot">
          Already have an account?
          <RouterLink
            class="inline-link"
            :to="{ path: '/login', query: route.query }"
          >
            Sign in
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
  max-width: 480px;
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

.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
}

@media (max-width: 480px) {
  .field-grid { grid-template-columns: 1fr; }
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

.field-hint {
  font-family: var(--font-body);
  font-size: 0.75rem;
  color: var(--color-secondary);
  margin-top: 4px;
}

.field-error {
  font-family: var(--font-body);
  font-size: 0.75rem;
  color: var(--color-tertiary);
  margin-top: 4px;
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
