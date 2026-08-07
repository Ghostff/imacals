<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import {
  integrationService,
  attributeService,
  INTEGRATION_TYPE_LABELS,
  CATEGORY_LABELS,
  CATEGORY_HINTS,
  type Integration,
  type IntegrationAttribute,
  type IntegrationCategory,
  type IntegrationType,
  type ProviderTypeDef,
  type FieldDef,
  type InlineAttribute,
} from '@/services/integration';
import { domainService } from '@/services/domain';
import { ApiException } from '@/services/api';

const integrations: Ref<Integration[]>                   = ref([]);
const providerTypes: Ref<ProviderTypeDef[]>              = ref([]);
const domainNames: Ref<Record<string, string>>           = ref({});
const domainOptions: Ref<{ id: string; name: string }[]> = ref([]);
const loading: Ref<boolean>                              = ref(true);
const error: Ref<string | null>                          = ref(null);

// Fixed order so the sending providers — the ones that decide whether mail goes out at all —
// always come first.
const SECTION_ORDER: IntegrationCategory[] = ['email', 'email-validation', 'other'];

interface Section {
  category: IntegrationCategory;
  label: string;
  hint: string;
  rows: Integration[];
}

const sections: ComputedRef<Section[]> = computed<Section[]>(() =>
  SECTION_ORDER
    .map((category) => ({
      category,
      label: CATEGORY_LABELS[category],
      hint: CATEGORY_HINTS[category],
      rows: integrations.value.filter((i) => i.integration_category === category),
    }))
    .filter((s) => s.rows.length > 0),
);

// An operator's first question is "what is sending?" — answer it above the tables.
const liveSender: ComputedRef<Integration | null> = computed<Integration | null>(
  () => integrations.value.find((i) => i.integration_category === 'email' && i.is_enabled) ?? null,
);

function typeLabel(t: IntegrationType): string {
  return INTEGRATION_TYPE_LABELS[t] ?? t;
}

function fieldsFor(t: IntegrationType): FieldDef[] {
  return providerTypes.value.find((p) => p.integration_type === t)?.fields ?? [];
}

async function load(): Promise<void> {
  loading.value = true;
  error.value = null;
  try {
    const [rows, types] = await Promise.all([
      integrationService.index(),
      integrationService.providerTypes(),
    ]);
    integrations.value = rows;
    providerTypes.value = types;

    const domains = await domainService.index();
    domainNames.value = Object.fromEntries(domains.map((d) => [d.id, d.name]));
    domainOptions.value = domains.map((d) => ({ id: d.id, name: d.name }));
  } catch (e: unknown) {
    error.value =
      e instanceof ApiException || e instanceof Error ? e.message : 'Could not load integrations.';
  } finally {
    loading.value = false;
  }
}

onMounted(load);

// ── Switching the live provider ────────────────────────────────────────────
const switching: Ref<string | null> = ref(null);

// A full re-read, not a local flip: enabling one provider disables its siblings on the server, so
// every other badge in the category is stale until we reload.
async function makeLive(row: Integration): Promise<void> {
  switching.value = row.id;
  error.value = null;
  try {
    await integrationService.setEnabled(row.id, true);
    integrations.value = await integrationService.index();
  } catch (e: unknown) {
    error.value =
      e instanceof ApiException || e instanceof Error ? e.message : 'Could not switch provider.';
  } finally {
    switching.value = null;
  }
}

async function turnOff(row: Integration): Promise<void> {
  switching.value = row.id;
  error.value = null;
  try {
    const updated = await integrationService.setEnabled(row.id, false);
    integrations.value = integrations.value.map((i) => (i.id === updated.id ? updated : i));
  } catch (e: unknown) {
    error.value =
      e instanceof ApiException || e instanceof Error ? e.message : 'Could not disable provider.';
  } finally {
    switching.value = null;
  }
}

// ── Add provider ───────────────────────────────────────────────────────────
const showAdd: Ref<boolean> = ref(false);
const submitting: Ref<boolean> = ref(false);
const submitError: Ref<string | null> = ref(null);
const slugTouched: Ref<boolean> = ref(false);

interface AddForm {
  name: string;
  slug: string;
  domain_id: string;
  integration_type: IntegrationType;
}

const form: Ref<AddForm> = ref({ name: '', slug: '', domain_id: '', integration_type: 'smtp' });
const credentials: Ref<Record<string, string>> = ref({});
const customRows: Ref<{ name: string; value: string; encrypted: boolean }[]> = ref([]);

const addFields: ComputedRef<FieldDef[]> = computed<FieldDef[]>(() =>
  fieldsFor(form.value.integration_type),
);

function slugify(s: string): string {
  return s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '');
}

function onNameInput(): void {
  if (!slugTouched.value) form.value.slug = slugify(form.value.name);
}

function openAdd(): void {
  form.value = {
    name: '',
    slug: '',
    domain_id: domainOptions.value[0]?.id ?? '',
    integration_type: 'smtp',
  };
  credentials.value = {};
  customRows.value = [];
  slugTouched.value = false;
  submitError.value = null;
  showAdd.value = true;
}

function closeAdd(): void {
  if (submitting.value) return;
  showAdd.value = false;
}

function addCustomRow(): void {
  customRows.value = [...customRows.value, { name: '', value: '', encrypted: false }];
}

function removeCustomRow(index: number): void {
  customRows.value = customRows.value.filter((_, i) => i !== index);
}

function buildAttributes(): InlineAttribute[] {
  if (form.value.integration_type === 'custom') {
    return customRows.value
      .filter((r) => r.name.trim().length > 0)
      .map((r) => ({
        name: r.name.trim(),
        value: r.value,
        type: r.encrypted ? 'password' : 'text',
        is_encrypted: r.encrypted,
      }));
  }
  return addFields.value
    .map((f) => ({
      name: f.name,
      value: credentials.value[f.name] ?? '',
      type: f.type,
      is_encrypted: f.is_encrypted,
    }))
    // Blank optional fields are dropped rather than stored as empty attributes.
    .filter((a) => (a.value ?? '').length > 0);
}

async function submitAdd(): Promise<void> {
  submitting.value = true;
  submitError.value = null;
  try {
    await integrationService.create({
      domain_id: form.value.domain_id,
      name: form.value.name.trim(),
      slug: form.value.slug.trim(),
      integration_type: form.value.integration_type,
      attributes: buildAttributes(),
    });
    // Re-read rather than push: the server decides whether a new provider goes live.
    integrations.value = await integrationService.index();
    showAdd.value = false;
  } catch (e: unknown) {
    submitError.value =
      e instanceof ApiException || e instanceof Error
        ? e.message
        : 'Could not create the integration.';
  } finally {
    submitting.value = false;
  }
}

// ── Credentials ────────────────────────────────────────────────────────────
const showCreds: Ref<boolean> = ref(false);
const credsRow: Ref<Integration | null> = ref(null);
const credsAttributes: Ref<IntegrationAttribute[]> = ref([]);
const credsLoading: Ref<boolean> = ref(false);
const credsError: Ref<string | null> = ref(null);
// Only fields the operator actually retyped get sent — an untouched secret keeps its stored value.
const credsEdits: Ref<Record<string, string>> = ref({});
const credsSaving: Ref<boolean> = ref(false);
const credsSaved: Ref<boolean> = ref(false);

interface CredsField {
  field: FieldDef | null;
  attribute: IntegrationAttribute | null;
  name: string;
  label: string;
  isSecret: boolean;
}

async function openCreds(row: Integration): Promise<void> {
  credsRow.value = row;
  credsAttributes.value = [];
  credsEdits.value = {};
  credsError.value = null;
  credsSaved.value = false;
  credsLoading.value = true;
  showCreds.value = true;
  try {
    credsAttributes.value = await integrationService.getAttributes(row.id);
  } catch (e: unknown) {
    credsError.value =
      e instanceof ApiException || e instanceof Error ? e.message : 'Could not load credentials.';
  } finally {
    credsLoading.value = false;
  }
}

function closeCreds(): void {
  if (credsSaving.value) return;
  showCreds.value = false;
}

// Template fields first, in the provider's declared order, then anything added by hand — so the
// form reads the same way regardless of insert order.
const credsFields: ComputedRef<CredsField[]> = computed<CredsField[]>(() => {
  const row = credsRow.value;
  if (!row) return [];
  const template = fieldsFor(row.integration_type);
  const byName = new Map(credsAttributes.value.map((a) => [a.name, a]));

  const fromTemplate: CredsField[] = template.map((field) => ({
    field,
    attribute: byName.get(field.name) ?? null,
    name: field.name,
    label: field.label,
    isSecret: field.is_encrypted,
  }));

  const extras: CredsField[] = credsAttributes.value
    .filter((a) => !template.some((f) => f.name === a.name))
    .map((attribute) => ({
      field: null,
      attribute,
      name: attribute.name,
      label: attribute.name,
      isSecret: attribute.is_encrypted,
    }));

  return [...fromTemplate, ...extras];
});

function inputValue(entry: CredsField): string {
  const typed = credsEdits.value[entry.name];
  if (typed !== undefined) return typed;
  // Secrets come back as null from the API, so there is nothing to prefill.
  return entry.isSecret ? '' : entry.attribute?.value ?? '';
}

function setEdited(name: string, value: string): void {
  credsEdits.value = { ...credsEdits.value, [name]: value };
  credsSaved.value = false;
}

async function saveCreds(): Promise<void> {
  const row = credsRow.value;
  if (!row) return;
  credsSaving.value = true;
  credsError.value = null;
  try {
    for (const entry of credsFields.value) {
      const typed = credsEdits.value[entry.name];
      if (typed === undefined || typed.length === 0) continue;

      if (entry.attribute) {
        await attributeService.update(entry.attribute.id, { value: typed });
      } else {
        await attributeService.create({
          attributeable_type: 'integrations',
          attributeable_id: row.id,
          name: entry.name,
          value: typed,
          type: entry.field?.type ?? 'text',
          is_encrypted: entry.isSecret,
        });
      }
    }
    credsAttributes.value = await integrationService.getAttributes(row.id);
    credsEdits.value = {};
    credsSaved.value = true;
  } catch (e: unknown) {
    credsError.value =
      e instanceof ApiException || e instanceof Error ? e.message : 'Could not save credentials.';
  } finally {
    credsSaving.value = false;
  }
}

// ── Delete ─────────────────────────────────────────────────────────────────
const showDelete: Ref<boolean> = ref(false);
const deleteRow: Ref<Integration | null> = ref(null);
const deleting: Ref<boolean> = ref(false);
const deleteError: Ref<string | null> = ref(null);

function openDelete(row: Integration): void {
  deleteRow.value = row;
  deleteError.value = null;
  showDelete.value = true;
}

async function confirmDelete(): Promise<void> {
  const row = deleteRow.value;
  if (!row) return;
  deleting.value = true;
  deleteError.value = null;
  try {
    await integrationService.delete(row.id);
    integrations.value = integrations.value.filter((i) => i.id !== row.id);
    showDelete.value = false;
  } catch (e: unknown) {
    deleteError.value =
      e instanceof ApiException || e instanceof Error
        ? e.message
        : 'Could not delete the integration.';
  } finally {
    deleting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <p class="page-label">Settings</p>
    <h1 class="page-title">Integrations</h1>
    <p class="page-intro">
      Providers imacals sends and verifies campaign mail through. Credentials live here, not in
      environment files — a change applies to the next send, with no restart.
    </p>

    <div v-if="loading" class="state-msg">Loading…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <div class="live-banner">
        <div>
          <p class="live-label">Currently sending through</p>
          <p v-if="liveSender" class="live-value">{{ liveSender.name }}</p>
          <p v-else class="live-value live-value--none">
            No sending provider is live — campaigns cannot send.
          </p>
        </div>
        <button class="btn-add" type="button" @click="openAdd">+ Add Provider</button>
      </div>

      <p v-if="integrations.length === 0" class="state-msg">No integrations configured yet.</p>

      <section v-for="section in sections" :key="section.category" class="section">
        <div class="section-header">
          <h2 class="section-title">{{ section.label }}</h2>
          <p class="section-hint">{{ section.hint }}</p>
        </div>

        <div class="table-card">
          <div class="table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Name</th>
                  <th>Provider</th>
                  <th>Domain</th>
                  <th>Status</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="row in section.rows" :key="row.id">
                  <td class="cell-name">{{ row.name }}</td>
                  <td class="cell-muted">{{ typeLabel(row.integration_type) }}</td>
                  <td class="cell-muted">{{ domainNames[row.domain_id] ?? row.domain_id }}</td>
                  <td>
                    <span v-if="row.is_enabled" class="badge badge--live">Live</span>
                    <span v-else class="badge badge--off">Off</span>
                  </td>
                  <td class="cell-actions">
                    <div class="row-actions">
                      <button
                        v-if="!row.is_enabled"
                        class="btn-row-action"
                        type="button"
                        :disabled="switching === row.id"
                        @click="makeLive(row)"
                      >
                        {{ switching === row.id ? 'Switching…' : 'Make live' }}
                      </button>
                      <button
                        v-else
                        class="btn-row-action"
                        type="button"
                        :disabled="switching === row.id"
                        @click="turnOff(row)"
                      >
                        {{ switching === row.id ? 'Working…' : 'Turn off' }}
                      </button>
                      <button class="btn-row-action" type="button" @click="openCreds(row)">
                        Credentials
                      </button>
                      <button class="btn-row-delete" type="button" @click="openDelete(row)">
                        Delete
                      </button>
                    </div>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>
    </template>
  </div>

  <!-- Add provider -->
  <Teleport to="body">
    <div v-if="showAdd" class="modal-backdrop" @click.self="closeAdd">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="add-int-title">
        <div class="modal-header">
          <h2 id="add-int-title" class="modal-title">Add Provider</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeAdd">✕</button>
        </div>

        <form class="modal-body" @submit.prevent="submitAdd">
          <div class="field-row">
            <div class="field-group">
              <label class="field-label" for="int-name">Name <span class="req">*</span></label>
              <input
                id="int-name"
                v-model="form.name"
                class="field-input"
                type="text"
                required
                placeholder="Campaign Relay"
                @input="onNameInput"
              />
            </div>
            <div class="field-group">
              <label class="field-label" for="int-slug">Slug <span class="req">*</span></label>
              <input
                id="int-slug"
                v-model="form.slug"
                class="field-input"
                type="text"
                required
                placeholder="campaign-relay"
                @input="slugTouched = true"
              />
            </div>
          </div>

          <div class="field-row">
            <div class="field-group">
              <label class="field-label" for="int-type">Provider <span class="req">*</span></label>
              <select id="int-type" v-model="form.integration_type" class="field-input">
                <option
                  v-for="p in providerTypes"
                  :key="p.integration_type"
                  :value="p.integration_type"
                >
                  {{ typeLabel(p.integration_type) }}
                </option>
              </select>
            </div>
            <div class="field-group">
              <label class="field-label" for="int-domain">Domain <span class="req">*</span></label>
              <select id="int-domain" v-model="form.domain_id" class="field-input" required>
                <option v-for="d in domainOptions" :key="d.id" :value="d.id">{{ d.name }}</option>
              </select>
            </div>
          </div>

          <!-- Credentials rendered from the provider's field template -->
          <template v-if="form.integration_type !== 'custom'">
            <div class="secrets-header">
              <p class="section-label">Credentials</p>
              <p class="secrets-hint">Secrets are encrypted before storing and never read back.</p>
            </div>
            <p v-if="addFields.length === 0" class="secrets-hint">
              This provider needs no credentials.
            </p>
            <div v-else class="secrets-list">
              <div v-for="field in addFields" :key="field.name" class="field-group">
                <label class="field-label" :for="`cred-${field.name}`">
                  {{ field.label }}
                  <span v-if="field.is_required" class="req">*</span>
                </label>
                <input
                  :id="`cred-${field.name}`"
                  v-model="credentials[field.name]"
                  class="field-input"
                  :type="field.type === 'password' ? 'password' : 'text'"
                  :required="field.is_required"
                  :placeholder="field.label"
                  autocomplete="off"
                />
              </div>
            </div>
          </template>

          <!-- Free-form attributes for Custom -->
          <template v-else>
            <div class="secrets-header">
              <p class="section-label">Attributes</p>
              <button class="btn-row-action" type="button" @click="addCustomRow">
                + Add attribute
              </button>
            </div>
            <div v-if="customRows.length > 0" class="secrets-list">
              <div v-for="(row, index) in customRows" :key="index" class="custom-row">
                <input v-model="row.name" class="field-input" type="text" placeholder="KEY" />
                <input
                  v-model="row.value"
                  class="field-input"
                  :type="row.encrypted ? 'password' : 'text'"
                  placeholder="value"
                  autocomplete="off"
                />
                <label class="checkbox-label">
                  <input v-model="row.encrypted" type="checkbox" />
                  Encrypt
                </label>
                <button class="btn-row-delete" type="button" @click="removeCustomRow(index)">
                  Remove
                </button>
              </div>
            </div>
          </template>

          <p v-if="submitError" class="state-msg state-msg--error" role="alert">
            {{ submitError }}
          </p>

          <div class="modal-footer">
            <button class="btn-secondary" type="button" :disabled="submitting" @click="closeAdd">
              Cancel
            </button>
            <button class="btn-primary" type="submit" :disabled="submitting">
              {{ submitting ? 'Saving…' : 'Add Provider' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </Teleport>

  <!-- Credentials -->
  <Teleport to="body">
    <div v-if="showCreds" class="modal-backdrop" @click.self="closeCreds">
      <div class="modal" role="dialog" aria-modal="true" aria-labelledby="creds-title">
        <div class="modal-header">
          <h2 id="creds-title" class="modal-title">{{ credsRow?.name }} credentials</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="closeCreds">✕</button>
        </div>

        <div class="modal-body">
          <div v-if="credsLoading" class="state-msg">Loading…</div>
          <div v-else-if="credsError" class="state-msg state-msg--error">{{ credsError }}</div>

          <template v-else>
            <p class="secrets-hint">
              Leave a field blank to keep its stored value. Encrypted values are never returned —
              type a new one to replace it.
            </p>

            <div class="secrets-list">
              <div v-for="entry in credsFields" :key="entry.name" class="field-group">
                <label class="field-label" :for="`edit-${entry.name}`">
                  {{ entry.label }}
                  <span v-if="entry.isSecret" class="stored-tag">
                    {{ entry.attribute ? 'stored' : 'not set' }}
                  </span>
                </label>
                <input
                  :id="`edit-${entry.name}`"
                  class="field-input"
                  :type="entry.isSecret ? 'password' : 'text'"
                  :value="inputValue(entry)"
                  :placeholder="entry.isSecret ? '••••••••' : entry.label"
                  autocomplete="off"
                  @input="setEdited(entry.name, ($event.target as HTMLInputElement).value)"
                />
              </div>
            </div>

            <p v-if="credsSaved" class="state-msg">Saved. The next send uses these values.</p>

            <div class="modal-footer">
              <button
                class="btn-secondary"
                type="button"
                :disabled="credsSaving"
                @click="closeCreds"
              >
                Close
              </button>
              <button
                class="btn-primary"
                type="button"
                :disabled="credsSaving"
                @click="saveCreds"
              >
                {{ credsSaving ? 'Saving…' : 'Save changes' }}
              </button>
            </div>
          </template>
        </div>
      </div>
    </div>
  </Teleport>

  <!-- Delete -->
  <Teleport to="body">
    <div v-if="showDelete" class="modal-backdrop" @click.self="showDelete = false">
      <div class="modal modal--sm" role="dialog" aria-modal="true" aria-labelledby="del-int-title">
        <div class="modal-header">
          <h2 id="del-int-title" class="modal-title">Delete integration</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="showDelete = false">
            ✕
          </button>
        </div>
        <div class="modal-body">
          <p class="confirm-text">
            Delete <strong>{{ deleteRow?.name }}</strong>? Its credentials are removed with it.
          </p>
          <p v-if="deleteRow?.is_enabled" class="state-msg state-msg--error">
            This provider is currently live — deleting it leaves nothing sending.
          </p>
          <p v-if="deleteError" class="state-msg state-msg--error" role="alert">
            {{ deleteError }}
          </p>
          <div class="modal-footer">
            <button
              class="btn-secondary"
              type="button"
              :disabled="deleting"
              @click="showDelete = false"
            >
              Cancel
            </button>
            <button class="btn-danger" type="button" :disabled="deleting" @click="confirmDelete">
              {{ deleting ? 'Deleting…' : 'Delete' }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.page { padding: var(--spacing-lg); }

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
  font-size: 2rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
}

.page-intro {
  font-family: var(--font-body);
  font-size: 0.9375rem;
  color: var(--color-secondary);
  max-width: 62ch;
  margin-top: var(--spacing-sm);
}

.live-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  flex-wrap: wrap;
  background-color: var(--color-surface);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-md) var(--spacing-lg);
  margin: var(--spacing-lg) 0;
}

.live-label {
  font-family: var(--font-label);
  font-size: 0.6875rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.live-value {
  font-family: var(--font-body);
  font-size: 1.0625rem;
  color: var(--color-primary);
  margin-top: 4px;
}

.live-value--none { color: var(--color-tertiary); }

.section { margin-bottom: var(--spacing-lg); }

.section-header { margin-bottom: var(--spacing-md); }

.section-title {
  font-family: var(--font-body);
  font-size: 1.0625rem;
  font-weight: 600;
  color: var(--color-primary);
}

.section-hint {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.table-card {
  background-color: var(--color-surface);
  border-radius: var(--rounded-lg);
  overflow: hidden;
}

.table-wrap { overflow-x: auto; }

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-family: var(--font-body);
  font-size: 0.9375rem;
}

.data-table th {
  font-family: var(--font-label);
  font-size: 0.6875rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  text-align: left;
  padding: 12px var(--spacing-md);
  border-bottom: 1px solid var(--color-border);
  white-space: nowrap;
}

.data-table td {
  padding: 14px var(--spacing-md);
  border-bottom: 1px solid var(--color-divider);
  color: var(--color-primary);
  vertical-align: middle;
}

.data-table tbody tr:last-child td { border-bottom: none; }

.cell-name { font-weight: 500; }
.cell-muted { color: var(--color-secondary); }
.cell-actions { text-align: right; }

.badge {
  font-family: var(--font-label);
  font-size: 0.6875rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  border-radius: var(--rounded-sm);
  padding: 3px 8px;
}

.badge--live { color: var(--color-on-primary); background-color: var(--color-primary); }
.badge--off { color: var(--color-secondary); background-color: var(--color-neutral); }

.row-actions {
  display: flex;
  gap: var(--spacing-sm);
  justify-content: flex-end;
}

.btn-add {
  font-family: var(--font-body);
  font-size: 0.9375rem;
  font-weight: 500;
  color: var(--color-on-primary);
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 10px 18px;
  cursor: pointer;
  white-space: nowrap;
}

.btn-add:hover { opacity: 0.9; }

.btn-row-action,
.btn-row-delete {
  font-family: var(--font-label);
  font-size: 0.75rem;
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-sm);
  padding: 5px 10px;
  cursor: pointer;
  white-space: nowrap;
}

.btn-row-action { color: var(--color-primary); }
.btn-row-delete { color: var(--color-tertiary); }

.btn-row-action:disabled,
.btn-row-delete:disabled { opacity: 0.5; cursor: not-allowed; }

.state-msg { font-family: var(--font-body); color: var(--color-secondary); }
.state-msg--error { color: var(--color-tertiary); }

/* ── Modals ─────────────────────────────────────────────────────────────── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--spacing-lg);
  z-index: 50;
}

.modal {
  width: 100%;
  max-width: 620px;
  max-height: 88vh;
  overflow-y: auto;
  background-color: var(--color-surface);
  border-radius: var(--rounded-lg);
}

.modal--sm { max-width: 420px; }

.modal-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md) var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
}

.modal-title {
  font-family: var(--font-body);
  font-size: 1.0625rem;
  font-weight: 600;
  color: var(--color-primary);
}

.modal-close {
  background: none;
  border: none;
  font-size: 1rem;
  color: var(--color-secondary);
  cursor: pointer;
}

.modal-body {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  padding: var(--spacing-lg);
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
}

.field-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--spacing-md);
}

.field-group { display: flex; flex-direction: column; gap: 6px; }

.field-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.req { color: var(--color-tertiary); }

.stored-tag {
  font-size: 0.625rem;
  letter-spacing: 0.06em;
  color: var(--color-secondary);
  margin-left: 6px;
  text-transform: none;
}

.field-input {
  font-family: var(--font-body);
  font-size: 0.9375rem;
  color: var(--color-primary);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-sm);
  padding: 9px 11px;
  outline: none;
  width: 100%;
}

.field-input:focus { border-color: var(--color-primary); }

.secrets-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--spacing-md);
  border-top: 1px solid var(--color-border);
  padding-top: var(--spacing-md);
}

.section-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  color: var(--color-primary);
}

.secrets-hint {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
}

.secrets-list { display: flex; flex-direction: column; gap: var(--spacing-md); }

.custom-row {
  display: grid;
  grid-template-columns: 1fr 1fr auto auto;
  gap: var(--spacing-sm);
  align-items: center;
}

.checkbox-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
  display: flex;
  align-items: center;
  gap: 6px;
  white-space: nowrap;
}

.confirm-text { font-family: var(--font-body); color: var(--color-primary); }

.btn-primary,
.btn-secondary,
.btn-danger {
  font-family: var(--font-body);
  font-size: 0.9375rem;
  font-weight: 500;
  border-radius: var(--rounded-md);
  padding: 10px 18px;
  cursor: pointer;
  border: none;
}

.btn-primary { color: var(--color-on-primary); background-color: var(--color-tertiary); }

.btn-secondary {
  color: var(--color-primary);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
}

.btn-danger { color: var(--color-on-primary); background-color: var(--color-tertiary); }

.btn-primary:disabled,
.btn-secondary:disabled,
.btn-danger:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
