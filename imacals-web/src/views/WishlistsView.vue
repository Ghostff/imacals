<script setup lang="ts">
import { ref, onMounted, computed, type Ref } from 'vue';
import { RouterLink } from 'vue-router';
import { useWishlist } from '@/composables/useWishlist';
import type { WishlistSummary } from '@/services/wishlist';

const { lists, loaded, loading, refresh, create, rename, remove } = useWishlist();

const loadError: Ref<string | null>      = ref(null);
const creating: Ref<boolean>          = ref(false);
const newName: Ref<string>            = ref('');
const newDescription: Ref<string>    = ref('');
const savingNew: Ref<boolean>         = ref(false);
const createError: Ref<string | null> = ref(null);

const editingId: Ref<string | null> = ref(null);
const editName: Ref<string>         = ref('');
const editDescription: Ref<string>  = ref('');
const savingEdit: Ref<boolean>      = ref(false);

const sorted: Ref<WishlistSummary[]> = computed<WishlistSummary[]>(() =>
  [...lists.value].sort((a, b) => a.name.localeCompare(b.name)),
);

function openCreate(): void {
  creating.value = true;
  newName.value = '';
  newDescription.value = '';
  createError.value = null;
}

function cancelCreate(): void {
  creating.value = false;
  createError.value = null;
}

async function submitCreate(): Promise<void> {
  const trimmed: string = newName.value.trim();
  if (!trimmed) {
    createError.value = 'Give the list a name.';
    return;
  }
  savingNew.value = true;
  createError.value = null;
  try {
    const made = await create(trimmed, newDescription.value.trim() || undefined);
    if (!made) {
      createError.value = 'Could not create the list. Try again.';
    } else {
      creating.value = false;
      newName.value = '';
      newDescription.value = '';
    }
  } finally {
    savingNew.value = false;
  }
}

function startEdit(w: WishlistSummary): void {
  editingId.value = w.id;
  editName.value = w.name;
  editDescription.value = w.description ?? '';
}

function cancelEdit(): void {
  editingId.value = null;
}

async function submitEdit(): Promise<void> {
  if (!editingId.value) return;
  const trimmed: string = editName.value.trim();
  if (!trimmed) return;
  savingEdit.value = true;
  try {
    await rename(editingId.value, trimmed, editDescription.value.trim() || null);
    editingId.value = null;
  } finally {
    savingEdit.value = false;
  }
}

async function confirmRemove(w: WishlistSummary): Promise<void> {
  const ok: boolean = window.confirm(`Delete the list "${w.name}"? Saved products will be removed.`);
  if (!ok) return;
  await remove(w.id);
}

onMounted(async () => {
  loadError.value = null;
  try {
    await refresh();
  } catch (e: unknown) {
    loadError.value = e instanceof Error ? e.message : 'Could not load your wishlists.';
  }
});
</script>

<template>
  <div class="page">
    <header class="head">
      <p class="eyebrow">My Account</p>
      <div class="head-row">
        <h1 class="section-title">Wishlists</h1>
        <button
          v-if="!creating"
          class="btn-primary"
          type="button"
          @click="openCreate"
        >
          New list
        </button>
      </div>
      <p class="lede">
        Save products you want to come back to. Each list has its own link so you can share it with
        a colleague or call the order desk and say "send me everything on my list".
      </p>
    </header>

    <form v-if="creating" class="card create-card" @submit.prevent="submitCreate">
      <h2 class="card-title">New wishlist</h2>

      <div class="field">
        <label class="field-label" for="wl-name">Name</label>
        <input
          id="wl-name"
          v-model="newName"
          class="field-input"
          type="text"
          placeholder="e.g. Stocking the Lagos branch"
          maxlength="120"
          required
        />
      </div>

      <div class="field">
        <label class="field-label" for="wl-desc">Description (optional)</label>
        <input
          id="wl-desc"
          v-model="newDescription"
          class="field-input"
          type="text"
          placeholder="What this list is for"
          maxlength="255"
        />
      </div>

      <p v-if="createError" class="alert alert--error">{{ createError }}</p>

      <div class="form-actions">
        <button class="btn-secondary" type="button" :disabled="savingNew" @click="cancelCreate">
          Cancel
        </button>
        <button class="btn-primary" type="submit" :disabled="savingNew">
          {{ savingNew ? 'Creating…' : 'Create list' }}
        </button>
      </div>
    </form>

    <p v-if="loading && !loaded" class="state-msg">Loading your wishlists…</p>

    <p v-else-if="loadError" class="state-msg state-msg--error">
      {{ loadError }}
    </p>

    <p
      v-else-if="loaded && lists.length === 0 && !creating"
      class="state-msg empty"
    >
      You have no wishlists yet. Start one to save products as you browse the catalogue.
    </p>

    <p
      v-else-if="!loading && createError"
      class="state-msg state-msg--error"
    >
      {{ createError }}
    </p>

    <ul v-else class="lists" role="list">
      <li v-for="w in sorted" :key="w.id" class="list-card card">
        <template v-if="editingId === w.id">
          <h2 class="card-title">Rename list</h2>
          <div class="field">
            <label class="field-label" :for="'edit-name-' + w.id">Name</label>
            <input
              :id="'edit-name-' + w.id"
              v-model="editName"
              class="field-input"
              type="text"
              maxlength="120"
              required
            />
          </div>
          <div class="field">
            <label class="field-label" :for="'edit-desc-' + w.id">Description</label>
            <input
              :id="'edit-desc-' + w.id"
              v-model="editDescription"
              class="field-input"
              type="text"
              maxlength="255"
            />
          </div>
          <div class="form-actions">
            <button class="btn-secondary" type="button" :disabled="savingEdit" @click="cancelEdit">
              Cancel
            </button>
            <button class="btn-primary" type="button" :disabled="savingEdit" @click="submitEdit">
              {{ savingEdit ? 'Saving…' : 'Save' }}
            </button>
          </div>
        </template>

        <template v-else>
          <RouterLink class="list-link" :to="`/wishlists/${w.id}`">
            <h2 class="list-name">{{ w.name }}</h2>
            <p v-if="w.description" class="list-desc">{{ w.description }}</p>
            <p class="list-meta">
              {{ w.item_count }} {{ w.item_count === 1 ? 'item' : 'items' }}
            </p>
          </RouterLink>

          <div class="list-actions">
            <button class="btn-link" type="button" @click="startEdit(w)">Rename</button>
            <button class="btn-link btn-link--danger" type="button" @click="confirmRemove(w)">
              Delete
            </button>
          </div>
        </template>
      </li>
    </ul>
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
  flex-wrap: wrap;
  gap: var(--spacing-md);
  margin-top: var(--spacing-sm);
}

.lede {
  margin-top: var(--spacing-sm);
  color: var(--color-secondary);
  max-width: 60ch;
}

.card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.create-card {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.field {
  display: flex;
  flex-direction: column;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: var(--spacing-sm);
}

.alert {
  padding: 10px 14px;
  border-radius: var(--rounded-md);
  font-size: 0.85rem;
}

.alert--error {
  border: 1px solid var(--color-tertiary);
  color: var(--color-tertiary);
}

.card-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
}

.empty {
  margin-top: var(--spacing-lg);
}

.lists {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--spacing-md);
  list-style: none;
}

.list-card {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.list-link {
  display: flex;
  flex-direction: column;
  gap: 4px;
  text-decoration: none;
  color: inherit;
}

.list-name {
  font-family: var(--font-display);
  font-size: 1.15rem;
  font-weight: 500;
}

.list-desc {
  color: var(--color-secondary);
  font-size: 0.875rem;
}

.list-meta {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  margin-top: 4px;
}

.list-actions {
  display: flex;
  gap: var(--spacing-md);
  margin-top: auto;
  padding-top: var(--spacing-sm);
  border-top: 1px solid var(--color-divider);
}

.btn-link {
  background: none;
  border: none;
  padding: 0;
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  cursor: pointer;
}

.btn-link:hover {
  color: var(--color-tertiary);
}

.btn-link--danger:hover {
  color: var(--color-tertiary);
}
</style>