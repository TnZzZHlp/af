import { requestJson } from "./client";

export type LoginRequest = {
  username: string;
  password: string;
};

export type LoginResponse = {
  id: string;
  username: string;
  name: string | null;
};

export function login(payload: LoginRequest) {
  return requestJson<LoginResponse>("/auth/login", {
    method: "POST",
    body: payload,
  });
}
