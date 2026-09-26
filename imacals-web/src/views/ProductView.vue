<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import { useRoute, useRouter, RouterLink } from 'vue-router';
import {
  catalogService,
  formatNaira,
  effectivePriceKobo,
  discountPercent,
  discountSavingsKobo,
  isDiscounted,
  type Product,
} from '@/services/catalog';
import { useCart } from '@/composables/useCart';
import { useWishlist } from '@/composables/useWishlist';
import { useAuth } from '@/composables/useAuth';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

const route  = useRoute();
const router = useRouter();
const { add } = useCart();
const { isAuthenticated } = useAuth();
const { lists, refresh: refreshWishlist, create: createWishlist, addItem } = useWishlist();

const product: Ref<Product | null> = ref(null);
const activeImage: Ref<string | null> = ref(null);
const quantity: Ref<number>        = ref(1);
const loading: Ref<boolean>        = ref(true);
const error: Ref<string | null>    = ref(null);
const added: Ref<boolean>          = ref(false);

const savedTo: Ref<string | null> = ref(null);
const savingWishlist: Ref<boolean> = ref(false);
const wishlistError: Ref<string | null> = ref(null);

onMounted(async () => {
  try {
    const found = await catalogService.findProduct(route.params.slug as string);
    product.value  = found;
    activeImage.value = (found.images && found.images.length > 0) ? found.images[0] : found.image_url;
    // Open on the smallest quantity the warehouse will actually pick.
    quantity.value = found.min_order_quantity;
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Could not load this product.';
  } finally {
    loading.value = false;
  }
  if (isAuthenticated.value) await refreshWishlist();
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

async function saveToWishlist(): Promise<void> {
  if (!product.value || !isAuthenticated.value) {
    await router.push({ name: 'login', query: { redirect: route.fullPath } });
    return;
  }
  savingWishlist.value = true;
  wishlistError.value = null;
  try {
    let target = lists.value[0] ?? null;
    if (!target) {
      const created = await createWishlist('Saved items');
      if (!created) throw new Error('Could not create a list.');
      target = created;
    }
    const detail = await addItem(target.id, product.value.id);
    if (!detail) throw new Error('Could not save this product.');
    savedTo.value = target.name;
  } catch (e: unknown) {
    wishlistError.value = e instanceof Error ? e.message : 'Could not save this product.';
  } finally {
    savingWishlist.value = false;
  }
}
</script>

<template>
  <div class="page">
    <p v-if="loading" class="state-msg">Loading…</p>
    <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>

    <div v-else-if="product" class="detail">
      <div class="media-column">
        <div class="media-main">
          <img
            v-if="activeImage || product.image_url"
            :src="(activeImage || product.image_url)!"
            :alt="product.name"
            class="media-img"
          />
          <span v-else class="media-placeholder" aria-hidden="true">{{ product.name.charAt(0) }}</span>
        </div>

        <!-- Thumbnail Gallery -->
        <div
          v-if="product.images && product.images.length > 1"
          class="gallery-thumbs"
          role="tablist"
          aria-label="Product images"
        >
          <button
            v-for="(imgUrl, idx) in product.images"
            :key="idx"
            type="button"
            class="thumb-btn"
            :class="{ 'thumb-btn--active': (activeImage || product.image_url) === imgUrl }"
            :aria-label="'View image ' + (idx + 1)"
            @click="activeImage = imgUrl"
          >
            <img :src="imgUrl" :alt="product.name + ' thumbnail ' + (idx + 1)" class="thumb-img" />
          </button>
        </div>
      </div>

      <div class="info">
        <div class="header-tags">
          <p class="eyebrow">{{ product.category_name }}</p>
          <span v-if="isDiscounted(product)" class="discount-deal-badge">
            PROMO · {{ discountPercent(product) }}% OFF
          </span>
        </div>
        <h1 class="title">{{ product.name }}</h1>

        <div class="price-container">
          <p class="price">
            <template v-if="isDiscounted(product)">
              <span class="price-effective">{{ formatNaira(effectivePriceKobo(product)) }}</span>
              <del class="price-original">{{ formatNaira(product.unit_price_kobo) }}</del>
            </template>
            <template v-else>
              {{ formatNaira(product.unit_price_kobo) }}
            </template>
            <span class="unit">/ {{ product.unit }}</span>
          </p>
          <p v-if="isDiscounted(product)" class="savings-callout">
            You save {{ formatNaira(discountSavingsKobo(product)) }} per {{ product.unit }} ({{ discountPercent(product) }}% discount)
          </p>
        </div>

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

          <div class="wishlist-row">
            <button
              class="btn-secondary"
              type="button"
              :disabled="savingWishlist"
              @click="saveToWishlist"
            >
              {{ savingWishlist ? 'Saving…' : (savedTo ? 'Saved again' : 'Save to wishlist') }}
            </button>
            <p v-if="savedTo" class="saved-note" role="status">
              Saved to <RouterLink class="inline-link" :to="`/wishlists`">{{ savedTo }}</RouterLink>
            </p>
            <p v-else-if="wishlistError" class="saved-note saved-note--error" role="alert">
              {{ wishlistError }}
            </p>
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

.media-column {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-sm);
}

.media-main {
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

.gallery-thumbs {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.thumb-btn {
  width: 64px;
  height: 64px;
  flex-shrink: 0;
  border-radius: var(--rounded-md);
  border: 1.5px solid var(--color-border);
  background: var(--color-surface);
  overflow: hidden;
  padding: 0;
  cursor: pointer;
  transition: border-color 0.2s ease, transform 0.15s ease;
}

.thumb-btn:hover {
  border-color: var(--color-primary);
  transform: translateY(-1px);
}

.thumb-btn--active {
  border-color: var(--color-tertiary);
  box-shadow: 0 0 0 1px var(--color-tertiary);
}

.thumb-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
  display: block;
}

.header-tags {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  flex-wrap: wrap;
}

.discount-deal-badge {
  font-family: var(--font-label);
  font-size: 0.6875rem;
  font-weight: 600;
  letter-spacing: 0.04em;
  padding: 3px 8px;
  border-radius: var(--rounded-sm);
  background-color: color-mix(in srgb, var(--color-secondary) 20%, transparent);
  border: 1px solid var(--color-border);
  color: var(--color-primary);
}

.title {
  font-family: var(--font-display);
  font-size: 2.25rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  margin: var(--spacing-sm) 0;
}

.price-container {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.price {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 8px;
  font-family: var(--font-label);
  font-size: 1.5rem;
}

.price-effective {
  font-weight: 600;
  color: var(--color-primary);
}

.price-original {
  font-size: 1.1rem;
  color: var(--color-secondary);
}

.savings-callout {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
  letter-spacing: 0.01em;
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

.wishlist-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--spacing-sm);
  margin-top: var(--spacing-sm);
}

.saved-note {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}

.saved-note--error {
  color: var(--color-tertiary);
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
