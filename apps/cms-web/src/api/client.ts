import { API_BASE_URL, ApiError } from "../types";
import { getToken } from "../auth/storage";

interface RequestOptions extends Omit<RequestInit, "body"> {
  body?: BodyInit | object | null;
}

export async function apiFetch<T>(
  path: string,
  options: RequestOptions = {},
): Promise<T> {
  const url = `${API_BASE_URL}${path}`;
  const token = getToken();

  const headers = new Headers(options.headers);
  headers.set("Accept", "application/json");
  if (token) {
    headers.set("Authorization", `Bearer ${token}`);
  }

  let body: BodyInit | undefined;
  if (options.body instanceof FormData) {
    body = options.body;
  } else if (options.body !== undefined && options.body !== null) {
    headers.set("Content-Type", "application/json");
    body = JSON.stringify(options.body);
  }

  let response: Response;
  try {
    response = await fetch(url, { ...options, headers, body });
  } catch {
    throw new ApiError(
      0,
      "network_error",
      "Impossibile contattare il server. Verifica la connessione.",
      null,
    );
  }

  if (response.status === 204) {
    return undefined as T;
  }

  let data: unknown;
  try {
    data = await response.json();
  } catch {
    data = null;
  }

  if (!response.ok) {
    const errorBody = (
      data as {
        error?: {
          code: string;
          message: string;
          details: Record<string, unknown> | null;
        };
      } | null
    )?.error;
    const code = errorBody?.code ?? "unknown_error";
    const message = errorBody?.message ?? response.statusText;
    const details = errorBody?.details ?? null;
    throw new ApiError(response.status, code, message, details);
  }

  return data as T;
}
