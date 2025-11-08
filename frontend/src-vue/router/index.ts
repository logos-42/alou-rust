import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import HomeView from '../views/HomeView.vue'

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('../views/LoginView.vue'),
      meta: { requiresAuth: false, hideForAuth: true },
    },
    {
      path: '/auth/callback',
      name: 'auth-callback',
      component: () => import('../views/AuthCallbackView.vue'),
      meta: { requiresAuth: false },
    },
    {
      path: '/',
      name: 'home',
      component: HomeView,
      meta: { requiresAuth: false },
    },
    {
      path: '/wallet',
      name: 'wallet',
      component: () => import('../views/WalletView.vue').catch(() => import('../views/HomeView.vue')),
      meta: { requiresAuth: false },
    },
    {
      path: '/about',
      name: 'about',
      component: () => import('../views/AboutView.vue'),
      meta: { requiresAuth: false },
    },
  ],
})

// Navigation guard
router.beforeEach(async (to, from, next) => {
  const authStore = useAuthStore()
  
  // Initialize auth state on first load
  if (!authStore.isAuthenticated) {
    await authStore.init()
  }

  const requiresAuth = to.matched.some((record) => record.meta.requiresAuth)
  const hideForAuth = to.matched.some((record) => record.meta.hideForAuth)

  if (requiresAuth && !authStore.isAuthenticated) {
    // Redirect to login if authentication is required
    next({ name: 'login', query: { redirect: to.fullPath } })
  } else if (hideForAuth && authStore.isAuthenticated) {
    // Redirect to home if already authenticated and trying to access login
    next({ name: 'home' })
  } else {
    next()
  }
})

export default router
