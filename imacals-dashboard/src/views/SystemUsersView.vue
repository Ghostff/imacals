<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import {
  systemUserService,
  type EligibleRole,
  type DomainSystemUser,
  type UpsertSystemUserPayload,
} from '@/services/systemUser';
import { domainService, type Domain } from '@/services/domain';
import { userService, type User } from '@/services/user';
import { ApiException } from '@/services/api';

const assignments: Ref<DomainSystemUser[]> = ref([]);
const domains: Ref<Domain[]>               = ref([]);
const users: Ref<User[]>                   = ref([]);
const eligibleRoles: Ref<EligibleRole[]>   = ref([]);
const loading: Ref<boolean>                = ref(true);
const error: Ref<string | null>            = ref(null);

// ── Set modal ─────────────────────────────────────────────────────────────
const showModal: Ref<boolean>         = ref(false);
const editingId: Ref<string | null>   = ref(null);
const submitting: Ref<boolean>        = ref(false);
const submitError: Ref<string | null> = ref(null);
const userSearch: Ref<string>         = ref('');

const form: Ref<UpsertSystemUserPayload> = ref({ domain_id: '', user_id: '', user_role_id: '' });

const filteredUsers: ComputedRef<User[]> = computed(() => {
  const q = userSearch.value.trim().toLowerCase();
  if (!q) return users.value;
  return users.value.filter(
    (u) =>
      `${u.first_name} ${u.last_name}`.toLowerCase().includes(q) ||
      u.email.toLowerCase().includes(q),
  );
});

function openSet(existing?: DomainSystemUser): void {
  editingId.value   = existing?.id ?? null;
  submitError.value = null;
  userSearch.value  = '';
  form.value = {
    domain_id:    existing?.domain_id    ?? '',
    user_id:      existing?.user_id      ?? '',
    user_role_id: existing?.user_role_id ?? '',
  };
  showModal.value = true;
}

function closeModal(): void {
  if (submitting.value) return;
  showModal.value = false;
}

async function submitForm(): Promise<void> {
  submitError.value = null;
  submitting.value  = true;
  try {
    const saved = await systemUserService.upsert({ ...form.value });
    assignments.value = [
      ...assignments.value.filter(
        (a) => !(a.domain_id === saved.domain_id && a.user_role_id === saved.user_role_id),
      ),
      saved,
    ].sort((a, b) => a.domain_name.localeCompare(b.domain_name) || a.role_title.localeCompare(b.role_title));
    showModal.value = false;
  } catch (e: unknown) {
    submitError.value = e instanceof ApiException ? e.message : 'Failed to save assignment.';
  } finally {
    submitting.value = false;
  }
}

// ── Delete ────────────────────────────────────────────────────────────────
const toDelete: Ref<DomainSystemUser | null> = ref(null);
const deleting: Ref<boolean>                 = ref(false);
const deleteError: Ref<string | null>        = ref(null);

function openDelete(a: DomainSystemUser): void {
  toDelete.value    = a;
  deleteError.value = null;
}

function closeDelete(): void {
  if (deleting.value) return;
  toDelete.value = null;
}

async function confirmDelete(): Promise<void> {
  if (!toDelete.value) return;
  deleteError.value = null;
  deleting.value    = true;
  try {
    await systemUserService.delete(toDelete.value.id);
    assignments.value = assignments.value.filter((a) => a.id !== toDelete.value!.id);
    toDelete.value    = null;
  } catch (e: unknown) {
    deleteError.value = e instanceof ApiException ? e.message : 'Failed to remove assignment.';
  } finally {
    deleting.value = false;
  }
}

function userName(a: DomainSystemUser): string {
  return `${a.user_first_name} ${a.user_last_name}`;
}

// ── Load ──────────────────────────────────────────────────────────────────
onMounted(async () => {
  try {
    [assignments.value, domains.value, users.value, eligibleRoles.value] = await Promise.all([
      systemUserService.index(),
      domainService.index(),
      userService.index(),
      systemUserService.eligibleRoles(),
    ]);
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load system users.';
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="page">
    <p class="page-label">Users</p>
    <h1 class="page-title">System Users</h1>

    <p class="page-desc">
      System users are the broker, realtor, HML, and insurance contacts the platform assigns to
      offers and contracts, scoped per domain so different agents can serve different regions.
    </p>

    <div v-if="loading" class="state-msg">Loading…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <div class="toolbar">
        <p class="result-count">{{ assignments.length }} assignment{{ assignments.length !== 1 ? 's' : '' }}</p>
        <button class="btn-add" type="button" @click="openSet()">+ Set System User</button>
      </div>

      <div class="table-card">
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th>Domain</th>
                <th>Role</th>
                <th>User</th>
                <th>Email</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="assignments.length === 0">
                <td colspan="5" class="empty-cell">No system users configured.</td>
              </tr>
              <tr v-for="a in assignments" :key="a.id">
                <td class="cell-name">{{ a.domain_name }}</td>
                <td>
                  <span class="badge">{{ a.role_title }}</span>
                </td>
                <td class="cell-name">{{ userName(a) }}</td>
                <td class="cell-muted">{{ a.user_email }}</td>
                <td class="cell-actions">
                  <div class="row-actions">
                    <button class="btn-row-action" type="button" @click="openSet(a)">Change</button>
                    <button class="btn-row-delete" type="button" @click="openDelete(a)">Remove</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>

  <!-- Set / Change modal -->
  <Teleport to="body">
    <div v-if="showModal" class="modal-backdrop" @click.self="closeModal">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="su-modal-title">
        <div class="modal-header">
          <h2 id="su-modal-title" class="modal-title">
            {{ editingId ? 'Change System User' : 'Set System User' }}
          </h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeModal">✕</button>
        </div>
        <form class="modal-body" @submit.prevent="submitForm">
          <div class="field">
            <label class="field-label">Domain</label>
            <select v-model="form.domain_id" class="field-input" required>
              <option value="" disabled>Select a domain…</option>
              <option v-for="d in domains" :key="d.id" :value="d.id">{{ d.name }}</option>
            </select>
          </div>
          <div class="field">
            <label class="field-label">Role</label>
            <select v-model="form.user_role_id" class="field-input" required>
              <option value="" disabled>Select a role…</option>
              <option v-for="r in eligibleRoles" :key="r.id" :value="r.id">{{ r.title }}</option>
            </select>
          </div>
          <div class="field">
            <label class="field-label">User</label>
            <input
              v-model="userSearch"
              class="field-input field-input--search"
              type="search"
              placeholder="Search by name or email…"
            />
            <div class="user-list">
              <div v-if="filteredUsers.length === 0" class="user-list-empty">No users found.</div>
              <label
                v-for="u in filteredUsers"
                :key="u.id"
                class="user-option"
                :class="{ 'user-option--selected': form.user_id === u.id }"
              >
                <input v-model="form.user_id" type="radio" :value="u.id" class="user-radio" />
                <span class="user-name">{{ u.first_name }} {{ u.last_name }}</span>
                <span class="user-email">{{ u.email }}</span>
              </label>
            </div>
          </div>
          <div v-if="submitError" class="modal-error">{{ submitError }}</div>
          <div class="modal-footer">
            <button type="button" class="btn-cancel" :disabled="submitting" @click="closeModal">Cancel</button>
            <button
              type="submit"
              class="btn-submit"
              :disabled="submitting || !form.domain_id || !form.user_role_id || !form.user_id"
            >
              {{ submitting ? 'Saving…' : (editingId ? 'Save Changes' : 'Set User') }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>

  <!-- Remove confirmation -->
  <Teleport to="body">
    <div v-if="toDelete" class="modal-backdrop" @click.self="closeDelete">
      <div class="modal modal--sm" role="dialog" aria-modal="true" aria-labelledby="del-su-title">
        <div class="modal-header">
          <h2 id="del-su-title" class="modal-title">Remove System User</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeDelete">✕</button>
        </div>
        <div class="modal-body">
          <p class="confirm-msg">
            Remove <strong>{{ userName(toDelete) }}</strong> as
            <strong>{{ toDelete.role_title }}</strong> for
            <strong>{{ toDelete.domain_name }}</strong>?
          </p>
          <div v-if="deleteError" class="modal-error">{{ deleteError }}</div>
          <div class="modal-footer">
            <button type="button" class="btn-cancel" :disabled="deleting" @click="closeDelete">Cancel</button>
            <button type="button" class="btn-delete" :disabled="deleting" @click="confirmDelete">
              {{ deleting ? 'Removing…' : 'Remove' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
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
  font-size: 2.5rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
  margin-bottom: var(--spacing-sm);
}

.page-desc {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-lg);
  max-width: 560px;
  line-height: 1.5;
}

/* ── Toolbar ── */
.toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.result-count {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

/* ── Table ── */
.table-card {
  background: var(--color-surface);
  border: 1px solid #E5E2DE;
  border-radius: var(--rounded-lg, 8px);
  overflow: hidden;
}

.table-wrap { overflow-x: auto; }

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--font-body);
  font-size: 0.875rem;
}

.data-table th {
  text-align: left;
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  padding: var(--spacing-sm) var(--spacing-md);
  border-bottom: 1px solid #E5E2DE;
  white-space: nowrap;
}

.data-table td {
  padding: var(--spacing-sm) var(--spacing-md);
  color: var(--color-primary);
  border-bottom: 1px solid #E5E2DE;
  vertical-align: middle;
}

.data-table tbody tr:last-child td { border-bottom: none; }
.data-table tbody tr:hover td { background: #F0EDE9; }

.cell-name    { font-weight: 500; white-space: nowrap; }
.cell-muted   { color: var(--color-secondary); }
.cell-actions { width: 1px; white-space: nowrap; }
.row-actions  { display: flex; gap: 12px; align-items: center; }

.empty-cell {
  text-align: center;
  color: var(--color-secondary);
  padding: var(--spacing-lg) !important;
}

/* ── Badge ── */
.badge {
  display: inline-block;
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.05em;
  text-transform: uppercase;
  padding: 2px 8px;
  border-radius: var(--rounded-sm);
  background: #E5E2DE;
  color: var(--color-secondary);
}

/* ── Row action buttons ── */
.btn-row-action,
.btn-row-delete {
  font-family: var(--font-body);
  font-size: 0.8rem;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.15s;
}

.btn-row-action { color: var(--color-primary); }
.btn-row-delete { color: var(--color-tertiary); }

.data-table tbody tr:hover .btn-row-action,
.data-table tbody tr:hover .btn-row-delete { opacity: 0.7; }

.btn-row-action:hover,
.btn-row-delete:hover { opacity: 1 !important; }

/* ── Add button ── */
.btn-add {
  margin-left: auto;
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: #fff;
  background: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 8px 18px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-add:hover { opacity: 0.88; }

/* ── State ── */
.state-msg        { font-family: var(--font-body); color: var(--color-secondary); }
.state-msg--error { color: var(--color-tertiary); }

/* ── Modal ── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.modal {
  background: var(--color-surface);
  border-radius: var(--rounded-lg, 12px);
  width: 100%;
  max-width: 460px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.18);
}

.modal--sm { max-width: 380px; }

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

.field { display: flex; flex-direction: column; gap: 5px; }

.field-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.field-input {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  background: var(--color-surface);
  border: 1px solid #E0DED9;
  border-radius: var(--rounded-md);
  padding: 8px 10px;
  outline: none;
  transition: border-color 0.15s;
  width: 100%;
  box-sizing: border-box;
}

.field-input:focus { border-color: var(--color-primary); }
.field-input--search::-webkit-search-cancel-button { display: none; }

/* ── User picker ── */
.user-list {
  border: 1px solid #E0DED9;
  border-radius: var(--rounded-md);
  max-height: 180px;
  overflow-y: auto;
}

.user-list-empty {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
  padding: 10px 12px;
}

.user-option {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 0.1s;
  border-bottom: 1px solid #F0EDE9;
}

.user-option:last-child { border-bottom: none; }
.user-option:hover         { background: #F7F5F2; }
.user-option--selected     { background: #F0EDE9; }

.user-radio { flex-shrink: 0; accent-color: var(--color-tertiary); }

.user-name {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  font-weight: 500;
}

.user-email {
  font-family: var(--font-body);
  font-size: 0.8rem;
  color: var(--color-secondary);
  margin-left: auto;
}

.confirm-msg {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--color-primary);
  line-height: 1.5;
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
  border: 1px solid #E0DED9;
  border-radius: var(--rounded-md);
  padding: 8px 16px;
  cursor: pointer;
  transition: border-color 0.15s, color 0.15s;
}

.btn-cancel:hover { border-color: var(--color-primary); color: var(--color-primary); }
.btn-cancel:disabled { opacity: 0.55; cursor: not-allowed; }

.btn-submit {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: #fff;
  background: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 8px 18px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-submit:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-submit:not(:disabled):hover { opacity: 0.88; }

.btn-delete {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: #fff;
  background: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 8px 18px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-delete:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-delete:not(:disabled):hover { opacity: 0.88; }
</style>
