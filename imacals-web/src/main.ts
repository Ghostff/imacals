import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { initTheme } from '@/composables/useTheme';
import { initCart } from '@/composables/useCart';
import './style.css';

// Runs before mount so a dark session never flashes light.
initTheme();
// Restores a cart left behind by an earlier visit before the header renders its badge.
initCart();

createApp(App).use(router).mount('#app');
