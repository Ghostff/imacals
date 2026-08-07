<script setup lang="ts">
import { computed, type ComputedRef } from 'vue';
import { useRoute } from 'vue-router';

const route = useRoute();

const title: ComputedRef<string> = computed(() => {
  const name = route.name as string | undefined;
  if (!name) return 'Page';
  return name
    .split('-')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join(' ');
});
</script>

<template>
  <div class="page">
    <p class="page-label">Section</p>
    <h1 class="page-title">{{ title }}</h1>
  </div>
</template>

<style scoped>
.page {
  padding: 48px var(--spacing-lg);
}

.page-label {
  font-family: var(--font-label);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-secondary);
  margin-bottom: var(--spacing-sm);
}

.page-title {
  font-family: var(--font-display);
  font-size: 2.5rem;
  font-weight: 500;
  letter-spacing: -0.02em;
  color: var(--color-primary);
}
</style>
