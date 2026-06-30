// Origin-relative API access. In production Caddy proxies /v1/* to the services on the same origin; in dev
// the Vite proxy (vite.config.ts) forwards /v1/* to the local services. Every call is just "/v1/...".

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
  const res = await fetch(path, {
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
