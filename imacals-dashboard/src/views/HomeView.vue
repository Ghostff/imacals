<script setup lang="ts">
</script>

<template>
  <div class="page">
    <p class="page-label">Overview</p>
    <h1 class="page-title">Dashboard</h1>
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
