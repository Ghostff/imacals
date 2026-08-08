<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import { useRoute } from 'vue-router';
import { userService, type User, type UpdateUserPayload } from '@/services/user';
import { roleService, type Role } from '@/services/role';
import { ApiException } from '@/services/api';

const route  = useRoute();
const userId = route.params.id as string;

const targetUser: Ref<User | null> = ref(null);
const roles: Ref<Role[]>           = ref([]);
const loading: Ref<boolean>        = ref(true);
const error: Ref<string | null>    = ref(null);

const form: Ref<UpdateUserPayload> = ref({
  first_name: '', last_name: '', email: '', phone: '', date_of_birth: '', role_id: '',
});
const saving: Ref<boolean>          = ref(false);
const saveError: Ref<string | null> = ref(null);
const saved: Ref<boolean>           = ref(false);

onMounted(async () => {
  try {
    const [users, roleList] = await Promise.all([
      userService.index(),
      roleService.index().catch(() => []),
    ]);
    roles.value      = roleList;
    targetUser.value = users.find((u) => u.id === userId) ?? null;

    if (!targetUser.value) {
      error.value = 'User not found.';
      return;
    }

    form.value = {
      first_name:    targetUser.value.first_name,
      last_name:     targetUser.value.last_name,
      email:         targetUser.value.email,
      phone:         targetUser.value.phone ?? '',
      date_of_birth: targetUser.value.date_of_birth ?? '',
      role_id:       targetUser.value.role_id ?? '',
    };
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load profile.';
  } finally {
    loading.value = false;
  }
});

async function save(): Promise<void> {
  saveError.value = null;
  saved.value     = false;
  saving.value    = true;
  try {
    const payload: UpdateUserPayload = {
      first_name:    form.value.first_name.trim(),
      last_name:     form.value.last_name.trim(),
      email:         form.value.email.trim(),
      phone:         form.value.phone?.trim() || undefined,
      date_of_birth: form.value.date_of_birth || undefined,
    };
    // Only send role_id when it actually changed — the API treats a role change as a permission
    // change and gates it more strictly than a name edit.
    if (form.value.role_id && form.value.role_id !== targetUser.value?.role_id) {
      payload.role_id = form.value.role_id;
    }

    await userService.update(userId, payload);

    if (targetUser.value) {
      targetUser.value = {
        ...targetUser.value,
        first_name:    payload.first_name,
        last_name:     payload.last_name,
        email:         payload.email,
        phone:         payload.phone ?? null,
        date_of_birth: payload.date_of_birth ?? null,
        role_id:       form.value.role_id || null,
        role:          roles.value.find((r) => r.id === form.value.role_id) ?? null,
      };
    }
    saved.value = true;
    setTimeout(() => { saved.value = false; }, 3000);
  } catch (e: unknown) {
    saveError.value = e instanceof ApiException ? e.message : 'Failed to save.';
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div v-if="loading" class="state-msg">Loading…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <p class="page-label">Users</p>
      <div class="profile-header">
        <div>
          <h1 class="page-title">{{ targetUser?.first_name }} {{ targetUser?.last_name }}</h1>
          <p class="profile-email">{{ targetUser?.email }}</p>
        </div>
        <span v-if="targetUser?.role" class="role-badge">{{ targetUser.role.title }}</span>
      </div>

      <div class="card">
        <h3 class="card-title">Basic Info</h3>
        <form @submit.prevent="save">
          <div class="field-grid">
            <div class="field">
              <label class="field-label" for="first_name">First Name</label>
              <input id="first_name" v-model="form.first_name" class="field-input" type="text" required />
            </div>
            <div class="field">
              <label class="field-label" for="last_name">Last Name</label>
              <input id="last_name" v-model="form.last_name" class="field-input" type="text" required />
            </div>
            <div class="field">
              <label class="field-label" for="email">Email</label>
              <input id="email" v-model="form.email" class="field-input" type="email" required />
            </div>
            <div class="field">
              <label class="field-label" for="phone">Phone</label>
              <input id="phone" v-model="form.phone" class="field-input" type="tel" placeholder="0800 000 0000" />
            </div>
            <div class="field">
              <label class="field-label" for="dob">Date of Birth</label>
              <input id="dob" v-model="form.date_of_birth" class="field-input" type="date" />
            </div>
            <div class="field">
              <label class="field-label" for="role">Role</label>
              <select id="role" v-model="form.role_id" class="field-input">
                <option value="">— None —</option>
                <option v-for="r in roles" :key="r.id" :value="r.id">{{ r.title }}</option>
              </select>
            </div>
          </div>

          <div v-if="saveError" class="form-error">{{ saveError }}</div>
          <div v-if="saved" class="form-success">Saved.</div>

          <button type="submit" class="btn-primary" :disabled="saving">
            {{ saving ? 'Saving…' : 'Save' }}
          </button>
        </form>
      </div>
    </template>
  </div>
</template>

<style scoped>
.page { padding: 48px var(--spacing-lg); }

.page-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-sm);
}

.page-title {
  font-family: var(--font-display);
  font-size: 2.25rem;
  font-weight: 500;
  letter-spacing: -0.02em;
}

.profile-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.profile-email {
  font-size: 0.9rem;
  color: var(--color-secondary);
}

.role-badge {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 4px 10px;
  color: var(--color-secondary);
}

.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
  max-width: 720px;
}

.card-title {
  font-family: var(--font-display);
  font-size: 1.15rem;
  font-weight: 500;
  margin-bottom: var(--spacing-md);
}

.field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

@media (max-width: 620px) {
  .field-grid { grid-template-columns: 1fr; }
}

.field-label {
  display: block;
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  margin-bottom: 6px;
}

.field-input {
  width: 100%;
  font-family: var(--font-body);
  font-size: 0.95rem;
  color: var(--color-primary);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 10px 12px;
}

.field-input:focus {
  outline: 2px solid var(--color-tertiary);
  outline-offset: -1px;
}

.form-error {
  color: var(--color-tertiary);
  font-size: 0.875rem;
  margin-bottom: var(--spacing-sm);
}

.form-success {
  font-size: 0.875rem;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-sm);
}

.btn-primary {
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  background-color: var(--color-tertiary);
  color: var(--color-on-primary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 12px 20px;
  cursor: pointer;
}

.btn-primary:disabled { opacity: 0.45; cursor: not-allowed; }

.state-msg {
  font-family: var(--font-label);
  font-size: 0.8rem;
  color: var(--color-secondary);
  padding: var(--spacing-lg) 0;
}

.state-msg--error { color: var(--color-tertiary); }
</style>
