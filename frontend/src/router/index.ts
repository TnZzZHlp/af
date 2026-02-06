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
      ],
    },
  ],
})

export default router
