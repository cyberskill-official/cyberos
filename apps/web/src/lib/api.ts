// API access. In production Caddy proxies /v1/* to the services on the same origin; in dev the Vite proxy
// (vite.config.ts) forwards /v1/* to the local services. On both, an origin-relative "/v1/..." is correct
// and API_BASE is "".
//
// Capacitor is the exception, and it fails silently, so it is worth spelling out. A native build serves the
// bundle from its own local origin - capacitor://localhost on iOS, http://localhost on Android - so a
// relative "/v1/auth/token" resolves against THAT origin, not the server. The local bundle server answers
// every unknown path with index.html and HTTP 200. The request therefore "succeeds": res.ok is true, then
// res.json() chokes on HTML, the .catch(() => null) swallows it, and the caller throws with the status
// attached - which is how sign-in reported "(200)" on a request that never left the device. A genuine auth
// failure would have been 401.
//
// Native builds therefore need an absolute origin. Detected from location rather than by importing
// @capacitor/core, which is a devDependency and should not become runtime code in the web bundle.
//
// Not affected: the Tauri desktop shell, which navigates to https://os.cyberskill.world/ on boot
// (apps/desktop/src/index.html) and is genuinely same-origin from then on.
const NATIVE_ORIGIN = "https://os.cyberskill.world";

function resolveApiBase(): string {
  if (typeof window === "undefined") return "";
  const { protocol, hostname, port } = window.location;
  if (protocol === "capacitor:") return NATIVE_ORIGIN; // iOS
  // Android Capacitor serves from http://localhost with no port. The Vite dev server always has one
  // (5173), so requiring an empty port keeps local development on the proxy.
  if (protocol === "http:" && hostname === "localhost" && port === "") return NATIVE_ORIGIN;
  return "";
}

export const API_BASE: string = resolveApiBase();

// Absolute on native, unchanged on web/desktop. Pass any origin-relative path through this before fetch.
export function apiUrl(path: string): string {
  return API_BASE && path.startsWith("/") ? API_BASE + path : path;
}

// ws:// or wss:// origin for the chat sockets. On web this mirrors the page origin; on native it is derived
// from API_BASE, because location.origin there is capacitor://localhost - a scheme the WebSocket
// constructor rejects outright, and one that "replace(/^http/, 'ws')" silently fails to rewrite at all.
export function wsOrigin(): string {
  const base = API_BASE || window.location.origin;
  return base.replace(/^http/, "ws");
}

export class ApiError extends Error {
  status: number;
  constructor(status: number, message: string) {
    super(message);
    this.status = status;
    this.name = "ApiError";
  }
}

export function decodeJwt(token: string): Record<string, unknown> | null {
  try {
    let p = token.split(".")[1].replace(/-/g, "+").replace(/_/g, "/");
    p += "====".slice(p.length % 4 || 4);
    return JSON.parse(atob(p));
  } catch {
    return null;
  }
}

export function tokenValid(token: string | null): boolean {
  if (!token) return false;
  const c = decodeJwt(token);
  const exp = c && typeof c.exp === "number" ? c.exp : 0;
  return exp * 1000 > Date.now();
}

// Authenticated JSON request. Throws ApiError on a non-2xx response, surfacing the service's `error` field
// when present so the UI can show a real message instead of a bare status code.
export async function apiFetch<T = unknown>(
  token: string,
  method: string,
  path: string,
  body?: unknown,
): Promise<T> {
  const res = await fetch(apiUrl(path), {
    method,
    headers: {
      Authorization: "Bearer " + token,
      ...(body !== undefined ? { "content-type": "application/json" } : {}),
    },
    body: body !== undefined ? JSON.stringify(body) : undefined,
  });
  if (!res.ok) {
    let msg = `request failed (${res.status})`;
    try {
      const j = (await res.json()) as { error?: string };
      if (j && j.error) msg = j.error;
    } catch {
      /* non-JSON error body */
    }
    throw new ApiError(res.status, msg);
  }
  if (res.status === 204) return undefined as T;
  const ct = res.headers.get("content-type") || "";
  return (ct.includes("application/json") ? await res.json() : await res.text()) as T;
}

// Authenticated raw-body upload: POSTs the file bytes as the request body (no base64 inflation), content
// type from the file itself. Same error shape as apiFetch so callers can branch on ApiError.status.
export async function apiUploadRaw<T = unknown>(token: string, path: string, file: File | Blob): Promise<T> {
  const res = await fetch(apiUrl(path), {
    method: "POST",
    headers: {
      Authorization: "Bearer " + token,
      "content-type": (file as File).type || "application/octet-stream",
    },
    body: file,
  });
  if (!res.ok) {
    let msg = `upload failed (${res.status})`;
    try {
      const j = (await res.json()) as { error?: string };
      if (j && j.error) msg = j.error;
    } catch {
      /* non-JSON error body */
    }
    throw new ApiError(res.status, msg);
  }
  return (await res.json()) as T;
}
