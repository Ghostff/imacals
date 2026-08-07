<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import { catalogService, formatNaira, type Product } from '@/services/catalog';
import { useCart } from '@/composables/useCart';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

const route  = useRoute();
const router = useRouter();
const { add } = useCart();

const product: Ref<Product | null> = ref(null);
const quantity: Ref<number>        = ref(1);
const loading: Ref<boolean>        = ref(true);
const error: Ref<string | null>    = ref(null);
const added: Ref<boolean>          = ref(false);

onMounted(async () => {
  try {
    const found = await catalogService.findProduct(route.params.slug as string);
    product.value  = found;
    // Open on the smallest quantity the warehouse will actually pick.
    quantity.value = found.min_order_quantity;
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Could not load this product.';
  } finally {
    loading.value = false;
  }
});

function addToCart(): void {
  if (!product.value) return;
  add(product.value, quantity.value);
  added.value = true;
}

function buyNow(): void {
  addToCart();
  void router.push({ name: 'cart' });
}
</script>

<template>
  <div class="page">
    <p v-if="loading" class="state-msg">Loading…</p>
    <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>

    <div v-else-if="product" class="detail">
      <div class="media">
        <img v-if="product.image_url" :src="product.image_url" :alt="product.name" class="media-img" />
        <span v-else class="media-placeholder" aria-hidden="true">{{ product.name.charAt(0) }}</span>
      </div>

      <div class="info">
        <p class="eyebrow">{{ product.category_name }}</p>
        <h1 class="title">{{ product.name }}</h1>

        <p class="price">
          {{ formatNaira(product.unit_price_kobo) }}
          <span class="unit">/ {{ product.unit }}</span>
        </p>

        <p class="description">{{ product.description }}</p>

        <dl class="specs">
          <div class="spec">
            <dt class="spec-label">Sold by</dt>
            <dd class="spec-value">{{ product.unit }}</dd>
          </div>
          <div class="spec">
            <dt class="spec-label">Minimum order</dt>
            <dd class="spec-value">{{ product.min_order_quantity }}</dd>
          </div>
          <div class="spec">
            <dt class="spec-label">Availability</dt>
            <dd class="spec-value">{{ product.in_stock ? 'In stock in Aba' : 'Out of stock' }}</dd>
          </div>
        </dl>

        <div v-if="product.in_stock" class="buy">
          <div class="qty">
            <label class="field-label" for="qty">Quantity</label>
            <input
              id="qty"
              v-model.number="quantity"
              class="field-input qty-input"
              type="number"
              :min="product.min_order_quantity"
              :step="1"
            />
          </div>

          <div class="buy-actions">
            <!-- The one Tertiary action on this screen. -->
            <button class="btn-primary" type="button" @click="buyNow">Add and view cart</button>
            <button class="btn-secondary" type="button" @click="addToCart">Add to cart</button>
          </div>
        </div>

        <p v-else class="oos-note">
          This line is out of stock. Call <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
          and we will tell you when the next load arrives.
        </p>

        <p v-if="added" class="added" role="status">
          Added to your cart. <RouterLink class="inline-link" to="/cart">Go to cart</RouterLink>
        </p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.detail {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: var(--spacing-lg);
}

@media (max-width: 820px) {
  .detail { grid-template-columns: 1fr; }
}

.media {
  display: flex;
  align-items: center;
  justify-content: center;
  aspect-ratio: 4 / 3;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  overflow: hidden;
}

.media-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.media-placeholder {
  font-family: var(--font-display);
  font-size: 5rem;
  color: var(--color-secondary);
}

.title {
  font-family: var(--font-display);
  font-size: 2.25rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  margin: var(--spacing-sm) 0;
}

.price {
  font-family: var(--font-label);
  font-size: 1.5rem;
}

.unit {
  font-size: 0.85rem;
  color: var(--color-secondary);
}

.description {
  margin: var(--spacing-md) 0;
  color: var(--color-secondary);
  max-width: 52ch;
}

.specs {
  border-top: 1px solid var(--color-divider);
  margin: var(--spacing-md) 0;
}

.spec {
  display: flex;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 10px 0;
  border-bottom: 1px solid var(--color-divider);
}

.spec-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}

.spec-value {
  font-size: 0.875rem;
}

.buy {
  margin-top: var(--spacing-lg);
}

.qty-input {
  max-width: 120px;
}

.buy-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-md);
}

.oos-note,
.added {
  margin-top: var(--spacing-md);
  font-size: 0.875rem;
  color: var(--color-secondary);
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
