<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import { useRouter } from 'vue-router';
import { userService, type User, type UserOrganization, type CreateUserPayload } from '@/services/user';
import { organizationUserRoleService, type OrganizationUserRole } from '@/services/organization_user_role';
import { organizationService, type Org } from '@/services/organization';
import { ApiException } from '@/services/api';

const router = useRouter();
const users: Ref<User[]>                    = ref([]);
const userRoles: Ref<OrganizationUserRole[]> = ref([]);
const orgs: Ref<Org[]>                      = ref([]);
const loading: Ref<boolean>                 = ref(true);
const error: Ref<string | null>             = ref(null);

// ── Add User modal ────────────────────────────────────────────────────────
const showModal: Ref<boolean>       = ref(false);
const submitting: Ref<boolean>      = ref(false);
const submitError: Ref<string|null> = ref(null);

const IMACALS_SLUG = 'imacals';

const form: Ref<CreateUserPayload & { password: string }> = ref({
  first_name: '',
  last_name: '',
  email: '',
  password: '',
  phone: '',
  organization_ids: [],
  user_role_id: '',
});

function openModal(): void {
  const imacalsOrg = orgs.value.find((o) => o.slug === IMACALS_SLUG);
  form.value = {
    first_name: '',
    last_name: '',
    email: '',
    password: '',
    phone: '',
    organization_ids: imacalsOrg ? [imacalsOrg.id] : [],
    user_role_id: userRoles.value[0]?.id ?? '',
  };
  submitError.value = null;
  showModal.value   = true;
}

function closeModal(): void {
  showModal.value = false;
}

async function submitUser(): Promise<void> {
  submitError.value = null;
  submitting.value  = true;
  try {
    const payload: CreateUserPayload = {
      first_name:       form.value.first_name.trim(),
      last_name:        form.value.last_name.trim(),
      email:            form.value.email.trim(),
      phone:            form.value.phone?.trim() || undefined,
      organization_ids: form.value.organization_ids,
      user_role_id:     form.value.user_role_id || undefined,
    };
    if (form.value.password.trim()) payload.password = form.value.password.trim();

    const result = await userService.create(payload);
    // Prepend the new user to the list
    users.value = [{ ...result.user, organizations: [], role: null, user_role: null }, ...users.value];
    closeModal();
  } catch (e: unknown) {
    submitError.value = e instanceof ApiException ? e.message : 'Failed to create user.';
  } finally {
    submitting.value = false;
  }
}

const search: Ref<string>        = ref('');
const roleFilter: Ref<string>    = ref('all'); // role id or 'all'
const orgFilter: Ref<string>     = ref('all'); // org  id or 'all'

const availableOrgs: ComputedRef<UserOrganization[]> = computed(() => {
  const map = new Map<string, UserOrganization>();
  for (const u of users.value) {
    for (const o of u.organizations) map.set(o.id, o);
  }
  return [...map.values()].sort((a, b) => a.name.localeCompare(b.name));
});

const filteredUsers: ComputedRef<User[]> = computed(() => {
  const q = search.value.trim().toLowerCase();
  return users.value.filter((u) => {
    if (q) {
      const fullName = `${u.first_name} ${u.last_name}`.toLowerCase();
      if (!fullName.includes(q) && !u.email.toLowerCase().includes(q) && !(u.phone ?? '').toLowerCase().includes(q)) {
        return false;
      }
    }
    if (roleFilter.value !== 'all' && u.user_role?.id !== roleFilter.value) return false;
    if (orgFilter.value  !== 'all' && !u.organizations.some((o) => o.id === orgFilter.value)) return false;
    return true;
  });
});

const activeFilterCount: ComputedRef<number> = computed(() =>
  (search.value.trim() ? 1 : 0) +
  (roleFilter.value !== 'all' ? 1 : 0) +
  (orgFilter.value  !== 'all' ? 1 : 0),
);

function clearFilters(): void {
  search.value     = '';
  roleFilter.value = 'all';
  orgFilter.value  = 'all';
}

// ── Delete confirmation ───────────────────────────────────────────────────
const userToDelete: Ref<User | null>    = ref(null);
const deleting: Ref<boolean>            = ref(false);
const deleteError: Ref<string | null>   = ref(null);

function openDeleteModal(u: User): void {
  userToDelete.value = u;
  deleteError.value  = null;
}

function closeDeleteModal(): void {
  if (deleting.value) return;
  userToDelete.value = null;
}

async function confirmDelete(): Promise<void> {
  if (!userToDelete.value) return;
  deleteError.value = null;
  deleting.value    = true;
  try {
    await userService.delete(userToDelete.value.id);
    users.value        = users.value.filter((u) => u.id !== userToDelete.value!.id);
    userToDelete.value = null;
  } catch (e: unknown) {
    deleteError.value = e instanceof ApiException ? e.message : 'Failed to delete user.';
  } finally {
    deleting.value = false;
  }
}

function formatDate(iso: string | null): string {
  if (!iso) return '—';
  return new Date(iso).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' });
}

onMounted(async () => {
  try {
    const [usersResult, userRolesResult, orgsResult] = await Promise.all([
      userService.index(),
      organizationUserRoleService.index().catch(() => []),
      organizationService.index().catch(() => []),
    ]);
    users.value     = usersResult;
    userRoles.value = userRolesResult;
    orgs.value      = orgsResult;
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load users.';
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="page">
    <p class="page-label">Staff & Customers</p>
    <h1 class="page-title">All Users</h1>

    <div v-if="loading" class="state-msg">Loading…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <div class="toolbar">
        <div class="search-wrap">
          <svg class="search-icon" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
            <circle cx="6.5" cy="6.5" r="4.5" stroke="currentColor" stroke-width="1.25"/>
            <path d="M10.5 10.5L14 14" stroke="currentColor" stroke-width="1.25" stroke-linecap="round"/>
          </svg>
          <input
            v-model="search"
            class="search-input"
            type="search"
            placeholder="Search by name or email…"
          />
        </div>

        <div class="filters">
          <select v-model="roleFilter" class="filter-select">
            <option value="all">All roles</option>
            <option v-for="r in userRoles" :key="r.id" :value="r.id">{{ r.title }}</option>
          </select>

          <select v-model="orgFilter" class="filter-select">
            <option value="all">All organizations</option>
            <option v-for="o in availableOrgs" :key="o.id" :value="o.id">{{ o.name }}</option>
          </select>

          <button
            v-if="activeFilterCount > 0"
            class="clear-btn"
            type="button"
            @click="clearFilters"
          >
            Clear <span class="clear-count">{{ activeFilterCount }}</span>
          </button>
        </div>

        <button class="btn-add" type="button" @click="openModal">+ Add User</button>
      </div>

      <div class="table-card card">
        <div class="table-wrap">
          <table class="users-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Email</th>
                <th>Phone</th>
                <th>Role</th>
                <th>Organizations</th>
                <th>Last Login</th>
                <th>Created</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="filteredUsers.length === 0">
                <td colspan="8" class="empty-cell">
                  {{ activeFilterCount > 0 ? 'No users match the current filters.' : 'No users found.' }}
                </td>
              </tr>
              <tr v-for="u in filteredUsers" :key="u.id">
                <td class="cell-name">
                  <button class="name-link" type="button" @click="router.push(`/users/${u.id}`)">
                    {{ u.first_name }} {{ u.last_name }}
                  </button>
                </td>
                <td>{{ u.email }}</td>
                <td>{{ u.phone ?? '—' }}</td>
                <td>
                  <span v-if="u.user_role" class="badge">{{ u.user_role.title }}</span>
                  <span v-else class="cell-muted">—</span>
                </td>
                <td>
                  <span
                    v-for="o in u.organizations"
                    :key="o.id"
                    class="org-chip"
                  >{{ o.name }}</span>
                  <span v-if="u.organizations.length === 0" class="cell-muted">—</span>
                </td>
                <td>{{ formatDate(u.last_logged_in_at) }}</td>
                <td>{{ formatDate(u.created_at) }}</td>
                <td class="cell-actions">
                  <button class="btn-row-delete" type="button" @click.stop="openDeleteModal(u)">Delete</button>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <p class="result-count">
        Showing {{ filteredUsers.length }} of {{ users.length }} users
      </p>
    </template>
  </div>

  <!-- Delete confirmation modal -->
  <Teleport to="body">
    <div v-if="userToDelete" class="modal-backdrop" @click.self="closeDeleteModal">
      <div class="modal modal--sm" role="dialog" aria-modal="true" aria-labelledby="delete-modal-title">
        <div class="modal-header">
          <h2 id="delete-modal-title" class="modal-title">Delete User</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeDeleteModal">✕</button>
        </div>
        <div class="modal-body modal-body--confirm">
          <p class="confirm-msg">
            Are you sure you want to delete
            <strong>{{ userToDelete.first_name }} {{ userToDelete.last_name }}</strong>?
            This cannot be undone.
          </p>
          <div v-if="deleteError" class="modal-error">{{ deleteError }}</div>
          <div class="modal-footer">
            <button type="button" class="btn-cancel" :disabled="deleting" @click="closeDeleteModal">Cancel</button>
            <button type="button" class="btn-delete" :disabled="deleting" @click="confirmDelete">
              {{ deleting ? 'Deleting…' : 'Delete' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Add User modal -->
  <Teleport to="body">
    <div v-if="showModal" class="modal-backdrop" @click.self="closeModal">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="modal-title">
        <div class="modal-header">
          <h2 id="modal-title" class="modal-title">Add User</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeModal">✕</button>
        </div>

        <form class="modal-body" @submit.prevent="submitUser">
          <div class="field-row">
            <div class="field">
              <label class="field-label">First Name</label>
              <input v-model="form.first_name" class="field-input" type="text" required placeholder="Jane" />
            </div>
            <div class="field">
              <label class="field-label">Last Name</label>
              <input v-model="form.last_name" class="field-input" type="text" required placeholder="Doe" />
            </div>
          </div>

          <div class="field">
            <label class="field-label">Email</label>
            <input v-model="form.email" class="field-input" type="email" required placeholder="jane@example.com" />
          </div>

          <div class="field">
            <label class="field-label">Phone Number <span class="field-optional">(optional)</span></label>
            <input v-model="form.phone" class="field-input" type="tel" placeholder="0800 000 0000" />
          </div>

          <div class="field">
            <label class="field-label">Password <span class="field-optional">(optional)</span></label>
            <input v-model="form.password" class="field-input" type="password" placeholder="Leave blank to set later" />
          </div>

          <div class="field">
            <label class="field-label">Organization</label>
            <select v-model="form.organization_ids[0]" class="field-input" required>
              <option v-for="o in orgs" :key="o.id" :value="o.id">{{ o.name }}</option>
            </select>
          </div>

          <div class="field">
            <label class="field-label">Job Title</label>
            <select v-model="form.user_role_id" class="field-input" required>
              <option v-for="r in userRoles" :key="r.id" :value="r.id">{{ r.title }}</option>
            </select>
          </div>

          <div v-if="submitError" class="modal-error">{{ submitError }}</div>

          <div class="modal-footer">
            <button type="button" class="btn-cancel" @click="closeModal">Cancel</button>
            <button type="submit" class="btn-submit" :disabled="submitting">
              {{ submitting ? 'Creating…' : 'Create User' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.page {
  padding: 48px var(--spacing-lg);
}

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
  font-size: 2.5rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
  margin-bottom: var(--spacing-lg);
}

/* ── Toolbar ── */
.toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
  margin-bottom: var(--spacing-md);
}

.search-wrap {
  position: relative;
  flex: 1;
  min-width: 200px;
  max-width: 320px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  width: 14px;
  height: 14px;
  color: var(--color-secondary);
  pointer-events: none;
}

.search-input {
  width: 100%;
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 7px 10px 7px 32px;
  outline: none;
  transition: border-color 0.15s;
}

.search-input:focus {
  border-color: var(--color-primary);
}

.search-input::placeholder {
  color: var(--color-secondary);
  opacity: 0.7;
}

.search-input::-webkit-search-cancel-button { display: none; }

.filters {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.filter-select {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 7px 28px 7px 10px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' fill='none'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%236C7278' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  outline: none;
  cursor: pointer;
  transition: border-color 0.15s;
}

.filter-select:focus {
  border-color: var(--color-primary);
}

.clear-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 7px 12px;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s;
}

.clear-btn:hover {
  color: var(--color-tertiary);
  border-color: var(--color-tertiary);
}

.clear-count {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background-color: var(--color-tertiary);
  color: var(--color-on-primary);
  font-size: 0.65rem;
  font-family: var(--font-label);
  font-weight: 500;
}

/* ── Table ── */
.table-card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg, 8px);
  padding: 0;
  overflow: hidden;
}

.table-wrap {
  overflow-x: auto;
}

.users-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--font-body);
  font-size: 0.875rem;
}

.users-table th {
  text-align: left;
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  white-space: nowrap;
}

.users-table td {
  padding: var(--spacing-sm) var(--spacing-md);
  color: var(--color-primary);
  border-bottom: 1px solid var(--color-divider);
  vertical-align: middle;
}

.users-table tbody tr:last-child td {
  border-bottom: none;
}

.users-table tbody tr:hover td {
  background-color: var(--color-neutral);
}

.cell-name {
  white-space: nowrap;
  font-weight: 500;
}

.name-link {
  background: none;
  border: none;
  padding: 0;
  font-family: inherit;
  font-size: inherit;
  font-weight: 500;
  color: var(--color-primary);
  cursor: pointer;
  text-decoration: underline;
  text-underline-offset: 3px;
  text-decoration-color: transparent;
  transition: text-decoration-color 0.15s;
}

.name-link:hover {
  text-decoration-color: var(--color-primary);
}

.cell-muted {
  color: var(--color-secondary);
}

.empty-cell {
  text-align: center;
  color: var(--color-secondary);
  padding: var(--spacing-lg) !important;
}

/* ── Badges ── */
.badge {
  display: inline-block;
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 2px 8px;
  border-radius: var(--rounded-sm);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
}

.org-chip {
  display: inline-block;
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.04em;
  padding: 2px 7px;
  border-radius: var(--rounded-sm);
  border: 1px solid var(--color-border);
  color: var(--color-secondary);
  margin-right: 4px;
  white-space: nowrap;
}

/* ── Footer ── */
.result-count {
  margin-top: var(--spacing-md);
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

/* ── Loading / error ── */
.state-msg {
  font-family: var(--font-body);
  color: var(--color-secondary);
  margin-top: var(--spacing-lg);
}

.state-msg--error {
  color: var(--color-tertiary);
}

.btn-add {
  margin-left: auto;
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-on-primary);
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 8px 18px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-add:hover { opacity: 0.88; }

/* ── Modal ── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg, 12px);
  width: 100%;
  max-width: 480px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.25);
}

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 20px 24px 0;
}

.modal-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
  color: var(--color-primary);
}

.modal-close {
  background: none;
  border: none;
  font-size: 1rem;
  color: var(--color-secondary);
  cursor: pointer;
  padding: 4px;
  line-height: 1;
}

.modal-close:hover { color: var(--color-primary); }

.modal-body {
  padding: 20px 24px 24px;
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
}

.field-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.field-optional {
  font-size: 0.65rem;
  text-transform: none;
  letter-spacing: 0;
}

.field-input {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  background: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 8px 10px;
  outline: none;
  transition: border-color 0.15s;
  width: 100%;
  box-sizing: border-box;
}

.field-input:focus { border-color: var(--color-primary); }

select.field-input {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' fill='none'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%236C7278' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 28px;
  cursor: pointer;
}

.modal-error {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-tertiary);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  padding-top: 4px;
}

.btn-cancel {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 8px 16px;
  cursor: pointer;
  transition: border-color 0.15s;
}

.btn-cancel:hover { border-color: var(--color-primary); color: var(--color-primary); }

/* ── Delete row button ── */
.cell-actions { width: 1px; white-space: nowrap; }

.btn-row-delete {
  font-family: var(--font-body);
  font-size: 0.8rem;
  color: var(--color-tertiary);
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s;
}

.users-table tbody tr:hover .btn-row-delete { opacity: 0.7; }
.btn-row-delete:hover { opacity: 1 !important; }

/* ── Confirm modal ── */
.modal--sm { max-width: 400px; }

.modal-body--confirm { gap: 16px; }

.confirm-msg {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--color-primary);
  line-height: 1.5;
}

.btn-delete {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-on-primary);
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 8px 18px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-delete:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-delete:not(:disabled):hover { opacity: 0.88; }

.btn-submit {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--color-on-primary);
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 8px 18px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-submit:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-submit:not(:disabled):hover { opacity: 0.88; }
</style>
