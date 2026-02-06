import { defineStore } from "pinia";
import { ref } from "vue";
import {
  listGatewayKeys,
  createGatewayKey,
  updateGatewayKey,
  deleteGatewayKey,
  type GatewayKey,
  type CreateGatewayKeyRequest,
  type UpdateGatewayKeyRequest,
} from "@/api/gateway-keys";
import { ApiError } from "@/api/client";

export const useGatewayKeysStore = defineStore("gateway-keys", () => {
  const keys = ref<GatewayKey[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchKeys() {
    loading.value = true;
    error.value = null;
    try {
      keys.value = await listGatewayKeys();
    } catch (e) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to fetch gateway keys";
      }
    } finally {
      loading.value = false;
    }
  }

  async function createKey(payload: CreateGatewayKeyRequest) {
    loading.value = true;
    error.value = null;
    try {
      const newKey = await createGatewayKey(payload);
      keys.value.unshift(newKey);
      return newKey;
    } catch (e) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to create gateway key";
      }
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function updateKey(id: string, payload: UpdateGatewayKeyRequest) {
    loading.value = true;
    error.value = null;
    try {
      const updatedKey = await updateGatewayKey(id, payload);
      const index = keys.value.findIndex((k) => k.id === id);
      if (index !== -1) {
        keys.value[index] = updatedKey;
      }
      return updatedKey;
    } catch (e) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to update gateway key";
      }
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function deleteKey(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await deleteGatewayKey(id);
      keys.value = keys.value.filter((k) => k.id !== id);
      return true;
    } catch (e) {
      if (e instanceof ApiError) {
        error.value = e.message;
      } else {
        error.value = "Failed to delete gateway key";
      }
      return false;
    } finally {
      loading.value = false;
    }
  }

  return {
    keys,
    loading,
    error,
    fetchKeys,
    createKey,
    updateKey,
    deleteKey,
  };
});
