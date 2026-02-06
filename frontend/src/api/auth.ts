import { requestJson } from "./client";

export type LoginRequest = {
  username: string;
  password: string;
};

export type LoginResponse = {
  id: string;
  username: string;
  name: string | null;
  token: string;
};

export function login(payload: LoginRequest) {
  return requestJson<LoginResponse>("/auth/login", {
    method: "POST",
    body: payload,
  });
}
