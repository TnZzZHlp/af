<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRequestLogsStore } from "@/stores/request-logs";
import { Button } from "@/components/ui/button";
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
import { Loader2, Eye, ChevronLeft, ChevronRight, RefreshCw, Copy, Check } from "lucide-vue-next";
import type { RequestLogSummary } from "@/api/request-logs";
import { useClipboard } from "@vueuse/core";
import Prism from "prismjs";
import "prismjs/themes/prism-tomorrow.css";
import "prismjs/components/prism-json";

const store = useRequestLogsStore();
const { copy, copied } = useClipboard({ legacy: true });

const isDetailSheetOpen = ref(false);

const limit = ref(20);
const offset = ref(0);

// Track which section was just copied to show the checkmark correctly
const copiedSection = ref<string | null>(null);

onMounted(() => {
  loadLogs();
});

async function loadLogs() {
  await store.loadRequestLogs(limit.value, offset.value);
}

async function openDetailSheet(log: RequestLogSummary) {
  isDetailSheetOpen.value = true;
  await store.loadRequestLog(log.request_id);
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString();
}

function decodeBody(body: number[] | null): string {
  if (!body) return "Empty";
  try {
    const text = new TextDecoder().decode(new Uint8Array(body));
    try {
      // Try to format JSON
      return JSON.stringify(JSON.parse(text), null, 2);
    } catch {
      return text;
    }
  } catch {
    return `[Binary Data: ${body.length} bytes]`;
  }
}

function highlightBody(body: number[] | null): string {
  const text = decodeBody(body);
  // Basic detection if it looks like JSON or if decodeBody formatted it (it would be valid JSON)
  // We can try to parse it again to be sure, or just rely on decodeBody's formatting.
  // Since decodeBody already formatted it, it should be valid JSON if it was formatted.
  // But decodeBody also returns plain text if it failed to parse.
  
  // Let's check if it starts with { or [
  const trimmed = text.trim();
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
     return Prism.highlight(text, Prism.languages.json as Prism.Grammar, "json");
  }
  return text; // Return plain text (will be rendered as HTML, so be careful with XSS? 
               // decodeBody returns JSON stringify or text. 
               // If text contains HTML, v-html will render it. 
               // We should probably escape plain text if we are not highlighting it, 
               // OR trust that the API returns safe content? 
               // Logs can contain anything. We should escape it if not highlighting.
}

// Helper to safely render: if highlighted, it's HTML. If not, we should probably escape it?
// Actually Prism.highlight returns HTML.
// If we return plain text, we should escape it to be safe when using v-html.
function getHighlightedHtml(body: number[] | null): string {
  const text = decodeBody(body);
  const trimmed = text.trim();
  if (trimmed.startsWith("{") || trimmed.startsWith("[")) {
     try {
       return Prism.highlight(text, Prism.languages.json as Prism.Grammar, "json");
     } catch (e) {
       console.warn("Prism highlight failed", e);
     }
  }
  // Escape HTML characters for safety since we are using v-html
  return text.replace(/&/g, "&amp;")
             .replace(/</g, "&lt;")
             .replace(/>/g, "&gt;")
             .replace(/"/g, "&quot;")
             .replace(/'/g, "&#039;");
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
  if (store.requestLogs.length === limit.value) {
    offset.value += limit.value;
    loadLogs();
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Request Logs</h1>
        <p class="text-muted-foreground">
          View and analyze API request logs.
        </p>
      </div>
      <div class="flex gap-2">
        <Button variant="outline" size="icon" @click="loadLogs" :disabled="store.isLoading">
           <RefreshCw class="h-4 w-4" :class="{ 'animate-spin': store.isLoading }" />
        </Button>
        <Button variant="outline" size="icon" @click="previousPage" :disabled="offset === 0 || store.isLoading">
          <ChevronLeft class="h-4 w-4" />
        </Button>
        <Button variant="outline" size="icon" @click="nextPage" :disabled="store.requestLogs.length < limit || store.isLoading">
          <ChevronRight class="h-4 w-4" />
        </Button>
      </div>
    </div>

    <div v-if="store.error"
      class="rounded-md bg-destructive/15 p-4 text-destructive text-sm flex justify-between items-center">
      <span>{{ store.error }}</span>
    </div>

    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Time</TableHead>
            <TableHead>Method/Path</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Latency</TableHead>
            <TableHead>Client IP</TableHead>
            <TableHead>Model</TableHead>
            <TableHead class="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="store.isLoading && store.requestLogs.length === 0">
            <TableCell colspan="7" class="h-24 text-center">
              <Loader2 class="mx-auto h-6 w-6 animate-spin text-muted-foreground" />
            </TableCell>
          </TableRow>
          <TableRow v-else-if="store.requestLogs.length === 0">
            <TableCell colspan="7" class="h-24 text-center text-muted-foreground">
              No logs found.
            </TableCell>
          </TableRow>

          <TableRow v-for="log in store.requestLogs" :key="log.request_id">
            <TableCell class="text-xs whitespace-nowrap">
              {{ formatDate(log.created_at) }}
            </TableCell>
            <TableCell>
              <div class="flex flex-col">
                <span class="font-medium">{{ log.provider || '-' }} / {{ log.endpoint || '-' }}</span>
                <span class="text-xs text-muted-foreground">{{ log.alias ? `Alias: ${log.alias}` : '' }}</span>
              </div>
            </TableCell>
            <TableCell>
              <Badge :variant="log.status_code && log.status_code >= 200 && log.status_code < 300 ? 'default' : 'destructive'">
                {{ log.status_code || '-' }}
              </Badge>
            </TableCell>
            <TableCell>{{ log.latency_ms ? `${log.latency_ms}ms` : '-' }}</TableCell>
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
      <SheetContent class="w-full sm:max-w-2xl md:max-w-[900px] overflow-y-auto p-0 gap-0">
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
                  <Badge :variant="store.currentLog.status_code && store.currentLog.status_code >= 200 && store.currentLog.status_code < 300 ? 'default' : 'destructive'">
                    {{ store.currentLog.status_code || '-' }}
                  </Badge>
                </div>
                <div>
                  <span class="font-semibold block text-muted-foreground">Latency</span>
                  <span>{{ store.currentLog.latency_ms ? `${store.currentLog.latency_ms}ms` : '-' }}</span>
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
              </div>
            </TabsContent>
            
            <TabsContent value="request" class="p-0">
              <div class="flex items-center justify-between px-6 py-4 border-b">
                <h3 class="font-semibold">Request Body</h3>
                <Button variant="ghost" size="sm" class="h-8 w-8 p-0" @click="handleCopy(decodeBody(store.currentLog.request_body), 'request')">
                  <Check v-if="copied && copiedSection === 'request'" class="h-4 w-4 text-green-500" />
                  <Copy v-else class="h-4 w-4" />
                </Button>
              </div>
              <div class="bg-muted overflow-x-auto max-h-[600px]">
                <pre class="text-sm font-mono whitespace-pre-wrap break-all p-4" v-html="getHighlightedHtml(store.currentLog.request_body)"></pre>
              </div>
            </TabsContent>
            
            <TabsContent value="response" class="p-0">
              <div class="flex items-center justify-between px-6 py-4 border-b">
                <h3 class="font-semibold">Response Body</h3>
                 <Button variant="ghost" size="sm" class="h-8 w-8 p-0" @click="handleCopy(decodeBody(store.currentLog.response_body), 'response')">
                  <Check v-if="copied && copiedSection === 'response'" class="h-4 w-4 text-green-500" />
                  <Copy v-else class="h-4 w-4" />
                </Button>
              </div>
              <div class="bg-muted overflow-x-auto max-h-[600px]">
                <pre class="text-sm font-mono whitespace-pre-wrap break-all p-4" v-html="getHighlightedHtml(store.currentLog.response_body)"></pre>
              </div>
            </TabsContent>
          </Tabs>
        </div>
      </SheetContent>
    </Sheet>
  </div>
</template>
