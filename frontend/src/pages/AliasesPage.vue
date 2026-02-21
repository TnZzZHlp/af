<script setup lang="ts">
import { onMounted, ref, watch, computed } from "vue";
import { useAliasesStore } from "@/stores/aliases";
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
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import Combobox from "@/components/Combobox.vue";
import type { Alias, AliasTargetDetail } from "@/api/aliases";
import { listProviderModels, type Model } from "@/api/providers";

const store = useAliasesStore();
const providersStore = useProvidersStore();

const isAliasSheetOpen = ref(false);
const isEditingAlias = ref(false);
const editingAliasId = ref<string | null>(null);

const aliasForm = ref({
  name: "",
});

// Target State
const isTargetSheetOpen = ref(false);
const isEditingTarget = ref(false);
const editingTargetId = ref<string | null>(null);
const currentAliasId = ref<string | null>(null);
const availableModels = ref<Model[]>([]);
const loadingModels = ref(false);
const modelsFetchFailed = ref(false);

const targetForm = ref({
  provider_id: "",
  model_id: "",
});

const modelOptions = computed(() =>
  availableModels.value.map((m) => ({ value: m.id, label: m.id }))
);

watch(() => targetForm.value.provider_id, async (newProviderId) => {
  if (newProviderId) {
    loadingModels.value = true;
    modelsFetchFailed.value = false;
    try {
      availableModels.value = await listProviderModels(newProviderId);
    } catch (error) {
      console.error("Failed to fetch models:", error);
      availableModels.value = [];
      modelsFetchFailed.value = true;
    } finally {
      loadingModels.value = false;
    }
  } else {
    availableModels.value = [];
    modelsFetchFailed.value = false;
  }
});

// Delete Confirmation State
const isDeleteAliasDialogOpen = ref(false);
const aliasToDeleteId = ref<string | null>(null);

const isDeleteTargetDialogOpen = ref(false);
const targetToDelete = ref<{ aliasId: string; targetId: string } | null>(null);

onMounted(() => {
  store.fetchAliases();
  providersStore.fetchProviders();
});

function openCreateAliasSheet() {
  isEditingAlias.value = false;
  editingAliasId.value = null;
  aliasForm.value = {
    name: "",
  };
  isAliasSheetOpen.value = true;
}

function openEditAliasSheet(alias: Alias) {
  isEditingAlias.value = true;
  editingAliasId.value = alias.id;
  aliasForm.value = {
    name: alias.name,
  };
  isAliasSheetOpen.value = true;
}

async function handleAliasSubmit() {
  const payload = {
    name: aliasForm.value.name,
  };

  if (isEditingAlias.value && editingAliasId.value) {
    await store.patchAlias(editingAliasId.value, payload);
  } else {
    await store.addAlias(payload);
  }

  if (!store.error) {
    isAliasSheetOpen.value = false;
  }
}

function handleDeleteAlias(id: string) {
  aliasToDeleteId.value = id;
  isDeleteAliasDialogOpen.value = true;
}

async function confirmDeleteAlias() {
  if (aliasToDeleteId.value) {
    await store.removeAlias(aliasToDeleteId.value);
    isDeleteAliasDialogOpen.value = false;
    aliasToDeleteId.value = null;
  }
}

async function toggleAliasEnabled(alias: Alias) {
  await store.patchAlias(alias.id, { enabled: !alias.enabled });
}

// Target Handlers
function openCreateTargetSheet(aliasId: string) {
  currentAliasId.value = aliasId;
  isEditingTarget.value = false;
  editingTargetId.value = null;
  availableModels.value = [];
  modelsFetchFailed.value = false;
  targetForm.value = {
    provider_id: "",
    model_id: "",
  };
  isTargetSheetOpen.value = true;
}

function openEditTargetSheet(aliasId: string, target: AliasTargetDetail) {
  currentAliasId.value = aliasId;
  isEditingTarget.value = true;
  editingTargetId.value = target.alias_target_id;
  targetForm.value = {
    provider_id: target.provider_id,
    model_id: target.model_id,
  };
  // Trigger fetch models immediately since we set provider_id
  // The watch will handle it, but since we are synchronous here, it might trigger after.
  // Actually, watch is triggered on next tick usually or if reactive value changes.
  isTargetSheetOpen.value = true;
}

async function handleTargetSubmit() {
  if (!currentAliasId.value) return;

  if (isEditingTarget.value && editingTargetId.value) {
    await store.patchTarget(currentAliasId.value, editingTargetId.value, {
      provider_id: targetForm.value.provider_id,
      model_id: targetForm.value.model_id,
    });
  } else {
    await store.addTarget(currentAliasId.value, {
      provider_id: targetForm.value.provider_id,
      model_id: targetForm.value.model_id,
    });
  }

  if (!store.error) isTargetSheetOpen.value = false;
}

async function handleDeleteTarget(aliasId: string, id: string) {
  targetToDelete.value = { aliasId, targetId: id };
  isDeleteTargetDialogOpen.value = true;
}

async function confirmDeleteTarget() {
  if (targetToDelete.value) {
    await store.removeTarget(targetToDelete.value.aliasId, targetToDelete.value.targetId);
    isDeleteTargetDialogOpen.value = false;
    targetToDelete.value = null;
  }
}

async function toggleTargetEnabled(aliasId: string, target: AliasTargetDetail) {
  await store.patchTarget(aliasId, target.alias_target_id, { enabled: !target.enabled });
}

function onAccordionItemOpen(aliasId: string) {
  if (!store.targets[aliasId]) {
    store.fetchTargets(aliasId);
  }
}

function formatDate(dateStr: string) {
  return new Date(dateStr).toLocaleString();
}
</script>

<template>
  <div class="space-y-6 h-full flex flex-col min-h-0">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Aliases</h1>
        <p class="text-muted-foreground">
          Manage routing aliases and their targets.
        </p>
      </div>
      <Button @click="openCreateAliasSheet">
        <Plus class="mr-2 h-4 w-4" />
        Add Alias
      </Button>
    </div>

    <div v-if="store.error"
      class="rounded-md bg-destructive/15 p-4 text-destructive text-sm flex justify-between items-center">
      <span>{{ store.error }}</span>
      <Button variant="ghost" size="sm" @click="store.clearError">Dismiss</Button>
    </div>

    <div class="rounded-md border flex-1 min-h-0 flex flex-col">
      <Table class="flex-1 min-h-0">
        <TableHeader class="sticky top-0 bg-background z-10 shadow-sm">
          <TableRow>
            <TableHead class="w-7.5"></TableHead>
            <TableHead>Name</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Created At</TableHead>
            <TableHead class="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="store.loading && store.aliases.length === 0">
            <TableCell colspan="5" class="h-24 text-center">
              <Loader2 class="mx-auto h-6 w-6 animate-spin text-muted-foreground" />
            </TableCell>
          </TableRow>
          <TableRow v-else-if="store.aliases.length === 0">
            <TableCell colspan="5" class="h-24 text-center text-muted-foreground">
              No aliases found.
            </TableCell>
          </TableRow>

          <template v-for="alias in store.aliases" :key="alias.id">
            <TableRow>
              <TableCell></TableCell>
              <TableCell class="font-medium">
                {{ alias.name }}
                <div class="text-xs text-muted-foreground font-mono">{{ alias.id }}</div>
              </TableCell>
              <TableCell>
                <Badge :variant="alias.enabled ? 'default' : 'secondary'" class="cursor-pointer"
                  @click="toggleAliasEnabled(alias)">
                  {{ alias.enabled ? 'Active' : 'Disabled' }}
                </Badge>
              </TableCell>
              <TableCell class="text-sm">
                {{ formatDate(alias.created_at) }}
              </TableCell>
              <TableCell class="text-right">
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                      <MoreVertical class="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem @click="openEditAliasSheet(alias)">
                      <Edit class="mr-2 h-4 w-4" />
                      Edit
                    </DropdownMenuItem>
                    <DropdownMenuItem class="text-destructive" @click="handleDeleteAlias(alias.id)">
                      <Trash2 class="mr-2 h-4 w-4" />
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
            <TableRow>
              <TableCell colspan="5" class="p-0 border-t-0">
                <Accordion type="single" collapsible
                  @update:model-value="(val) => val && onAccordionItemOpen(alias.id)">
                  <AccordionItem :value="alias.id" class="border-0">
                    <AccordionTrigger class="px-8 py-2 text-xs text-muted-foreground hover:no-underline">
                      View Targets
                    </AccordionTrigger>
                    <AccordionContent class="px-8 pb-6">
                      <div v-if="store.loading && !store.targets[alias.id]"
                        class="flex items-center justify-center py-10">
                        <Loader2 class="h-5 w-5 animate-spin text-muted-foreground" />
                      </div>
                      <div v-else class="space-y-4">
                        <div class="flex items-center justify-between">
                          <h3 class="text-sm font-semibold flex items-center gap-2">
                            <ExternalLink class="h-4 w-4 text-primary" />
                            Targets
                          </h3>
                          <Button size="sm" variant="outline" @click="openCreateTargetSheet(alias.id)">
                            <Plus class="h-3 w-3 mr-1" /> Add
                          </Button>
                        </div>
                        <div class="rounded-md border bg-muted/30 overflow-hidden">
                          <Table class="text-xs">
                            <TableHeader>
                              <TableRow>
                                <TableHead>Provider</TableHead>
                                <TableHead>Model</TableHead>
                                <TableHead>Usage</TableHead>
                                <TableHead>Status</TableHead>
                                <TableHead class="text-right">Actions</TableHead>
                              </TableRow>
                            </TableHeader>
                            <TableBody>
                              <TableRow v-for="target in store.targets[alias.id]" :key="target.alias_target_id">
                                <TableCell>{{ target.provider_name }}</TableCell>
                                <TableCell>{{ target.model_id }}</TableCell>
                                <TableCell>{{ target.usage_count }}</TableCell>
                                <TableCell>
                                  <div class="h-2 w-2 rounded-full cursor-pointer"
                                    :class="target.enabled ? 'bg-green-500' : 'bg-gray-400'"
                                    @click="toggleTargetEnabled(alias.id, target)">
                                  </div>
                                </TableCell>
                                <TableCell class="text-right">
                                  <div class="flex justify-end gap-1">
                                    <Button variant="ghost" size="icon" class="h-6 w-6"
                                      @click="openEditTargetSheet(alias.id, target)">
                                      <Edit class="h-3 w-3" />
                                    </Button>
                                    <Button variant="ghost" size="icon"
                                      class="h-6 w-6 text-destructive hover:text-destructive/70 cursor-pointer"
                                      @click="handleDeleteTarget(alias.id, target.alias_target_id)">
                                      <Trash2 class="h-3 w-3" />
                                    </Button>
                                  </div>
                                </TableCell>
                              </TableRow>
                              <TableRow v-if="!store.targets[alias.id]?.length">
                                <TableCell colspan="4" class="text-center py-6 text-muted-foreground italic">
                                  No targets configured
                                </TableCell>
                              </TableRow>
                            </TableBody>
                          </Table>
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

    <!-- Alias Sheet -->
    <Sheet :open="isAliasSheetOpen" @update:open="isAliasSheetOpen = $event">
      <SheetContent class="p-0">
        <div class="h-full flex flex-col py-6">
          <SheetHeader class="px-6 mb-6">
            <SheetTitle>{{ isEditingAlias ? 'Edit Alias' : 'Add Alias' }}</SheetTitle>
            <SheetDescription>
              Create a new alias to route requests to multiple provider models.
            </SheetDescription>
          </SheetHeader>
          <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
            <div class="grid gap-2">
              <Label for="alias-name">Name</Label>
              <Input id="alias-name" v-model="aliasForm.name" placeholder="e.g. gpt-4-turbo"
                :disabled="isEditingAlias" />
              <p v-if="isEditingAlias" class="text-xs text-muted-foreground">Name cannot be changed.</p>
            </div>
          </div>
          <SheetFooter class="px-6 mt-6 flex gap-2">
            <Button type="submit" :disabled="store.loading" @click="handleAliasSubmit">
              <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ isEditingAlias ? 'Save Changes' : 'Add Alias' }}
            </Button>
            <Button variant="outline" @click="isAliasSheetOpen = false">
              Cancel
            </Button>
          </SheetFooter>
        </div>
      </SheetContent>
    </Sheet>

    <!-- Target Sheet -->
    <Sheet :open="isTargetSheetOpen" @update:open="isTargetSheetOpen = $event">
      <SheetContent class="p-0">
        <div class="h-full flex flex-col py-6">
          <SheetHeader class="px-6 mb-6">
            <SheetTitle>{{ isEditingTarget ? 'Edit Target' : 'Add Target' }}</SheetTitle>
            <SheetDescription>
              Route traffic to a specific provider and model.
            </SheetDescription>
          </SheetHeader>
          <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
            <div class="grid gap-2">
              <Label for="target-provider-id">Provider</Label>
              <Select v-model="targetForm.provider_id">
                <SelectTrigger id="target-provider-id">
                  <SelectValue placeholder="Select a provider" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem v-for="provider in providersStore.providers" :key="provider.id" :value="provider.id">
                    {{ provider.name }}
                  </SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div class="grid gap-2">
              <Label for="target-model-id">Model</Label>
              <template v-if="availableModels.length > 0 || loadingModels">
                <Combobox
                  v-model="targetForm.model_id"
                  :options="modelOptions"
                  :loading="loadingModels"
                  placeholder="Select a model..."
                  search-placeholder="Search model..."
                  empty-text="No model found."
                />
              </template>
              <Input id="target-model-id" v-model="targetForm.model_id" placeholder="e.g. gpt-4o"
                :disabled="!targetForm.provider_id" />
              <p v-if="modelsFetchFailed && targetForm.provider_id" class="text-xs text-amber-600">
                Could not fetch models from provider. Please enter model ID manually.
              </p>
              <p v-else-if="targetForm.provider_id && availableModels.length > 0" class="text-xs text-muted-foreground">
                Select from the list above or enter a model ID manually.
              </p>
              <p v-else-if="targetForm.provider_id && !loadingModels" class="text-xs text-muted-foreground">
                Enter the model identifier manually.
              </p>
              <p v-else class="text-xs text-muted-foreground">Select a provider first.</p>
            </div>
          </div>
          <SheetFooter class="px-6 mt-6 flex gap-2">
            <Button type="submit" :disabled="store.loading" @click="handleTargetSubmit">
              <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ isEditingTarget ? 'Save Changes' : 'Add Target' }}
            </Button>
            <Button variant="outline" @click="isTargetSheetOpen = false">
              Cancel
            </Button>
          </SheetFooter>
        </div>
      </SheetContent>
    </Sheet>

    <!-- Delete Alias Alert Dialog -->
    <AlertDialog :open="isDeleteAliasDialogOpen" @update:open="isDeleteAliasDialogOpen = $event">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This action cannot be undone. This will permanently delete the alias and remove all associated
            targets.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="aliasToDeleteId = null" class="cursor-pointer">Cancel</AlertDialogCancel>
          <AlertDialogAction class="bg-destructive  hover:bg-destructive/70 cursor-pointer" @click="confirmDeleteAlias">
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <!-- Delete Target Alert Dialog -->
    <AlertDialog :open="isDeleteTargetDialogOpen" @update:open="isDeleteTargetDialogOpen = $event">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete this target?</AlertDialogTitle>
          <AlertDialogDescription>
            This will remove this routing target from the alias. Traffic will no longer be routed to this
            provider/model.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="targetToDelete = null" class="cursor-pointer">Cancel</AlertDialogCancel>
          <AlertDialogAction class="bg-destructive  hover:bg-destructive/70 cursor-pointer"
            @click="confirmDeleteTarget">
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
