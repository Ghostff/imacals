<script setup lang="ts">
import { RouterLink } from 'vue-router';
import { useCart } from '@/composables/useCart';
import { useTheme } from '@/composables/useTheme';
import { SITE } from '@/site';

const { itemCount } = useCart();
const { isDark, toggleTheme } = useTheme();
</script>

<template>
  <header class="header">
    <div class="header-inner">
      <RouterLink class="brand" to="/">imacals</RouterLink>

      <nav class="nav" aria-label="Primary">
        <RouterLink class="nav-link" to="/catalog">Catalogue</RouterLink>
        <RouterLink class="nav-link" to="/delivery">Delivery</RouterLink>
        <RouterLink class="nav-link" to="/track">Track order</RouterLink>
      </nav>

      <div class="header-actions">
        <!-- Phone ordering is a first-class channel, not a fallback — keep it visible. -->
        <a class="order-line" :href="SITE.orderLineHref">
          <span class="order-line-label">Order by phone</span>
          <span class="order-line-number">{{ SITE.orderLine }}</span>
        </a>

        <button
          class="icon-btn"
          type="button"
          :aria-label="isDark ? 'Switch to light theme' : 'Switch to dark theme'"
          @click="toggleTheme"
        >
          {{ isDark ? 'Light' : 'Dark' }}
        </button>

        <RouterLink class="cart-link" to="/cart">
          Cart
          <span v-if="itemCount > 0" class="cart-badge">{{ itemCount }}</span>
        </RouterLink>
      </div>
    </div>
  </header>
</template>

<style scoped>
.header {
  position: sticky;
  top: 0;
  z-index: 10;
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
}

.header-inner {
  max-width: var(--page-max);
  margin: 0 auto;
  min-height: var(--header-height);
  padding: var(--spacing-sm) var(--spacing-md);
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
  flex-wrap: wrap;
}

.brand {
  font-family: var(--font-display);
  font-size: 1.35rem;
  font-weight: 500;
  letter-spacing: -0.01em;
  color: var(--color-primary);
  text-decoration: none;
}

.nav {
  display: flex;
  gap: var(--spacing-md);
  margin-left: var(--spacing-md);
}

.nav-link {
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  color: var(--color-secondary);
  text-decoration: none;
  padding: 4px 0;
  border-bottom: 1px solid transparent;
}

.nav-link:hover,
.nav-link.router-link-active {
  color: var(--color-primary);
  border-bottom-color: var(--color-primary);
}

.header-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: var(--spacing-md);
}

.order-line {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
  text-decoration: none;
}

.order-line-label {
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
  text-transform: uppercase;
  color: var(--color-secondary);
}

.order-line-number {
  font-family: var(--font-label);
  font-size: 0.85rem;
  color: var(--color-primary);
}

.icon-btn {
  font-family: var(--font-label);
  font-size: 0.7rem;
  letter-spacing: 0.02em;
  background: transparent;
  color: var(--color-secondary);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 6px 10px;
  cursor: pointer;
}

.icon-btn:hover {
  color: var(--color-primary);
}

.cart-link {
  position: relative;
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  text-decoration: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 7px 12px;
}

.cart-badge {
  display: inline-block;
  min-width: 18px;
  margin-left: 6px;
  padding: 0 5px;
  border-radius: 9px;
  background-color: var(--color-primary);
  color: var(--color-on-primary);
  font-size: 0.7rem;
  text-align: center;
}

@media (max-width: 720px) {
  .nav { margin-left: 0; order: 3; width: 100%; }
  .header-actions { gap: var(--spacing-sm); }
  .order-line-label { display: none; }
}
</style>
