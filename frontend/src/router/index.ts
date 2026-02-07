import { createRouter, createWebHistory } from "vue-router";
import LoginPage from "@/pages/LoginPage.vue";
import { useAuthStore } from "@/stores/auth";

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "login",
      component: LoginPage,
      meta: { guestOnly: true },
    },
    {
      path: "/manage",
      component: () => import("@/layouts/DashboardLayout.vue"),
      meta: { requiresAuth: true },
      children: [
        {
          path: "dashboard",
          name: "dashboard",
          component: () => import("@/pages/DashboardPage.vue"),
        },
        {
          path: "gateway-keys",
          name: "gateway-keys",
          component: () => import("@/pages/GatewayKeysPage.vue"),
        },
        {
          path: "providers",
          name: "providers",
          component: () => import("@/pages/ProvidersPage.vue"),
        },
        {
          path: "aliases",
          name: "aliases",
          component: () => import("@/pages/AliasesPage.vue"),
        },
        {
          path: "request-logs",
          name: "request-logs",
          component: () => import("@/pages/RequestLogsPage.vue"),
        },
        {
          path: "users",
          name: "users",
          component: () => import("@/pages/UsersPage.vue"),
        },
      ],
    },
  ],
});

router.beforeEach((to, from, next) => {
  const authStore = useAuthStore();
  const isAuthenticated = !!authStore.user;

  if (to.meta.requiresAuth && !isAuthenticated) {
    next({ name: "login", query: { redirect: to.fullPath } });
  } else if (to.meta.guestOnly && isAuthenticated) {
    next({ name: "dashboard" });
  } else {
    next();
  }
});

export default router;
