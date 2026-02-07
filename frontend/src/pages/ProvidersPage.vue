<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useProvidersStore } from "@/stores/providers";
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
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import {
  Plus,
  Trash2,
  Edit,
  Loader2,
  MoreVertical,
  ExternalLink,
  ShieldCheck,
  Check,
  Copy,
} from "lucide-vue-next";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { ApiType, Provider, ProviderEndpoint, ProviderKey } from "@/api/providers";

const store = useProvidersStore();

const isProviderSheetOpen = ref(false);
const isEditingProvider = ref(false);
const editingProviderId = ref<string | null>(null);

const providerForm = ref({
  name: "",
  description: "",
});

// Endpoint State
const isEndpointSheetOpen = ref(false);
const isEditingEndpoint = ref(false);
const editingEndpointId = ref<string | null>(null);
const currentProviderId = ref<string | null>(null);

const endpointForm = ref({
  api_type: 'openai_chat_completions' as ApiType,
  url: "",
});

// Key State
const isKeySheetOpen = ref(false);
const isEditingKey = ref(false);
const editingKeyId = ref<string | null>(null);

const keyForm = ref({
  name: "",
  key: "",
});

onMounted(() => {
  store.fetchProviders();
});

function openCreateProviderSheet() {
  isEditingProvider.value = false;
  editingProviderId.value = null;
  providerForm.value = {
    name: "",
    description: "",
  };
  isProviderSheetOpen.value = true;
}

function openEditProviderSheet(provider: Provider) {
  isEditingProvider.value = true;
  editingProviderId.value = provider.id;
  providerForm.value = {
    name: provider.name,
    description: provider.description || "",
  };
  isProviderSheetOpen.value = true;
}

async function handleProviderSubmit() {
  const payload = {
    name: providerForm.value.name,
    description: providerForm.value.description || undefined,
  };

  if (isEditingProvider.value && editingProviderId.value) {
    await store.patchProvider(editingProviderId.value, payload);
  } else {
    await store.addProvider(payload);
  }

  if (!store.error) {
    isProviderSheetOpen.value = false;
  }
}

async function handleDeleteProvider(id: string) {
  if (confirm("Are you sure you want to delete this provider? This will also delete all associated endpoints and keys.")) {
    await store.removeProvider(id);
  }
}

async function toggleProviderEnabled(provider: Provider) {
  await store.patchProvider(provider.id, { enabled: !provider.enabled });
}

// Endpoint Handlers
function openCreateEndpointSheet(providerId: string) {
  currentProviderId.value = providerId;
  isEditingEndpoint.value = false;
  editingEndpointId.value = null;
  endpointForm.value = {
    api_type: 'openai_chat_completions',
    url: "",
  };
  isEndpointSheetOpen.value = true;
}

function openEditEndpointSheet(providerId: string, ep: ProviderEndpoint) {
  currentProviderId.value = providerId;
  isEditingEndpoint.value = true;
  editingEndpointId.value = ep.id;
  endpointForm.value = {
    api_type: ep.api_type,
    url: ep.url,
  };
  isEndpointSheetOpen.value = true;
}

async function handleEndpointSubmit() {
  if (!currentProviderId.value) return;

  if (isEditingEndpoint.value && editingEndpointId.value) {
    await store.patchEndpoint(currentProviderId.value, editingEndpointId.value, endpointForm.value);
  } else {
    await store.addEndpoint(currentProviderId.value, endpointForm.value);
  }

  if (!store.error) isEndpointSheetOpen.value = false;
}

async function handleDeleteEndpoint(providerId: string, id: string) {
  if (confirm("Delete this endpoint?")) {
    await store.removeEndpoint(providerId, id);
  }
}

async function toggleEndpointEnabled(providerId: string, ep: ProviderEndpoint) {
  await store.patchEndpoint(providerId, ep.id, { enabled: !ep.enabled });
}

// Key Handlers
function openCreateKeySheet(providerId: string) {
  currentProviderId.value = providerId;
  isEditingKey.value = false;
  editingKeyId.value = null;
  keyForm.value = {
    name: "",
    key: "",
  };
  isKeySheetOpen.value = true;
}

function openEditKeySheet(providerId: string, key: ProviderKey) {
  currentProviderId.value = providerId;
  isEditingKey.value = true;
  editingKeyId.value = key.id;
  keyForm.value = {
    name: key.name || "",
    key: "", // Keys are not sent back from server for editing usually
  };
  isKeySheetOpen.value = true;
}

async function handleKeySubmit() {
  if (!currentProviderId.value) return;

  if (isEditingKey.value && editingKeyId.value) {
    // Only send name for updates
    await store.patchKey(currentProviderId.value, editingKeyId.value, {
      name: keyForm.value.name || undefined,
    });
  } else {
    await store.addKey(currentProviderId.value, keyForm.value);
  }

  if (!store.error) isKeySheetOpen.value = false;
}

async function handleDeleteKey(providerId: string, id: string) {
  if (confirm("Delete this API key?")) {
    await store.removeKey(providerId, id);
  }
}

async function toggleKeyEnabled(providerId: string, key: ProviderKey) {
  await store.patchKey(providerId, key.id, { enabled: !key.enabled });
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString();
}

// Sub-resources loading logic
function onAccordionItemOpen(providerId: string) {
  if (!store.endpoints[providerId]) {
    store.fetchEndpoints(providerId);
  }
  if (!store.keys[providerId]) {
    store.fetchKeys(providerId);
  }
}

const copiedId = ref<string | null>(null);
async function copyToClipboard(text: string, id: string) {
  await navigator.clipboard.writeText(text);
  copiedId.value = id;
  setTimeout(() => { copiedId.value = null; }, 2000);
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Providers</h1>
        <p class="text-muted-foreground">
          Manage AI providers, their endpoints, and API keys.
        </p>
      </div>
      <Button @click="openCreateProviderSheet">
        <Plus class="mr-2 h-4 w-4" />
        Add Provider
      </Button>
    </div>

    <div v-if="store.error"
      class="rounded-md bg-destructive/15 p-4 text-destructive text-sm flex justify-between items-center">
      <span>{{ store.error }}</span>
      <Button variant="ghost" size="sm" @click="store.clearError">Dismiss</Button>
    </div>

    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-7.5"></TableHead>
            <TableHead>Name</TableHead>
            <TableHead>Description</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Created At</TableHead>
            <TableHead class="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="store.loading && store.providers.length === 0">
            <TableCell colspan="6" class="h-24 text-center">
              <Loader2 class="mx-auto h-6 w-6 animate-spin text-muted-foreground" />
            </TableCell>
          </TableRow>
          <TableRow v-else-if="store.providers.length === 0">
            <TableCell colspan="6" class="h-24 text-center text-muted-foreground">
              No providers found.
            </TableCell>
          </TableRow>

          <template v-for="provider in store.providers" :key="provider.id">
            <TableRow>
              <TableCell></TableCell>
              <TableCell class="font-medium">
                {{ provider.name }}
                <div class="text-xs text-muted-foreground font-mono">{{ provider.id }}</div>
              </TableCell>
              <TableCell>{{ provider.description || '-' }}</TableCell>
              <TableCell>
                <Badge :variant="provider.enabled ? 'default' : 'secondary'" class="cursor-pointer"
                  @click="toggleProviderEnabled(provider)">
                  {{ provider.enabled ? 'Active' : 'Disabled' }}
                </Badge>
              </TableCell>
              <TableCell class="text-sm">
                {{ formatDate(provider.created_at) }}
              </TableCell>
              <TableCell class="text-right">
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                      <MoreVertical class="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem @click="openEditProviderSheet(provider)">
                      <Edit class="mr-2 h-4 w-4" />
                      Edit
                    </DropdownMenuItem>
                    <DropdownMenuItem class="text-destructive" @click="handleDeleteProvider(provider.id)">
                      <Trash2 class="mr-2 h-4 w-4" />
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
            <TableRow>
              <TableCell colspan="6" class="p-0 border-t-0">
                <Accordion type="single" collapsible
                  @update:model-value="(val) => val && onAccordionItemOpen(provider.id)">
                  <AccordionItem :value="provider.id" class="border-0">
                    <AccordionTrigger class="px-8 py-2 text-xs text-muted-foreground hover:no-underline">
                      View Endpoints & Keys
                    </AccordionTrigger>
                    <AccordionContent class="px-8 pb-6">
                      <div class="grid grid-cols-1 lg:grid-cols-2 gap-10 pt-4">
                        <!-- Endpoints Section -->
                        <div class="space-y-4">
                          <div class="flex items-center justify-between">
                            <h3 class="text-sm font-semibold flex items-center gap-2">
                              <ExternalLink class="h-4 w-4 text-primary" />
                              Endpoints
                            </h3>
                            <Button size="sm" variant="outline" @click="openCreateEndpointSheet(provider.id)">
                              <Plus class="h-3 w-3 mr-1" /> Add
                            </Button>
                          </div>
                          <div class="rounded-md border bg-muted/30 overflow-hidden">
                            <Table class="text-xs">
                              <TableHeader>
                                <TableRow>
                                  <TableHead>Type</TableHead>
                                  <TableHead>URL</TableHead>
                                  <TableHead>Status</TableHead>
                                  <TableHead class="text-right">Actions</TableHead>
                                </TableRow>
                              </TableHeader>
                              <TableBody>
                                <TableRow v-for="ep in store.endpoints[provider.id]" :key="ep.id">
                                  <TableCell class="font-medium">{{ ep.api_type.split('_').pop() }}</TableCell>
                                  <TableCell class="max-w-30 truncate" :title="ep.url">{{ ep.url }}</TableCell>
                                  <TableCell>
                                    <div class="h-2 w-2 rounded-full cursor-pointer"
                                      :class="ep.enabled ? 'bg-green-500' : 'bg-gray-400'"
                                      @click="toggleEndpointEnabled(provider.id, ep)"></div>
                                  </TableCell>
                                  <TableCell class="text-right">
                                    <div class="flex justify-end gap-1">
                                      <Button variant="ghost" size="icon" class="h-6 w-6"
                                        @click="openEditEndpointSheet(provider.id, ep)">
                                        <Edit class="h-3 w-3" />
                                      </Button>
                                      <Button variant="ghost" size="icon" class="h-6 w-6 text-destructive"
                                        @click="handleDeleteEndpoint(provider.id, ep.id)">
                                        <Trash2 class="h-3 w-3" />
                                      </Button>
                                    </div>
                                  </TableCell>
                                </TableRow>
                                <TableRow v-if="!store.endpoints[provider.id]?.length">
                                  <TableCell colspan="4" class="text-center py-6 text-muted-foreground italic">
                                    No endpoints configured
                                  </TableCell>
                                </TableRow>
                              </TableBody>
                            </Table>
                          </div>
                        </div>

                        <!-- Keys Section -->
                        <div class="space-y-4">
                          <div class="flex items-center justify-between">
                            <h3 class="text-sm font-semibold flex items-center gap-2">
                              <ShieldCheck class="h-4 w-4 text-primary" />
                              API Keys
                            </h3>
                            <Button size="sm" variant="outline" @click="openCreateKeySheet(provider.id)">
                              <Plus class="h-3 w-3 mr-1" /> Add
                            </Button>
                          </div>
                          <div class="rounded-md border bg-muted/30 overflow-hidden">
                            <Table class="text-xs">
                              <TableHeader>
                                <TableRow>
                                  <TableHead>Name</TableHead>
                                  <TableHead>Key</TableHead>
                                  <TableHead>Status</TableHead>
                                  <TableHead class="text-right">Actions</TableHead>
                                </TableRow>
                              </TableHeader>
                              <TableBody>
                                <TableRow v-for="key in store.keys[provider.id]" :key="key.id">
                                  <TableCell>{{ key.name || 'Default' }}</TableCell>
                                  <TableCell>
                                    <div class="flex items-center gap-1">
                                      <span>••••{{ key.key.slice(-4) }}</span>
                                      <Button variant="ghost" size="icon" class="h-5 w-5"
                                        @click="copyToClipboard(key.key, key.id)">
                                        <Check v-if="copiedId === key.id" class="h-3 w-3 text-green-500" />
                                        <Copy v-else class="h-3 w-3" />
                                      </Button>
                                    </div>
                                  </TableCell>
                                  <TableCell>
                                    <div class="h-2 w-2 rounded-full cursor-pointer"
                                      :class="key.enabled ? 'bg-green-500' : 'bg-gray-400'"
                                      @click="toggleKeyEnabled(provider.id, key)"></div>
                                  </TableCell>
                                  <TableCell class="text-right">
                                    <div class="flex justify-end gap-1">
                                      <Button variant="ghost" size="icon" class="h-6 w-6"
                                        @click="openEditKeySheet(provider.id, key)">
                                        <Edit class="h-3 w-3" />
                                      </Button>
                                      <Button variant="ghost" size="icon" class="h-6 w-6 text-destructive"
                                        @click="handleDeleteKey(provider.id, key.id)">
                                        <Trash2 class="h-3 w-3" />
                                      </Button>
                                    </div>
                                  </TableCell>
                                </TableRow>
                                <TableRow v-if="!store.keys[provider.id]?.length">
                                  <TableCell colspan="4" class="text-center py-6 text-muted-foreground italic">
                                    No keys configured
                                  </TableCell>
                                </TableRow>
                              </TableBody>
                            </Table>
                          </div>
                        </div>
                      </div>
                    </AccordionContent>
                  </AccordionItem>
                </Accordion>
              </TableCell>
            </TableRow>
          </template>
        </TableBody>
      </Table>
    </div>

    <!-- Provider Sheet -->
    <Sheet :open="isProviderSheetOpen" @update:open="isProviderSheetOpen = $event">
      <SheetContent class="p-0">
        <div class="h-full flex flex-col py-6">
          <SheetHeader class="px-6 mb-6">
            <SheetTitle>{{ isEditingProvider ? 'Edit Provider' : 'Add Provider' }}</SheetTitle>
            <SheetDescription>
              {{ isEditingProvider ? "Update the details for this AI provider." :
                "Add a new vendor like OpenAI or Anthropic." }}
            </SheetDescription>
          </SheetHeader>
          <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
            <div class="grid gap-2">
              <Label for="provider-name">Name</Label>
              <Input id="provider-name" v-model="providerForm.name" placeholder="e.g. OpenAI" />
            </div>
            <div class="grid gap-2">
              <Label for="provider-description">Description (Optional)</Label>
              <Input id="provider-description" v-model="providerForm.description"
                placeholder="Main production account" />
            </div>
          </div>
          <SheetFooter class="px-6 mt-6 flex gap-2">
            <Button type="submit" :disabled="store.loading" @click="handleProviderSubmit">
              <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ isEditingProvider ? 'Save Changes' : 'Add Provider' }}
            </Button>
            <Button variant="outline" @click="isProviderSheetOpen = false">
              Cancel
            </Button>
          </SheetFooter>
        </div>
      </SheetContent>
    </Sheet>

    <!-- Endpoint Sheet -->
    <Sheet :open="isEndpointSheetOpen" @update:open="isEndpointSheetOpen = $event">
      <SheetContent class="p-0">
        <div class="h-full flex flex-col py-6">
          <SheetHeader class="px-6 mb-6">
            <SheetTitle>{{ isEditingEndpoint ? 'Edit Endpoint' : 'Add Endpoint' }}</SheetTitle>
            <SheetDescription>
              Configure the API endpoint for this provider.
            </SheetDescription>
          </SheetHeader>
          <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
            <div class="grid gap-2">
              <Label>API Type</Label>
              <Select v-model="endpointForm.api_type" :disabled="isEditingEndpoint">
                <SelectTrigger>
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="openai_chat_completions">OpenAI Chat Completions</SelectItem>
                  <SelectItem value="openai_responses">OpenAI Responses</SelectItem>
                  <SelectItem value="anthropic_messages">Anthropic Messages</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="grid gap-2">
              <Label for="ep-url">URL</Label>
              <Input id="ep-url" v-model="endpointForm.url" placeholder="https://api.openai.com/v1/chat/completions" />
            </div>
          </div>
          <SheetFooter class="px-6 mt-6 flex gap-2">
            <Button type="submit" :disabled="store.loading" @click="handleEndpointSubmit">
              <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ isEditingEndpoint ? 'Save Changes' : 'Add Endpoint' }}
            </Button>
            <Button variant="outline" @click="isEndpointSheetOpen = false">
              Cancel
            </Button>
          </SheetFooter>
        </div>
      </SheetContent>
    </Sheet>

    <!-- Key Sheet -->
    <Sheet :open="isKeySheetOpen" @update:open="isKeySheetOpen = $event">
      <SheetContent class="p-0">
        <div class="h-full flex flex-col py-6">
          <SheetHeader class="px-6 mb-6">
            <SheetTitle>{{ isEditingKey ? 'Edit API Key' : 'Add API Key' }}</SheetTitle>
            <SheetDescription>
              Set up a credential to be used with this provider's endpoints.
            </SheetDescription>
          </SheetHeader>
          <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
            <div class="grid gap-2">
              <Label for="key-name">Name (Optional)</Label>
              <Input id="key-name" v-model="keyForm.name" placeholder="Primary Key" />
            </div>
            <div class="grid gap-2" v-if="!isEditingKey">
              <Label for="key-value">API Key</Label>
              <Input id="key-value" v-model="keyForm.key" type="password" />
            </div>
          </div>
          <SheetFooter class="px-6 mt-6 flex gap-2">
            <Button type="submit" :disabled="store.loading" @click="handleKeySubmit">
              <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ isEditingKey ? 'Save Changes' : 'Add Key' }}
            </Button>
            <Button variant="outline" @click="isKeySheetOpen = false">
              Cancel
            </Button>
          </SheetFooter>
        </div>
      </SheetContent>
    </Sheet>
  </div>
</template>