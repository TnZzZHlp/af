import { createRouter, createWebHistory } from "vue-router"
import LoginPage from "@/pages/LoginPage.vue"

const router = createRouter({
  history: createWebHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: "/",
      name: "login",
      component: LoginPage,
    },
    {
      path: "/dashboard",
      component: () => import("@/layouts/DashboardLayout.vue"),
      children: [
        {
          path: "",
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
})

export default router
