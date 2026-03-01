<script setup lang="ts">
import { onMounted, onUnmounted, ref, computed } from "vue";
import { useRequestLogsStore } from "@/stores/request-logs";
import { useAliasesStore } from "@/stores/aliases";
import { useProvidersStore } from "@/stores/providers";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { Loader2, Eye, ChevronLeft, ChevronRight, RefreshCw, Copy, Check, Play, Pause } from "lucide-vue-next";
import type { RequestLogSummary } from "@/api/request-logs";
import { useClipboard } from "@vueuse/core";
import "prismjs/themes/prism-tomorrow.css";
import { decodeBody, getHighlightedHtml, extractAiContent } from "@/lib/utils";
import Combobox from "@/components/Combobox.vue";

const store = useRequestLogsStore();
const aliasesStore = useAliasesStore();
const providersStore = useProvidersStore();
const { copy, copied } = useClipboard({ legacy: true });

const isDetailSheetOpen = ref(false);

const limit = ref(20);
const offset = ref(0);

const autoRefresh = ref(false);
let refreshInterval: ReturnType<typeof setInterval> | null = null;

// Track which section was just copied to show the checkmark correctly
const copiedSection = ref<string | null>(null);

const aliasOptions = computed(() =>
  aliasesStore.aliases.map((a) => ({ value: a.name, label: a.name }))
);

const providerOptions = computed(() =>
  providersStore.providers.map((p) => ({ value: p.name, label: p.name }))
);

onMounted(() => {
  loadLogs();
  aliasesStore.fetchAliases();
  providersStore.fetchProviders();
});

async function loadLogs() {
  await store.loadRequestLogs(limit.value, offset.value);
}

function searchLogs() {
  offset.value = 0;
  loadLogs();
}

function clearFilters() {
  store.filter = {};
  offset.value = 0;
  loadLogs();
}

async function openDetailSheet(log: RequestLogSummary) {
  isDetailSheetOpen.value = true;
  await store.loadRequestLog(log.request_id);
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString();
}

function formatLatency(latencyMs: number | null) {
  return latencyMs != null ? `${latencyMs}ms` : "-";
}

function formatCacheLayer(cacheLayer: string | null) {
  return cacheLayer ?? "miss";
}

function handleCopy(text: string, section: string) {
  copy(text);
  copiedSection.value = section;
  setTimeout(() => {
    if (copiedSection.value === section) {
      copiedSection.value = null;
    }
  }, 2000);
}

function previousPage() {
  if (offset.value >= limit.value) {
    offset.value -= limit.value;
    loadLogs();
  }
}

function nextPage() {
  if (offset.value + limit.value < store.total) {
    offset.value += limit.value;
    loadLogs();
  }
}

function toggleAutoRefresh() {
  autoRefresh.value = !autoRefresh.value;
  if (autoRefresh.value) {
    refreshInterval = setInterval(loadLogs, 5000);
  } else if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
}

onUnmounted(() => {
  if (refreshInterval) {
    clearInterval(refreshInterval);
    refreshInterval = null;
  }
});
</script>

<template>
  <div class="space-y-6 h-full flex flex-col min-h-0">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Request Logs</h1>
        <p class="text-muted-foreground">
          View and analyze API request logs.
        </p>
      </div>
      <div class="flex gap-2 items-center">
        <span class="text-sm text-muted-foreground mr-2">
          Page {{ Math.floor(offset / limit) + 1 }} of {{ Math.ceil(store.total / limit) || 1 }} ({{ store.total }}
          entries)
        </span>
        <Button variant="outline" size="icon" @click="toggleAutoRefresh" :class="{ 'bg-primary/10': autoRefresh }">
          <Pause v-if="autoRefresh" class="h-4 w-4" />
          <Play v-else class="h-4 w-4" />
        </Button>
        <Button variant="outline" size="icon" @click="loadLogs" :disabled="store.isLoading">
          <RefreshCw class="h-4 w-4" :class="{ 'animate-spin': store.isLoading }" />
        </Button>
        <Button variant="outline" size="icon" @click="previousPage" :disabled="offset === 0 || store.isLoading">
          <ChevronLeft class="h-4 w-4" />
        </Button>
        <Button variant="outline" size="icon" @click="nextPage"
          :disabled="offset + limit >= store.total || store.isLoading">
          <ChevronRight class="h-4 w-4" />
        </Button>
      </div>
    </div>

    <!-- Filters -->
    <div class="grid gap-2 grid-cols-2 md:grid-cols-3 lg:grid-cols-6">
      <Input v-model="store.filter.model" placeholder="Model" class="h-8" @keyup.enter="searchLogs" />

      <Combobox
        :model-value="store.filter.alias"
        :options="aliasOptions"
        placeholder="Select alias..."
        search-placeholder="Search alias..."
        empty-text="No alias found."
        unselectable
        class="h-8"
        @update:model-value="(val) => { store.filter.alias = val ? String(val) : undefined; searchLogs(); }"
      />

      <Combobox
        :model-value="store.filter.provider"
        :options="providerOptions"
        placeholder="Select provider..."
        search-placeholder="Search provider..."
        empty-text="No provider found."
        unselectable
        class="h-8"
        @update:model-value="(val) => { store.filter.provider = val ? String(val) : undefined; searchLogs(); }"
      />

      <Input v-model="store.filter.client_ip" placeholder="Client IP" class="h-8" @keyup.enter="searchLogs" />
    </div>
    <div class="flex justify-end gap-2">
      <Button variant="secondary" size="sm" @click="clearFilters">Clear Filters</Button>
      <Button size="sm" @click="searchLogs">Search</Button>
    </div>

    <div v-if="store.error"
      class="rounded-md bg-destructive/15 p-4 text-destructive text-sm flex justify-between items-center">
      <span>{{ store.error }}</span>
    </div>

    <div class="rounded-md border flex-1 min-h-0 flex flex-col">
      <Table class="flex-1 min-h-0">
        <TableHeader class="sticky top-0 bg-background z-10 shadow-sm">
          <TableRow>
            <TableHead class="w-7.5"></TableHead>
            <TableHead>Time</TableHead>
            <TableHead>Provider</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Latency</TableHead>
            <TableHead>Cache layer</TableHead>
            <TableHead>Client IP</TableHead>
            <TableHead>Model</TableHead>
            <TableHead class="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="store.isLoading && store.requestLogs.length === 0">
            <TableCell colspan="9" class="h-24 text-center">
              <Loader2 class="mx-auto h-6 w-6 animate-spin text-muted-foreground" />
            </TableCell>
          </TableRow>
          <TableRow v-else-if="store.requestLogs.length === 0">
            <TableCell colspan="9" class="h-24 text-center text-muted-foreground">
              No logs found.
            </TableCell>
          </TableRow>

          <TableRow v-for="log in store.requestLogs" :key="log.request_id">
            <TableCell></TableCell>

            <TableCell class="text-xs whitespace-nowrap">
              {{ formatDate(log.created_at) }}
            </TableCell>
            <TableCell>
              <div class="flex flex-col">
                <span class="font-medium">{{ log.provider || '-' }}</span>
                <span class="text-xs text-muted-foreground">{{ log.alias ? `Alias: ${log.alias}` : '' }}</span>
              </div>
            </TableCell>
            <TableCell>
              <Badge
                :variant="log.status_code && log.status_code >= 200 && log.status_code < 300 ? 'default' : 'destructive'">
                {{ log.status_code || '-' }}
              </Badge>
            </TableCell>
            <TableCell>
              {{ formatLatency(log.latency_ms) }}
            </TableCell>
            <TableCell>
              <Badge variant="secondary" class="h-5 text-[10px] px-1.5">
                {{ formatCacheLayer(log.cache_layer) }}
              </Badge>
            </TableCell>
            <TableCell>{{ log.client_ip || '-' }}</TableCell>
            <TableCell>{{ log.model || '-' }}</TableCell>
            <TableCell class="text-right">
              <Button variant="ghost" size="icon" @click="openDetailSheet(log)">
                <Eye class="h-4 w-4" />
              </Button>
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>

    <!-- Detail Sheet -->
    <Sheet :open="isDetailSheetOpen" @update:open="isDetailSheetOpen = $event">
      <SheetContent class="w-full sm:max-w-2xl md:max-w-225 overflow-y-auto p-0 gap-0">
        <SheetHeader class="p-6">
          <SheetTitle>Request Details</SheetTitle>
          <SheetDescription v-if="store.currentLog">
            ID: {{ store.currentLog.request_id }}
          </SheetDescription>
          <SheetDescription v-else-if="store.isLoading">
            Loading details...
          </SheetDescription>
        </SheetHeader>

        <div v-if="store.isLoading && !store.currentLog" class="p-6 flex justify-center">
          <Loader2 class="h-8 w-8 animate-spin text-muted-foreground" />
        </div>

        <div v-if="store.currentLog">
          <Tabs default-value="overview" class="w-full">
            <TabsList class="mx-6">
              <TabsTrigger value="overview">Overview</TabsTrigger>
              <TabsTrigger value="content">Content</TabsTrigger>
              <TabsTrigger value="request">Request Body</TabsTrigger>
              <TabsTrigger value="response">Response Body</TabsTrigger>
            </TabsList>

            <TabsContent value="overview" class="p-6 space-y-4">
              <div class="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span class="font-semibold block text-muted-foreground">Created At</span>
                  <span>{{ formatDate(store.currentLog.created_at) }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Status Code</span>
                  <Badge
                    :variant="store.currentLog.status_code && store.currentLog.status_code >= 200 && store.currentLog.status_code < 300 ? 'default' : 'destructive'">
                    {{ store.currentLog.status_code || '-' }}
                  </Badge>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Latency</span>
                  <div class="flex items-center gap-2">
                    <span>{{ formatLatency(store.currentLog.latency_ms) }}</span>
                    <Badge variant="secondary" class="h-5 text-[10px] px-1.5">
                      {{ formatCacheLayer(store.currentLog.cache_layer) }}
                    </Badge>
                  </div>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Client IP</span>
                  <span>{{ store.currentLog.client_ip || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Provider</span>
                  <span>{{ store.currentLog.provider || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Endpoint</span>
                  <span>{{ store.currentLog.endpoint || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Model</span>
                  <span>{{ store.currentLog.model || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Alias</span>
                  <span>{{ store.currentLog.alias || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Request Content Type</span>
                  <span>{{ store.currentLog.request_content_type || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Response Content Type</span>
                  <span>{{ store.currentLog.response_content_type || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Gateway Key ID</span>
                  <span class="font-mono text-xs">{{ store.currentLog.gateway_key_id || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">User Agent</span>
                  <span class="text-xs break-all">{{ store.currentLog.user_agent || '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Prompt Tokens</span>
                  <span>{{ store.currentLog.prompt_tokens ?? '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Completion Tokens</span>
                  <span>{{ store.currentLog.completion_tokens ?? '-' }}</span>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Total Tokens</span>
                  <span>{{ store.currentLog.total_tokens ?? '-' }}</span>
                </div>
              </div>
            </TabsContent>

            <TabsContent value="content" class="p-0">
              <div class="flex items-center justify-between px-6 py-4 border-b">
                <h3 class="font-semibold">Generated Content</h3>
                <Button variant="ghost" size="sm" class="h-8 w-8 p-0"
                  @click="handleCopy(extractAiContent(store.currentLog.response_body, store.currentLog.api_type) || '', 'content')">
                  <Check v-if="copied && copiedSection === 'content'" class="h-4 w-4 text-green-500" />
                  <Copy v-else class="h-4 w-4" />
                </Button>
              </div>
              <div class="overflow-x-auto max-h-150 p-6">
                <div class="text-sm whitespace-pre-wrap leading-relaxed">
                  {{ extractAiContent(store.currentLog.response_body, store.currentLog.api_type) ||
                    'No content extracted.' }}
                </div>
              </div>
            </TabsContent>

            <TabsContent value="request" class="p-0">
              <div class="flex items-center justify-between px-6 py-4 border-b">
                <h3 class="font-semibold">Request Body</h3>
                <Button variant="ghost" size="sm" class="h-8 w-8 p-0"
                  @click="handleCopy(decodeBody(store.currentLog.request_body), 'request')">
                  <Check v-if="copied && copiedSection === 'request'" class="h-4 w-4 text-green-500" />
                  <Copy v-else class="h-4 w-4" />
                </Button>
              </div>
              <div class="overflow-x-auto max-h-150">
                <pre class="text-sm font-mono whitespace-pre-wrap! break-all! p-4 language-json"
                  v-html="getHighlightedHtml(store.currentLog.request_body)"></pre>
              </div>
            </TabsContent>

            <TabsContent value="response" class="p-0">
              <div class="flex items-center justify-between px-6 py-4 border-b">
                <h3 class="font-semibold">Response Body</h3>
                <Button variant="ghost" size="sm" class="h-8 w-8 p-0"
                  @click="handleCopy(decodeBody(store.currentLog.response_body), 'response')">
                  <Check v-if="copied && copiedSection === 'response'" class="h-4 w-4 text-green-500" />
                  <Copy v-else class="h-4 w-4" />
                </Button>
              </div>
              <div class="overflow-x-auto max-h-150">
                <pre class="text-sm font-mono whitespace-pre-wrap! break-all! p-4 language-json"
                  v-html="getHighlightedHtml(store.currentLog.response_body)"></pre>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </SheetContent>
    </Sheet>
  </div>
</template>
