<script setup lang="ts">
import { RouterLink } from 'vue-router';
import { SITE } from '@/site';
</script>

<template>
  <footer class="footer">
    <div class="footer-inner">
      <div class="footer-col">
        <p class="eyebrow">Warehouse</p>
        <p class="footer-text">{{ SITE.warehouse.line1 }}</p>
        <p class="footer-text">
          {{ SITE.warehouse.city }}, {{ SITE.warehouse.state }}
        </p>
        <p class="footer-text">{{ SITE.warehouse.country }}</p>
      </div>

      <div class="footer-col">
        <p class="eyebrow">Order line</p>
        <p class="footer-text">
          <a class="footer-link" :href="SITE.orderLineHref">{{ SITE.orderLine }}</a>
        </p>
        <p class="footer-text">WhatsApp {{ SITE.whatsapp }}</p>
        <p class="footer-text">{{ SITE.hours }}</p>
      </div>

      <div class="footer-col">
        <p class="eyebrow">Shop</p>
        <p class="footer-text"><RouterLink class="footer-link" to="/catalog">Catalogue</RouterLink></p>
        <p class="footer-text"><RouterLink class="footer-link" to="/delivery">Delivery &amp; coverage</RouterLink></p>
        <p class="footer-text"><RouterLink class="footer-link" to="/track">Track an order</RouterLink></p>
      </div>
    </div>

    <div class="footer-bar">
      <p class="footer-note">
        {{ SITE.name }} — distribution from {{ SITE.warehouse.city }}, {{ SITE.warehouse.state }}.
      </p>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  margin-top: var(--spacing-lg);
  background-color: var(--color-surface);
  border-top: 1px solid var(--color-border);
}

.footer-inner {
  max-width: var(--page-max);
  margin: 0 auto;
  padding: var(--spacing-lg) var(--spacing-md);
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: var(--spacing-lg);
}

.footer-col .eyebrow {
  margin-bottom: var(--spacing-sm);
}

.footer-text {
  font-size: 0.875rem;
  color: var(--color-secondary);
}

.footer-link {
  color: var(--color-primary);
  text-decoration: none;
  border-bottom: 1px solid var(--color-border);
}

.footer-link:hover {
  border-bottom-color: var(--color-primary);
}

.footer-bar {
  border-top: 1px solid var(--color-divider);
}

.footer-note {
  max-width: var(--page-max);
  margin: 0 auto;
  padding: var(--spacing-md);
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
}
</style>
