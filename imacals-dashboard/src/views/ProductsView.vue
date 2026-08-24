<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import {
  productService,
  formatNaira,
  type Product,
  type CreateProductPayload,
  type UpdateProductPayload,
} from '@/services/product';
import { categoryService, type Category } from '@/services/category';
import { ApiException } from '@/services/api';

const products: Ref<Product[]>     = ref([]);
const categories: Ref<Category[]> = ref([]);
const loading: Ref<boolean>        = ref(true);
const error: Ref<string | null>    = ref(null);

const search: Ref<string>          = ref('');
const selectedCategory: Ref<string> = ref('all');

const STANDARD_UNITS: string[] = [
  'bag (50kg)',
  'bag (25kg)',
  'bag (20)',
  'carton (24)',
  'carton (48)',
  'keg (25L)',
  'keg (10L)',
  'crate (24)',
  'pack (12)',
  'piece',
];

// ── Modals state ─────────────────────────────────────────────────────────────
const showAddModal: Ref<boolean>       = ref(false);
const showEditModal: Ref<boolean>      = ref(false);
const showCategoryModal: Ref<boolean>  = ref(false);
const showDeleteModal: Ref<boolean>    = ref(false);

const submitting: Ref<boolean>         = ref(false);
const modalError: Ref<string | null>   = ref(null);

// Selected product for edit / delete
const activeProduct: Ref<Product | null> = ref(null);

// Form state for creating product
interface ProductForm {
  name: string;
  slug: string;
  category_id: string;
  unit: string;
  price_naira: number | null;
  min_order_quantity: number;
  in_stock: boolean;
  description: string;
  image_file: File | null;
  image_preview: string | null;
}

const addForm: Ref<ProductForm> = ref({
  name: '',
  slug: '',
  category_id: '',
  unit: 'bag (50kg)',
  price_naira: null,
  min_order_quantity: 1,
  in_stock: true,
  description: '',
  image_file: null,
  image_preview: null,
});

const editForm: Ref<ProductForm> = ref({
  name: '',
  slug: '',
  category_id: '',
  unit: '',
  price_naira: null,
  min_order_quantity: 1,
  in_stock: true,
  description: '',
  image_file: null,
  image_preview: null,
});

// Category quick create
const categoryName: Ref<string>        = ref('');
const categorySlug: Ref<string>        = ref('');
const categoryDescription: Ref<string> = ref('');

// ── Helpers ──────────────────────────────────────────────────────────────────
function slugify(text: string): string {
  return text
    .toLowerCase()
    .trim()
    .replace(/[^\w\s-]/g, '')
    .replace(/[\s_-]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function onAddNameChange(): void {
  if (!addForm.value.slug || addForm.value.slug === slugify(addForm.value.name.slice(0, -1))) {
    addForm.value.slug = slugify(addForm.value.name);
  }
}

function onAddCategoryNameChange(): void {
  categorySlug.value = slugify(categoryName.value);
}

function onImageSelect(event: Event, isEdit: boolean): void {
  const input = event.target as HTMLInputElement;
  if (!input.files || input.files.length === 0) return;
  const file = input.files[0];
  const preview = URL.createObjectURL(file);

  if (isEdit) {
    editForm.value.image_file = file;
    editForm.value.image_preview = preview;
  } else {
    addForm.value.image_file = file;
    addForm.value.image_preview = preview;
  }
}

// ── Filtering ────────────────────────────────────────────────────────────────
const filteredProducts: ComputedRef<Product[]> = computed<Product[]>(() => {
  const q = search.value.trim().toLowerCase();
  return products.value.filter((p) => {
    if (q) {
      const matchName = p.name.toLowerCase().includes(q);
      const matchSlug = p.slug.toLowerCase().includes(q);
      const matchCat  = p.category_name.toLowerCase().includes(q);
      if (!matchName && !matchSlug && !matchCat) return false;
    }
    if (selectedCategory.value !== 'all' && p.category_id !== selectedCategory.value) {
      return false;
    }
    return true;
  });
});

// ── Data loading ─────────────────────────────────────────────────────────────
async function loadData(): Promise<void> {
  loading.value = true;
  error.value = null;
  try {
    const [prods, cats] = await Promise.all([
      productService.index(),
      categoryService.index(),
    ]);
    products.value = prods;
    categories.value = cats;
    if (cats.length > 0 && !addForm.value.category_id) {
      addForm.value.category_id = cats[0].id;
    }
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error ? e.message : 'Could not load products.';
  } finally {
    loading.value = false;
  }
}

onMounted(loadData);

// ── Modal Actions ────────────────────────────────────────────────────────────
function openAddModal(): void {
  addForm.value = {
    name: '',
    slug: '',
    category_id: categories.value[0]?.id ?? '',
    unit: 'bag (50kg)',
    price_naira: null,
    min_order_quantity: 1,
    in_stock: true,
    description: '',
    image_file: null,
    image_preview: null,
  };
  modalError.value = null;
  showAddModal.value = true;
}

function openEditModal(prod: Product): void {
  activeProduct.value = prod;
  editForm.value = {
    name: prod.name,
    slug: prod.slug,
    category_id: prod.category_id,
    unit: prod.unit,
    price_naira: Math.round(prod.unit_price_kobo / 100),
    min_order_quantity: prod.min_order_quantity,
    in_stock: prod.in_stock,
    description: prod.description ?? '',
    image_file: null,
    image_preview: prod.image_url,
  };
  modalError.value = null;
  showEditModal.value = true;
}

function openDeleteModal(prod: Product): void {
  activeProduct.value = prod;
  modalError.value = null;
  showDeleteModal.value = true;
}

function openCategoryModal(): void {
  categoryName.value = '';
  categorySlug.value = '';
  categoryDescription.value = '';
  modalError.value = null;
  showCategoryModal.value = true;
}

// ── Submit Handlers ──────────────────────────────────────────────────────────
async function submitAddProduct(): Promise<void> {
  modalError.value = null;
  const f = addForm.value;

  if (!f.name.trim()) {
    modalError.value = 'Product name is required';
    return;
  }
  if (!f.slug.trim()) {
    modalError.value = 'Product slug is required';
    return;
  }
  if (!f.category_id) {
    modalError.value = 'Please select a category';
    return;
  }
  if (!f.unit.trim()) {
    modalError.value = 'Unit is required';
    return;
  }
  if (!f.price_naira || f.price_naira <= 0) {
    modalError.value = 'Price must be greater than zero Naira';
    return;
  }

  submitting.value = true;
  try {
    const payload: CreateProductPayload = {
      name: f.name.trim(),
      slug: f.slug.trim(),
      category_id: f.category_id,
      unit: f.unit.trim(),
      unit_price_kobo: Math.round(f.price_naira * 100),
      min_order_quantity: Number(f.min_order_quantity) || 1,
      in_stock: f.in_stock,
      description: f.description.trim() || undefined,
    };

    let created = await productService.create(payload);

    // If an image was chosen, upload it now
    if (f.image_file) {
      created = await productService.uploadImage(created.id, f.image_file);
    }

    products.value = [created, ...products.value];
    showAddModal.value = false;
  } catch (e: unknown) {
    modalError.value = e instanceof ApiException || e instanceof Error ? e.message : 'Failed to create product.';
  } finally {
    submitting.value = false;
  }
}

async function submitEditProduct(): Promise<void> {
  if (!activeProduct.value) return;
  modalError.value = null;
  const f = editForm.value;

  if (!f.name.trim()) {
    modalError.value = 'Product name is required';
    return;
  }
  if (!f.slug.trim()) {
    modalError.value = 'Product slug is required';
    return;
  }
  if (!f.unit.trim()) {
    modalError.value = 'Unit is required';
    return;
  }
  if (!f.price_naira || f.price_naira <= 0) {
    modalError.value = 'Price must be greater than zero Naira';
    return;
  }

  submitting.value = true;
  try {
    const payload: UpdateProductPayload = {
      name: f.name.trim(),
      slug: f.slug.trim(),
      category_id: f.category_id,
      unit: f.unit.trim(),
      unit_price_kobo: Math.round(f.price_naira * 100),
      min_order_quantity: Number(f.min_order_quantity) || 1,
      in_stock: f.in_stock,
      description: f.description.trim() || undefined,
    };

    let updated = await productService.update(activeProduct.value.id, payload);

    if (f.image_file) {
      updated = await productService.uploadImage(activeProduct.value.id, f.image_file);
    }

    products.value = products.value.map((p) => (p.id === updated.id ? updated : p));
    showEditModal.value = false;
  } catch (e: unknown) {
    modalError.value = e instanceof ApiException || e instanceof Error ? e.message : 'Failed to update product.';
  } finally {
    submitting.value = false;
  }
}

async function toggleStockStatus(prod: Product): Promise<void> {
  const newStatus = !prod.in_stock;
  try {
    const updated = await productService.update(prod.id, { in_stock: newStatus });
    products.value = products.value.map((p) => (p.id === updated.id ? updated : p));
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error ? e.message : 'Could not update stock status.';
  }
}

async function submitDelete(): Promise<void> {
  if (!activeProduct.value) return;
  submitting.value = true;
  try {
    await productService.delete(activeProduct.value.id);
    products.value = products.value.filter((p) => p.id !== activeProduct.value?.id);
    showDeleteModal.value = false;
  } catch (e: unknown) {
    modalError.value = e instanceof ApiException || e instanceof Error ? e.message : 'Failed to delete product.';
  } finally {
    submitting.value = false;
  }
}

async function submitAddCategory(): Promise<void> {
  modalError.value = null;
  if (!categoryName.value.trim()) {
    modalError.value = 'Category name is required';
    return;
  }
  if (!categorySlug.value.trim()) {
    modalError.value = 'Category slug is required';
    return;
  }

  submitting.value = true;
  try {
    const created = await categoryService.create({
      name: categoryName.value.trim(),
      slug: categorySlug.value.trim(),
      description: categoryDescription.value.trim() || undefined,
    });
    categories.value = [...categories.value, created];
    addForm.value.category_id = created.id;
    showCategoryModal.value = false;
  } catch (e: unknown) {
    modalError.value = e instanceof ApiException || e instanceof Error ? e.message : 'Failed to create category.';
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="products-view">
    <!-- Header -->
    <div class="header-row">
      <div>
        <p class="eyebrow">Catalogue & Inventory</p>
        <h1 class="page-title">Products</h1>
      </div>
      <div class="header-actions">
        <button class="btn-secondary" type="button" @click="openCategoryModal">
          + Add Category
        </button>
        <!-- The single Tertiary action on this screen -->
        <button class="btn-primary" type="button" @click="openAddModal">
          + Add Product
        </button>
      </div>
    </div>

    <!-- Filters & Search -->
    <div class="toolbar">
      <div class="search-box">
        <input
          v-model="search"
          type="search"
          class="field-input"
          placeholder="Search products by name or slug…"
          aria-label="Search products"
        />
      </div>
      <div class="filter-box">
        <select v-model="selectedCategory" class="field-select" aria-label="Filter by category">
          <option value="all">All Categories ({{ categories.length }})</option>
          <option v-for="cat in categories" :key="cat.id" :value="cat.id">
            {{ cat.name }}
          </option>
        </select>
      </div>
    </div>

    <!-- State Messages -->
    <p v-if="loading" class="state-msg">Loading products…</p>
    <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>

    <div v-else-if="filteredProducts.length === 0" class="empty-card">
      <p class="empty-title">No products found</p>
      <p class="empty-desc">
        {{ search || selectedCategory !== 'all' ? 'Try adjusting your search or category filter.' : 'Get started by uploading your first product to the Aba warehouse catalogue.' }}
      </p>
      <button v-if="!search && selectedCategory === 'all'" class="btn-primary" type="button" @click="openAddModal">
        + Add Product
      </button>
    </div>

    <!-- Products Table -->
    <div v-else class="table-container">
      <table class="data-table">
        <thead>
          <tr>
            <th class="th-img">Image</th>
            <th>Product Name</th>
            <th>Category</th>
            <th>Unit</th>
            <th>Price</th>
            <th>MOQ</th>
            <th>Stock Status</th>
            <th class="th-actions">Actions</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="prod in filteredProducts" :key="prod.id">
            <td class="td-img">
              <div class="thumb-wrapper">
                <img v-if="prod.image_url" :src="prod.image_url" :alt="prod.name" class="thumb-img" />
                <span v-else class="thumb-placeholder">{{ prod.name.charAt(0).toUpperCase() }}</span>
              </div>
            </td>
            <td>
              <div class="prod-title">{{ prod.name }}</div>
              <div class="prod-slug">{{ prod.slug }}</div>
            </td>
            <td>
              <span class="badge-cat">{{ prod.category_name }}</span>
            </td>
            <td class="prod-unit">{{ prod.unit }}</td>
            <td class="prod-price">{{ formatNaira(prod.unit_price_kobo) }}</td>
            <td>{{ prod.min_order_quantity }}</td>
            <td>
              <button
                class="badge-stock"
                :class="prod.in_stock ? 'badge-stock--in' : 'badge-stock--out'"
                type="button"
                :title="'Click to mark as ' + (prod.in_stock ? 'out of stock' : 'in stock')"
                @click="toggleStockStatus(prod)"
              >
                {{ prod.in_stock ? '● In Stock' : '○ Out of Stock' }}
              </button>
            </td>
            <td class="td-actions">
              <button class="action-link" type="button" @click="openEditModal(prod)">
                Edit
              </button>
              <span class="action-divider">|</span>
              <button class="action-link action-link--danger" type="button" @click="openDeleteModal(prod)">
                Delete
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- ── Add Product Modal ───────────────────────────────────────────────── -->
    <div v-if="showAddModal" class="modal-backdrop" @click.self="showAddModal = false">
      <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="add-title">
        <header class="modal-head">
          <h2 id="add-title" class="modal-title">Add New Product</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="showAddModal = false">✕</button>
        </header>

        <form class="modal-body" @submit.prevent="submitAddProduct">
          <p v-if="modalError" class="modal-error">{{ modalError }}</p>

          <div class="form-row">
            <div class="form-group flex-2">
              <label class="form-label" for="add-name">Product Name *</label>
              <input
                id="add-name"
                v-model="addForm.name"
                type="text"
                class="field-input"
                placeholder="e.g. Long Grain Rice — 50kg Bag"
                required
                @input="onAddNameChange"
              />
            </div>
            <div class="form-group flex-1">
              <label class="form-label" for="add-slug">Slug *</label>
              <input
                id="add-slug"
                v-model="addForm.slug"
                type="text"
                class="field-input"
                placeholder="rice-50kg"
                required
              />
            </div>
          </div>

          <div class="form-row">
            <div class="form-group flex-1">
              <label class="form-label" for="add-cat">Category *</label>
              <select id="add-cat" v-model="addForm.category_id" class="field-select" required>
                <option v-for="cat in categories" :key="cat.id" :value="cat.id">
                  {{ cat.name }}
                </option>
              </select>
            </div>
            <div class="form-group flex-1">
              <label class="form-label" for="add-unit">Sold By (Unit) *</label>
              <input
                id="add-unit"
                v-model="addForm.unit"
                type="text"
                list="unit-options"
                class="field-input"
                placeholder="e.g. bag (50kg), carton (24)"
                required
              />
              <datalist id="unit-options">
                <option v-for="u in STANDARD_UNITS" :key="u" :value="u" />
              </datalist>
            </div>
          </div>

          <div class="form-row">
            <div class="form-group flex-1">
              <label class="form-label" for="add-price">Price (₦ Naira) *</label>
              <input
                id="add-price"
                v-model.number="addForm.price_naira"
                type="number"
                min="1"
                step="1"
                class="field-input"
                placeholder="e.g. 89500"
                required
              />
            </div>
            <div class="form-group flex-1">
              <label class="form-label" for="add-moq">Min Order Qty *</label>
              <input
                id="add-moq"
                v-model.number="addForm.min_order_quantity"
                type="number"
                min="1"
                step="1"
                class="field-input"
                required
              />
            </div>
          </div>

          <div class="form-group">
            <label class="form-label" for="add-desc">Description</label>
            <textarea
              id="add-desc"
              v-model="addForm.description"
              rows="3"
              class="field-input field-textarea"
              placeholder="Detailed specifications, brand, packaging notes…"
            ></textarea>
          </div>

          <div class="form-group">
            <label class="form-label" for="add-img">Product Image</label>
            <input
              id="add-img"
              type="file"
              accept="image/*"
              class="file-input"
              @change="(e) => onImageSelect(e, false)"
            />
            <div v-if="addForm.image_preview" class="image-preview-box">
              <img :src="addForm.image_preview" alt="Preview" class="preview-img" />
              <span class="preview-label">Image Selected</span>
            </div>
          </div>

          <div class="form-group-checkbox">
            <label class="checkbox-label">
              <input v-model="addForm.in_stock" type="checkbox" />
              <span>Available in stock at the Aba warehouse</span>
            </label>
          </div>

          <footer class="modal-foot">
            <button class="btn-secondary" type="button" @click="showAddModal = false">
              Cancel
            </button>
            <button class="btn-primary" type="submit" :disabled="submitting">
              {{ submitting ? 'Uploading…' : 'Save Product' }}
            </button>
          </footer>
        </form>
      </div>
    </div>

    <!-- ── Edit Product Modal ──────────────────────────────────────────────── -->
    <div v-if="showEditModal" class="modal-backdrop" @click.self="showEditModal = false">
      <div class="modal-card" role="dialog" aria-modal="true" aria-labelledby="edit-title">
        <header class="modal-head">
          <h2 id="edit-title" class="modal-title">Edit Product</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="showEditModal = false">✕</button>
        </header>

        <form class="modal-body" @submit.prevent="submitEditProduct">
          <p v-if="modalError" class="modal-error">{{ modalError }}</p>

          <div class="form-row">
            <div class="form-group flex-2">
              <label class="form-label" for="edit-name">Product Name *</label>
              <input
                id="edit-name"
                v-model="editForm.name"
                type="text"
                class="field-input"
                required
              />
            </div>
            <div class="form-group flex-1">
              <label class="form-label" for="edit-slug">Slug *</label>
              <input
                id="edit-slug"
                v-model="editForm.slug"
                type="text"
                class="field-input"
                required
              />
            </div>
          </div>

          <div class="form-row">
            <div class="form-group flex-1">
              <label class="form-label" for="edit-cat">Category *</label>
              <select id="edit-cat" v-model="editForm.category_id" class="field-select" required>
                <option v-for="cat in categories" :key="cat.id" :value="cat.id">
                  {{ cat.name }}
                </option>
              </select>
            </div>
            <div class="form-group flex-1">
              <label class="form-label" for="edit-unit">Sold By (Unit) *</label>
              <input
                id="edit-unit"
                v-model="editForm.unit"
                type="text"
                list="edit-unit-options"
                class="field-input"
                required
              />
              <datalist id="edit-unit-options">
                <option v-for="u in STANDARD_UNITS" :key="u" :value="u" />
              </datalist>
            </div>
          </div>

          <div class="form-row">
            <div class="form-group flex-1">
              <label class="form-label" for="edit-price">Price (₦ Naira) *</label>
              <input
                id="edit-price"
                v-model.number="editForm.price_naira"
                type="number"
                min="1"
                step="1"
                class="field-input"
                required
              />
            </div>
            <div class="form-group flex-1">
              <label class="form-label" for="edit-moq">Min Order Qty *</label>
              <input
                id="edit-moq"
                v-model.number="editForm.min_order_quantity"
                type="number"
                min="1"
                step="1"
                class="field-input"
                required
              />
            </div>
          </div>

          <div class="form-group">
            <label class="form-label" for="edit-desc">Description</label>
            <textarea
              id="edit-desc"
              v-model="editForm.description"
              rows="3"
              class="field-input field-textarea"
            ></textarea>
          </div>

          <div class="form-group">
            <label class="form-label" for="edit-img">Replace Product Image</label>
            <input
              id="edit-img"
              type="file"
              accept="image/*"
              class="file-input"
              @change="(e) => onImageSelect(e, true)"
            />
            <div v-if="editForm.image_preview" class="image-preview-box">
              <img :src="editForm.image_preview" alt="Preview" class="preview-img" />
              <span class="preview-label">Image Selected</span>
            </div>
          </div>

          <div class="form-group-checkbox">
            <label class="checkbox-label">
              <input v-model="editForm.in_stock" type="checkbox" />
              <span>Available in stock at the Aba warehouse</span>
            </label>
          </div>

          <footer class="modal-foot">
            <button class="btn-secondary" type="button" @click="showEditModal = false">
              Cancel
            </button>
            <button class="btn-primary" type="submit" :disabled="submitting">
              {{ submitting ? 'Updating…' : 'Save Changes' }}
            </button>
          </footer>
        </form>
      </div>
    </div>

    <!-- ── Add Category Modal ──────────────────────────────────────────────── -->
    <div v-if="showCategoryModal" class="modal-backdrop" @click.self="showCategoryModal = false">
      <div class="modal-card modal-card--sm" role="dialog" aria-modal="true" aria-labelledby="cat-title">
        <header class="modal-head">
          <h2 id="cat-title" class="modal-title">Add Category</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="showCategoryModal = false">✕</button>
        </header>

        <form class="modal-body" @submit.prevent="submitAddCategory">
          <p v-if="modalError" class="modal-error">{{ modalError }}</p>

          <div class="form-group">
            <label class="form-label" for="cat-name">Category Name *</label>
            <input
              id="cat-name"
              v-model="categoryName"
              type="text"
              class="field-input"
              placeholder="e.g. Building Materials"
              required
              @input="onAddCategoryNameChange"
            />
          </div>

          <div class="form-group">
            <label class="form-label" for="cat-slug">Slug *</label>
            <input
              id="cat-slug"
              v-model="categorySlug"
              type="text"
              class="field-input"
              placeholder="building-materials"
              required
            />
          </div>

          <div class="form-group">
            <label class="form-label" for="cat-desc">Description</label>
            <textarea
              id="cat-desc"
              v-model="categoryDescription"
              rows="2"
              class="field-input field-textarea"
              placeholder="Short description of this category…"
            ></textarea>
          </div>

          <footer class="modal-foot">
            <button class="btn-secondary" type="button" @click="showCategoryModal = false">
              Cancel
            </button>
            <button class="btn-primary" type="submit" :disabled="submitting">
              {{ submitting ? 'Saving…' : 'Create Category' }}
            </button>
          </footer>
        </form>
      </div>
    </div>

    <!-- ── Delete Confirmation Modal ───────────────────────────────────────── -->
    <div v-if="showDeleteModal" class="modal-backdrop" @click.self="showDeleteModal = false">
      <div class="modal-card modal-card--sm" role="dialog" aria-modal="true" aria-labelledby="del-title">
        <header class="modal-head">
          <h2 id="del-title" class="modal-title">Delete Product</h2>
          <button class="modal-close" type="button" aria-label="Close" @click="showDeleteModal = false">✕</button>
        </header>

        <div class="modal-body">
          <p v-if="modalError" class="modal-error">{{ modalError }}</p>
          <p class="modal-confirm-msg">
            Are you sure you want to delete <strong>{{ activeProduct?.name }}</strong>?
            This will remove the product from the public catalogue.
          </p>

          <footer class="modal-foot">
            <button class="btn-secondary" type="button" @click="showDeleteModal = false">
              Cancel
            </button>
            <button class="btn-primary btn-primary--danger" type="button" :disabled="submitting" @click="submitDelete">
              {{ submitting ? 'Deleting…' : 'Confirm Delete' }}
            </button>
          </footer>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.products-view {
  max-width: 1200px;
  margin: 0 auto;
  padding: var(--spacing-lg) var(--spacing-md);
}

.header-row {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.eyebrow {
  font-family: var(--font-label);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-secondary);
  margin-bottom: 4px;
}

.page-title {
  font-family: var(--font-display);
  font-size: 2rem;
  font-weight: 500;
  color: var(--color-primary);
  letter-spacing: -0.02em;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
}

.toolbar {
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
  margin-bottom: var(--spacing-lg);
}

.search-box {
  flex: 1;
  min-width: 260px;
}

.filter-box {
  min-width: 200px;
}

/* ── Buttons ────────────────────────────────────────────────────────────── */
.btn-primary {
  background-color: var(--color-tertiary);
  color: var(--color-on-primary);
  border: none;
  padding: 8px 16px;
  border-radius: var(--rounded-md);
  font-family: var(--font-body);
  font-size: 0.875rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity 0.1s;
}

.btn-primary:hover {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary--danger {
  background-color: #d32f2f;
}

.btn-secondary {
  background: transparent;
  color: var(--color-primary);
  border: 1px solid var(--color-border);
  padding: 8px 14px;
  border-radius: var(--rounded-md);
  font-family: var(--font-body);
  font-size: 0.875rem;
  cursor: pointer;
  transition: background-color 0.1s;
}

.btn-secondary:hover {
  background-color: var(--color-neutral);
}

/* ── Inputs ─────────────────────────────────────────────────────────────── */
.field-input,
.field-select {
  width: 100%;
  padding: 8px 12px;
  background-color: var(--color-surface);
  color: var(--color-primary);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  font-family: var(--font-body);
  font-size: 0.875rem;
  outline: none;
}

.field-input:focus,
.field-select:focus {
  border-color: var(--color-primary);
}

.field-textarea {
  resize: vertical;
}

.file-input {
  display: block;
  width: 100%;
  font-size: 0.875rem;
  color: var(--color-secondary);
}

/* ── Table ──────────────────────────────────────────────────────────────── */
.table-container {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  overflow: hidden;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  text-align: left;
}

.data-table th {
  background-color: var(--color-neutral);
  padding: 12px 16px;
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--color-secondary);
  border-bottom: 1px solid var(--color-border);
}

.data-table td {
  padding: 12px 16px;
  border-bottom: 1px solid var(--color-divider);
  font-size: 0.875rem;
  vertical-align: middle;
}

.data-table tbody tr:last-child td {
  border-bottom: none;
}

.data-table tbody tr:hover {
  background-color: color-mix(in srgb, var(--color-neutral) 40%, transparent);
}

.th-img, .td-img {
  width: 54px;
}

.thumb-wrapper {
  width: 44px;
  height: 44px;
  border-radius: var(--rounded-md);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.thumb-placeholder {
  font-family: var(--font-display);
  font-size: 1.25rem;
  color: var(--color-secondary);
}

.prod-title {
  font-weight: 500;
  color: var(--color-primary);
}

.prod-slug {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.badge-cat {
  display: inline-block;
  padding: 3px 8px;
  border-radius: var(--rounded-sm);
  background-color: var(--color-neutral);
  border: 1px solid var(--color-border);
  font-family: var(--font-label);
  font-size: 0.6875rem;
  letter-spacing: 0.02em;
}

.prod-unit {
  color: var(--color-secondary);
}

.prod-price {
  font-family: var(--font-label);
  font-weight: 500;
}

.badge-stock {
  border: none;
  background: none;
  cursor: pointer;
  font-family: var(--font-label);
  font-size: 0.75rem;
  padding: 4px 8px;
  border-radius: var(--rounded-sm);
  transition: opacity 0.1s;
}

.badge-stock--in {
  background-color: #e8f5e9;
  color: #2e7d32;
}

.badge-stock--out {
  background-color: #ffebee;
  color: #c62828;
}

.th-actions, .td-actions {
  text-align: right;
  white-space: nowrap;
}

.action-link {
  background: none;
  border: none;
  color: var(--color-primary);
  font-size: 0.8125rem;
  cursor: pointer;
  padding: 2px 4px;
}

.action-link:hover {
  text-decoration: underline;
}

.action-link--danger {
  color: #d32f2f;
}

.action-divider {
  margin: 0 4px;
  color: var(--color-divider);
}

/* ── States ─────────────────────────────────────────────────────────────── */
.state-msg {
  padding: var(--spacing-lg);
  text-align: center;
  color: var(--color-secondary);
}

.state-msg--error {
  color: #d32f2f;
}

.empty-card {
  text-align: center;
  padding: var(--spacing-lg) var(--spacing-md);
  background-color: var(--color-surface);
  border: 1px dashed var(--color-border);
  border-radius: var(--rounded-lg);
}

.empty-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  margin-bottom: 6px;
}

.empty-desc {
  color: var(--color-secondary);
  font-size: 0.875rem;
  max-width: 44ch;
  margin: 0 auto var(--spacing-md);
}

/* ── Modals ─────────────────────────────────────────────────────────────── */
.modal-backdrop {
  position: fixed;
  inset: 0;
  background-color: var(--color-overlay);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: var(--spacing-md);
}

.modal-card {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  width: 100%;
  max-width: 600px;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.modal-card--sm {
  max-width: 440px;
}

.modal-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--spacing-md) var(--spacing-lg);
  border-bottom: 1px solid var(--color-border);
}

.modal-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
}

.modal-close {
  background: none;
  border: none;
  font-size: 1rem;
  color: var(--color-secondary);
  cursor: pointer;
}

.modal-body {
  padding: var(--spacing-lg);
}

.modal-error {
  padding: 8px 12px;
  background-color: #ffebee;
  color: #c62828;
  border-radius: var(--rounded-md);
  font-size: 0.8125rem;
  margin-bottom: var(--spacing-md);
}

.modal-confirm-msg {
  font-size: 0.875rem;
  color: var(--color-secondary);
  line-height: 1.5;
  margin-bottom: var(--spacing-lg);
}

.form-row {
  display: flex;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.flex-1 { flex: 1; }
.flex-2 { flex: 2; }

.form-group {
  margin-bottom: var(--spacing-md);
}

.form-label {
  display: block;
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  margin-bottom: 4px;
}

.form-group-checkbox {
  margin: var(--spacing-md) 0;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.875rem;
  cursor: pointer;
}

.image-preview-box {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  margin-top: 8px;
}

.preview-img {
  width: 50px;
  height: 50px;
  object-fit: cover;
  border-radius: var(--rounded-md);
  border: 1px solid var(--color-border);
}

.preview-label {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.modal-foot {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-lg);
  padding-top: var(--spacing-md);
  border-top: 1px solid var(--color-divider);
}
</style>
