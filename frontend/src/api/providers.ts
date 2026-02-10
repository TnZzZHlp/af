import { requestJson } from "./client";

export type ApiType = 'openai_chat_completions' | 'openai_responses' | 'openai_models' | 'anthropic_messages';

export interface Provider {
  id: string;
  name: string;
  description: string | null;
  brief: string | null;
  enabled: boolean;
  created_at: string;
}

export interface ProviderEndpoint {
  id: string;
  provider_id: string;
  api_type: ApiType;
  url: string;
  enabled: boolean;
  created_at: string;
}

export interface ProviderKey {
  id: string;
  provider_id: string;
  name: string | null;
  key: string;
  usage_count: number;
  enabled: boolean;
  created_at: string;
}

export interface ListProvidersParams {
  page?: number;
  page_size?: number;
}

export interface CreateProviderRequest {
  name: string;
  description?: string;
  brief?: string;
}

export interface UpdateProviderRequest {
  name?: string;
  description?: string;
  brief?: string;
  enabled?: boolean;
}

export interface CreateEndpointRequest {
  api_type: ApiType;
  url: string;
}

export interface UpdateEndpointRequest {
  url?: string;
  enabled?: boolean;
}

export interface CreateKeyRequest {
  name?: string;
  key: string;
}

export interface UpdateKeyRequest {
  name?: string;
  enabled?: boolean;
}

// Provider API
export async function listProviders(params: ListProvidersParams = {}): Promise<Provider[]> {
  const query = new URLSearchParams();
  if (params.page !== undefined) query.set("page", params.page.toString());
  if (params.page_size !== undefined) query.set("page_size", params.page_size.toString());

  const queryString = query.toString();
  return requestJson<Provider[]>(`/providers${queryString ? `?${queryString}` : ""}`);
}

export async function createProvider(payload: CreateProviderRequest): Promise<Provider> {
  return requestJson<Provider>("/providers", {
    method: "POST",
    body: payload,
  });
}

export async function getProvider(id: string): Promise<Provider> {
  return requestJson<Provider>(`/providers/${id}`);
}

export async function updateProvider(id: string, payload: UpdateProviderRequest): Promise<Provider> {
  return requestJson<Provider>(`/providers/${id}`, {
    method: "PUT",
    body: payload,
  });
}

export async function deleteProvider(id: string): Promise<void> {
  await requestJson<void>(`/providers/${id}`, {
    method: "DELETE",
  });
}

// Endpoint API
export async function listEndpoints(providerId: string): Promise<ProviderEndpoint[]> {
  return requestJson<ProviderEndpoint[]>(`/providers/${providerId}/endpoints`);
}

export async function createEndpoint(providerId: string, payload: CreateEndpointRequest): Promise<ProviderEndpoint> {
  return requestJson<ProviderEndpoint>(`/providers/${providerId}/endpoints`, {
    method: "POST",
    body: payload,
  });
}

export async function updateEndpoint(providerId: string, endpointId: string, payload: UpdateEndpointRequest): Promise<ProviderEndpoint> {
  return requestJson<ProviderEndpoint>(`/providers/${providerId}/endpoints/${endpointId}`, {
    method: "PUT",
    body: payload,
  });
}

export async function deleteEndpoint(providerId: string, endpointId: string): Promise<void> {
  await requestJson<void>(`/providers/${providerId}/endpoints/${endpointId}`, {
    method: "DELETE",
  });
}

// Key API
export async function listKeys(providerId: string): Promise<ProviderKey[]> {
  return requestJson<ProviderKey[]>(`/providers/${providerId}/keys`);
}

export async function createKey(providerId: string, payload: CreateKeyRequest): Promise<ProviderKey> {
  return requestJson<ProviderKey>(`/providers/${providerId}/keys`, {
    method: "POST",
    body: payload,
  });
}

export async function updateKey(providerId: string, keyId: string, payload: UpdateKeyRequest): Promise<ProviderKey> {
  return requestJson<ProviderKey>(`/providers/${providerId}/keys/${keyId}`, {
    method: "PUT",
    body: payload,
  });
}

export async function deleteKey(providerId: string, keyId: string): Promise<void> {
  await requestJson<void>(`/providers/${providerId}/keys/${keyId}`, {
    method: "DELETE",
  });
}