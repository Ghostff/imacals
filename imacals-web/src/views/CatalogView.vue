<script setup lang="ts">
import { ref, computed, onMounted, type Ref, type ComputedRef } from 'vue';
import ProductCard from '@/components/ProductCard.vue';
import { catalogService, type Category, type Product } from '@/services/catalog';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

const products: Ref<Product[]>   = ref([]);
const categories: Ref<Category[]> = ref([]);
const activeCategory: Ref<string> = ref('');
const search: Ref<string>         = ref('');
const loading: Ref<boolean>       = ref(true);
const error: Ref<string | null>   = ref(null);

// Filtering runs client-side over the loaded page so typing does not fire a request per keystroke.
const visible: ComputedRef<Product[]> = computed<Product[]>(() => {
  const term = search.value.trim().toLowerCase();
  return products.value.filter((p) => {
    const matchesCategory = !activeCategory.value || p.category_slug === activeCategory.value;
    const matchesTerm = !term
      || p.name.toLowerCase().includes(term)
      || p.description.toLowerCase().includes(term);
    return matchesCategory && matchesTerm;
  });
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

    <nav v-if="categories.length" class="filters" aria-label="Categories">
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
    </nav>

    <p v-if="loading" class="state-msg">Loading the catalogue…</p>
    <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>
    <p v-else-if="!products.length" class="state-msg">
      The catalogue is empty right now. Call {{ SITE.orderLine }} and we will tell you what is in.
    </p>
    <p v-else-if="!visible.length" class="state-msg">
      Nothing matches that search. Try a different term, or call {{ SITE.orderLine }}.
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

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--spacing-md);
}
</style>
