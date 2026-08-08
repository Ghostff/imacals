import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { initTheme } from '@/composables/useTheme';
import './style.css';

// Runs before mount so a dark session never flashes light.
initTheme();

createApp(App).use(router).mount('#app');
