import { useState } from "react";
import type { FormEvent } from "react";
import { useAuth } from "../lib/auth";

export function Login() {
  const { loginPassword, googleSignIn } = useAuth();
  const [tenant, setTenant] = useState("cyberskill");
  const [handle, setHandle] = useState("@stephen");
  const [password, setPassword] = useState("");
  const [showPw, setShowPw] = useState(false);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [gerr, setGerr] = useState("");

  async function onPassword(e: FormEvent) {
    e.preventDefault();
    setErr("");
    setBusy(true);
    try {
      await loginPassword(tenant.trim(), handle.trim(), password);
    } catch (x) {
      setErr(x instanceof Error ? x.message : String(x));
    } finally {
      setBusy(false);
    }
  }

  async function onGoogle() {
    setGerr("");
    try {
      await googleSignIn(tenant.trim() || "cyberskill");
    } catch (x) {
      setGerr(x instanceof Error ? x.message : String(x));
    }
  }

  return (
    <div className="app">
      <header className="topbar">
        <span className="wordmark">
          <span className="cyber">Cyber</span>
          <span className="os">OS</span>
        </span>
        <span className="slogan">Turn Your Will Into Real</span>
      </header>
      <div className="center">
        <div className="card">
          <h1>Sign in</h1>
          <div className="sub">to your CyberOS workspace</div>

          <button className="btn-google" onClick={onGoogle} type="button">
            <span className="g">G</span> Sign in with Google
          </button>
          <div className="err">{gerr}</div>

          {!showPw && (
            <button className="linkish" type="button" onClick={() => setShowPw(true)}>
              Admin sign-in
            </button>
          )}

          {showPw && (
            <form onSubmit={onPassword}>
              <div className="field">
                <label>Workspace (tenant)</label>
                <input value={tenant} onChange={(e) => setTenant(e.target.value)} spellCheck={false} />
              </div>
              <div className="field">
                <label>Handle</label>
                <input value={handle} onChange={(e) => setHandle(e.target.value)} spellCheck={false} />
              </div>
              <div className="field">
                <label>Password</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  autoComplete="current-password"
                />
              </div>
              <button className="btn-primary" disabled={busy} type="submit">
                {busy ? "Signing in..." : "Sign in"}
              </button>
              <div className="err">{err}</div>
            </form>
          )}
        </div>
      </div>
    </div>
  );
}
