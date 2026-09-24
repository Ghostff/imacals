<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import ProductCard from '@/components/ProductCard.vue';
import { catalogService, type Category, type Product } from '@/services/catalog';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

type SortKey = 'featured' | 'name-asc' | 'price-asc' | 'price-desc';
type PriceInput = number | '';

const products: Ref<Product[]>          = ref<Product[]>([]);
const categories: Ref<Category[]>       = ref<Category[]>([]);
const activeCategory: Ref<string>       = ref<string>('');
const search: Ref<string>               = ref<string>('');
// Empty string = no constraint; a number means "filter to this naira amount".
const minPriceNaira: Ref<PriceInput>    = ref<PriceInput>('');
const maxPriceNaira: Ref<PriceInput>    = ref<PriceInput>('');
const sortBy: Ref<SortKey>              = ref<SortKey>('featured');
const loading: Ref<boolean>             = ref<boolean>(true);
const error: Ref<string | null>         = ref<string | null>(null);

// Filtering runs client-side over the loaded page so typing does not fire a request per keystroke.
const visible: ComputedRef<Product[]> = computed<Product[]>(() => {
  const term = search.value.trim().toLowerCase();
  const minKobo = typeof minPriceNaira.value === 'number' ? minPriceNaira.value * 100 : null;
  const maxKobo = typeof maxPriceNaira.value === 'number' ? maxPriceNaira.value * 100 : null;

  const filtered = products.value.filter((p) => {
    const matchesCategory = !activeCategory.value || p.category_slug === activeCategory.value;
    const matchesTerm = !term
      || p.name.toLowerCase().includes(term)
      || p.description.toLowerCase().includes(term);
    const matchesMin = minKobo === null || p.unit_price_kobo >= minKobo;
    const matchesMax = maxKobo === null || p.unit_price_kobo <= maxKobo;
    return matchesCategory && matchesTerm && matchesMin && matchesMax;
  });

  if (sortBy.value === 'featured') return filtered;

  const sorted = [...filtered];
  switch (sortBy.value) {
    case 'name-asc':    sorted.sort((a, b) => a.name.localeCompare(b.name)); break;
    case 'price-asc':   sorted.sort((a, b) => a.unit_price_kobo - b.unit_price_kobo); break;
    case 'price-desc':  sorted.sort((a, b) => b.unit_price_kobo - a.unit_price_kobo); break;
  }
  return sorted;
});

onMounted(async () => {
  try {
    const [productResult, categoryResult] = await Promise.all([
      catalogService.listProducts(),
      catalogService.listCategories().catch(() => [] as Category[]),
    ]);
    products.value   = productResult;
    categories.value = categoryResult;
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Could not load the catalogue.';
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <div class="page">
    <header class="head">
      <div>
        <p class="eyebrow">Catalogue</p>
        <h1 class="section-title">Everything in the Aba warehouse</h1>
      </div>
      <input
        v-model="search"
        class="field-input search"
        type="search"
        placeholder="Search products…"
        aria-label="Search products"
      />
    </header>

    <nav v-if="categories.length" class="filters" aria-label="Filters">
      <button
        class="filter"
        :class="{ 'filter--active': activeCategory === '' }"
        type="button"
        @click="activeCategory = ''"
      >
        All
      </button>
      <button
        v-for="c in categories"
        :key="c.slug"
        class="filter"
        :class="{ 'filter--active': activeCategory === c.slug }"
        type="button"
        @click="activeCategory = c.slug"
      >
        {{ c.name }}
      </button>

      <div class="filter-group" role="group" aria-label="Price range">
        <label class="filter-label" for="price-min">Price ₦</label>
        <input
          id="price-min"
          v-model.number="minPriceNaira"
          type="number"
          min="0"
          step="100"
          placeholder="Min"
          class="price-input"
          aria-label="Minimum price in naira"
        />
        <span class="filter-dash" aria-hidden="true">–</span>
        <input
          v-model.number="maxPriceNaira"
          type="number"
          min="0"
          step="100"
          placeholder="Max"
          class="price-input"
          aria-label="Maximum price in naira"
        />
      </div>

      <div class="filter-group" role="group" aria-label="Sort">
        <label class="filter-label" for="sort">Sort</label>
        <select id="sort" v-model="sortBy" class="sort-select" aria-label="Sort products">
          <option value="featured">Featured</option>
          <option value="name-asc">Name A–Z</option>
          <option value="price-asc">Price: Low to High</option>
          <option value="price-desc">Price: High to Low</option>
        </select>
      </div>
    </nav>

    <p v-if="loading" class="state-msg">Loading the catalogue…</p>
    <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>
    <p v-else-if="!products.length" class="state-msg">
      The catalogue is empty right now. Call {{ SITE.orderLine }} and we will tell you what is in.
    </p>
    <p v-else-if="!visible.length" class="state-msg">
      Nothing matches those filters. Call {{ SITE.orderLine }} and we will check the warehouse for you.
    </p>

    <div v-else class="grid">
      <ProductCard v-for="p in visible" :key="p.id" :product="p" />
    </div>
  </div>
</template>

<style scoped>
.head {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

.search {
  max-width: 280px;
}

.filters {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
  margin-bottom: var(--spacing-lg);
}

.filter {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  background: transparent;
  color: var(--color-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 7px 12px;
  cursor: pointer;
}

.filter:hover {
  color: var(--color-primary);
}

.filter--active {
  color: var(--color-on-primary);
  background-color: var(--color-primary);
  border-color: var(--color-primary);
}

.filter-group {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  padding-left: var(--spacing-md);
  border-left: 1px solid var(--color-divider);
  margin-left: var(--spacing-sm);
}

.filter-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}

.filter-dash {
  font-family: var(--font-label);
  color: var(--color-secondary);
}

.price-input,
.sort-select {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 7px 10px;
  cursor: pointer;
}

.price-input {
  width: 90px;
  cursor: text;
}

.price-input:focus,
.sort-select:focus {
  outline: 2px solid var(--color-tertiary);
  outline-offset: -1px;
}

@media (max-width: 600px) {
  .filter-group {
    border-left: none;
    padding-left: 0;
    margin-left: 0;
  }
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--spacing-md);
}
</style>
