import { defineStore } from "pinia";
import { ref } from "vue";
import {
  listProviders,
  createProvider,
  updateProvider,
  deleteProvider,
  listEndpoints,
  createEndpoint,
  updateEndpoint,
  deleteEndpoint,
  listKeys,
  createKey,
  updateKey,
  deleteKey,
  type Provider,
  type ProviderEndpoint,
  type ProviderKey,
  type CreateProviderRequest,
  type UpdateProviderRequest,
  type CreateEndpointRequest,
  type UpdateEndpointRequest,
  type CreateKeyRequest,
  type UpdateKeyRequest,
} from "@/api/providers";
import { ApiError } from "@/api/client";

export const useProvidersStore = defineStore("providers", () => {
  const providers = ref<Provider[]>([]);
  const endpoints = ref<Record<string, ProviderEndpoint[]>>({});
  const keys = ref<Record<string, ProviderKey[]>>({});
  const loading = ref(false);
  const error = ref<string | null>(null);

  function clearError() {
    error.value = null;
  }

  async function fetchProviders() {
    loading.value = true;
    error.value = null;
    try {
      providers.value = await listProviders();
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to fetch providers";
    } finally {
      loading.value = false;
    }
  }

  async function addProvider(payload: CreateProviderRequest) {
    loading.value = true;
    error.value = null;
    try {
      const newProvider = await createProvider(payload);
      providers.value.unshift(newProvider);
      return newProvider;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to create provider";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function patchProvider(id: string, payload: UpdateProviderRequest) {
    loading.value = true;
    error.value = null;
    try {
      const updatedProvider = await updateProvider(id, payload);
      const index = providers.value.findIndex((p) => p.id === id);
      if (index !== -1) {
        providers.value[index] = updatedProvider;
      }
      return updatedProvider;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to update provider";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function removeProvider(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await deleteProvider(id);
      providers.value = providers.value.filter((p) => p.id !== id);
      delete endpoints.value[id];
      delete keys.value[id];
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to delete provider";
      return false;
    } finally {
      loading.value = false;
    }
  }

  // Endpoints
  async function fetchEndpoints(providerId: string) {
    loading.value = true;
    error.value = null;
    try {
      const data = await listEndpoints(providerId);
      endpoints.value[providerId] = data;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to fetch endpoints";
    } finally {
      loading.value = false;
    }
  }

  async function addEndpoint(providerId: string, payload: CreateEndpointRequest) {
    loading.value = true;
    error.value = null;
    try {
      const newEndpoint = await createEndpoint(providerId, payload);
      if (!endpoints.value[providerId]) {
        endpoints.value[providerId] = [];
      }
      endpoints.value[providerId].unshift(newEndpoint);
      return newEndpoint;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to create endpoint";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function patchEndpoint(providerId: string, endpointId: string, payload: UpdateEndpointRequest) {
    loading.value = true;
    error.value = null;
    try {
      const updatedEndpoint = await updateEndpoint(providerId, endpointId, payload);
      const list = endpoints.value[providerId];
      if (list) {
        const index = list.findIndex((e) => e.id === endpointId);
        if (index !== -1) {
          list[index] = updatedEndpoint;
        }
      }
      return updatedEndpoint;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to update endpoint";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function removeEndpoint(providerId: string, endpointId: string) {
    loading.value = true;
    error.value = null;
    try {
      await deleteEndpoint(providerId, endpointId);
      if (endpoints.value[providerId]) {
        endpoints.value[providerId] = endpoints.value[providerId].filter((e) => e.id !== endpointId);
      }
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to delete endpoint";
      return false;
    } finally {
      loading.value = false;
    }
  }

  // Keys
  async function fetchKeys(providerId: string) {
    loading.value = true;
    error.value = null;
    try {
      const data = await listKeys(providerId);
      keys.value[providerId] = data;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to fetch keys";
    } finally {
      loading.value = false;
    }
  }

  async function addKey(providerId: string, payload: CreateKeyRequest) {
    loading.value = true;
    error.value = null;
    try {
      const newKey = await createKey(providerId, payload);
      if (!keys.value[providerId]) {
        keys.value[providerId] = [];
      }
      keys.value[providerId].unshift(newKey);
      return newKey;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to create key";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function patchKey(providerId: string, keyId: string, payload: UpdateKeyRequest) {
    loading.value = true;
    error.value = null;
    try {
      const updatedKey = await updateKey(providerId, keyId, payload);
      const list = keys.value[providerId];
      if (list) {
        const index = list.findIndex((k) => k.id === keyId);
        if (index !== -1) {
          list[index] = updatedKey;
        }
      }
      return updatedKey;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to update key";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function removeKey(providerId: string, keyId: string) {
    loading.value = true;
    error.value = null;
    try {
      await deleteKey(providerId, keyId);
      if (keys.value[providerId]) {
        keys.value[providerId] = keys.value[providerId].filter((k) => k.id !== keyId);
      }
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to delete key";
      return false;
    } finally {
      loading.value = false;
    }
  }

  return {
    providers,
    endpoints,
    keys,
    loading,
    error,
    clearError,
    fetchProviders,
    addProvider,
    patchProvider,
    removeProvider,
    fetchEndpoints,
    addEndpoint,
    patchEndpoint,
    removeEndpoint,
    fetchKeys,
    addKey,
    patchKey,
    removeKey,
  };
});