import { createApp } from 'vue';
import App from './App.vue';
import router from './router';
import { initTheme } from '@/composables/useTheme';
import './style.css';

// FontAwesome Pro is an optional dependency (private registry — a clone without a token skips it),
// so its stylesheet loads at runtime and its absence is not fatal. The specifier is held in a
// variable and marked @vite-ignore so the bundler defers resolution to runtime instead of failing
// the build when the package isn't installed.
const FONTAWESOME_STYLES = '@fortawesome/fontawesome-svg-core/styles.css';
import(/* @vite-ignore */ FONTAWESOME_STYLES).catch(() => {
  // No icon font available; AdminMapView falls back to text labels on its map toolbar.
});

initTheme();

createApp(App).use(router).mount('#app');
