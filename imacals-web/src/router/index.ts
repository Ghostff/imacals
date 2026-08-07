import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import CatalogView from '@/views/CatalogView.vue';
import ProductView from '@/views/ProductView.vue';
import CartView from '@/views/CartView.vue';
import CheckoutView from '@/views/CheckoutView.vue';
import DeliveryView from '@/views/DeliveryView.vue';
import TrackOrderView from '@/views/TrackOrderView.vue';
import NotFoundView from '@/views/NotFoundView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/',               name: 'home',     component: HomeView },
    { path: '/catalog',        name: 'catalog',  component: CatalogView },
    { path: '/product/:slug',  name: 'product',  component: ProductView },
    { path: '/cart',           name: 'cart',     component: CartView },
    { path: '/checkout',       name: 'checkout', component: CheckoutView },
    { path: '/delivery',       name: 'delivery', component: DeliveryView },
    { path: '/track',          name: 'track',    component: TrackOrderView },
    { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView },
  ],
  // Storefront browsing is vertical: land at the top of each product or category page.
  scrollBehavior: () => ({ top: 0 }),
});

export default router;
