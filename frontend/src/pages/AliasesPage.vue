<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useAliasesStore } from "@/stores/aliases";
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
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import type { ApiType } from "@/api/providers";
import type { Alias, AliasTargetDetail } from "@/api/aliases";

const store = useAliasesStore();

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

const targetForm = ref({
  provider_id: "",
  model_id: "",
});

onMounted(() => {
  store.fetchAliases();
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

async function handleDeleteAlias(id: string) {
  if (confirm("Delete this alias?")) {
    await store.removeAlias(id);
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
  isTargetSheetOpen.value = true;
}

async function handleTargetSubmit() {
  if (!currentAliasId.value) return;

  if (isEditingTarget.value && editingTargetId.value) {
    // Only enabled is usually updatable for targets now, but let's keep basic structure
    await store.patchTarget(currentAliasId.value, editingTargetId.value, {});
  } else {
    await store.addTarget(currentAliasId.value, {
        provider_id: targetForm.value.provider_id,
        model_id: targetForm.value.model_id,
    });
  }

  if (!store.error) isTargetSheetOpen.value = false;
}

async function handleDeleteTarget(aliasId: string, id: string) {
  if (confirm("Delete this target?")) {
    await store.removeTarget(aliasId, id);
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
  <div class="space-y-6">
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

    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead class="w-[30px]"></TableHead>
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
                      <div class="space-y-4">
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
                                <TableHead>Endpoint URL</TableHead>
                                <TableHead>API Type</TableHead>
                                <TableHead>Status</TableHead>
                                <TableHead class="text-right">Actions</TableHead>
                              </TableRow>
                            </TableHeader>
                            <TableBody>
                              <TableRow v-for="target in store.targets[alias.id]" :key="target.alias_target_id">
                                <TableCell>{{ target.provider_name }}</TableCell>
                                <TableCell>{{ target.model_name }}</TableCell>
                                <TableCell class="max-w-[200px] truncate" :title="target.endpoint_url || 'N/A'">
                                  {{ target.endpoint_url || 'N/A' }}
                                </TableCell>
                                <TableCell>{{ target.api_type?.split('_').pop() || '-' }}</TableCell>
                                <TableCell>
                                  <div class="h-2 w-2 rounded-full cursor-pointer"
                                    :class="target.enabled ? 'bg-green-500' : 'bg-gray-400'"
                                    @click="toggleTargetEnabled(alias.id, target)"></div>
                                </TableCell>
                                <TableCell class="text-right">
                                  <div class="flex justify-end gap-1">
                                    <Button variant="ghost" size="icon" class="h-6 w-6"
                                      @click="openEditTargetSheet(alias.id, target)">
                                      <Edit class="h-3 w-3" />
                                    </Button>
                                    <Button variant="ghost" size="icon" class="h-6 w-6 text-destructive"
                                      @click="handleDeleteTarget(alias.id, target.alias_target_id)">
                                      <Trash2 class="h-3 w-3" />
                                    </Button>
                                  </div>
                                </TableCell>
                              </TableRow>
                              <TableRow v-if="!store.targets[alias.id]?.length">
                                <TableCell colspan="6" class="text-center py-6 text-muted-foreground italic">
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
              <Input id="alias-name" v-model="aliasForm.name" placeholder="e.g. gpt-4-turbo" :disabled="isEditingAlias" />
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
            <div class="grid gap-2" v-if="!isEditingTarget">
              <Label for="target-provider-id">Provider ID</Label>
              <Input id="target-provider-id" v-model="targetForm.provider_id" placeholder="UUID of Provider" />
            </div>
            <div class="grid gap-2" v-if="!isEditingTarget">
              <Label for="target-model-id">Model ID</Label>
              <Input id="target-model-id" v-model="targetForm.model_id" placeholder="UUID of Model" />
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
  </div>
</template>
