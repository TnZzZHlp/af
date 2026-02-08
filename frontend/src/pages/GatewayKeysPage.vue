<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useGatewayKeysStore } from "@/stores/gateway-keys";
import type { GatewayKey } from "@/api/gateway-keys";
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
    Accordion,
    AccordionContent,
    AccordionItem,
    AccordionTrigger,
} from "@/components/ui/accordion";
import {
    Plus,
    Trash2,
    Edit,
    Copy,
    Check,
    Eye,
    EyeOff,
    Loader2,
    MoreVertical,
    ShieldCheck,
    Activity,
} from "lucide-vue-next";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
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

const store = useGatewayKeysStore();

const isSheetOpen = ref(false);
const isEditing = ref(false);
const editingId = ref<string | null>(null);

const form = ref({
    name: "",
    rate_limit_rps: "" as string | number,
    rate_limit_rpm: "" as string | number,
});

const visibleKeys = ref<Record<string, boolean>>({});
const copiedId = ref<string | null>(null);

const isDeleteAlertOpen = ref(false);
const deleteKeyId = ref<string | null>(null);

onMounted(() => {
    store.fetchKeys();
});

function openCreateSheet() {
    isEditing.value = false;
    editingId.value = null;
    form.value = {
        name: "",
        rate_limit_rps: "",
        rate_limit_rpm: "",
    };
    isSheetOpen.value = true;
}

function openEditSheet(key: GatewayKey) {
    isEditing.value = true;
    editingId.value = key.id;
    form.value = {
        name: key.name || "",
        rate_limit_rps: key.rate_limit_rps || "",
        rate_limit_rpm: key.rate_limit_rpm || "",
    };
    isSheetOpen.value = true;
}

async function handleSubmit() {
    const payload = {
        name: form.value.name || null,
        rate_limit_rps: form.value.rate_limit_rps ? Number(form.value.rate_limit_rps) : null,
        rate_limit_rpm: form.value.rate_limit_rpm ? Number(form.value.rate_limit_rpm) : null,
    };

    if (isEditing.value && editingId.value) {
        await store.updateKey(editingId.value, payload);
    } else {
        await store.createKey(payload);
    }

    if (!store.error) {
        isSheetOpen.value = false;
    }
}

async function handleDelete(id: string) {
    deleteKeyId.value = id;
    isDeleteAlertOpen.value = true;
}

async function confirmDelete() {
    if (deleteKeyId.value) {
        await store.deleteKey(deleteKeyId.value);
        isDeleteAlertOpen.value = false;
        deleteKeyId.value = null;
    }
}

async function toggleEnabled(key: GatewayKey) {
    await store.updateKey(key.id, { name: key.name, enabled: !key.enabled, rate_limit_rps: key.rate_limit_rps, rate_limit_rpm: key.rate_limit_rpm });
}

function toggleKeyVisibility(id: string) {
    visibleKeys.value[id] = !visibleKeys.value[id];
}

async function copyToClipboard(text: string, id: string) {
    await navigator.clipboard.writeText(text);
    copiedId.value = id;
    setTimeout(() => {
        copiedId.value = null;
    }, 2000);
}

function formatDate(dateStr: string) {
    return new Date(dateStr).toLocaleString();
}
</script>

<template>
    <div class="space-y-6 h-full flex flex-col min-h-0">
        <div class="flex items-center justify-between">
            <div>
                <h1 class="text-3xl font-bold tracking-tight">Gateway API Keys</h1>
                <p class="text-muted-foreground">
                    Manage API keys for accessing the AI Gateway.
                </p>
            </div>
            <Button @click="openCreateSheet">
                <Plus class="mr-2 h-4 w-4" />
                Create New Key
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
                    <TableRow v-if="store.loading && store.keys.length === 0">
                        <TableCell colspan="5" class="h-24 text-center">
                            <Loader2 class="mx-auto h-6 w-6 animate-spin text-muted-foreground" />
                        </TableCell>
                    </TableRow>
                    <TableRow v-else-if="store.keys.length === 0">
                        <TableCell colspan="5" class="h-24 text-center text-muted-foreground">
                            No API keys found.
                        </TableCell>
                    </TableRow>

                    <template v-for="key in store.keys" :key="key.id">
                        <TableRow>
                            <TableCell></TableCell>
                            <TableCell class="font-medium">
                                {{ key.name || 'Unnamed Key' }}
                                <div class="text-xs text-muted-foreground font-mono">{{ key.id }}</div>
                            </TableCell>
                            <TableCell>
                                <Badge :variant="key.enabled ? 'default' : 'secondary'" class="cursor-pointer"
                                    @click="toggleEnabled(key)">
                                    {{ key.enabled ? 'Active' : 'Disabled' }}
                                </Badge>
                            </TableCell>
                            <TableCell class="text-sm">
                                {{ formatDate(key.created_at) }}
                            </TableCell>
                            <TableCell class="text-right">
                                <DropdownMenu>
                                    <DropdownMenuTrigger as-child>
                                        <Button variant="ghost" size="icon">
                                            <MoreVertical class="h-4 w-4" />
                                        </Button>
                                    </DropdownMenuTrigger>
                                    <DropdownMenuContent align="end">
                                        <DropdownMenuItem @click="openEditSheet(key)">
                                            <Edit class="mr-2 h-4 w-4" />
                                            Edit
                                        </DropdownMenuItem>
                                        <DropdownMenuItem variant="destructive" @click="handleDelete(key.id)">
                                            <Trash2 class="mr-2 h-4 w-4" />
                                            Delete
                                        </DropdownMenuItem>
                                    </DropdownMenuContent>
                                </DropdownMenu>
                            </TableCell>
                        </TableRow>
                        <TableRow>
                            <TableCell colspan="5" class="p-0 border-t-0">
                                <Accordion type="single" collapsible>
                                    <AccordionItem :value="key.id" class="border-0">
                                        <AccordionTrigger
                                            class="px-8 py-2 text-xs text-muted-foreground hover:no-underline">
                                            View Key & Details
                                        </AccordionTrigger>
                                        <AccordionContent class="px-8 pb-6">
                                            <div class="grid grid-cols-1 lg:grid-cols-2 gap-10 pt-4">

                                                <!-- Key Information -->
                                                <div class="space-y-4">
                                                    <h3 class="text-sm font-semibold flex items-center gap-2">
                                                        <ShieldCheck class="h-4 w-4 text-primary" />
                                                        Key Details
                                                    </h3>
                                                    <div class="rounded-md border bg-muted/30 p-4">
                                                        <div class="flex items-center gap-2">
                                                            <code
                                                                class="relative rounded bg-muted px-[0.3rem] py-[0.2rem] font-mono text-sm font-semibold flex-1">
                                {{ visibleKeys[key.id] ? key.key : '••••••••••••••••' }}
                              </code>
                                                            <Button variant="ghost" size="icon" class="h-8 w-8"
                                                                @click="toggleKeyVisibility(key.id)">
                                                                <Eye v-if="!visibleKeys[key.id]" class="h-4 w-4" />
                                                                <EyeOff v-else class="h-4 w-4" />
                                                            </Button>
                                                            <Button variant="ghost" size="icon" class="h-8 w-8"
                                                                @click="copyToClipboard(key.key, key.id)">
                                                                <Check v-if="copiedId === key.id"
                                                                    class="h-4 w-4 text-green-500" />
                                                                <Copy v-else class="h-4 w-4" />
                                                            </Button>
                                                        </div>
                                                        <p class="text-xs text-muted-foreground mt-2">
                                                            Use this key to authenticate requests to the AI Gateway.
                                                        </p>
                                                    </div>
                                                </div>

                                                <!-- Configuration -->
                                                <div class="space-y-4">
                                                    <h3 class="text-sm font-semibold flex items-center gap-2">
                                                        <Activity class="h-4 w-4 text-primary" />
                                                        Configuration
                                                    </h3>
                                                    <div class="rounded-md border bg-muted/30 p-4">
                                                        <div class="grid grid-cols-2 gap-4 text-sm">
                                                            <div>
                                                                <span class="text-muted-foreground">Requests /
                                                                    Second:</span>
                                                                <div class="font-medium mt-1">{{ key.rate_limit_rps ||
                                                                    'Unlimited' }}</div>
                                                            </div>
                                                            <div>
                                                                <span class="text-muted-foreground">Requests /
                                                                    Minute:</span>
                                                                <div class="font-medium mt-1">{{ key.rate_limit_rpm ||
                                                                    'Unlimited' }}</div>
                                                            </div>
                                                        </div>
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

        <Sheet :open="isSheetOpen" @update:open="isSheetOpen = $event">
            <SheetContent class="p-0">
                <div class="h-full flex flex-col py-6">
                    <SheetHeader class="px-6 mb-6">
                        <SheetTitle>{{ isEditing ? 'Edit API Key' : 'Create API Key' }}</SheetTitle>
                        <SheetDescription>
                            {{ isEditing ? 'Update the details for this API key.' : 'Create a new API key to access the
                            AI
                            Gateway.'
                            }}
                        </SheetDescription>
                    </SheetHeader>
                    <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
                        <div class="grid gap-2">
                            <Label for="name">Name (Optional)</Label>
                            <Input id="name" v-model="form.name" placeholder="My Application" />
                        </div>
                        <div class="grid gap-2">
                            <Label for="rps">Rate Limit (Requests Per Second)</Label>
                            <Input id="rps" v-model="form.rate_limit_rps" type="number" placeholder="e.g. 10" />
                        </div>
                        <div class="grid gap-2">
                            <Label for="rpm">Rate Limit (Requests Per Minute)</Label>
                            <Input id="rpm" v-model="form.rate_limit_rpm" type="number" placeholder="e.g. 600" />
                        </div>
                    </div>
                    <SheetFooter class="px-6 mt-6 flex gap-2">
                        <Button type="submit" :disabled="store.loading" @click="handleSubmit">
                            <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
                            {{ isEditing ? 'Save Changes' : 'Create Key' }}
                        </Button>
                        <Button variant="outline" @click="isSheetOpen = false">
                            Cancel
                        </Button>
                    </SheetFooter>
                </div>
            </SheetContent>
        </Sheet>

        <AlertDialog :open="isDeleteAlertOpen" @update:open="isDeleteAlertOpen = $event">
            <AlertDialogContent>
                <AlertDialogHeader>
                    <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
                    <AlertDialogDescription>
                        This action cannot be undone. This will permanently delete the API key and revoke access for any
                        applications
                        using it.
                    </AlertDialogDescription>
                </AlertDialogHeader>
                <AlertDialogFooter>
                    <AlertDialogCancel @click="deleteKeyId = null" class="cursor-pointer">Cancel</AlertDialogCancel>
                    <AlertDialogAction class="bg-destructive hover:bg-destructive/70 cursor-pointer"
                        @click="confirmDelete">
                        Delete
                    </AlertDialogAction>
                </AlertDialogFooter>
            </AlertDialogContent>
        </AlertDialog>
    </div>
</template>
