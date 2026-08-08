import { createRouter, createWebHistory } from 'vue-router';
import AuthView from '@/views/AuthView.vue';
import DashboardView from '@/views/DashboardView.vue';
import HomeView from '@/views/HomeView.vue';
import UsersAllView from '@/views/UsersAllView.vue';
import UserProfileView from '@/views/UserProfileView.vue';

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/login', name: 'login', component: AuthView, meta: { public: true } },
    {
      path: '/',
      component: DashboardView,
      children: [
        { path: '',            name: 'dashboard',    component: HomeView },
        { path: 'users/all',   name: 'users-all',    component: UsersAllView },
        { path: 'users/:id',   name: 'user-profile', component: UserProfileView },
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
  if (to.name === 'login' && token) return { name: 'users-all' };
});

export default router;
