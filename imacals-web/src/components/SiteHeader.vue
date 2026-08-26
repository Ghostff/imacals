<script setup lang="ts">
import { ref, onMounted, onUnmounted, type Ref } from 'vue';
import { RouterLink, useRouter } from 'vue-router';
import { useCart } from '@/composables/useCart';
import { useTheme } from '@/composables/useTheme';
import { useAuth } from '@/composables/useAuth';
import { SITE } from '@/site';

const router = useRouter();
const { itemCount } = useCart();
const { isDark, toggleTheme } = useTheme();
const { user, isAuthenticated, displayName, initials, logout } = useAuth();

const menuOpen: Ref<boolean> = ref(false);

function toggleMenu(): void {
  menuOpen.value = !menuOpen.value;
}

function closeMenu(): void {
  menuOpen.value = false;
}

function onSignOut(): void {
  closeMenu();
  logout();
  router.push('/');
}

function onDocClick(e: MouseEvent): void {
  const target = e.target as HTMLElement;
  if (!target.closest('.user-menu')) closeMenu();
}

onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));
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

        <!-- Authenticated user dropdown or Sign in link -->
        <div v-if="isAuthenticated" class="user-menu">
          <button
            class="user-btn"
            :class="{ 'user-btn--open': menuOpen }"
            type="button"
            aria-label="Account menu"
            :aria-expanded="menuOpen"
            @click.stop="toggleMenu"
          >
            <span class="user-avatar" aria-hidden="true">{{ initials }}</span>
            <span class="user-name">{{ user?.first_name || displayName }}</span>
            <span class="nav-chevron" aria-hidden="true">{{ menuOpen ? '▴' : '▾' }}</span>
          </button>

          <ul v-if="menuOpen" class="dropdown dropdown--right">
            <li>
              <RouterLink class="dropdown-item" to="/account" @click="closeMenu">
                <span class="dropdown-icon" aria-hidden="true">👤</span>
                My Account
              </RouterLink>
            </li>
            <li>
              <RouterLink class="dropdown-item" to="/track" @click="closeMenu">
                <span class="dropdown-icon" aria-hidden="true">📦</span>
                Track Order
              </RouterLink>
            </li>
            <li class="dropdown-divider" aria-hidden="true"></li>
            <li>
              <button class="dropdown-item dropdown-item--btn" type="button" @click.stop="toggleTheme">
                <span class="dropdown-icon" aria-hidden="true">{{ isDark ? '☀' : '☾' }}</span>
                {{ isDark ? 'Light theme' : 'Dark theme' }}
              </button>
            </li>
            <li class="dropdown-divider" aria-hidden="true"></li>
            <li>
              <button class="dropdown-item dropdown-item--btn" type="button" @click="onSignOut">
                <span class="dropdown-icon" aria-hidden="true">⇥</span>
                Sign out
              </button>
            </li>
          </ul>
        </div>

        <RouterLink v-else class="auth-link" to="/login">
          Sign in
        </RouterLink>

        <button
          v-if="!isAuthenticated"
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

.auth-link {
  font-family: var(--font-label);
  font-size: 0.8rem;
  letter-spacing: 0.02em;
  color: var(--color-primary);
  text-decoration: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  padding: 6px 12px;
  transition: border-color 0.15s, background-color 0.15s;
}

.auth-link:hover {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
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

/* ── User menu ── */
.user-menu {
  position: relative;
  display: flex;
  align-items: center;
}

.user-btn {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: none;
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  cursor: pointer;
  padding: 4px 8px;
  font-family: var(--font-body);
  font-size: 0.85rem;
  color: var(--color-primary);
  transition: background-color 0.15s, border-color 0.15s;
}

.user-btn:hover,
.user-btn--open {
  background-color: var(--color-neutral);
  border-color: var(--color-primary);
}

.user-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background-color: var(--color-primary);
  color: var(--color-on-primary);
  font-family: var(--font-label);
  font-size: 0.65rem;
  letter-spacing: 0.02em;
}

.user-name {
  white-space: nowrap;
}

.nav-chevron {
  font-size: 0.55rem;
  color: var(--color-secondary);
}

.dropdown {
  position: absolute;
  top: calc(100% + 6px);
  right: 0;
  z-index: 100;
  list-style: none;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  box-shadow: 0 4px 20px var(--color-overlay);
  min-width: 180px;
  padding: 4px 0;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  width: 100%;
  padding: 8px var(--spacing-md);
  font-family: var(--font-body);
  font-size: 0.85rem;
  color: var(--color-secondary);
  text-decoration: none;
  text-align: left;
  border: none;
  background: none;
  cursor: pointer;
  transition: color 0.1s, background-color 0.1s;
}

.dropdown-item:hover {
  color: var(--color-primary);
  background-color: var(--color-neutral);
}

.dropdown-icon {
  font-size: 0.85rem;
  width: 1.2em;
  text-align: center;
}

.dropdown-divider {
  height: 1px;
  margin: 4px 0;
  background-color: var(--color-divider);
}

@media (max-width: 720px) {
  .nav { margin-left: 0; order: 3; width: 100%; }
  .header-actions { gap: var(--spacing-sm); }
  .order-line-label { display: none; }
}
</style>
