<script setup lang="ts">
import { SidebarProvider, SidebarTrigger, SidebarInset } from "@/components/ui/sidebar"
import AppSidebar from "@/components/AppSidebar.vue"
import { Separator } from "@/components/ui/separator"
import { RouterView, useRoute } from "vue-router"
import { computed } from "vue"

const route = useRoute()
const pageTitle = computed(() => {
  switch (route.name) {
    case 'dashboard': return 'Overview'
    case 'gateway-keys': return 'Gateway API Keys'
    case 'providers': return 'Providers'
    default: return 'Dashboard'
  }
})
</script>

<template>
  <SidebarProvider class="h-screen overflow-hidden">
    <AppSidebar />
    <SidebarInset class="flex flex-col overflow-hidden">
      <header class="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger class="-ml-1" />
        <Separator orientation="vertical" class="mr-2 h-4" />
        <span class="font-semibold">{{ pageTitle }}</span>
      </header>
      <div class="flex flex-1 flex-col overflow-hidden p-4 min-h-0">
        <RouterView />
      </div>
    </SidebarInset>
  </SidebarProvider>
</template>
