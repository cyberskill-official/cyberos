import { useState } from "react";
import { useAuth } from "./lib/auth";
import { useTheme } from "./lib/theme";
import { currentLang, setLang, t } from "./lib/i18n";
import { Icon } from "./components/icons";
import { Login } from "./pages/Login";
import { Dashboard } from "./pages/Dashboard";
import { Chat } from "./pages/Chat";

type View = "dashboard" | "chat";

export function App() {
  const { ready, signedIn, email, logout } = useAuth();
  const [theme, toggleTheme] = useTheme();
  // Team default: land straight in chat. "All modules" reveals the operator dashboard.
  const [view, setView] = useState<View>("chat");

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

  return (
    <div className="app">
      <header className="topbar">
        <span className="wordmark">
          <span className="cyber">Cyber</span>
          <span className="os">OS</span>
        </span>
        <span className="slogan">{t("brand.slogan")}</span>
        {view === "chat" ? (
          <button className="btn-ghost" onClick={() => setView("dashboard")}>
            {t("top.allModules")}
          </button>
        ) : (
          <button className="btn-ghost" onClick={() => setView("chat")}>
            {t("top.backToChat")}
          </button>
        )}
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
      {view === "dashboard" ? <Dashboard onOpenChat={() => setView("chat")} /> : <Chat />}
    </div>
  );
}
