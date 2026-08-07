<script setup lang="ts">
import { ref, computed, type Ref, type ComputedRef } from 'vue';
import { RouterLink } from 'vue-router';
import { useCart } from '@/composables/useCart';
import { formatNaira } from '@/services/catalog';
import { orderService, type PlaceOrderInput, type PlacedOrder } from '@/services/order';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

const { lines, subtotalKobo, clear } = useCart();

const form: Ref<Omit<PlaceOrderInput, 'lines'>> = ref({
  customer_name: '',
  phone: '',
  email: '',
  delivery_address: '',
  city: '',
  state: 'Abia State',
  note: '',
});

const submitting: Ref<boolean>        = ref(false);
const error: Ref<string | null>       = ref(null);
const placed: Ref<PlacedOrder | null> = ref(null);

// A phone number is the one contact detail we cannot dispatch without — the driver calls ahead.
const canSubmit: ComputedRef<boolean> = computed<boolean>(() =>
  lines.value.length > 0
  && form.value.customer_name.trim().length > 0
  && form.value.phone.trim().length > 0
  && form.value.delivery_address.trim().length > 0
  && form.value.city.trim().length > 0
  && !submitting.value,
);

async function submit(): Promise<void> {
  if (!canSubmit.value) return;
  error.value      = null;
  submitting.value = true;
  try {
    const result = await orderService.place({
      ...form.value,
      email: form.value.email?.trim() || undefined,
      note: form.value.note?.trim() || undefined,
      lines: lines.value.map((l) => ({ product_id: l.product.id, quantity: l.quantity })),
    });
    placed.value = result;
    clear();
  } catch (e: unknown) {
    error.value = e instanceof ApiException || e instanceof Error
      ? e.message
      : 'Could not place the order.';
  } finally {
    submitting.value = false;
  }
}
</script>

<template>
  <div class="page">
    <div v-if="placed" class="confirmed">
      <p class="eyebrow">Order placed</p>
      <h1 class="section-title">Reference {{ placed.reference }}</h1>
      <p class="confirmed-copy">
        We have your order and the dispatch desk will call {{ form.phone }} to confirm delivery.
        Total {{ formatNaira(placed.total_kobo) }}.
      </p>
      <RouterLink class="btn-primary confirmed-cta" to="/track">Track this order</RouterLink>
    </div>

    <template v-else>
      <header class="head">
        <p class="eyebrow">Checkout</p>
        <h1 class="section-title">Where are we delivering?</h1>
      </header>

      <div v-if="!lines.length" class="empty">
        <p class="state-msg">Your cart is empty, so there is nothing to check out.</p>
        <RouterLink class="btn-primary empty-cta" to="/catalog">Browse the catalogue</RouterLink>
      </div>

      <div v-else class="layout">
        <form class="form" @submit.prevent="submit" novalidate>
          <div class="field-grid">
            <div class="field">
              <label class="field-label" for="name">Full name</label>
              <input id="name" v-model="form.customer_name" class="field-input" type="text" required />
            </div>

            <div class="field">
              <label class="field-label" for="phone">Phone number</label>
              <input id="phone" v-model="form.phone" class="field-input" type="tel" placeholder="0800 000 0000" required />
            </div>

            <div class="field">
              <label class="field-label" for="email">Email (optional)</label>
              <input id="email" v-model="form.email" class="field-input" type="email" />
            </div>

            <div class="field field--wide">
              <label class="field-label" for="address">Delivery address</label>
              <input id="address" v-model="form.delivery_address" class="field-input" type="text" required />
            </div>

            <div class="field">
              <label class="field-label" for="city">Town / city</label>
              <input id="city" v-model="form.city" class="field-input" type="text" required />
            </div>

            <div class="field">
              <label class="field-label" for="state">State</label>
              <input id="state" v-model="form.state" class="field-input" type="text" required />
            </div>

            <div class="field field--wide">
              <label class="field-label" for="note">Landmark or delivery note (optional)</label>
              <textarea id="note" v-model="form.note" class="field-input note" rows="3"></textarea>
            </div>
          </div>

          <p v-if="error" class="form-error" role="alert">{{ error }}</p>

          <!-- The one Tertiary action on this screen. -->
          <button class="btn-primary" type="submit" :disabled="!canSubmit">
            {{ submitting ? 'Placing order…' : 'Place order' }}
          </button>

          <p class="form-note">
            We confirm every order by phone before it leaves the warehouse. Prefer to skip the form?
            Call <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>.
          </p>
        </form>

        <aside class="summary">
          <h2 class="summary-title">Your order</h2>

          <div v-for="line in lines" :key="line.product.id" class="summary-row">
            <span class="summary-label">{{ line.quantity }} × {{ line.product.name }}</span>
            <span class="summary-value">{{ formatNaira(line.product.unit_price_kobo * line.quantity) }}</span>
          </div>

          <div class="summary-row summary-row--total">
            <span class="summary-label">Subtotal</span>
            <span class="summary-value">{{ formatNaira(subtotalKobo) }}</span>
          </div>

          <p class="summary-note">
            Delivery is quoted once we have the destination — from
            {{ SITE.warehouse.city }}, {{ SITE.coverage[0].eta.toLowerCase() }} within
            {{ SITE.coverage[0].area.toLowerCase() }}.
          </p>
        </aside>
      </div>
    </template>
  </div>
</template>

<style scoped>
.head {
  margin-bottom: var(--spacing-lg);
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

.field-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-lg);
}

@media (max-width: 620px) {
  .field-grid { grid-template-columns: 1fr; }
}

.field--wide {
  grid-column: 1 / -1;
}

.note {
  font-family: var(--font-body);
  resize: vertical;
}

.form-error {
  color: var(--color-tertiary);
  font-size: 0.875rem;
  margin-bottom: var(--spacing-md);
}

.form-note {
  margin-top: var(--spacing-md);
  font-size: 0.8rem;
  color: var(--color-secondary);
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

.summary-row--total {
  border-bottom: none;
  border-top: 1px solid var(--color-border);
  margin-top: var(--spacing-sm);
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
  white-space: nowrap;
}

.summary-note {
  margin-top: var(--spacing-md);
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.confirmed-copy {
  max-width: 52ch;
  color: var(--color-secondary);
  margin: var(--spacing-md) 0;
}

.confirmed-cta,
.empty-cta {
  display: inline-block;
  text-decoration: none;
  margin-top: var(--spacing-md);
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
