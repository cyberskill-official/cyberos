import { useState } from "react";
import type { FormEvent } from "react";
import { useAuth } from "../lib/auth";
import { currentLang, setLang, t } from "../lib/i18n";

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
        <img className="brandmark" src="/cyberskill-logo.svg" alt="CyberSkill" />
        <span className="wordmark">
          <span className="cyber">Cyber</span>
          <span className="os">OS</span>
        </span>
        <span className="slogan">{t("brand.slogan")}</span>
      </header>
      <div className="center">
        <div className="card">
          <h1>{t("login.signIn")}</h1>
          <div className="sub">{t("login.subtitle")}</div>

          <button className="btn-google" onClick={onGoogle} type="button">
            <span className="g">G</span> {t("login.google")}
          </button>
          <div className="err">{gerr}</div>

          {showPw && (
            <form onSubmit={onPassword}>
              <div className="field">
                <label>{t("login.workspace")}</label>
                <input value={tenant} onChange={(e) => setTenant(e.target.value)} spellCheck={false} />
              </div>
              <div className="field">
                <label>{t("login.handle")}</label>
                <input value={handle} onChange={(e) => setHandle(e.target.value)} spellCheck={false} />
              </div>
              <div className="field">
                <label>{t("login.password")}</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  autoComplete="current-password"
                />
              </div>
              <button className="btn-primary" disabled={busy} type="submit">
                {busy ? t("login.signingIn") : t("login.signIn")}
              </button>
              <div className="err">{err}</div>
            </form>
          )}

          {/* Both links share one flex row. They used to be two bare inline-block
              buttons rendered back to back: JSX strips the newline between sibling
              elements, so they butted together and read as "Admin sign-inTiengViet"
              on the first screen a store reviewer sees. */}
          <div className="link-row">
            {!showPw && (
              <button className="linkish" type="button" onClick={() => setShowPw(true)}>
                {t("login.adminSignIn")}
              </button>
            )}
            <button className="linkish" type="button" onClick={() => setLang(currentLang() === "vi" ? "en" : "vi")}>
              {t("top.language")}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
