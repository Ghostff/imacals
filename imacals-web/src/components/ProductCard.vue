<script setup lang="ts">
import { ref, type Ref } from 'vue';
import { useRouter } from 'vue-router';
import { RouterLink } from 'vue-router';
import { formatNaira, effectivePriceKobo, discountPercent, isDiscounted, type Product } from '@/services/catalog';
import { useWishlist } from '@/composables/useWishlist';
import { useAuth } from '@/composables/useAuth';

const props = defineProps<{ product: Product }>();

const router = useRouter();
const { isAuthenticated } = useAuth();
const { lists, loaded, refresh, create, addItem } = useWishlist();

const savingWish: Ref<boolean> = ref(false);
const savedOk: Ref<boolean> = ref(false);

// One-tap save from the grid: first list wins, a "Saved items" list is created on
// first use — the same convention the product page follows.
async function saveToWishlist(): Promise<void> {
  if (savedOk.value || savingWish.value) return;
  if (!isAuthenticated.value) {
    await router.push({ name: 'login', query: { redirect: `/product/${props.product.slug}` } });
    return;
  }
  savingWish.value = true;
  try {
    if (!loaded.value) await refresh();
    let target = lists.value[0] ?? null;
    if (!target) {
      const created = await create('Saved items');
      if (!created) throw new Error('Could not create a list.');
      target = created;
    }
    const detail = await addItem(target.id, props.product.id);
    if (!detail) throw new Error('Could not save this product.');
    savedOk.value = true;
  } catch {
    // Silent on the card — the product page surfaces full error copy for the same action.
  } finally {
    savingWish.value = false;
  }
}
</script>

<template>
  <article class="card">
    <RouterLink class="card-media" :to="`/product/${product.slug}`">
      <img v-if="product.image_url" :src="product.image_url" :alt="product.name" class="card-img" />
      <!-- No photography yet: the initial keeps the grid rhythm without a broken-image box. -->
      <span v-else class="card-placeholder" aria-hidden="true">{{ product.name.charAt(0) }}</span>
      <span v-if="isDiscounted(product)" class="card-discount-badge">-{{ discountPercent(product) }}%</span>
    </RouterLink>

    <button
      class="card-save"
      :class="{ 'card-save--saved': savedOk }"
      type="button"
      :disabled="savingWish"
      :aria-label="savedOk ? `${product.name} saved to wishlist` : `Save ${product.name} to wishlist`"
      @click="saveToWishlist"
    >
      {{ savedOk ? '♥' : '♡' }}
    </button>

    <div class="card-body">
      <p class="eyebrow">{{ product.category_name }}</p>
      <h3 class="card-name">
        <RouterLink class="card-name-link" :to="`/product/${product.slug}`">{{ product.name }}</RouterLink>
      </h3>

      <p class="card-price">
        <template v-if="isDiscounted(product)">
          <span class="card-price-current">{{ formatNaira(effectivePriceKobo(product)) }}</span>
          <del class="card-price-original">{{ formatNaira(product.unit_price_kobo) }}</del>
        </template>
        <template v-else>
          {{ formatNaira(product.unit_price_kobo) }}
        </template>
        <span class="card-unit">/ {{ product.unit }}</span>
      </p>

      <p class="card-meta">
        <span v-if="!product.in_stock" class="card-oos">Out of stock</span>
        <span v-else-if="product.min_order_quantity > 1">
          Minimum {{ product.min_order_quantity }} {{ product.unit }}
        </span>
        <span v-else>In stock</span>
      </p>
    </div>
  </article>
</template>

<style scoped>
.card {
  position: relative;
  display: flex;
  flex-direction: column;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  overflow: hidden;
}

.card-save {
  position: absolute;
  top: var(--spacing-sm);
  right: var(--spacing-sm);
  z-index: 1;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 30px;
  height: 30px;
  border-radius: 50%;
  border: 1px solid var(--color-border);
  background-color: var(--color-surface);
  color: var(--color-secondary);
  font-size: 0.95rem;
  line-height: 1;
  cursor: pointer;
  transition: color 0.15s, border-color 0.15s, background-color 0.15s;
}

.card-save:hover {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.card-save--saved {
  color: var(--color-on-primary);
  background-color: var(--color-primary);
  border-color: var(--color-primary);
}

.card-media {
  display: flex;
  align-items: center;
  justify-content: center;
  aspect-ratio: 4 / 3;
  background-color: var(--color-neutral);
  text-decoration: none;
}

.card-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.card-placeholder {
  font-family: var(--font-display);
  font-size: 2.5rem;
  color: var(--color-secondary);
}

.card-body {
  padding: var(--spacing-md);
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.card-name {
  font-family: var(--font-body);
  font-size: 0.95rem;
  font-weight: 500;
}

.card-name-link {
  color: var(--color-primary);
  text-decoration: none;
}

.card-name-link:hover {
  border-bottom: 1px solid var(--color-primary);
}

.card-discount-badge {
  position: absolute;
  top: var(--spacing-sm);
  left: var(--spacing-sm);
  z-index: 1;
  padding: 3px 7px;
  border-radius: var(--rounded-sm);
  background-color: var(--color-surface);
  color: var(--color-primary);
  border: 1px solid var(--color-border);
  font-family: var(--font-label);
  font-size: 0.6875rem;
  font-weight: 600;
  letter-spacing: 0.02em;
}

.card-price {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 6px;
  font-family: var(--font-label);
  font-size: 1rem;
  color: var(--color-primary);
}

.card-price-current {
  font-weight: 600;
  color: var(--color-primary);
}

.card-price-original {
  font-size: 0.8125rem;
  color: var(--color-secondary);
}

.card-unit {
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.card-meta {
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.card-oos {
  color: var(--color-tertiary);
}
</style>
