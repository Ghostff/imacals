<script setup lang="ts">
import { RouterLink } from 'vue-router';
import { formatNaira, type Product } from '@/services/catalog';

defineProps<{ product: Product }>();
</script>

<template>
  <article class="card">
    <RouterLink class="card-media" :to="`/product/${product.slug}`">
      <img v-if="product.image_url" :src="product.image_url" :alt="product.name" class="card-img" />
      <!-- No photography yet: the initial keeps the grid rhythm without a broken-image box. -->
      <span v-else class="card-placeholder" aria-hidden="true">{{ product.name.charAt(0) }}</span>
    </RouterLink>

    <div class="card-body">
      <p class="eyebrow">{{ product.category_name }}</p>
      <h3 class="card-name">
        <RouterLink class="card-name-link" :to="`/product/${product.slug}`">{{ product.name }}</RouterLink>
      </h3>

      <p class="card-price">
        {{ formatNaira(product.unit_price_kobo) }}
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
  display: flex;
  flex-direction: column;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  overflow: hidden;
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

.card-price {
  font-family: var(--font-label);
  font-size: 1rem;
  color: var(--color-primary);
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
