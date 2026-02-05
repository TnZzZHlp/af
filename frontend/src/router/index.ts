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
  ],
})

export default router
