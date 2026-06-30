import { createContext, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { decodeJwt, tokenValid } from "./api";

// Token persistence mirrors the legacy console (app.html): access + refresh tokens in localStorage, email
// derived from the JWT. This is a first-party app on a real origin, so localStorage is appropriate.
const LS = { token: "cyberos_token", refresh: "cyberos_refresh", email: "cyberos_email" } as const;
const lsGet = (k: string) => {
  try {
    return localStorage.getItem(k) || "";
  } catch {
    return "";
  }
};
const lsSet = (k: string, v: string) => {
  try {
    localStorage.setItem(k, v);
  } catch {
    /* storage disabled */
  }
};
const lsDel = (k: string) => {
  try {
    localStorage.removeItem(k);
  } catch {
    /* storage disabled */
  }
};

interface AuthState {
  token: string | null;
  email: string;
  ready: boolean;
  signedIn: boolean;
  loginPassword(tenant: string, handle: string, password: string): Promise<void>;
  googleSignIn(tenant: string): Promise<void>;
  logout(): void;
}

const Ctx = createContext<AuthState | null>(null);

export function useAuth(): AuthState {
  const c = useContext(Ctx);
  if (!c) throw new Error("useAuth used outside AuthProvider");
  return c;
}

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(null);
  const [email, setEmail] = useState("");
  const [ready, setReady] = useState(false);

  function adopt(at: string, rt?: string | null) {
    lsSet(LS.token, at);
    if (rt) lsSet(LS.refresh, rt);
    const claims = decodeJwt(at);
    const e = claims && typeof claims.email === "string" ? claims.email : "";
    lsSet(LS.email, e);
    setEmail(e);
    setToken(at);
  }

  async function refresh(): Promise<boolean> {
    const rt = lsGet(LS.refresh);
    if (!rt) return false;
    try {
      const res = await fetch("/v1/auth/token", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ grant_type: "refresh_token", refresh_token: rt }),
      });
      if (!res.ok) return false;
      const data = (await res.json()) as { access_token?: string; refresh_token?: string };
      if (!data || !data.access_token) return false;
      adopt(data.access_token, data.refresh_token);
      return true;
    } catch {
      return false;
    }
  }

  // Boot: capture an OIDC hand-back fragment (#access_token=...), else a valid stored token, else refresh.
  useEffect(() => {
    let alive = true;
    (async () => {
      if (location.hash && location.hash.indexOf("access_token") !== -1) {
        const p = new URLSearchParams(location.hash.replace(/^#/, ""));
        const at = p.get("access_token");
        if (at) {
          adopt(at, p.get("refresh_token"));
          try {
            history.replaceState(null, "", location.pathname + location.search);
          } catch {
            location.hash = "";
          }
          if (alive) setReady(true);
          return;
        }
      }
      const stored = lsGet(LS.token);
      if (tokenValid(stored)) {
        setToken(stored);
        setEmail(lsGet(LS.email));
      } else if (!(await refresh())) {
        lsDel(LS.token);
      }
      if (alive) setReady(true);
    })();
    // Periodic silent refresh so a long session does not dead-end when the ~1h access token expires.
    const iv = window.setInterval(() => {
      if (lsGet(LS.refresh)) void refresh();
    }, 45 * 60 * 1000);
    return () => {
      alive = false;
      window.clearInterval(iv);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loginPassword(tenant: string, handle: string, password: string) {
    const res = await fetch("/v1/auth/token", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ grant_type: "password", tenant_slug: tenant, handle, password }),
    });
    const data = (await res.json().catch(() => null)) as
      | { access_token?: string; refresh_token?: string; error?: string }
      | null;
    if (!res.ok || !data || !data.access_token) {
      throw new Error(data && data.error ? data.error : `sign-in failed (${res.status})`);
    }
    adopt(data.access_token, data.refresh_token);
  }

  async function googleSignIn(tenant: string) {
    const ret = location.origin + location.pathname; // return here; the boot effect captures the hand-back
    const res = await fetch(
      `/v1/auth/oidc/initiate?tenant_slug=${encodeURIComponent(tenant)}&idp=google&return_to=${encodeURIComponent(ret)}`,
    );
    const data = (await res.json().catch(() => null)) as
      | { authorization_url?: string; error?: string }
      | null;
    if (!res.ok || !data || !data.authorization_url) {
      throw new Error(data && data.error ? data.error : `could not start Google sign-in (${res.status})`);
    }
    window.location.href = data.authorization_url;
  }

  function logout() {
    lsDel(LS.token);
    lsDel(LS.refresh);
    lsDel(LS.email);
    setToken(null);
    setEmail("");
  }

  const value = useMemo<AuthState>(
    () => ({ token, email, ready, signedIn: !!token, loginPassword, googleSignIn, logout }),
    [token, email, ready],
  );

  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}
