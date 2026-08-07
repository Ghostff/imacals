<script setup lang="ts">
import { RouterLink } from 'vue-router';
import { SITE } from '@/site';
</script>

<template>
  <div class="page">
    <p class="eyebrow">404</p>
    <h1 class="section-title">That page is not here</h1>
    <p class="copy">
      The link may be old, or the product may have been taken off the catalogue. Try the catalogue,
      or call <a class="inline-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>.
    </p>
    <RouterLink class="btn-primary cta" to="/catalog">Browse the catalogue</RouterLink>
  </div>
</template>

<style scoped>
.copy {
  max-width: 52ch;
  color: var(--color-secondary);
  margin: var(--spacing-md) 0;
}

.cta {
  display: inline-block;
  text-decoration: none;
}

.inline-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
