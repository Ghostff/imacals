<script setup lang="ts">
import { RouterLink } from 'vue-router';
import { useCart } from '@/composables/useCart';
import {
  formatNaira,
  effectivePriceKobo,
  isDiscounted,
  discountSavingsKobo,
  discountPercent,
} from '@/services/catalog';
import { SITE } from '@/site';

const {
  lines,
  itemCount,
  subtotalKobo,
  originalSubtotalKobo,
  totalSavingsKobo,
  taxKobo,
  exemptSubtotalKobo,
  setQuantity,
  remove,
  clear,
} = useCart();
</script>

<template>
  <div class="page">
    <header class="head">
      <p class="eyebrow">Cart</p>
      <h1 class="section-title">Your order</h1>
    </header>

    <div v-if="!lines.length" class="empty">
      <p class="state-msg">Your cart is empty.</p>
      <RouterLink class="btn-primary empty-cta" to="/catalog">Browse the catalogue</RouterLink>
    </div>

    <div v-else class="layout">
      <section class="lines">
        <article v-for="line in lines" :key="line.product.id" class="line">
          <div class="line-main">
            <RouterLink class="line-name" :to="`/product/${line.product.slug}`">
              {{ line.product.name }}
            </RouterLink>
            <div class="line-meta">
              <template v-if="isDiscounted(line.product)">
                <span class="line-price-discounted">{{ formatNaira(effectivePriceKobo(line.product)) }}</span>
                <del class="line-price-original">{{ formatNaira(line.product.unit_price_kobo) }}</del>
                <span class="line-discount-tag">-{{ discountPercent(line.product) }}%</span>
              </template>
              <template v-else>
                <span>{{ formatNaira(line.product.unit_price_kobo) }}</span>
              </template>
              <span class="line-unit">/ {{ line.product.unit }}</span>
              <template v-if="line.product.min_order_quantity > 1">
                · min {{ line.product.min_order_quantity }}
              </template>
            </div>
            <p v-if="isDiscounted(line.product)" class="line-savings">
              Save {{ formatNaira(discountSavingsKobo(line.product) * line.quantity) }}
            </p>
          </div>

          <div class="line-qty">
            <label class="field-label" :for="`qty-${line.product.id}`">Qty</label>
            <input
              :id="`qty-${line.product.id}`"
              class="field-input qty-input"
              type="number"
              :min="line.product.min_order_quantity"
              :value="line.quantity"
              @change="setQuantity(line.product.id, Number(($event.target as HTMLInputElement).value))"
            />
          </div>

          <p class="line-total">{{ formatNaira(effectivePriceKobo(line.product) * line.quantity) }}</p>

          <button class="line-remove" type="button" @click="remove(line.product.id)">Remove</button>
        </article>

        <button class="line-remove clear-all" type="button" @click="clear">Clear cart</button>
      </section>

      <aside class="summary">
        <h2 class="summary-title">Summary</h2>

        <div class="summary-row">
          <span class="summary-label">Items</span>
          <span class="summary-value">{{ itemCount }}</span>
        </div>
        <div v-if="totalSavingsKobo > 0" class="summary-row">
          <span class="summary-label">Regular total</span>
          <span class="summary-value"><del>{{ formatNaira(originalSubtotalKobo) }}</del></span>
        </div>
        <div v-if="totalSavingsKobo > 0" class="summary-row summary-row--savings">
          <span class="summary-label">Promotional savings</span>
          <span class="summary-value">-{{ formatNaira(totalSavingsKobo) }}</span>
        </div>
        <div class="summary-row summary-row--total">
          <span class="summary-label">Subtotal</span>
          <span class="summary-value">{{ formatNaira(subtotalKobo) }}</span>
        </div>
        <div class="summary-row summary-row--tax">
          <span class="summary-label">Est. VAT (7.5%)</span>
          <span class="summary-value">{{ taxKobo > 0 ? formatNaira(taxKobo) : '₦0 (Exempt)' }}</span>
        </div>
        <div v-if="exemptSubtotalKobo > 0" class="tax-exempt-note">
          Includes {{ formatNaira(exemptSubtotalKobo) }} in VAT-exempt agricultural foodstuffs
        </div>
        <div class="summary-row">
          <span class="summary-label">Delivery</span>
          <!-- The fee depends on the destination, which we only know at checkout. -->
          <span class="summary-value summary-muted">Quoted at checkout</span>
        </div>

        <!-- The one Tertiary action on this screen. -->
        <RouterLink class="btn-primary checkout-cta" to="/checkout">Continue to checkout</RouterLink>

        <p class="summary-note">
          Rather order by phone? Call
          <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
          and read out your list.
        </p>
      </aside>
    </div>
  </div>
</template>

<style scoped>
.head {
  margin-bottom: var(--spacing-lg);
}

.empty-cta {
  display: inline-block;
  text-decoration: none;
  margin-top: var(--spacing-md);
}

.layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 320px;
  gap: var(--spacing-lg);
  align-items: start;
}

@media (max-width: 820px) {
  .layout { grid-template-columns: 1fr; }
}

.line {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto auto;
  align-items: center;
  gap: var(--spacing-md);
  padding: var(--spacing-md) 0;
  border-bottom: 1px solid var(--color-divider);
}

@media (max-width: 620px) {
  .line { grid-template-columns: minmax(0, 1fr) auto; }
}

.line-name {
  color: var(--color-primary);
  text-decoration: none;
  font-weight: 500;
}

.line-name:hover {
  border-bottom: 1px solid var(--color-primary);
}

.line-meta {
  display: flex;
  align-items: baseline;
  flex-wrap: wrap;
  gap: 4px;
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.line-price-discounted {
  font-family: var(--font-label);
  font-weight: 600;
  color: var(--color-primary);
}

.line-price-original {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
}

.line-discount-tag {
  display: inline-block;
  padding: 1px 4px;
  border-radius: var(--rounded-sm);
  background-color: color-mix(in srgb, var(--color-secondary) 22%, transparent);
  border: 1px solid var(--color-border);
  color: var(--color-primary);
  font-family: var(--font-label);
  font-size: 0.6875rem;
  font-weight: 600;
}

.line-savings {
  font-family: var(--font-label);
  font-size: 0.75rem;
  color: var(--color-secondary);
  margin: 3px 0 0 0;
}

.qty-input {
  width: 88px;
}

.line-total {
  font-family: var(--font-label);
  white-space: nowrap;
}

.line-remove {
  font-family: var(--font-label);
  font-size: 0.75rem;
  background: none;
  border: none;
  color: var(--color-secondary);
  cursor: pointer;
  padding: 0;
  text-align: left;
}

.line-remove:hover {
  color: var(--color-primary);
}

.clear-all {
  margin-top: var(--spacing-md);
}

.summary {
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.summary-title {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
  margin-bottom: var(--spacing-md);
}

.summary-row {
  display: flex;
  justify-content: space-between;
  gap: var(--spacing-md);
  padding: 8px 0;
  border-bottom: 1px solid var(--color-divider);
}

.summary-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}

.summary-value {
  font-family: var(--font-label);
  font-size: 0.875rem;
}

.summary-muted {
  color: var(--color-secondary);
}

.summary-row--savings {
  color: var(--color-primary);
  font-weight: 500;
}

.summary-row--savings .summary-value {
  color: var(--color-primary);
  font-weight: 600;
}

.tax-exempt-note {
  font-size: 0.72rem;
  color: var(--color-secondary);
  padding: 4px 0 8px;
  line-height: 1.3;
}

.checkout-cta {
  display: block;
  text-align: center;
  text-decoration: none;
  margin-top: var(--spacing-lg);
}

.summary-note {
  margin-top: var(--spacing-md);
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
