import { requestJson } from "./client";

export interface Alias {
  id: string;
  name: string;
  enabled: boolean;
  created_at: string;
}

export interface AliasTarget {
  id: string;
  alias_id: string;
  provider_id: string;
  model_id: string;
  enabled: boolean;
  created_at: string;
}

export interface AliasTargetDetail extends AliasTarget {
  alias_name: string;
  alias_target_id: string; // duplicate of id
  provider_id: string;
  provider_name: string;
  provider_usage_count: number;
  provider_endpoint_id?: string;
  endpoint_url?: string;
  // api_type is no longer returned in detail list
}

export interface ListAliasesParams {
  page?: number;
  page_size?: number;
}

export interface CreateAliasRequest {
  name: string;
}

export interface UpdateAliasRequest {
  name?: string;
  enabled?: boolean;
}

export interface CreateAliasTargetRequest {
  provider_id: string;
  model_id: string;
}

export interface UpdateAliasTargetRequest {
  provider_id?: string;
  model_id?: string;
  enabled?: boolean;
}

// Alias API
export async function listAliases(params: ListAliasesParams = {}): Promise<Alias[]> {
  const query = new URLSearchParams();
  if (params.page !== undefined) query.set("page", params.page.toString());
  if (params.page_size !== undefined) query.set("page_size", params.page_size.toString());

  const queryString = query.toString();
  return requestJson<Alias[]>(`/aliases${queryString ? `?${queryString}` : ""}`);
}

export async function createAlias(payload: CreateAliasRequest): Promise<Alias> {
  return requestJson<Alias>("/aliases", {
    method: "POST",
    body: payload,
  });
}

export async function getAlias(id: string): Promise<Alias> {
  return requestJson<Alias>(`/aliases/${id}`);
}

export async function updateAlias(id: string, payload: UpdateAliasRequest): Promise<Alias> {
  return requestJson<Alias>(`/aliases/${id}`, {
    method: "PUT",
    body: payload,
  });
}

export async function deleteAlias(id: string): Promise<void> {
  await requestJson<void>(`/aliases/${id}`, {
    method: "DELETE",
  });
}

// Alias Target API
export async function listAliasTargetDetails(aliasId: string): Promise<AliasTargetDetail[]> {
  return requestJson<AliasTargetDetail[]>(`/aliases/${aliasId}/targets/details`);
}

export async function createAliasTarget(
  aliasId: string,
  payload: CreateAliasTargetRequest,
): Promise<AliasTarget> {
  return requestJson<AliasTarget>(`/aliases/${aliasId}/targets`, {
    method: "POST",
    body: payload,
  });
}

export async function updateAliasTarget(
  aliasId: string,
  targetId: string,
  payload: UpdateAliasTargetRequest,
): Promise<AliasTarget> {
  return requestJson<AliasTarget>(`/aliases/${aliasId}/targets/${targetId}`, {
    method: "PUT",
    body: payload,
  });
}

export async function deleteAliasTarget(aliasId: string, targetId: string): Promise<void> {
  await requestJson<void>(`/aliases/${aliasId}/targets/${targetId}`, {
    method: "DELETE",
  });
}
