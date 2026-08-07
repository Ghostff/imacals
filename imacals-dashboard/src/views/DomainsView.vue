<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import { domainService, type Domain, type CreateDomainPayload } from '@/services/domain';
import { geo, type GeoCountry, type GeoState, type GeoCity } from '@/services/geo';
import { ApiException } from '@/services/api';

const domains: Ref<Domain[]>                    = ref([]);
const countries: Ref<GeoCountry[]>              = ref([]);
const countryMap: Ref<Record<string, string>>   = ref({});
const loading: Ref<boolean>                     = ref(true);
const error: Ref<string | null>                 = ref(null);

// ── Add / Edit modal ──────────────────────────────────────────────────────
type ModalMode = 'create' | 'edit';
const showModal: Ref<boolean>         = ref(false);
const modalMode: Ref<ModalMode>       = ref('create');
const editingId: Ref<string | null>   = ref(null);
const submitting: Ref<boolean>        = ref(false);
const submitError: Ref<string | null> = ref(null);

const form: Ref<{ name: string; slug: string; country_id: string; state_id: string; city_id: string }> = ref({
  name: '', slug: '', country_id: '', state_id: '', city_id: '',
});

// Cascading geo dropdowns inside the modal
const formStates: Ref<GeoState[]>     = ref([]);
const formCities: Ref<GeoCity[]>      = ref([]);
const loadingStates: Ref<boolean>     = ref(false);
const loadingCities: Ref<boolean>     = ref(false);

// Auto-generate slug from name while the user hasn't manually edited it
const slugTouched: Ref<boolean> = ref(false);

function slugify(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
}

function onNameInput(): void {
  if (!slugTouched.value) form.value.slug = slugify(form.value.name);
}

async function onCountryChange(): Promise<void> {
  form.value.state_id = '';
  form.value.city_id  = '';
  formStates.value    = [];
  formCities.value    = [];
  if (!form.value.country_id) return;
  loadingStates.value = true;
  try {
    formStates.value = await geo.states(form.value.country_id);
  } finally {
    loadingStates.value = false;
  }
}

async function onStateChange(): Promise<void> {
  form.value.city_id = '';
  formCities.value   = [];
  if (!form.value.state_id) return;
  loadingCities.value = true;
  try {
    formCities.value = await geo.cities(form.value.state_id);
  } finally {
    loadingCities.value = false;
  }
}

function openCreate(): void {
  modalMode.value   = 'create';
  editingId.value   = null;
  slugTouched.value = false;
  form.value        = { name: '', slug: '', country_id: '', state_id: '', city_id: '' };
  formStates.value  = [];
  formCities.value  = [];
  submitError.value = null;
  showModal.value   = true;
}

async function openEdit(domain: Domain): Promise<void> {
  modalMode.value   = 'edit';
  editingId.value   = domain.id;
  slugTouched.value = true;
  form.value        = {
    name:       domain.name,
    slug:       domain.slug,
    country_id: domain.country_id,
    state_id:   domain.state_id ?? '',
    city_id:    domain.city_id  ?? '',
  };
  formStates.value  = [];
  formCities.value  = [];
  submitError.value = null;
  showModal.value   = true;

  if (domain.country_id) {
    loadingStates.value = true;
    try {
      formStates.value = await geo.states(domain.country_id);
    } finally {
      loadingStates.value = false;
    }
  }

  if (domain.state_id) {
    loadingCities.value = true;
    try {
      formCities.value = await geo.cities(domain.state_id);
    } finally {
      loadingCities.value = false;
    }
  }
}

function closeModal(): void {
  if (submitting.value) return;
  showModal.value = false;
}

async function submitForm(): Promise<void> {
  submitError.value = null;
  submitting.value  = true;
  try {
    const payload: CreateDomainPayload = {
      name:       form.value.name.trim(),
      slug:       form.value.slug.trim(),
      country_id: form.value.country_id,
      state_id:   form.value.state_id || null,
      city_id:    form.value.city_id  || null,
    };
    if (modalMode.value === 'create') {
      const created = await domainService.create(payload);
      domains.value = [...domains.value, created].sort((a, b) => a.name.localeCompare(b.name));
    } else {
      const updated = await domainService.update(editingId.value!, payload);
      domains.value = domains.value.map((d) => d.id === updated.id ? updated : d);
    }
    showModal.value = false;
  } catch (e: unknown) {
    submitError.value = e instanceof ApiException ? e.message : 'Failed to save domain.';
  } finally {
    submitting.value = false;
  }
}

// ── Delete ────────────────────────────────────────────────────────────────
const domainToDelete: Ref<Domain | null> = ref(null);
const deleting: Ref<boolean>             = ref(false);
const deleteError: Ref<string | null>    = ref(null);

function openDelete(domain: Domain): void {
  domainToDelete.value = domain;
  deleteError.value    = null;
}

function closeDelete(): void {
  if (deleting.value) return;
  domainToDelete.value = null;
}

async function confirmDelete(): Promise<void> {
  if (!domainToDelete.value) return;
  deleteError.value = null;
  deleting.value    = true;
  try {
    await domainService.delete(domainToDelete.value.id);
    domains.value        = domains.value.filter((d) => d.id !== domainToDelete.value!.id);
    domainToDelete.value = null;
  } catch (e: unknown) {
    deleteError.value = e instanceof ApiException ? e.message : 'Failed to delete domain.';
  } finally {
    deleting.value = false;
  }
}

// ── Search ────────────────────────────────────────────────────────────────
const search: Ref<string> = ref('');
const filtered: ComputedRef<Domain[]> = computed(() => {
  const q = search.value.trim().toLowerCase();
  if (!q) return domains.value;
  return domains.value.filter((d) => d.name.toLowerCase().includes(q) || d.slug.includes(q));
});

// ── Mount ─────────────────────────────────────────────────────────────────
onMounted(async () => {
  try {
    const [domainList, countryList] = await Promise.all([
      domainService.index(),
      geo.countries(),
    ]);
    domains.value   = domainList;
    countries.value = countryList;
    countryMap.value = Object.fromEntries(countryList.map((c) => [c.id, c.name]));
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load domains.';
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="page">
    <p class="page-label">Models</p>
    <h1 class="page-title">Domains</h1>

    <div v-if="loading" class="state-msg">Loading…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <div class="toolbar">
        <input v-model="search" class="search-input" type="search" placeholder="Search domains…" />
        <p class="result-count">{{ filtered.length }} of {{ domains.length }}</p>
        <button class="btn-add" type="button" @click="openCreate">+ Add Domain</button>
      </div>

      <div class="table-card">
        <div class="table-wrap">
          <table class="data-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Slug</th>
                <th>Country</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-if="filtered.length === 0">
                <td colspan="4" class="empty-cell">No domains found.</td>
              </tr>
              <tr v-for="d in filtered" :key="d.id">
                <td class="cell-name">{{ d.name }}</td>
                <td class="cell-slug">{{ d.slug }}</td>
                <td class="cell-muted">{{ countryMap[d.country_id] ?? d.country_id }}</td>
                <td class="cell-actions">
                  <div class="row-actions">
                    <button class="btn-row-action" type="button" @click="openEdit(d)">Edit</button>
                    <button class="btn-row-delete" type="button" @click="openDelete(d)">Delete</button>
                  </div>
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </template>
  </div>

  <!-- Add / Edit modal -->
  <Teleport to="body">
    <div v-if="showModal" class="modal-backdrop" @click.self="closeModal">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="domain-modal-title">
        <div class="modal-header">
          <h2 id="domain-modal-title" class="modal-title">{{ modalMode === 'create' ? 'Add Domain' : 'Edit Domain' }}</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeModal">✕</button>
        </div>
        <form class="modal-body" @submit.prevent="submitForm">
          <div class="field">
            <label class="field-label">Name</label>
            <input
              v-model="form.name"
              class="field-input"
              type="text"
              required
              placeholder="Miami Metro"
              @input="onNameInput"
            />
          </div>
          <div class="field">
            <label class="field-label">Slug</label>
            <input
              v-model="form.slug"
              class="field-input"
              type="text"
              required
              placeholder="miami-metro"
              @input="slugTouched = true"
            />
          </div>
          <div class="field">
            <label class="field-label">Country</label>
            <select v-model="form.country_id" class="field-input field-select" required @change="onCountryChange">
              <option value="" disabled>Select country…</option>
              <option v-for="c in countries" :key="c.id" :value="c.id">{{ c.name }}</option>
            </select>
          </div>
          <div v-if="form.country_id" class="field">
            <label class="field-label">
              State <span class="field-optional">(optional)</span>
            </label>
            <select
              v-model="form.state_id"
              class="field-input field-select"
              :disabled="loadingStates"
              @change="onStateChange"
            >
              <option value="">{{ loadingStates ? 'Loading…' : 'None' }}</option>
              <option v-for="s in formStates" :key="s.id" :value="s.id">{{ s.name }}</option>
            </select>
          </div>
          <div v-if="form.state_id" class="field">
            <label class="field-label">
              City <span class="field-optional">(optional)</span>
            </label>
            <select v-model="form.city_id" class="field-input field-select" :disabled="loadingCities">
              <option value="">{{ loadingCities ? 'Loading…' : 'None' }}</option>
              <option v-for="c in formCities" :key="c.id" :value="c.id">{{ c.name }}</option>
            </select>
          </div>
          <div v-if="submitError" class="modal-error">{{ submitError }}</div>
          <div class="modal-footer">
            <button type="button" class="btn-cancel" :disabled="submitting" @click="closeModal">Cancel</button>
            <button type="submit" class="btn-submit" :disabled="submitting">
              {{ submitting ? 'Saving…' : (modalMode === 'create' ? 'Add Domain' : 'Save Changes') }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>

  <!-- Delete confirmation -->
  <Teleport to="body">
    <div v-if="domainToDelete" class="modal-backdrop" @click.self="closeDelete">
      <div class="modal modal--sm" role="dialog" aria-modal="true" aria-labelledby="del-domain-title">
        <div class="modal-header">
          <h2 id="del-domain-title" class="modal-title">Delete Domain</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeDelete">✕</button>
        </div>
        <div class="modal-body">
          <p class="confirm-msg">
            Delete <strong>{{ domainToDelete.name }}</strong>? This cannot be undone.
          </p>
          <div v-if="deleteError" class="modal-error">{{ deleteError }}</div>
          <div class="modal-footer">
            <button type="button" class="btn-cancel" :disabled="deleting" @click="closeDelete">Cancel</button>
            <button type="button" class="btn-delete" :disabled="deleting" @click="confirmDelete">
              {{ deleting ? 'Deleting…' : 'Delete' }}
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
  margin-bottom: var(--spacing-lg);
}

/* ── Toolbar ── */
.toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.search-input {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  background: var(--color-surface);
  border: 1px solid #E0DED9;
  border-radius: var(--rounded-md);
  padding: 7px 10px;
  outline: none;
  width: 240px;
  transition: border-color 0.15s;
}

.search-input:focus { border-color: var(--color-primary); }
.search-input::-webkit-search-cancel-button { display: none; }

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

.cell-name { font-weight: 500; white-space: nowrap; }
.cell-slug {
  font-family: var(--font-label);
  font-size: 0.8rem;
  color: var(--color-secondary);
  letter-spacing: 0.02em;
}
.cell-muted { color: var(--color-secondary); }
.cell-actions { width: 1px; white-space: nowrap; }
.row-actions { display: flex; gap: 12px; align-items: center; }

.empty-cell {
  text-align: center;
  color: var(--color-secondary);
  padding: var(--spacing-lg) !important;
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
.state-msg { font-family: var(--font-body); color: var(--color-secondary); }
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
  max-width: 440px;
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

.field-optional {
  font-style: normal;
  text-transform: none;
  letter-spacing: 0;
  opacity: 0.7;
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

.field-select { cursor: pointer; }
.field-select:disabled { opacity: 0.55; cursor: not-allowed; }

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
