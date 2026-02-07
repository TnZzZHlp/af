import { defineStore } from 'pinia'
import { ref } from 'vue'
import { type RequestLog, fetchRequestLogs } from '@/api/request-logs'

export const useRequestLogsStore = defineStore('request-logs', () => {
  const requestLogs = ref<RequestLog[]>([])
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadRequestLogs = async (limit: number = 20, offset: number = 0) => {
    isLoading.value = true
    error.value = null
    try {
      requestLogs.value = await fetchRequestLogs(limit, offset)
    } catch (e: any) {
      console.error('Failed to load request logs:', e)
      error.value = e.message || 'Failed to load request logs'
    } finally {
      isLoading.value = false
    }
  }

  return {
    requestLogs,
    isLoading,
    error,
    loadRequestLogs,
  }
})
