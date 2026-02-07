import { defineStore } from 'pinia'
import { ref } from 'vue'
import { type RequestLog, type RequestLogSummary, fetchRequestLogs, fetchRequestLog } from '@/api/request-logs'

export const useRequestLogsStore = defineStore('request-logs', () => {
  const requestLogs = ref<RequestLogSummary[]>([])
  const currentLog = ref<RequestLog | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const loadRequestLogs = async (limit: number = 20, offset: number = 0) => {
    isLoading.value = true
    error.value = null
    try {
      requestLogs.value = await fetchRequestLogs(limit, offset)
    } catch (e) {
      console.error('Failed to load request logs:', e)
      error.value = e instanceof Error ? e.message : 'Failed to load request logs'
    } finally {
      isLoading.value = false
    }
  }

  const loadRequestLog = async (id: string) => {
    isLoading.value = true
    error.value = null
    try {
      currentLog.value = await fetchRequestLog(id)
    } catch (e) {
      console.error('Failed to load request log:', e)
      error.value = e instanceof Error ? e.message : 'Failed to load request log'
    } finally {
      isLoading.value = false
    }
  }

  return {
    requestLogs,
    currentLog,
    isLoading,
    error,
    loadRequestLogs,
    loadRequestLog,
  }
})
