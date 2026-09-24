<script setup lang="ts">
import { ref, onMounted, computed, type Ref } from 'vue';
import { RouterLink, useRoute } from 'vue-router';
import { useWishlist } from '@/composables/useWishlist';
import { useCart } from '@/composables/useCart';
import ProductCard from '@/components/ProductCard.vue';
import type { WishlistDetail } from '@/services/wishlist';

const route = useRoute();
const { loadOne, removeItem } = useWishlist();
const { add: addToCart } = useCart();

const wishlist: Ref<WishlistDetail | null> = ref(null);
const loading: Ref<boolean>               = ref(true);
const error: Ref<string | null>           = ref(null);

const itemCount: Ref<number> = computed<number>(() => wishlist.value?.items.length ?? 0);

async function refresh(): Promise<void> {
  loading.value = true;
  error.value   = null;
  try {
    const id: string = route.params.id as string;
    const data = await loadOne(id);
    if (!data) {
      error.value = 'This wishlist could not be loaded.';
      wishlist.value = null;
    } else {
      wishlist.value = data;
    }
  } finally {
    loading.value = false;
  }
}

function moveToCart(productId: string): void {
  const item = wishlist.value?.items.find((i) => i.product.id === productId);
  if (!item) return;
  addToCart(item.product, item.product.min_order_quantity);
}

async function dropItem(itemId: string): Promise<void> {
  if (!wishlist.value) return;
  const before: number = itemCount.value;
  await removeItem(wishlist.value.id, itemId);
  // Refresh the detail so itemCount and the rendered list stay in step.
  await refresh();
  // Touch `before` so the linter keeps the variable; the refresh above already replaces the
  // list with the canonical server state.
  void before;
}

onMounted(refresh);
</script>

<template>
  <div class="page">
    <p v-if="loading" class="state-msg">Loading wishlist…</p>
    <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>

    <template v-else-if="wishlist">
      <header class="head">
        <p class="eyebrow">
          <RouterLink class="crumb" to="/wishlists">My wishlists</RouterLink>
        </p>
        <h1 class="section-title">{{ wishlist.name }}</h1>
        <p v-if="wishlist.description" class="lede">{{ wishlist.description }}</p>
        <p class="meta">
          {{ itemCount }} {{ itemCount === 1 ? 'item' : 'items' }} saved
        </p>
      </header>

      <p v-if="itemCount === 0" class="state-msg empty">
        Nothing here yet. Open a product and save it to this list.
      </p>

      <ul v-else class="items" role="list">
        <li v-for="item in wishlist.items" :key="item.id" class="row">
          <ProductCard :product="item.product" />

          <div class="row-actions">
            <button class="btn-secondary" type="button" @click="moveToCart(item.product.id)">
              Move to cart
            </button>
            <button class="btn-link btn-link--danger" type="button" @click="dropItem(item.id)">
              Remove from list
            </button>
          </div>
        </li>
      </ul>
    </template>
  </div>
</template>

<style scoped>
.head {
  margin-bottom: var(--spacing-lg);
}

.crumb {
  color: var(--color-secondary);
  text-decoration: none;
}

.crumb:hover {
  color: var(--color-primary);
}

.lede {
  margin-top: var(--spacing-sm);
  color: var(--color-secondary);
  max-width: 60ch;
}

.meta {
  margin-top: var(--spacing-sm);
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}

.items {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
  gap: var(--spacing-md);
  list-style: none;
}

.row {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.row-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
  padding: 0 var(--spacing-sm);
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

.empty {
  margin-top: var(--spacing-lg);
}
</style>