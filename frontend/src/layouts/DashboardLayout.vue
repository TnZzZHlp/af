<script setup lang="ts">
import { SidebarProvider, SidebarTrigger, SidebarInset } from "@/components/ui/sidebar"
import AppSidebar from "@/components/AppSidebar.vue"
import { Separator } from "@/components/ui/separator"
import { RouterView } from "vue-router"
</script>

<template>
  <SidebarProvider>
    <AppSidebar />
    <SidebarInset>
      <header class="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger class="-ml-1" />
        <Separator orientation="vertical" class="mr-2 h-4" />
        <span class="font-semibold">Dashboard</span>
      </header>
      <div class="p-4">
        <RouterView />
      </div>
    </SidebarInset>
  </SidebarProvider>
</template>
