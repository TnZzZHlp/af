import { requestJson } from "./client";

export interface GatewayKey {
  id: string;
  name: string | null;
  key: string;
  enabled: boolean;
  rate_limit_rps: number | null;
  rate_limit_rpm: number | null;
  created_at: string;
}

export interface ListGatewayKeysParams {
  page?: number;
  page_size?: number;
}

export interface CreateGatewayKeyRequest {
  name?: string | null;
  rate_limit_rps?: number | null;
  rate_limit_rpm?: number | null;
}

export interface UpdateGatewayKeyRequest {
  name: string | null;
  enabled?: boolean;
  rate_limit_rps: number | null;
  rate_limit_rpm: number | null;
}

export async function listGatewayKeys(params: ListGatewayKeysParams = {}): Promise<GatewayKey[]> {
  const query = new URLSearchParams();
  if (params.page !== undefined) query.set("page", params.page.toString());
  if (params.page_size !== undefined) query.set("page_size", params.page_size.toString());

  const queryString = query.toString();
  return requestJson<GatewayKey[]>(`/gateway-keys${queryString ? `?${queryString}` : ""}`);
}

export async function createGatewayKey(payload: CreateGatewayKeyRequest): Promise<GatewayKey> {
  return requestJson<GatewayKey>("/gateway-keys", {
    method: "POST",
    body: payload,
  });
}

export async function getGatewayKey(id: string): Promise<GatewayKey> {
  return requestJson<GatewayKey>(`/gateway-keys/${id}`);
}

export async function updateGatewayKey(
  id: string,
  payload: UpdateGatewayKeyRequest,
): Promise<GatewayKey> {
  return requestJson<GatewayKey>(`/gateway-keys/${id}`, {
    method: "PUT",
    body: payload,
  });
}

export async function deleteGatewayKey(id: string): Promise<void> {
  await requestJson<void>(`/gateway-keys/${id}`, {
    method: "DELETE",
  });
}
