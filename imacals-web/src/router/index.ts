import { createRouter, createWebHistory } from 'vue-router';
import HomeView from '@/views/HomeView.vue';
import CatalogView from '@/views/CatalogView.vue';
import ProductView from '@/views/ProductView.vue';
import CartView from '@/views/CartView.vue';
import CheckoutView from '@/views/CheckoutView.vue';
import DeliveryView from '@/views/DeliveryView.vue';
import TrackOrderView from '@/views/TrackOrderView.vue';
import RegisterView from '@/views/RegisterView.vue';
import LoginView from '@/views/LoginView.vue';
import AccountView from '@/views/AccountView.vue';
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
    { path: '/register',       name: 'register', component: RegisterView, meta: { guestOnly: true } },
    { path: '/login',          name: 'login',    component: LoginView,    meta: { guestOnly: true } },
    { path: '/account',        name: 'account',  component: AccountView,  meta: { requiresAuth: true } },
    { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView },
  ],
  // Storefront browsing is vertical: land at the top of each product or category page.
  scrollBehavior: () => ({ top: 0 }),
});

router.beforeEach((to) => {
  const token = localStorage.getItem('token');
  if (to.meta.requiresAuth && !token) {
    return { name: 'login', query: { redirect: to.fullPath } };
  }
  if (to.meta.guestOnly && token) {
    return { name: 'account' };
  }
});

export default router;
