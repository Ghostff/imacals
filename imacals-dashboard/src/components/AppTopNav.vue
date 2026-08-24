<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, type Ref, type ComputedRef } from 'vue';
import { useRoute, RouterLink } from 'vue-router';
import { useAuth } from '@/composables/useAuth';
import { useTheme } from '@/composables/useTheme';

defineEmits<{ logout: [] }>();

interface NavItem {
  label: string;
  key: string;
  to: string;
}

const route = useRoute();
const { user } = useAuth();
const { isDark, toggleTheme } = useTheme();

const nav: NavItem[] = [
  { label: 'Products', key: 'products', to: '/products' },
  { label: 'Integrations', key: 'integrations', to: '/integrations' },
];

// `openKey` is a string rather than a boolean so more menus can join later without reworking the
// outside-click handling.
const openKey: Ref<string | null> = ref(null);

function toggle(key: string): void {
  openKey.value = openKey.value === key ? null : key;
}

function close(): void {
  openKey.value = null;
}

const displayName: ComputedRef<string> = computed<string>(() => {
  const full = `${user.value?.first_name ?? ''} ${user.value?.last_name ?? ''}`.trim();
  return full || user.value?.email || 'Account';
});

const initials: ComputedRef<string> = computed<string>(() => {
  const first = user.value?.first_name?.[0] ?? '';
  const last = user.value?.last_name?.[0] ?? '';
  const fromName = `${first}${last}`.trim();
  if (fromName) return fromName.toUpperCase();
  return (user.value?.email?.[0] ?? '?').toUpperCase();
});

// Close the menu on any click outside it.
function onDocClick(e: MouseEvent): void {
  const target = e.target as HTMLElement;
  if (!target.closest('.user-menu')) close();
}

onMounted(() => document.addEventListener('click', onDocClick));
onUnmounted(() => document.removeEventListener('click', onDocClick));
</script>

<template>
  <header class="topnav">
    <span class="topnav-brand">imacals</span>

    <ul class="nav-list">
      <li v-for="item in nav" :key="item.key" class="nav-item">
        <RouterLink
          :to="item.to"
          class="nav-link"
          :class="{ 'nav-link--active': route.path === item.to }"
          @click="close"
        >
          {{ item.label }}
        </RouterLink>
      </li>
    </ul>

    <!-- Account menu (right) -->
    <div class="user-menu">
      <button
        class="user-menu-trigger"
        :class="{ 'user-menu-trigger--open': openKey === 'user' }"
        :aria-expanded="openKey === 'user'"
        aria-label="Account menu"
        type="button"
        @click.stop="toggle('user')"
      >
        <span class="user-avatar" aria-hidden="true">{{ initials }}</span>
        <span class="user-name">{{ displayName }}</span>
        <span class="nav-chevron" aria-hidden="true">{{ openKey === 'user' ? '▴' : '▾' }}</span>
      </button>

      <ul v-if="openKey === 'user'" class="dropdown dropdown--right">
        <li>
          <button class="dropdown-item dropdown-item--btn" type="button" @click.stop="toggleTheme">
            <span class="dropdown-icon" aria-hidden="true">{{ isDark ? '☀' : '☾' }}</span>
            {{ isDark ? 'Light mode' : 'Dark mode' }}
          </button>
        </li>
        <li class="dropdown-divider" aria-hidden="true"></li>
        <li>
          <button class="dropdown-item dropdown-item--btn" type="button" @click="$emit('logout')">
            <span class="dropdown-icon" aria-hidden="true">⇥</span>
            Sign out
          </button>
        </li>
      </ul>
    </div>
  </header>
</template>

<style scoped>
.topnav {
  display: flex;
  align-items: stretch;
  background-color: var(--color-surface);
  border-bottom: 1px solid var(--color-border);
  padding: 0 var(--spacing-lg);
  position: sticky;
  top: 0;
  z-index: 10;
}

.topnav-brand {
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 500;
  color: var(--color-primary);
  display: flex;
  align-items: center;
  padding-right: var(--spacing-lg);
  margin-right: var(--spacing-md);
  border-right: 1px solid var(--color-border);
}

.nav-list {
  list-style: none;
  display: flex;
  align-items: stretch;
  gap: 0;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 12px var(--spacing-md);
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
  text-decoration: none;
  border-bottom: 2px solid transparent;
  white-space: nowrap;
  transition: color 0.1s;
}

.nav-link:hover {
  color: var(--color-primary);
}

.nav-link--active {
  color: var(--color-primary);
  border-bottom-color: var(--color-tertiary);
  font-weight: 500;
}

.nav-chevron {
  font-size: 0.5rem;
  color: var(--color-secondary);
}

/* ── Account menu ───────────────────────────────────────────────────────── */
.user-menu {
  position: relative;
  margin-left: auto;
  display: flex;
  align-items: center;
}

.user-menu-trigger {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  background: none;
  border: none;
  cursor: pointer;
  padding: 6px 8px;
  border-radius: var(--rounded-md);
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
}

.user-menu-trigger:hover,
.user-menu-trigger--open {
  color: var(--color-primary);
  background-color: var(--color-neutral);
}

.user-avatar {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border-radius: 50%;
  background-color: var(--color-primary);
  color: var(--color-on-primary);
  font-family: var(--font-label);
  font-size: 0.6875rem;
  letter-spacing: 0.02em;
}

.user-name {
  white-space: nowrap;
}

/* ── Dropdown ───────────────────────────────────────────────────────────── */
.dropdown {
  position: absolute;
  top: calc(100% + 4px);
  z-index: 100;
  list-style: none;
  background-color: var(--color-surface);
  border: 1px solid var(--color-border);
  border-radius: var(--rounded-md);
  box-shadow: 0 4px 16px color-mix(in srgb, var(--color-primary) 12%, transparent);
  min-width: 180px;
  padding: 4px 0;
}

/* Right-aligned so the menu can't overflow the viewport edge. */
.dropdown--right {
  right: 0;
  left: auto;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: var(--spacing-sm);
  width: 100%;
  padding: 9px var(--spacing-md);
  font-family: var(--font-body);
  font-size: 0.875rem;
  color: var(--color-secondary);
  text-decoration: none;
  text-align: left;
  border-left: 3px solid transparent;
  transition: color 0.1s, background-color 0.1s;
}

/* Clear the user-agent button styling so a button item matches a link item. */
.dropdown-item--btn {
  background: none;
  border-top: none;
  border-right: none;
  border-bottom: none;
  cursor: pointer;
}

.dropdown-item:hover {
  color: var(--color-primary);
  background-color: var(--color-neutral);
}

.dropdown-icon {
  width: 1em;
  text-align: center;
  font-size: 0.875rem;
}

.dropdown-divider {
  height: 1px;
  margin: 4px 0;
  background-color: var(--color-divider);
}
</style>
