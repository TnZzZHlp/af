export class ApiError extends Error {
  status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

type RequestOptions = Omit<RequestInit, "body"> & {
  body?: unknown;
};

export async function requestJson<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { body, headers, ...rest } = options;
  const requestHeaders = new Headers(headers);
  requestHeaders.set("Content-Type", "application/json");

  // Add token if available
  const storedUser = localStorage.getItem("af_auth_user");
  if (storedUser) {
    try {
      const user = JSON.parse(storedUser);
      if (user.token) {
        requestHeaders.set("Authorization", `Bearer ${user.token}`);
      }
    } catch {
      // Ignore
    }
  }

  const response = await fetch(`/api${path}`, {
    ...rest,
    headers: requestHeaders,
    body: body === undefined ? undefined : JSON.stringify(body),
  });

  const contentType = response.headers.get("content-type") ?? "";
  const isJson = contentType.includes("application/json");
  const payload = isJson ? await response.json().catch(() => null) : null;

  if (!response.ok) {
    const message =
      typeof payload?.error === "string"
        ? payload.error
        : `Request failed with status ${response.status}`;
    throw new ApiError(message, response.status);
  }

  if (!isJson && response.status !== 204 && response.headers.get("content-length") !== "0") {
    // If it's not JSON and not a known empty body status/length, we might still want to allow it if it's a success
    // but for now let's be a bit more permissive for 200/204
    if (response.status !== 200) {
      throw new ApiError("Unexpected response from server", response.status);
    }
  }

  return payload as T;
}
