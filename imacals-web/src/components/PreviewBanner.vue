<script setup lang="ts">
import { usingPreviewCatalog } from '@/services/catalog';
import { SITE } from '@/site';
</script>

<template>
  <!-- Renders only while the storefront is running on sample data, so nobody mistakes the
       preview catalogue for real stock. Disappears on its own once /catalog is live. -->
  <div v-if="usingPreviewCatalog" class="preview-banner" role="status">
    <strong class="preview-strong">Preview catalogue.</strong>
    These are sample products — the live catalogue and online checkout are still being built. To
    order today, call <a class="preview-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>.
  </div>
</template>

<style scoped>
.preview-banner {
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  padding: 10px var(--spacing-md);
  text-align: center;
  font-size: 0.8rem;
  color: var(--color-secondary);
}

.preview-strong {
  color: var(--color-primary);
  font-weight: 500;
}

.preview-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}
</style>
