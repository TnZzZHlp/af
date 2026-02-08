import { defineStore } from "pinia";
import { ref } from "vue";
import { type DashboardStats, type StatsQuery, getDashboardStats } from "@/api/stats";
import { ApiError } from "@/api/client";

export const useStatsStore = defineStore("stats", () => {
  const stats = ref<DashboardStats | null>(null);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function fetchStats(query: StatsQuery = {}) {
    loading.value = true;
    error.value = null;
    try {
      stats.value = await getDashboardStats(query);
    } catch (e) {
      error.value = e instanceof ApiError ? e.message : "Failed to fetch dashboard stats";
    } finally {
      loading.value = false;
    }
  }

  return {
    stats,
    loading,
    error,
    fetchStats,
  };
});
