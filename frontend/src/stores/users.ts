import { defineStore } from "pinia";
import { ref } from "vue";
import {
  listUsers as apiListUsers,
  createUser as apiCreateUser,
  updateUser as apiUpdateUser,
  updatePassword as apiUpdatePassword,
  deleteUser as apiDeleteUser,
  type User,
  type CreateUserRequest,
  type UpdateUserRequest,
  type UpdatePasswordRequest,
} from "@/api/users";
import { ApiError } from "@/api/client";

export const useUsersStore = defineStore("users", () => {
  const users = ref<User[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchUsers() {
    loading.value = true;
    error.value = null;
    try {
      users.value = await apiListUsers();
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to fetch users";
      }
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function createUser(payload: CreateUserRequest) {
    loading.value = true;
    error.value = null;
    try {
      const newUser = await apiCreateUser(payload);
      users.value.push(newUser);
      return newUser;
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to create user";
      }
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updateUser(id: string, payload: UpdateUserRequest) {
    loading.value = true;
    error.value = null;
    try {
      const updatedUser = await apiUpdateUser(id, payload);
      const index = users.value.findIndex((u) => u.id === id);
      if (index !== -1) {
        users.value[index] = updatedUser;
      }
      return updatedUser;
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to update user";
      }
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updatePassword(id: string, payload: UpdatePasswordRequest) {
    loading.value = true;
    error.value = null;
    try {
      await apiUpdatePassword(id, payload);
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to update password";
      }
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteUser(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await apiDeleteUser(id);
      users.value = users.value.filter((u) => u.id !== id);
    } catch (e: unknown) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to delete user";
      }
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    users,
    loading,
    error,
    fetchUsers,
    createUser,
    updateUser,
    updatePassword,
    deleteUser,
  };
});
