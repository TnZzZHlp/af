<script setup lang="ts">
import { onMounted, computed, ref, watch } from 'vue'
import { useStatsStore } from '@/stores/stats'
import { VisXYContainer, VisAxis, VisGroupedBar } from '@unovis/vue'
import { ChartContainer, ChartTooltip, ChartTooltipContent, componentToString, ChartCrosshair } from '@/components/ui/chart'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"

const statsStore = useStatsStore()
const timeRange = ref("24h")

function fetchStatsForRange(range: string) {
  const end = new Date()
  const start = new Date()

  switch (range) {
    case '24h':
      start.setTime(end.getTime() - 24 * 60 * 60 * 1000)
      break
    case '7d':
      start.setTime(end.getTime() - 7 * 24 * 60 * 60 * 1000)
      break
    default:
      start.setTime(end.getTime() - 7 * 24 * 60 * 60 * 1000)
  }

  statsStore.fetchStats({
    start: start.toISOString(),
    end: end.toISOString()
  })
}

onMounted(() => {
  fetchStatsForRange(timeRange.value)
})

watch(timeRange, (newVal) => {
  fetchStatsForRange(newVal)
})

const requestsData = computed(() => {
  if (!statsStore.stats) return []
  return statsStore.stats.requests_over_time.map(d => ({
    time: new Date(d.time).getTime(),
    requests: d.count
  }))
})

const totalRequests = computed(() => {
  if (!requestsData.value) return 0
  return requestsData.value.reduce((acc, curr) => acc + curr.requests, 0).toLocaleString()
})

const cacheHitRate = computed(() => {
  if (!statsStore.stats) return "0.00%"
  return `${(statsStore.stats.cache_hit_rate * 100).toFixed(2)}%`
})

const cacheHitRequests = computed(() => {
  return statsStore.stats ? statsStore.stats.cache_hit_requests.toLocaleString() : "0"
})

const cacheTotalRequests = computed(() => {
  return statsStore.stats ? statsStore.stats.cache_total_requests.toLocaleString() : "0"
})

const requestsChartConfig = {
  requests: {
    label: "Requests",
    color: "hsl(var(--primary))",
  },
}

function xTickFormat(d: number) {
  const options: Intl.DateTimeFormatOptions = {
    hour: 'numeric',
    minute: 'numeric',
  }

  if (['24h', '7d'].includes(timeRange.value)) {
    options.month = 'short'
    options.day = 'numeric'
  }

  return new Date(d).toLocaleString(undefined, options)
}
</script>

<template>
  <div class="space-y-6 h-full flex flex-col min-h-0 overflow-y-auto">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Dashboard</h1>
        <p class="text-muted-foreground">Overview of your AI Gateway usage.</p>
      </div>
    </div>

    <div v-if="statsStore.loading && !statsStore.stats" class="text-center py-10">
      Loading stats...
    </div>
    <div v-else-if="statsStore.error" class="rounded-md bg-destructive/15 p-4 text-destructive text-sm">
      {{ statsStore.error }}
    </div>

    <div v-if="statsStore.stats" class="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
      <Card class="col-span-4 py-0">
        <CardHeader class="flex flex-col items-stretch border-b p-0! sm:flex-row">
          <div class="flex flex-1 flex-col justify-center gap-1 px-6 py-5 sm:py-6">
            <div class="flex items-center justify-between">
              <CardTitle>Requests Over Time</CardTitle>
              <Select v-model="timeRange">
                <SelectTrigger class="w-40 rounded-lg" aria-label="Select a value">
                  <SelectValue placeholder="Last 24 hours" />
                </SelectTrigger>
                <SelectContent class="rounded-xl">
                  <SelectItem value="24h" class="rounded-lg">
                    Last 24 hours
                  </SelectItem>
                  <SelectItem value="7d" class="rounded-lg">
                    Last 7 days
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <CardDescription>
              Showing total requests for the selected period
            </CardDescription>
          </div>
          <div class="flex">
            <div
              class="flex flex-1 flex-col justify-center gap-1 border-t px-6 py-4 text-left even:border-l sm:border-t-0 sm:border-l sm:px-8 sm:py-6">
              <span class="text-muted-foreground text-xs">
                Total Requests
              </span>
              <span class="text-lg leading-none font-bold sm:text-3xl">
                {{ totalRequests }}
              </span>
            </div>
          </div>
        </CardHeader>
        <CardContent class="px-2 sm:p-6">
          <ChartContainer :config="requestsChartConfig" class="aspect-auto h-62.5 w-full">
            <VisXYContainer :data="requestsData" :margin="{ left: -24 }" :height="250">
              <VisGroupedBar :x="(d: any) => d.time" :y="(d: any) => d.requests" color="hsl(var(--primary))"
                :bar-padding="0.5" :rounded-corners="0.5" />
              <VisAxis type="x" :tickFormat="xTickFormat" :tickLine="false" :domainLine="false" :gridLine="false" />
              <VisAxis type="y" :numTicks="3" :tickLine="false" :domainLine="false" :gridLine="true" />
              <ChartTooltip :content="ChartTooltipContent" />
              <ChartCrosshair :template="componentToString(requestsChartConfig, ChartTooltipContent, {
                labelFormatter: (d) => {
                  return new Date(d).toLocaleString(undefined, {
                    month: 'short',
                    day: 'numeric',
                    hour: 'numeric',
                    minute: 'numeric'
                  })
                },
              })" />
            </VisXYContainer>
          </ChartContainer>
        </CardContent>
      </Card>

      <Card class="col-span-3">
        <CardHeader>
          <CardTitle>Cache Hit Rate</CardTitle>
          <CardDescription>
            Cache performance for the selected period
          </CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="text-4xl font-bold leading-none">{{ cacheHitRate }}</div>
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div class="space-y-1">
              <p class="text-muted-foreground">Cache Hits</p>
              <p class="font-semibold">{{ cacheHitRequests }}</p>
            </div>
            <div class="space-y-1">
              <p class="text-muted-foreground">Total Requests</p>
              <p class="font-semibold">{{ cacheTotalRequests }}</p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  </div>
</template>
