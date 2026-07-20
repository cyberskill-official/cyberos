import { createContext, useContext, useEffect, useMemo, useState } from "react";
import type { ReactNode } from "react";
import { apiUrl, decodeJwt, isNativeShell, tokenValid } from "./api";

// OIDC hand-back target on native. The web build returns to its own URL, but a Capacitor build cannot: its
// origin is capacitor://localhost (iOS) or http://localhost (Android), and the Google consent screen runs in
// SAFARI, not in the app's webview. Safari cannot redirect into another app's private scheme, so the
// hand-back has to go to a scheme iOS/Android will route back to us - one declared in CFBundleURLTypes
// (iOS) and an intent-filter (Android).
//
// The scheme is the bundle id rather than something short like "cyberos://". Any app may claim any custom
// scheme, and on iOS the winner of a collision is undefined; a reverse-DNS id we already own cannot be
// claimed out from under us. This is also the convention Apple documents for OAuth redirects.
//
// Must stay in lockstep with three other places, or Google sign-in breaks on native:
//   apps/web/ios/App/App/Info.plist            CFBundleURLTypes
//   apps/web/android/.../AndroidManifest.xml   intent-filter data android:scheme
//   deploy/vps/docker-compose.p0*.yml          AUTH_OIDC_RETURN_ALLOW (server-side allow-list)
const NATIVE_RETURN_TO = "os.cyberskill.world://auth";

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
      const res = await fetch(apiUrl("/v1/auth/token"), {
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

  // Adopt an OIDC hand-back fragment (#access_token=...&refresh_token=...). Shared by two callers that
  // receive the same payload through different doors: the web/desktop flow, where the browser navigates
  // back to us and it arrives in location.hash, and the native flow, where the OS hands the whole URL to
  // the appUrlOpen listener below and location never changes at all.
  function adoptFromHash(hash: string): boolean {
    if (!hash || hash.indexOf("access_token") === -1) return false;
    const p = new URLSearchParams(hash.replace(/^#/, ""));
    const at = p.get("access_token");
    if (!at) return false;
    adopt(at, p.get("refresh_token"));
    return true;
  }

  // Boot: capture an OIDC hand-back fragment (#access_token=...), else a valid stored token, else refresh.
  useEffect(() => {
    let alive = true;
    (async () => {
      if (adoptFromHash(location.hash)) {
        try {
          history.replaceState(null, "", location.pathname + location.search);
        } catch {
          location.hash = "";
        }
        if (alive) setReady(true);
        return;
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

  // Native only: the OIDC hand-back arrives as an app-open URL, not as a navigation. Safari finishes the
  // Google flow, and iOS/Android route os.cyberskill.world://auth#access_token=... back into the app, which
  // surfaces it here. location never changes, so the boot effect above would never see it - which is the
  // whole reason Google sign-in dead-ended on mobile.
  //
  // @capacitor/app is imported dynamically and only on native, so it never enters the web bundle - the same
  // rule api.ts follows for @capacitor/core. A failed import is swallowed: password sign-in is the path that
  // still works when any of this is misconfigured, and it must not be taken down by an absent plugin.
  useEffect(() => {
    if (!isNativeShell()) return;
    let alive = true;
    let remove: (() => void) | undefined;
    void (async () => {
      try {
        const { App } = await import("@capacitor/app");
        const handle = await App.addListener("appUrlOpen", ({ url }) => {
          const i = url.indexOf("#");
          if (i !== -1) adoptFromHash(url.slice(i));
        });
        if (alive) remove = () => void handle.remove();
        else void handle.remove();
      } catch {
        /* plugin absent or bridge unavailable - password sign-in is unaffected */
      }
    })();
    return () => {
      alive = false;
      remove?.();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  async function loginPassword(tenant: string, handle: string, password: string) {
    const res = await fetch(apiUrl("/v1/auth/token"), {
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
    // Where Google hands the user back to. On web and desktop that is this page, and the boot effect picks
    // the token out of location.hash. On native it is the registered custom scheme, and the appUrlOpen
    // listener picks it up instead - location.origin there is capacitor://localhost, which Safari cannot
    // redirect to and which the server rejects as return_to_not_allowed.
    const ret = isNativeShell() ? NATIVE_RETURN_TO : location.origin + location.pathname;
    const res = await fetch(
      apiUrl(
        `/v1/auth/oidc/initiate?tenant_slug=${encodeURIComponent(tenant)}&idp=google&return_to=${encodeURIComponent(ret)}`,
      ),
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
