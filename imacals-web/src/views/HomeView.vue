<script setup lang="ts">
import { ref, onMounted, type Ref } from 'vue';
import { RouterLink } from 'vue-router';
import ProductCard from '@/components/ProductCard.vue';
import { catalogService, type Product } from '@/services/catalog';
import { ApiException } from '@/services/api';
import { SITE } from '@/site';

const featured: Ref<Product[]> = ref([]);
const loading: Ref<boolean>    = ref(true);
const error: Ref<string | null> = ref(null);

onMounted(async () => {
  try {
    const all = await catalogService.listProducts();
    featured.value = all.slice(0, 6);
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
  <div>
    <section class="hero">
      <div class="hero-inner">
        <p class="eyebrow">{{ SITE.warehouse.city }} · {{ SITE.warehouse.state }} · {{ SITE.warehouse.country }}</p>
        <h1 class="hero-title">Wholesale supply, delivered from Aba.</h1>
        <p class="hero-lede">
          {{ SITE.tagline }} Browse the catalogue and check out online, or call the order line and
          we will take it down for you. Everything ships from our Aba distribution warehouse.
        </p>

        <div class="hero-actions">
          <!-- The one Tertiary action on this screen. -->
          <RouterLink class="btn-primary" to="/catalog">Browse the catalogue</RouterLink>
          <a class="btn-secondary" :href="SITE.orderLineHref">Call {{ SITE.orderLine }}</a>
        </div>
      </div>
    </section>

    <section class="page">
      <div class="strip">
        <div v-for="row in SITE.coverage" :key="row.area" class="strip-item">
          <p class="strip-eta">{{ row.eta }}</p>
          <p class="strip-area">{{ row.area }}</p>
        </div>
      </div>
    </section>

    <section class="page">
      <header class="section-head">
        <h2 class="section-title">Moving this week</h2>
        <RouterLink class="see-all" to="/catalog">See everything</RouterLink>
      </header>

      <p v-if="loading" class="state-msg">Loading the catalogue…</p>
      <p v-else-if="error" class="state-msg state-msg--error">{{ error }}</p>
      <p v-else-if="!featured.length" class="state-msg">
        The catalogue is empty right now. Call {{ SITE.orderLine }} and we will tell you what is in.
      </p>

      <div v-else class="grid">
        <ProductCard v-for="p in featured" :key="p.id" :product="p" />
      </div>
    </section>

    <section class="page">
      <div class="phone-panel">
        <div>
          <p class="eyebrow">Prefer to talk?</p>
          <h2 class="section-title">We take orders by phone.</h2>
          <p class="phone-copy">
            Call {{ SITE.hours.toLowerCase() }}. Tell us what you need and where it is going — we
            confirm the price, the delivery window and the payment details on the call.
          </p>
        </div>
        <a class="btn-secondary phone-cta" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
      </div>
    </section>
  </div>
</template>

<style scoped>
.hero {
  border-bottom: 1px solid var(--color-border);
  background-color: var(--color-surface);
}

.hero-inner {
  max-width: var(--page-max);
  margin: 0 auto;
  padding: 72px var(--spacing-md);
}

.hero-title {
  font-family: var(--font-display);
  font-size: clamp(2.25rem, 5vw, 4rem);
  font-weight: 500;
  letter-spacing: -0.03em;
  line-height: 1.05;
  margin: var(--spacing-md) 0;
  max-width: 16ch;
}

.hero-lede {
  max-width: 56ch;
  color: var(--color-secondary);
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  margin-top: var(--spacing-lg);
}

.hero-actions .btn-primary,
.hero-actions .btn-secondary {
  text-decoration: none;
  display: inline-block;
}

.strip {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 1px;
  background-color: var(--color-divider);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  overflow: hidden;
}

.strip-item {
  background-color: var(--color-surface);
  padding: var(--spacing-md);
}

.strip-eta {
  font-family: var(--font-label);
  font-size: 0.9rem;
  color: var(--color-primary);
}

.strip-area {
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.section-head {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--spacing-md);
  margin-bottom: var(--spacing-md);
}

.see-all {
  font-family: var(--font-label);
  font-size: 0.8rem;
  color: var(--color-secondary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}

.see-all:hover {
  color: var(--color-primary);
}

.grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
  gap: var(--spacing-md);
}

.phone-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--spacing-md);
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-lg);
  padding: var(--spacing-lg);
}

.phone-copy {
  max-width: 52ch;
  color: var(--color-secondary);
  margin-top: var(--spacing-sm);
}

.phone-cta {
  text-decoration: none;
  white-space: nowrap;
}
</style>
