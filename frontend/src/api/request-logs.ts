import { requestJson } from "./client";
import type { ApiType } from "./providers";

export interface RequestLogSummary {
  request_id: string;
  gateway_key_id: string | null;
  api_type: ApiType;
  model: string | null;
  alias: string | null;
  provider: string | null;
  endpoint: string | null;
  status_code: number | null;
  latency_ms: number | null;
  client_ip: string | null;
  user_agent: string | null;
  request_content_type: string | null;
  response_content_type: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  created_at: string;
}

export interface RequestLog {
  request_id: string;
  gateway_key_id: string | null;
  api_type: ApiType;
  model: string | null;
  alias: string | null;
  provider: string | null;
  endpoint: string | null;
  status_code: number | null;
  latency_ms: number | null;
  client_ip: string | null;
  user_agent: string | null;
  request_body: number[] | null; // Vec<u8> is array of numbers in JSON
  response_body: number[] | null;
  request_content_type: string | null;
  response_content_type: string | null;
  prompt_tokens: number | null;
  completion_tokens: number | null;
  total_tokens: number | null;
  created_at: string;
}

export interface RequestLogFilter {
  model?: string;
  alias?: string;
  provider?: string;
  client_ip?: string;
}

export const fetchRequestLogs = async (
  limit: number = 20,
  offset: number = 0,
  filter: RequestLogFilter = {},
): Promise<{ data: RequestLogSummary[]; total: number }> => {
  const params = new URLSearchParams({
    limit: limit.toString(),
    offset: offset.toString(),
  });

  if (filter.model) params.append("model", filter.model);
  if (filter.alias) params.append("alias", filter.alias);
  if (filter.provider) params.append("provider", filter.provider);
  if (filter.client_ip) params.append("client_ip", filter.client_ip);

  return requestJson<{ data: RequestLogSummary[]; total: number }>(
    `/request-logs?${params.toString()}`,
  );
};

export const fetchRequestLog = async (id: string): Promise<RequestLog> => {
  return requestJson<RequestLog>(`/request-logs/${id}`);
};
