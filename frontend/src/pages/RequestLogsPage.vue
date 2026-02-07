<script setup lang="ts">
import { onMounted, ref, computed } from "vue";
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
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { Badge } from "@/components/ui/badge";
import { Loader2, Eye, ChevronLeft, ChevronRight, RefreshCw } from "lucide-vue-next";
import type { RequestLog } from "@/api/request-logs";

const store = useRequestLogsStore();

const isDetailSheetOpen = ref(false);
const selectedLog = ref<RequestLog | null>(null);

const limit = ref(20);
const offset = ref(0);

onMounted(() => {
  loadLogs();
});

async function loadLogs() {
  await store.loadRequestLogs(limit.value, offset.value);
}

function openDetailSheet(log: RequestLog) {
  selectedLog.value = log;
  isDetailSheetOpen.value = true;
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
      <SheetContent class="w-[900px] sm:max-w-[900px] overflow-y-auto">
        <SheetHeader class="mb-6">
          <SheetTitle>Request Details</SheetTitle>
          <SheetDescription>
            ID: {{ selectedLog?.request_id }}
          </SheetDescription>
        </SheetHeader>
        
        <div v-if="selectedLog" class="space-y-6">
          <div class="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span class="font-semibold block text-muted-foreground">Created At</span>
              <span>{{ formatDate(selectedLog.created_at) }}</span>
            </div>
            <div>
              <span class="font-semibold block text-muted-foreground">Status Code</span>
              <Badge :variant="selectedLog.status_code && selectedLog.status_code >= 200 && selectedLog.status_code < 300 ? 'default' : 'destructive'">
                {{ selectedLog.status_code || '-' }}
              </Badge>
            </div>
            <div>
              <span class="font-semibold block text-muted-foreground">Latency</span>
              <span>{{ selectedLog.latency_ms ? `${selectedLog.latency_ms}ms` : '-' }}</span>
            </div>
            <div>
              <span class="font-semibold block text-muted-foreground">Client IP</span>
              <span>{{ selectedLog.client_ip || '-' }}</span>
            </div>
            <div>
              <span class="font-semibold block text-muted-foreground">Provider</span>
              <span>{{ selectedLog.provider || '-' }}</span>
            </div>
             <div>
              <span class="font-semibold block text-muted-foreground">Endpoint</span>
              <span>{{ selectedLog.endpoint || '-' }}</span>
            </div>
            <div>
              <span class="font-semibold block text-muted-foreground">Model</span>
              <span>{{ selectedLog.model || '-' }}</span>
            </div>
            <div>
              <span class="font-semibold block text-muted-foreground">Alias</span>
              <span>{{ selectedLog.alias || '-' }}</span>
            </div>
             <div>
              <span class="font-semibold block text-muted-foreground">Gateway Key ID</span>
              <span class="font-mono text-xs">{{ selectedLog.gateway_key_id || '-' }}</span>
            </div>
             <div>
              <span class="font-semibold block text-muted-foreground">User Agent</span>
              <span class="text-xs break-all">{{ selectedLog.user_agent || '-' }}</span>
            </div>
          </div>

          <div class="space-y-2">
            <h3 class="font-semibold">Request Body</h3>
            <div class="rounded-md bg-muted p-4 overflow-x-auto">
              <pre class="text-xs font-mono whitespace-pre-wrap break-all">{{ decodeBody(selectedLog.request_body) }}</pre>
            </div>
          </div>

          <div class="space-y-2">
            <h3 class="font-semibold">Response Body</h3>
            <div class="rounded-md bg-muted p-4 overflow-x-auto">
              <pre class="text-xs font-mono whitespace-pre-wrap break-all">{{ decodeBody(selectedLog.response_body) }}</pre>
            </div>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  </div>
</template>
