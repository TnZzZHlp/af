import { requestJson } from './client'
import type { ApiType } from './providers'

export interface RequestLog {
  request_id: string
  gateway_key_id: string | null
  api_type: ApiType
  model: string | null
  alias: string | null
  provider: string | null
  endpoint: string | null
  status_code: number | null
  latency_ms: number | null
  client_ip: string | null
  user_agent: string | null
  request_body: number[] | null // Vec<u8> is array of numbers in JSON
  response_body: number[] | null
  request_content_type: string | null
  response_content_type: string | null
  created_at: string
}

export const fetchRequestLogs = async (
  limit: number = 20,
  offset: number = 0,
): Promise<RequestLog[]> => {
  const params = new URLSearchParams({
    limit: limit.toString(),
    offset: offset.toString(),
  });
  return requestJson<RequestLog[]>(`/request-logs?${params.toString()}`)
}