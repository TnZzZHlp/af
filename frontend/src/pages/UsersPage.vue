<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useUsersStore } from "@/stores/users";
import type { User } from "@/api/users";
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
  KeyRound,
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

const store = useUsersStore();

const isSheetOpen = ref(false);
const isEditing = ref(false);
const editingId = ref<string | null>(null);

const form = ref({
  username: "",
  password: "",
  confirmPassword: "",
});

const isPasswordDialogOpen = ref(false);
const passwordForm = ref({
  password: "",
  confirmPassword: "",
});
const passwordUpdateId = ref<string | null>(null);

const isDeleteAlertOpen = ref(false);
const deleteUserId = ref<string | null>(null);

onMounted(() => {
  store.fetchUsers();
});

function openCreateSheet() {
  isEditing.value = false;
  editingId.value = null;
  form.value = {
    username: "",
    password: "",
    confirmPassword: "",
  };
  isSheetOpen.value = true;
}

function openEditSheet(user: User) {
  isEditing.value = true;
  editingId.value = user.id;
  form.value = {
    username: user.username,
    password: "",
    confirmPassword: "",
  };
  isSheetOpen.value = true;
}

function openPasswordDialog(user: User) {
  passwordUpdateId.value = user.id;
  passwordForm.value = {
    password: "",
    confirmPassword: "",
  };
  isPasswordDialogOpen.value = true;
}

async function handleSubmit() {
  if (!isEditing.value && form.value.password !== form.value.confirmPassword) {
    store.error = "Passwords do not match";
    return;
  }

  if (isEditing.value && editingId.value) {
    await store.updateUser(editingId.value, {
      username: form.value.username,
    });
  } else {
    await store.createUser({
      username: form.value.username,
      password: form.value.password,
    });
  }

  if (!store.error) {
    isSheetOpen.value = false;
  }
}

async function handlePasswordSubmit() {
  if (passwordForm.value.password !== passwordForm.value.confirmPassword) {
    store.error = "Passwords do not match";
    return;
  }

  if (passwordUpdateId.value) {
    await store.updatePassword(passwordUpdateId.value, {
      password: passwordForm.value.password,
    });
  }

  if (!store.error) {
    isPasswordDialogOpen.value = false;
  }
}

async function handleDelete(id: string) {
  deleteUserId.value = id;
  isDeleteAlertOpen.value = true;
}

async function confirmDelete() {
  if (deleteUserId.value) {
    await store.deleteUser(deleteUserId.value);
    isDeleteAlertOpen.value = false;
    deleteUserId.value = null;
  }
}

async function toggleEnabled(user: User) {
  await store.updateUser(user.id, { enabled: !user.enabled });
}

function formatDate(dateStr: string | null) {
  if (!dateStr) return "Never";
  return new Date(dateStr).toLocaleString();
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div>
        <h1 class="text-3xl font-bold tracking-tight">Users</h1>
        <p class="text-muted-foreground">
          Manage system users and their access.
        </p>
      </div>
      <Button @click="openCreateSheet">
        <Plus class="mr-2 h-4 w-4" />
        Create User
      </Button>
    </div>

    <div v-if="store.error"
      class="rounded-md bg-destructive/15 p-4 text-destructive text-sm flex justify-between items-center">
      <span>{{ store.error }}</span>
      <Button variant="ghost" size="sm" @click="store.error = null">Dismiss</Button>
    </div>

    <div class="rounded-md border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Username</TableHead>
            <TableHead>Status</TableHead>
            <TableHead>Created At</TableHead>
            <TableHead>Password Updated</TableHead>
            <TableHead class="text-right">Actions</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          <TableRow v-if="store.loading && store.users.length === 0">
            <TableCell colspan="5" class="h-24 text-center">
              <Loader2 class="mx-auto h-6 w-6 animate-spin text-muted-foreground" />
            </TableCell>
          </TableRow>
          <TableRow v-else-if="store.users.length === 0">
            <TableCell colspan="5" class="h-24 text-center text-muted-foreground">
              No users found.
            </TableCell>
          </TableRow>

          <template v-for="user in store.users" :key="user.id">
            <TableRow>
              <TableCell class="font-medium">
                {{ user.username }}
              </TableCell>
              <TableCell>
                <Badge :variant="user.enabled ? 'default' : 'secondary'" class="cursor-pointer"
                  @click="toggleEnabled(user)">
                  {{ user.enabled ? 'Active' : 'Disabled' }}
                </Badge>
              </TableCell>
              <TableCell class="text-sm">
                {{ formatDate(user.created_at) }}
              </TableCell>
              <TableCell class="text-sm">
                {{ formatDate(user.password_updated_at) }}
              </TableCell>
              <TableCell class="text-right">
                <DropdownMenu>
                  <DropdownMenuTrigger as-child>
                    <Button variant="ghost" size="icon">
                      <MoreVertical class="h-4 w-4" />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end">
                    <DropdownMenuItem @click="openEditSheet(user)">
                      <Edit class="mr-2 h-4 w-4" />
                      Edit
                    </DropdownMenuItem>
                    <DropdownMenuItem @click="openPasswordDialog(user)">
                      <KeyRound class="mr-2 h-4 w-4" />
                      Change Password
                    </DropdownMenuItem>
                    <DropdownMenuItem variant="destructive" @click="handleDelete(user.id)">
                      <Trash2 class="mr-2 h-4 w-4" />
                      Delete
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </TableCell>
            </TableRow>
          </template>
        </TableBody>
      </Table>
    </div>

    <!-- Create/Edit Sheet -->
    <Sheet :open="isSheetOpen" @update:open="isSheetOpen = $event">
      <SheetContent class="p-0">
        <div class="h-full flex flex-col py-6">
          <SheetHeader class="px-6 mb-6">
            <SheetTitle>{{ isEditing ? 'Edit User' : 'Create User' }}</SheetTitle>
            <SheetDescription>
              {{ isEditing ? 'Update user details.' : 'Create a new user.' }}
            </SheetDescription>
          </SheetHeader>
          <div class="grid flex-1 auto-rows-min gap-6 px-6 overflow-y-auto">
            <div class="grid gap-2">
              <Label for="username">Username</Label>
              <Input id="username" v-model="form.username" placeholder="jdoe" />
            </div>
            <div v-if="!isEditing" class="grid gap-2">
              <Label for="password">Password</Label>
              <Input id="password" type="password" v-model="form.password" />
            </div>
            <div v-if="!isEditing" class="grid gap-2">
              <Label for="confirmPassword">Confirm Password</Label>
              <Input id="confirmPassword" type="password" v-model="form.confirmPassword" />
            </div>
          </div>
          <SheetFooter class="px-6 mt-6 flex gap-2">
            <Button type="submit" :disabled="store.loading" @click="handleSubmit">
              <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
              {{ isEditing ? 'Save Changes' : 'Create User' }}
            </Button>
            <Button variant="outline" @click="isSheetOpen = false">
              Cancel
            </Button>
          </SheetFooter>
        </div>
      </SheetContent>
    </Sheet>

    <!-- Change Password Dialog -->
    <AlertDialog :open="isPasswordDialogOpen" @update:open="isPasswordDialogOpen = $event">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Change Password</AlertDialogTitle>
          <AlertDialogDescription>
            Enter a new password for the user.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <div class="grid gap-4 py-4">
          <div class="grid gap-2">
            <Label for="newPassword">New Password</Label>
            <Input id="newPassword" type="password" v-model="passwordForm.password" />
          </div>
          <div class="grid gap-2">
            <Label for="confirmNewPassword">Confirm New Password</Label>
            <Input id="confirmNewPassword" type="password" v-model="passwordForm.confirmPassword" />
          </div>
        </div>
        <AlertDialogFooter>
          <AlertDialogCancel @click="isPasswordDialogOpen = false">Cancel</AlertDialogCancel>
          <AlertDialogAction @click="handlePasswordSubmit" :disabled="store.loading">
            <Loader2 v-if="store.loading" class="mr-2 h-4 w-4 animate-spin" />
            Change Password
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>

    <!-- Delete Alert -->
    <AlertDialog :open="isDeleteAlertOpen" @update:open="isDeleteAlertOpen = $event">
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Are you absolutely sure?</AlertDialogTitle>
          <AlertDialogDescription>
            This action cannot be undone. This will permanently delete the user.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel @click="deleteUserId = null" class="cursor-pointer">Cancel</AlertDialogCancel>
          <AlertDialogAction class="bg-destructive hover:bg-destructive/70 cursor-pointer" @click="confirmDelete">
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  </div>
</template>
