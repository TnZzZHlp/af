<script setup lang="ts">
import { computed, reactive, ref } from "vue"
import { useAuthStore } from "@/stores/auth"
import { useRouter } from "vue-router"
import { Button } from "@/components/ui/button"
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert"
import { Activity, AlertCircle, CheckCircle2, Lock, User, Loader2 } from "lucide-vue-next"

const auth = useAuthStore()
const router = useRouter()

const form = reactive({
  username: "",
  password: "",
})

const successMessage = ref<string | null>(null)

const canSubmit = computed(() => {
  return !auth.loading && form.username.trim().length > 0 && form.password.length > 0
})

const handleSubmit = async () => {
  if (!canSubmit.value) {
    return
  }

  successMessage.value = null

  const success = await auth.login(form.username.trim(), form.password)

  if (success && auth.user) {
    const greetingName = auth.user.name ?? auth.user.username
    successMessage.value = `Welcome back, ${greetingName}.`
    form.password = ""
    setTimeout(() => {
      router.push("/dashboard")
    }, 1000)
  }
}

const animDuration = 300

function onEnter(el: Element, done: () => void) {
  const element = el as HTMLElement
  element.style.height = '0'
  element.style.opacity = '0'
  element.style.overflow = 'hidden'

  // Force reflow
  void element.offsetHeight

  element.style.transition = `height ${animDuration}ms ease-out, opacity ${animDuration}ms ease-out`
  element.style.height = `${element.scrollHeight}px`
  element.style.opacity = '1'

  setTimeout(done, animDuration)
}

function onAfterEnter(el: Element) {
  const element = el as HTMLElement
  element.style.height = 'auto'
  element.style.overflow = 'visible'
}

function onLeave(el: Element, done: () => void) {
  const element = el as HTMLElement
  element.style.height = `${element.scrollHeight}px`
  element.style.overflow = 'hidden'

  // Force reflow
  void element.offsetHeight

  element.style.transition = `height ${animDuration}ms ease-in, opacity ${animDuration}ms ease-in`
  element.style.height = '0'
  element.style.opacity = '0'

  setTimeout(done, animDuration)
}
</script>

<template>
  <div
    class="min-h-screen flex items-center justify-center p-6 bg-[radial-gradient(ellipse_at_top_left,var(--tw-gradient-stops))] from-slate-50 via-slate-100 to-slate-200">
    <!-- Ambient Background Elements -->
    <div class="fixed -top-[20%] -right-[10%] w-150 h-150 rounded-full bg-blue-100/40 blur-3xl -z-10 animate-pulse">
    </div>
    <div
      class="fixed -bottom-[20%] -left-[10%] w-125 h-125 rounded-full bg-emerald-50/60 blur-3xl -z-10 animate-pulse delay-1000">
    </div>

    <div class="w-full max-w-5xl grid lg:grid-cols-2 gap-12 items-center z-10">
      <!-- Hero / Info Section -->
      <div class="hidden lg:flex flex-col gap-6">
        <div class="space-y-4">
          <div
            class="inline-flex items-center gap-2 px-3 py-1 rounded-full border border-slate-200 bg-white/50 backdrop-blur text-xs font-semibold tracking-wider text-slate-600 uppercase">
            <Activity class="w-3.5 h-3.5 text-blue-600" />
            Gateway Console
          </div>
          <h1 class="text-5xl font-extrabold tracking-tight text-slate-900 leading-[1.1]">
            Route your models <br />
            <span class="text-transparent bg-clip-text bg-linear-to-r from-blue-600 to-emerald-600">
              with precision.
            </span>
          </h1>
          <p class="text-lg text-slate-600 leading-relaxed max-w-lg">
            Manage keys, track usage, and keep your infrastructure tuned. Secure, centralized control for your AI
            gateway.
          </p>
        </div>
      </div>

      <!-- Login Card -->
      <Card
        class="w-full max-w-100 mx-auto border-slate-200/60 shadow-xl shadow-slate-200/40 backdrop-blur-sm bg-white/90">
        <CardHeader class="space-y-1">
          <CardTitle class="text-2xl font-bold tracking-tight">Welcome back</CardTitle>
          <CardDescription>Enter your credentials to access your account.</CardDescription>
        </CardHeader>
        <CardContent>
          <form @submit.prevent="handleSubmit" class="space-y-4">
            <div class="space-y-2">
              <Label for="username">Username</Label>
              <div class="relative">
                <User class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input id="username" v-model="form.username" type="text" autocomplete="username" class="pl-9"
                  :disabled="auth.loading" />
              </div>
            </div>
            <div class="space-y-2">
              <Label for="password">Password</Label>
              <div class="relative">
                <Lock class="absolute left-3 top-2.5 h-4 w-4 text-muted-foreground" />
                <Input id="password" v-model="form.password" type="password" placeholder="••••••••"
                  autocomplete="current-password" class="pl-9" :disabled="auth.loading" />
              </div>
            </div>
            <Button type="submit" class="w-full" :disabled="!canSubmit">
              <Loader2 v-if="auth.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ auth.loading ? 'Signing in...' : 'Sign in' }}
            </Button>
          </form>
        </CardContent>
        <CardFooter class="flex flex-col gap-4">
          <Transition :css="false" @enter="onEnter" @after-enter="onAfterEnter" @leave="onLeave">
            <Alert v-if="auth.error" variant="destructive">
              <AlertCircle class="h-4 w-4" />
              <AlertTitle>Error</AlertTitle>
              <AlertDescription>{{ auth.error }}</AlertDescription>
            </Alert>
          </Transition>

          <Transition :css="false" @enter="onEnter" @after-enter="onAfterEnter" @leave="onLeave">
            <Alert v-if="successMessage"
              class="border-emerald-500/50 text-emerald-900 bg-emerald-50 dark:bg-emerald-900/20 dark:text-emerald-200">
              <CheckCircle2 class="h-4 w-4 text-emerald-600 dark:text-emerald-400" />
              <AlertTitle>Success</AlertTitle>
              <AlertDescription>{{ successMessage }}</AlertDescription>
            </Alert>
          </Transition>
        </CardFooter>
      </Card>
    </div>
  </div>
</template>
