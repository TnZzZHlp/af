import { defineStore } from 'pinia'
import { ref } from 'vue'
import { type RequestLog, type RequestLogSummary, type RequestLogFilter, fetchRequestLogs, fetchRequestLog } from '@/api/request-logs'

export const useRequestLogsStore = defineStore('request-logs', () => {
  const requestLogs = ref<RequestLogSummary[]>([])
  const total = ref(0)
  const currentLog = ref<RequestLog | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)
  const filter = ref<RequestLogFilter>({})

  const loadRequestLogs = async (limit: number = 20, offset: number = 0) => {
    isLoading.value = true
    error.value = null
    try {
      const response = await fetchRequestLogs(limit, offset, filter.value)
      requestLogs.value = response.data
      total.value = response.total
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
    total,
    currentLog,
    isLoading,
    error,
    filter,
    loadRequestLogs,
    loadRequestLog,
  }
})
