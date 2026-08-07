<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import { useRoute } from 'vue-router';
import {
  userProfileService,
  uploadDocumentXhr,
  type UserDocument,
  type UserBankAccount, type CreateBankAccountPayload,
} from '@/services/user_profile';
import { userService, type User, type UpdateUserPayload } from '@/services/user';
import { ApiException } from '@/services/api';

const route = useRoute();
const userId = route.params.id as string;

type Tab = 'profile' | 'documents' | 'bank';
const activeTab: Ref<Tab> = ref('profile');

const targetUser: Ref<User | null>            = ref(null);
const documents: Ref<UserDocument[]>          = ref([]);
const bankAccounts: Ref<UserBankAccount[]>    = ref([]);
const loading: Ref<boolean>                   = ref(true);
const error: Ref<string | null>               = ref(null);

// Proof of funds is only shown for project-manager role
const canUploadProofOfFunds: ComputedRef<boolean> = computed(() =>
  targetUser.value?.user_role?.name === 'project-manager',
);

interface DocumentSlotConfig {
  label: string;
  // Kebab-case value stored in files.file_type by the API.
  fileType: string;
  // When false: delete the existing file before uploading so only one is kept.
  allowMultiple: boolean;
}

const DOCUMENT_SLOTS: Record<string, DocumentSlotConfig> = {
  signature:      { label: 'Signature',     fileType: 'user-signature',      allowMultiple: false },
  initials:       { label: 'Initials',       fileType: 'user-initials',       allowMultiple: false },
  proof_of_funds: { label: 'Proof of Funds', fileType: 'user-proof-of-funds', allowMultiple: true  },
};

const availableDocumentTypes: ComputedRef<string[]> = computed(() => {
  const always = Object.keys(DOCUMENT_SLOTS).filter((k) => k !== 'proof_of_funds');
  if (canUploadProofOfFunds.value) always.push('proof_of_funds');
  return always;
});

// ── Basic info form ───────────────────────────────────────────────────────
const basicForm: Ref<UpdateUserPayload>     = ref({ first_name: '', last_name: '', email: '', phone: '', date_of_birth: '' });
const basicSaving: Ref<boolean>             = ref(false);
const basicError: Ref<string | null>        = ref(null);
const basicSuccess: Ref<boolean>            = ref(false);

// ── Per-slot upload state ─────────────────────────────────────────────────
// uploadProgress and uploadError are keyed by document_type
const uploadProgress: Ref<Record<string, number>>       = ref({});
const uploadError:    Ref<Record<string, string | null>> = ref({});
const uploading:      Ref<Record<string, boolean>>       = ref({});

// ── Add-bank form ─────────────────────────────────────────────────────────
const bankForm: Ref<CreateBankAccountPayload> = ref({
  bank_name: '', account_holder_name: '', account_type: 'checking',
  account_number: '', routing_number: '', is_primary: false,
});
const bankSaving: Ref<boolean>   = ref(false);
const bankError: Ref<string|null> = ref(null);


// ── Load ──────────────────────────────────────────────────────────────────
onMounted(async () => {
  try {
    const [userResult, docsResult, bankResult] = await Promise.all([
      userService.index().then((list) => list.find((u) => u.id === userId) ?? null),
      userProfileService.getDocuments(userId).catch(() => []),
      userProfileService.getBankAccounts(userId).catch(() => []),
    ]);

    targetUser.value   = userResult;
    documents.value    = docsResult;
    bankAccounts.value = bankResult;

    if (targetUser.value) {
      basicForm.value = {
        first_name:    targetUser.value.first_name,
        last_name:     targetUser.value.last_name,
        email:         targetUser.value.email,
        phone:         targetUser.value.phone ?? '',
        date_of_birth: targetUser.value.date_of_birth ?? '',
      };
    }
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to load profile.';
  } finally {
    loading.value = false;
  }
});

// ── Basic info save ───────────────────────────────────────────────────────
async function saveBasicInfo(): Promise<void> {
  basicError.value   = null;
  basicSuccess.value = false;
  basicSaving.value  = true;
  try {
    await userService.update(userId, {
      first_name:    basicForm.value.first_name.trim(),
      last_name:     basicForm.value.last_name.trim(),
      email:         basicForm.value.email.trim(),
      phone:         basicForm.value.phone?.trim() || undefined,
      date_of_birth: basicForm.value.date_of_birth || undefined,
    });
    if (targetUser.value) {
      targetUser.value = {
        ...targetUser.value,
        first_name:    basicForm.value.first_name.trim(),
        last_name:     basicForm.value.last_name.trim(),
        email:         basicForm.value.email.trim(),
        phone:         basicForm.value.phone?.trim() || null,
        date_of_birth: basicForm.value.date_of_birth || null,
      };
    }
    basicSuccess.value = true;
    setTimeout(() => { basicSuccess.value = false; }, 3000);
  } catch (e: unknown) {
    basicError.value = e instanceof ApiException ? e.message : 'Failed to save.';
  } finally {
    basicSaving.value = false;
  }
}

// ── Documents ─────────────────────────────────────────────────────────────
async function uploadDocument(docType: string, event: Event): Promise<void> {
  const input = event.target as HTMLInputElement;
  const file  = input.files?.[0];
  if (!file) return;

  uploadError.value    = { ...uploadError.value,    [docType]: null };
  uploadProgress.value = { ...uploadProgress.value, [docType]: 0 };
  uploading.value      = { ...uploading.value,      [docType]: true };

  try {
    const slot = DOCUMENT_SLOTS[docType];

    // Single-upload slots: silently replace by deleting every existing file
    // for this slot before uploading the new one.
    if (!slot.allowMultiple) {
      const existing = documents.value.filter((d) => d.file_type === slot.fileType);
      await Promise.all(existing.map((d) => userProfileService.deleteDocument(userId, d.id)));
      documents.value = documents.value.filter((d) => d.file_type !== slot.fileType);
    }

    const doc = await uploadDocumentXhr(userId, docType, file, (pct: number) => {
      uploadProgress.value = { ...uploadProgress.value, [docType]: pct };
    });
    documents.value = [doc, ...documents.value];
  } catch (e: unknown) {
    uploadError.value = { ...uploadError.value, [docType]: e instanceof Error ? e.message : 'Upload failed.' };
  } finally {
    uploading.value      = { ...uploading.value,      [docType]: false };
    uploadProgress.value = { ...uploadProgress.value, [docType]: 0 };
    input.value = '';
  }
}

async function removeDocument(docId: string): Promise<void> {
  await userProfileService.deleteDocument(userId, docId);
  documents.value = documents.value.filter((d) => d.id !== docId);
}

// ── Bank accounts ─────────────────────────────────────────────────────────
async function addBankAccount(): Promise<void> {
  bankError.value  = null;
  bankSaving.value = true;
  try {
    const account = await userProfileService.createBankAccount(userId, bankForm.value);
    bankAccounts.value = [...bankAccounts.value, account];
    bankForm.value     = { bank_name: '', account_holder_name: '', account_type: 'checking', account_number: '', routing_number: '', is_primary: false };
  } catch (e: unknown) {
    bankError.value = e instanceof ApiException ? e.message : 'Failed to add bank account.';
  } finally {
    bankSaving.value = false;
  }
}

async function removeBankAccount(accountId: string): Promise<void> {
  await userProfileService.deleteBankAccount(userId, accountId);
  bankAccounts.value = bankAccounts.value.filter((a) => a.id !== accountId);
}


function maskAccount(num: string): string {
  return '•••• ' + num.slice(-4);
}


</script>

<template>
  <div class="page">
    <div v-if="loading" class="state-msg">Loading…</div>
    <div v-else-if="error" class="state-msg state-msg--error">{{ error }}</div>

    <template v-else>
      <!-- Header -->
      <p class="page-label">Users</p>
      <div class="profile-header">
        <div>
          <h1 class="page-title">{{ targetUser?.first_name }} {{ targetUser?.last_name }}</h1>
          <p class="profile-email">{{ targetUser?.email }}</p>
        </div>
        <span v-if="targetUser?.user_role" class="role-badge">{{ targetUser.user_role.title }}</span>
      </div>

      <!-- Tabs -->
      <nav class="tabs">
        <button
          v-for="tab in (['profile', 'documents', 'bank'] as Tab[])"
          :key="tab"
          class="tab-btn"
          :class="{ 'tab-btn--active': activeTab === tab }"
          type="button"
          @click="activeTab = tab"
        >
          {{ { profile: 'Profile', documents: 'Documents', bank: 'Bank' }[tab] }}
        </button>
      </nav>

      <!-- ── Profile tab ── -->
      <section v-if="activeTab === 'profile'" class="section">
        <!-- Basic info -->
        <div class="card">
          <h3 class="card-title">Basic Info</h3>
          <form @submit.prevent="saveBasicInfo">
            <div class="field-grid">
              <div class="field">
                <label class="field-label">First Name</label>
                <input v-model="basicForm.first_name" class="field-input" type="text" required />
              </div>
              <div class="field">
                <label class="field-label">Last Name</label>
                <input v-model="basicForm.last_name" class="field-input" type="text" required />
              </div>
              <div class="field">
                <label class="field-label">Email</label>
                <input v-model="basicForm.email" class="field-input" type="email" required />
              </div>
              <div class="field">
                <label class="field-label">Phone</label>
                <input v-model="basicForm.phone" class="field-input" type="tel" placeholder="+1 (555) 000-0000" />
              </div>
              <div class="field">
                <label class="field-label">Date of Birth</label>
                <input v-model="basicForm.date_of_birth" class="field-input" type="date" />
              </div>
            </div>
            <div v-if="basicError" class="form-error">{{ basicError }}</div>
            <div v-if="basicSuccess" class="form-success">Saved.</div>
            <button type="submit" class="btn-primary" :disabled="basicSaving">
              {{ basicSaving ? 'Saving…' : 'Save' }}
            </button>
          </form>
        </div>

      </section>

      <!-- ── Documents tab ── -->
      <section v-if="activeTab === 'documents'" class="section">
        <div class="card">
          <div class="doc-slots">
            <div
              v-for="type in availableDocumentTypes"
              :key="type"
              class="doc-slot"
            >
              <p class="doc-slot-label">{{ DOCUMENT_SLOTS[type].label }}</p>

              <!-- Existing files -->
              <div class="doc-slot-files">
                <div
                  v-for="doc in documents.filter((d) => d.file_type === DOCUMENT_SLOTS[type].fileType)"
                  :key="doc.id"
                  class="doc-item"
                >
                  <span class="doc-name">{{ doc.name }}</span>
                  <button class="doc-remove" type="button" @click="removeDocument(doc.id)">Remove</button>
                </div>
                <p v-if="!documents.some((d) => d.file_type === DOCUMENT_SLOTS[type].fileType)" class="doc-empty">No file uploaded.</p>
              </div>

              <!-- Upload control -->
              <div class="doc-upload">
                <label class="upload-btn" :class="{ 'upload-btn--busy': uploading[type] }">
                  {{ uploading[type] ? 'Uploading…' : 'Choose file' }}
                  <input
                    type="file"
                    class="upload-input"
                    :disabled="uploading[type]"
                    @change="uploadDocument(type, $event)"
                  />
                </label>

                <!-- Progress bar -->
                <div v-if="uploading[type]" class="progress-wrap">
                  <div class="progress-bar" :style="{ width: (uploadProgress[type] ?? 0) + '%' }" />
                </div>

                <p v-if="uploadError[type]" class="upload-error">{{ uploadError[type] }}</p>
              </div>
            </div>
          </div>
        </div>
      </section>

      <!-- ── Bank tab ── -->
      <section v-if="activeTab === 'bank'" class="section">
        <div v-if="bankAccounts.length" class="card-list">
          <div v-for="a in bankAccounts" :key="a.id" class="card card--row">
            <div>
              <p class="card-primary">{{ a.bank_name }}</p>
              <p class="card-sub">{{ a.account_holder_name }} · {{ a.account_type }} · {{ maskAccount(a.account_number) }}</p>
            </div>
            <div class="card-actions">
              <span v-if="a.is_primary" class="badge-primary">Primary</span>
              <button class="btn-danger-sm" type="button" @click="removeBankAccount(a.id)">Remove</button>
            </div>
          </div>
        </div>

        <div class="card">
          <h3 class="card-title">Add Bank Account</h3>
          <form @submit.prevent="addBankAccount">
            <div class="field-grid">
              <div class="field">
                <label class="field-label">Bank Name</label>
                <input v-model="bankForm.bank_name" class="field-input" type="text" required />
              </div>
              <div class="field">
                <label class="field-label">Account Holder Name</label>
                <input v-model="bankForm.account_holder_name" class="field-input" type="text" required />
              </div>
              <div class="field">
                <label class="field-label">Account Type</label>
                <select v-model="bankForm.account_type" class="field-input">
                  <option value="checking">Checking</option>
                  <option value="savings">Savings</option>
                </select>
              </div>
              <div class="field">
                <label class="field-label">Account Number</label>
                <input v-model="bankForm.account_number" class="field-input" type="text" required />
              </div>
              <div class="field">
                <label class="field-label">Routing Number</label>
                <input v-model="bankForm.routing_number" class="field-input" type="text" required />
              </div>
              <div class="field field--checkbox">
                <label class="checkbox-label">
                  <input v-model="bankForm.is_primary" type="checkbox" />
                  Primary account
                </label>
              </div>
            </div>
            <div v-if="bankError" class="form-error">{{ bankError }}</div>
            <button type="submit" class="btn-primary" :disabled="bankSaving">
              {{ bankSaving ? 'Adding…' : 'Add Bank Account' }}
            </button>
          </form>
        </div>
      </section>

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

.profile-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: var(--spacing-lg);
}

.page-title {
  font-family: var(--font-display);
  font-size: 2.5rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
}

.profile-email {
  font-family: var(--font-body);
  font-size: 0.9rem;
  color: var(--color-secondary);
  margin-top: 4px;
}

.role-badge {
  display: inline-block;
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 4px 10px;
  border-radius: var(--rounded-sm);
  background-color: #E5E2DE;
  color: var(--color-secondary);
  margin-top: 8px;
}

/* ── Tabs ── */
.tabs {
  display: flex;
  gap: 0;
  border-bottom: 1px solid #E5E2DE;
  margin-bottom: var(--spacing-lg);
}

.tab-btn {
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
  background: none;
  border: none;
  border-bottom: 2px solid transparent;
  padding: 10px 20px;
  cursor: pointer;
  margin-bottom: -1px;
  transition: color 0.15s, border-color 0.15s;
}

.tab-btn:hover { color: var(--color-primary); }

.tab-btn--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
  font-weight: 500;
}

/* ── Section ── */
.section { max-width: 720px; }

/* ── Fields ── */
.field-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
  margin-bottom: 16px;
}

.field { display: flex; flex-direction: column; gap: 5px; }
.field--grow { flex: 1; }
.field--checkbox { justify-content: flex-end; padding-bottom: 4px; }

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
.field-textarea { resize: vertical; }

select.field-input {
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6' fill='none'%3E%3Cpath d='M1 1l4 4 4-4' stroke='%236C7278' stroke-width='1.5' stroke-linecap='round' stroke-linejoin='round'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 10px center;
  padding-right: 28px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
  cursor: pointer;
}

/* ── Buttons ── */
.btn-primary {
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  color: #fff;
  background-color: var(--color-tertiary);
  border: none;
  border-radius: var(--rounded-md);
  padding: 9px 20px;
  cursor: pointer;
  transition: opacity 0.15s;
}

.btn-primary:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-primary:not(:disabled):hover { opacity: 0.88; }
.btn--self-end { align-self: flex-end; }

.btn-danger-sm {
  font-family: var(--font-body);
  font-size: 0.8rem;
  color: var(--color-tertiary);
  background: none;
  border: 1px solid currentColor;
  border-radius: var(--rounded-sm);
  padding: 4px 10px;
  cursor: pointer;
  opacity: 0.7;
  transition: opacity 0.15s;
}

.btn-danger-sm:hover { opacity: 1; }

/* ── Cards ── */
.card-list { display: flex; flex-direction: column; gap: 10px; margin-bottom: var(--spacing-md); }

.card {
  background: var(--color-surface);
  border: 1px solid #E5E2DE;
  border-radius: var(--rounded-md);
  padding: 20px;
}

.card--row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 20px;
}

.card-title {
  font-family: var(--font-display);
  font-size: 1rem;
  font-weight: 500;
  color: var(--color-primary);
  margin-bottom: 14px;
}

.card-primary {
  font-family: var(--font-body);
  font-size: 0.9rem;
  font-weight: 500;
  color: var(--color-primary);
}

.card-sub {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
  margin-top: 2px;
}

.card-actions { display: flex; align-items: center; gap: 10px; }

.badge-primary {
  font-family: var(--font-label);
  font-size: 0.6rem;
  letter-spacing: 0.06em;
  text-transform: uppercase;
  padding: 2px 8px;
  border-radius: var(--rounded-sm);
  background: #D5EED5;
  color: #2A6A2A;
}

/* ── Document slots ── */
.doc-slots { display: flex; flex-direction: column; gap: 16px; }

.doc-slot {
  border: 1px solid #E5E2DE;
  border-radius: var(--rounded-md);
  padding: 16px;
}

.doc-slot-label {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: 8px;
}

.doc-slot-files { display: flex; flex-direction: column; gap: 6px; }

.doc-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-primary);
}

.doc-name { flex: 1; }

.doc-remove {
  font-family: var(--font-body);
  font-size: 0.8rem;
  color: var(--color-tertiary);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  opacity: 0.7;
}

.doc-remove:hover { opacity: 1; }

.doc-empty {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-secondary);
}

/* ── Upload control ── */
.doc-upload { margin-top: 10px; }

.upload-btn {
  display: inline-block;
  font-family: var(--font-body);
  font-size: 0.8125rem;
  font-weight: 500;
  color: var(--color-primary);
  background: #F5F3F0;
  border: 1px solid #E0DED9;
  border-radius: var(--rounded-sm);
  padding: 6px 14px;
  cursor: pointer;
  transition: background 0.15s, opacity 0.15s;
}

.upload-btn:hover { background: #EDE9E3; }
.upload-btn--busy { opacity: 0.55; cursor: not-allowed; }

/* Hide the native file input visually */
.upload-input {
  position: absolute;
  width: 1px;
  height: 1px;
  opacity: 0;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
}

.progress-wrap {
  margin-top: 8px;
  height: 4px;
  background: #E5E2DE;
  border-radius: 2px;
  overflow: hidden;
}

.progress-bar {
  height: 100%;
  background: var(--color-primary);
  border-radius: 2px;
  transition: width 0.15s linear;
}

.upload-error {
  font-family: var(--font-body);
  font-size: 0.8rem;
  color: var(--color-tertiary);
  margin-top: 6px;
}

.form-row {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: flex-end;
}

/* ── Feedback ── */
.form-error {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: var(--color-tertiary);
  margin-bottom: 10px;
}

.form-error--inline { width: 100%; margin-bottom: 0; }

.form-success {
  font-family: var(--font-body);
  font-size: 0.8125rem;
  color: #2A6A2A;
  margin-bottom: 10px;
}

/* ── State ── */
.state-msg { font-family: var(--font-body); color: var(--color-secondary); }
.state-msg--error { color: var(--color-tertiary); }
</style>
