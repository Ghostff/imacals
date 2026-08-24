import { createRouter, createWebHistory } from 'vue-router';
import AuthView from '@/views/AuthView.vue';
import DashboardView from '@/views/DashboardView.vue';
import HomeView from '@/views/HomeView.vue';
import AdminMapView from '@/views/AdminMapView.vue';
import PlaceholderView from '@/views/PlaceholderView.vue';
import DomainsView from '@/views/DomainsView.vue';
import UsersAllView from '@/views/UsersAllView.vue';
import UserProfileView from '@/views/UserProfileView.vue';
import SystemUsersView from '@/views/SystemUsersView.vue';
import IntegrationsView from '@/views/IntegrationsView.vue';
import ProductsView from '@/views/ProductsView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', name: 'login', component: AuthView, meta: { public: true } },
    {
      path: '/',
      component: DashboardView,
      children: [
        { path: '',                          name: 'dashboard',           component: HomeView },
        { path: 'products',                  name: 'products',            component: ProductsView },
        { path: 'find-property',            name: 'find-property',       component: PlaceholderView },
        { path: 'offers/draft',             name: 'offers-draft',        component: PlaceholderView },
        { path: 'offers/submitted',         name: 'offers-submitted',    component: PlaceholderView },
        { path: 'offers/under-contract',    name: 'offers-under-contract', component: PlaceholderView },
        { path: 'offers/closed',            name: 'offers-closed',       component: PlaceholderView },
        { path: 'offers/sold',              name: 'offers-sold',         component: PlaceholderView },
        { path: 'admin/map',                name: 'admin-map',           component: AdminMapView },
        { path: 'flow',                      name: 'flow',                    component: PlaceholderView },
        { path: 'models',                    name: 'models',                  component: PlaceholderView },
        { path: 'models/calendar',           name: 'models-calendar',         component: PlaceholderView },
        { path: 'models/domain',             name: 'models-domain',           component: DomainsView },
        { path: 'models/contract-template',  name: 'models-contract-template', component: PlaceholderView },
        { path: 'integrations',              name: 'integrations',            component: IntegrationsView },
        { path: 'models/integrations',       redirect: { name: 'integrations' } },
        { path: 'users/all',                name: 'users-all',           component: UsersAllView },
        { path: 'users/:id',                name: 'user-profile',        component: UserProfileView },
        { path: 'users/system',             name: 'users-system',        component: SystemUsersView },
      ],
    },
    { path: '/:pathMatch(.*)*', redirect: '/' },
  ],
});

// Redirect unauthenticated users to /login; redirect authenticated users away from /login.
router.beforeEach((to) => {
  const token = localStorage.getItem('token');
  // Carry the intended path so a deep link survives the sign-in round trip.
  if (!to.meta.public && !token) return { name: 'login', query: { redirect: to.fullPath } };
  if (to.name === 'login' && token) return { name: 'integrations' };
});

export default router;
