<script setup lang="ts">
import type { SidebarProps } from '@/components/ui/sidebar'

import {
  Command,
  LayoutDashboard,
  Key,
  Blocks,
  Route,
  Activity,
} from "lucide-vue-next"

import NavUser from '@/components/NavUser.vue'
import NavMain from '@/components/NavMain.vue'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/components/ui/sidebar'

const props = withDefaults(defineProps<SidebarProps>(), {
  variant: "inset",
})

const data = {
  user: {
    name: "Admin",
    email: "admin@example.com",
    avatar: "",
  },
  navMain: [
    {
      title: "Dashboard",
      url: "/dashboard",
      icon: LayoutDashboard,
    },
    {
      title: "Gateway Keys",
      url: "/dashboard/gateway-keys",
      icon: Key,
    },
    {
      title: "Providers",
      url: "/dashboard/providers",
      icon: Blocks,
    },
    {
      title: "Aliases",
      url: "/dashboard/aliases",
      icon: Route,
    },
    {
      title: "Request Logs",
      url: "/dashboard/request-logs",
      icon: Activity,
    },
  ],
}
</script>

<template>
  <Sidebar v-bind="props">
    <SidebarHeader>
      <SidebarMenu>
        <SidebarMenuItem>
          <SidebarMenuButton size="lg" as-child>
            <router-link to="/dashboard">
              <div class="flex aspect-square size-8 items-center justify-center rounded-lg bg-sidebar-primary text-sidebar-primary-foreground">
                <Command class="size-4" />
              </div>
              <div class="grid flex-1 text-left text-sm leading-tight">
                <span class="truncate font-medium">AI Gateway</span>
                <span class="truncate text-xs">Admin Console</span>
              </div>
            </router-link>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </SidebarMenu>
    </SidebarHeader>
    <SidebarContent>
      <NavMain :items="data.navMain" />
    </SidebarContent>
    <SidebarFooter>
      <NavUser :user="data.user" />
    </SidebarFooter>
  </Sidebar>
</template>