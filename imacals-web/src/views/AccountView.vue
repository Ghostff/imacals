<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import { useRouter, RouterLink } from 'vue-router';
import { useAuth, ApiException } from '@/composables/useAuth';
import { SITE } from '@/site';

const router = useRouter();
const { user, isAuthenticated, logout, updateProfile, fetchMe } = useAuth();

const editing: Ref<boolean>       = ref(false);
const saving: Ref<boolean>        = ref(false);
const saveError: Ref<string|null> = ref(null);
const saveSuccess: Ref<boolean>   = ref(false);

const form = ref({
  first_name: '',
  last_name: '',
  email: '',
  phone: '',
});

function initForm(): void {
  if (user.value) {
    form.value = {
      first_name: user.value.first_name ?? '',
      last_name:  user.value.last_name ?? '',
      email:      user.value.email ?? '',
      phone:      user.value.phone ?? '',
    };
  }
}

function startEdit(): void {
  initForm();
  saveError.value   = null;
  saveSuccess.value = false;
  editing.value     = true;
}

function cancelEdit(): void {
  initForm();
  editing.value = false;
}

async function save(): Promise<void> {
  saveError.value   = null;
  saveSuccess.value = false;
  saving.value      = true;

  try {
    await updateProfile({
      first_name: form.value.first_name.trim(),
      last_name:  form.value.last_name.trim(),
      email:      form.value.email.trim(),
      phone:      form.value.phone.trim() || undefined,
    });
    saveSuccess.value = true;
    editing.value     = false;
    setTimeout(() => { saveSuccess.value = false; }, 3000);
  } catch (e: unknown) {
    saveError.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Failed to update profile.';
  } finally {
    saving.value = false;
  }
}

function onSignOut(): void {
  logout();
  router.push('/');
}

onMounted(async () => {
  if (!isAuthenticated.value) {
    router.push('/login?redirect=/account');
    return;
  }
  await fetchMe();
  initForm();
});
</script>

<template>
  <div class="page">
    <header class="head">
      <p class="eyebrow">Customer Account</p>
      <div class="head-row">
        <h1 class="section-title">Hello, {{ user?.first_name || 'Customer' }}</h1>
        <button class="btn-secondary" type="button" @click="onSignOut">Sign out</button>
      </div>
    </header>

    <div class="layout">
      <!-- Profile Card -->
      <section class="card profile-card">
        <div class="card-head">
          <h2 class="card-title">Personal Details</h2>
          <button v-if="!editing" class="btn-edit" type="button" @click="startEdit">Edit</button>
        </div>

        <div v-if="saveSuccess" class="alert alert--success">Profile updated successfully.</div>
        <div v-if="saveError" class="alert alert--error">{{ saveError }}</div>

        <form v-if="editing" class="edit-form" @submit.prevent="save">
          <div class="field-grid">
            <div class="field">
              <label class="field-label" for="edit_fname">First name</label>
              <input id="edit_fname" v-model="form.first_name" class="field-input" type="text" required />
            </div>

            <div class="field">
              <label class="field-label" for="edit_lname">Last name</label>
              <input id="edit_lname" v-model="form.last_name" class="field-input" type="text" required />
            </div>
          </div>

          <div class="field">
            <label class="field-label" for="edit_email">Email address</label>
            <input id="edit_email" v-model="form.email" class="field-input" type="email" required />
          </div>

          <div class="field">
            <label class="field-label" for="edit_phone">Phone number</label>
            <input id="edit_phone" v-model="form.phone" class="field-input" type="tel" placeholder="0800 000 0000" />
          </div>

          <div class="form-actions">
            <button class="btn-secondary" type="button" :disabled="saving" @click="cancelEdit">Cancel</button>
            <button class="btn-primary" type="submit" :disabled="saving">
              {{ saving ? 'Saving…' : 'Save Changes' }}
            </button>
          </div>
        </form>

        <div v-else class="details-list">
          <div class="detail-row">
            <span class="detail-label">Full Name</span>
            <span class="detail-val">{{ user?.first_name }} {{ user?.last_name }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Email Address</span>
            <span class="detail-val">{{ user?.email }}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Phone Number</span>
            <span class="detail-val">{{ user?.phone || 'Not provided' }}</span>
          </div>
        </div>
      </section>

      <!-- Quick actions / assistance -->
      <aside class="sidebar">
        <div class="card help-card">
          <h2 class="card-title">Order Desk & Dispatch</h2>
          <p class="sidebar-copy">
            Orders from {{ SITE.warehouse.city }} are dispatched to Abia State and across Nigeria.
          </p>
          <div class="actions-list">
            <RouterLink class="action-link" to="/track">
              <span>Track an existing order</span>
              <span aria-hidden="true">→</span>
            </RouterLink>
            <RouterLink class="action-link" to="/catalog">
              <span>Browse wholesale catalogue</span>
              <span aria-hidden="true">→</span>
            </RouterLink>
            <RouterLink class="action-link" to="/cart">
              <span>View current cart</span>
              <span aria-hidden="true">→</span>
            </RouterLink>
          </div>

          <div class="phone-contact">
            <span class="phone-label">Need urgent help? Call the order desk:</span>
            <a class="phone-number" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.head {
  margin-bottom: var(--spacing-lg);
}

.head-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
  margin-top: var(--spacing-sm);
}

.layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 340px;
  gap: var(--spacing-lg);
  align-items: start;
}

@media (max-width: 820px) {
  .layout { grid-template-columns: 1fr; }
}

.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.card-head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: var(--spacing-md);
}

.card-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
}

.btn-edit {
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  padding: 5px 12px;
  cursor: pointer;
}

.btn-edit:hover {
  border-color: var(--color-primary);
}

.details-list {
  display: flex;
  flex-direction: column;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 12px 0;
  border-bottom: 1px solid var(--color-divider);
}

.detail-row:last-child {
  border-bottom: none;
}

.detail-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  text-transform: uppercase;
}

.detail-val {
  font-family: var(--font-body);
  font-size: 0.95rem;
  color: var(--color-primary);
}

.edit-form {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
}

.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
}

@media (max-width: 500px) {
  .field-grid { grid-template-columns: 1fr; }
}

.field {
  display: flex;
  flex-direction: column;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
}

.alert {
  padding: 10px 14px;
  border-radius: var(--rounded-md);
  font-size: 0.85rem;
  margin-bottom: var(--spacing-md);
}

.alert--success {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  color: var(--color-primary);
}

.alert--error {
  background-color: var(--color-neutral);
  border: 1px solid var(--color-tertiary);
  color: var(--color-tertiary);
}

.sidebar-copy {
  font-size: 0.85rem;
  color: var(--color-secondary);
  line-height: 1.4;
  margin-bottom: var(--spacing-md);
}

.actions-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: var(--spacing-lg);
}

.action-link {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  text-decoration: none;
  padding: 10px 12px;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  transition: border-color 0.15s, background-color 0.15s;
}

.action-link:hover {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
}

.phone-contact {
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--color-divider);
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.phone-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}

.phone-number {
  font-family: var(--font-label);
  font-size: 0.95rem;
  font-weight: 500;
  color: var(--color-primary);
  text-decoration: none;
}
</style>
