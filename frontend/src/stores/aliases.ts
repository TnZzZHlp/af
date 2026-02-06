import { defineStore } from "pinia";
import { ref } from "vue";
import {
  listAliases,
  createAlias,
  updateAlias,
  deleteAlias,
  listAliasTargetDetails,
  createAliasTarget,
  updateAliasTarget,
  deleteAliasTarget,
  type Alias,
  type AliasTargetDetail,
  type CreateAliasRequest,
  type UpdateAliasRequest,
  type CreateAliasTargetRequest,
  type UpdateAliasTargetRequest,
} from "@/api/aliases";
import { ApiError } from "@/api/client";

export const useAliasesStore = defineStore("aliases", () => {
  const aliases = ref<Alias[]>([]);
  const targets = ref<Record<string, AliasTargetDetail[]>>({});
  const loading = ref(false);
  const error = ref<string | null>(null);

  function clearError() {
    error.value = null;
  }

  async function fetchAliases() {
    loading.value = true;
    error.value = null;
    try {
      aliases.value = await listAliases();
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to fetch aliases";
    } finally {
      loading.value = false;
    }
  }

  async function addAlias(payload: CreateAliasRequest) {
    loading.value = true;
    error.value = null;
    try {
      const newAlias = await createAlias(payload);
      aliases.value.unshift(newAlias);
      return newAlias;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to create alias";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function patchAlias(id: string, payload: UpdateAliasRequest) {
    loading.value = true;
    error.value = null;
    try {
      const updatedAlias = await updateAlias(id, payload);
      const index = aliases.value.findIndex((a) => a.id === id);
      if (index !== -1) {
        aliases.value[index] = updatedAlias;
      }
      return updatedAlias;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to update alias";
      return null;
    } finally {
      loading.value = false;
    }
  }

  async function removeAlias(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await deleteAlias(id);
      aliases.value = aliases.value.filter((a) => a.id !== id);
      delete targets.value[id];
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to delete alias";
      return false;
    } finally {
      loading.value = false;
    }
  }

  // Targets
  async function fetchTargets(aliasId: string) {
    loading.value = true;
    error.value = null;
    try {
      const data = await listAliasTargetDetails(aliasId);
      targets.value[aliasId] = data;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to fetch alias targets";
    } finally {
      loading.value = false;
    }
  }

  async function addTarget(aliasId: string, payload: CreateAliasTargetRequest) {
    loading.value = true;
    error.value = null;
    try {
      await createAliasTarget(aliasId, payload);
      // Re-fetch targets to get detailed info (names)
      await fetchTargets(aliasId);
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to create alias target";
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function patchTarget(aliasId: string, targetId: string, payload: UpdateAliasTargetRequest) {
    loading.value = true;
    error.value = null;
    try {
      await updateAliasTarget(aliasId, targetId, payload);
      // Re-fetch targets to ensure consistency
      await fetchTargets(aliasId);
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to update alias target";
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function removeTarget(aliasId: string, targetId: string) {
    loading.value = true;
    error.value = null;
    try {
      await deleteAliasTarget(aliasId, targetId);
      if (targets.value[aliasId]) {
        targets.value[aliasId] = targets.value[aliasId].filter((t) => t.id !== targetId);
      }
      return true;
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to delete alias target";
      return false;
    } finally {
      loading.value = false;
    }
  }

  return {
    aliases,
    targets,
    loading,
    error,
    clearError,
    fetchAliases,
    addAlias,
    patchAlias,
    removeAlias,
    fetchTargets,
    addTarget,
    patchTarget,
    removeTarget,
  };
});
