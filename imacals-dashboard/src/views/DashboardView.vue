<script setup lang="ts">
import { onMounted } from 'vue';
import { RouterView, useRouter } from 'vue-router';
import { useAuth } from '@/composables/useAuth';
import AppTopNav from '@/components/AppTopNav.vue';

const router = useRouter();
const { fetchMe, logout } = useAuth();

onMounted(fetchMe);

function handleLogout(): void {
  logout();
  router.push({ name: 'login' });
}
</script>

<template>
  <div class="shell">
    <AppTopNav @logout="handleLogout" />
    <main class="content">
      <RouterView />
    </main>
  </div>
</template>

<style scoped>
.shell {
  min-height: 100vh;
  background-color: var(--color-neutral);
  display: flex;
  flex-direction: column;
}

.content {
  flex: 1;
}
</style>
