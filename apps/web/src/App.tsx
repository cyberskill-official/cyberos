import { useEffect, useState } from "react";
import { useAuth } from "./lib/auth";
import { useTheme } from "./lib/theme";
import { currentLang, setLang, t } from "./lib/i18n";
import { Icon } from "./components/icons";
import { Login } from "./pages/Login";
import { Dashboard } from "./pages/Dashboard";
import { Chat } from "./pages/Chat";
import { Moderation } from "./pages/Moderation";
import { isModerator } from "./lib/roles";
import { VersionBadge } from "./components/VersionBadge";

type View = "dashboard" | "chat" | "moderation" | "module";

// Every module owns a URL (os.cyberskill.world/chat, /dashboard, /<module>). Caddy serves index.html for
// any client route (SPA fallback), and this tiny pathname router maps it to a view - no router dependency.
// Unknown module paths render a stub that links the module's manual on the docs site (/docs/modules/<m>/).
function parsePath(path: string): { view: View; module?: string } {
  if (path === "/" || path === "/chat" || path.startsWith("/chat/")) return { view: "chat" };
  if (path === "/dashboard") return { view: "dashboard" };
  // TASK-CHAT-269. The route exists for everyone; the PAGE renders null and the SERVER 403s for a
  // non-admin, so a guessed URL leaks nothing.
  if (path === "/moderation") return { view: "moderation" };
  const seg = path.split("/")[1]?.toLowerCase() ?? "";
  if (!seg) return { view: "chat" };
  return { view: "module", module: seg };
}

function ModuleStub({ module, onBack }: { module: string; onBack: () => void }) {
  return (
    <div className="center">
      <div className="card">
        <h2 style={{ marginTop: 0 }}>{module}</h2>
        <div className="sub" style={{ marginBottom: 12 }}>
          {t("module.stub")}
        </div>
        <a className="btn" href={`/docs/modules/${module}/index.html`}>
          {t("module.manual")}
        </a>
        <button className="btn-ghost" onClick={onBack} style={{ marginLeft: 8 }} type="button">
          {t("top.backToChat")}
        </button>
      </div>
    </div>
  );
}

export function App() {
  const { ready, signedIn, email, logout, token } = useAuth();
  const [theme, toggleTheme] = useTheme();
  // Team default: land straight in chat. "All modules" reveals the operator dashboard.
  const [route, setRoute] = useState(() => parsePath(window.location.pathname));

  const nav = (to: string) => {
    window.history.pushState({}, "", to);
    setRoute(parsePath(to));
  };
  useEffect(() => {
    const onPop = () => setRoute(parsePath(window.location.pathname));
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

  if (!ready) {
    return (
      <div className="app">
        <div className="center">
          <div className="card">
            <div className="sub">{t("common.loading")}</div>
          </div>
        </div>
      </div>
    );
  }

  if (!signedIn) return <Login />;

  const view = route.view;
  return (
    <div className="app">
      <header className="topbar">
        <img className="brandmark" src="/cyberskill-logo.svg" alt="CyberSkill" />
        <span className="wordmark">
          <span className="cyber">Cyber</span>
          <span className="os">OS</span>
        </span>
        <span className="slogan">{t("brand.slogan")}</span>
        {view === "chat" ? (
          <button className="btn-ghost" onClick={() => nav("/dashboard")}>
            {t("top.allModules")}
          </button>
        ) : (
          <button className="btn-ghost" onClick={() => nav("/chat")}>
            {t("top.backToChat")}
          </button>
        )}
        {/* TASK-CHAT-269 §1 #18 — absent, not disabled. A visible-but-403 route teaches everyone in the
            workspace that a moderation surface exists and that they are not trusted with it. */}
        {isModerator(token) && view !== "moderation" && (
          <button className="btn-ghost" onClick={() => nav("/moderation")}>
            {t("top.moderation")}
          </button>
        )}
        <a className="btn-ghost" href="/docs/">
          {t("top.docs")}
        </a>
        <VersionBadge />
        <span className="spacer" />
        <button
          className="btn-ghost lang-btn"
          title={t("top.language")}
          onClick={() => setLang(currentLang() === "vi" ? "en" : "vi")}
          type="button"
        >
          {currentLang() === "vi" ? "EN" : "VI"}
        </button>
        <button
          className="icon-btn"
          title={theme === "dark" ? t("top.themeToLight") : t("top.themeToDark")}
          onClick={toggleTheme}
          type="button"
        >
          <Icon name={theme === "dark" ? "sun" : "moon"} size={17} />
        </button>
        <span className="who">{email}</span>
        <button className="btn-ghost" onClick={logout}>
          {t("top.signOut")}
        </button>
      </header>
      {view === "dashboard" ? (
        <Dashboard onOpenChat={() => nav("/chat")} />
      ) : view === "moderation" ? (
        <Moderation onBack={() => nav("/chat")} />
      ) : view === "module" ? (
        <ModuleStub module={route.module ?? ""} onBack={() => nav("/chat")} />
      ) : (
        <Chat />
      )}
    </div>
  );
}
