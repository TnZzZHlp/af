import { requestJson } from "./client";

export type User = {
  id: string;
  username: string;
  enabled: boolean;
  created_at: string;
  password_updated_at: string | null;
};

export type CreateUserRequest = {
  username: string;
  password?: string;
  enabled?: boolean;
};

export type UpdateUserRequest = {
  username?: string;
  enabled?: boolean;
};

export type UpdatePasswordRequest = {
  password: string;
};

export function listUsers() {
  return requestJson<User[]>("/users");
}

export function getUser(id: string) {
  return requestJson<User>(`/users/${id}`);
}

export function createUser(payload: CreateUserRequest) {
  return requestJson<User>("/users", {
    method: "POST",
    body: payload,
  });
}

export function updateUser(id: string, payload: UpdateUserRequest) {
  return requestJson<User>(`/users/${id}`, {
    method: "PUT",
    body: payload,
  });
}

export function updatePassword(id: string, payload: UpdatePasswordRequest) {
  return requestJson<void>(`/users/${id}/password`, {
    method: "PUT",
    body: payload,
  });
}

export function deleteUser(id: string) {
  return requestJson<void>(`/users/${id}`, {
    method: "DELETE",
  });
}
