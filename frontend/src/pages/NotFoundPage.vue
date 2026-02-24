<script setup lang="ts">
import { computed } from 'vue'
import { RouterLink, useRoute } from 'vue-router'
import { Button } from '@/components/ui/button'
import { useAuthStore } from '@/stores/auth'

const auth = useAuthStore()
const route = useRoute()

const missingPath = computed(() => route.fullPath)
const fallbackPath = computed(() => (auth.user ? '/manage/dashboard' : '/'))
const fallbackLabel = computed(() => (auth.user ? 'Back to dashboard' : 'Go to login'))
</script>

<template>
  <div
    class="min-h-screen grid place-items-center bg-[radial-gradient(ellipse_at_top_left,var(--tw-gradient-stops))] from-slate-50 via-slate-100 to-slate-200 dark:from-slate-950 dark:via-slate-900 dark:to-slate-950 p-6"
  >
    <section
      class="w-full max-w-2xl rounded-2xl border border-slate-200/70 dark:border-slate-800/80 bg-white/85 dark:bg-slate-950/80 shadow-xl shadow-slate-200/30 dark:shadow-slate-950/40 backdrop-blur px-8 py-12 text-center"
    >
      <p class="text-sm tracking-[0.2em] uppercase text-slate-500 dark:text-slate-400">Error 404</p>
      <h1 class="mt-3 text-4xl font-extrabold tracking-tight text-slate-900 dark:text-slate-50">
        Page not found
      </h1>
      <p class="mt-4 text-slate-600 dark:text-slate-300">
        The page
        <code class="rounded bg-slate-100 dark:bg-slate-800 px-2 py-1 text-sm">{{
          missingPath
        }}</code>
        does not exist.
      </p>
      <div class="mt-8 flex items-center justify-center">
        <Button as-child>
          <RouterLink :to="fallbackPath">{{ fallbackLabel }}</RouterLink>
        </Button>
      </div>
    </section>
  </div>
</template>
