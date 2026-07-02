import { useState } from "react";
import { useAuth } from "./lib/auth";
import { useTheme } from "./lib/theme";
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
            <div className="sub">Loading...</div>
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
        <span className="slogan">Turn Your Will Into Real</span>
        {view === "chat" ? (
          <button className="btn-ghost" onClick={() => setView("dashboard")}>
            All modules
          </button>
        ) : (
          <button className="btn-ghost" onClick={() => setView("chat")}>
            Back to chat
          </button>
        )}
        <span className="spacer" />
        <button
          className="icon-btn"
          title={theme === "dark" ? "Switch to light theme" : "Switch to dark theme"}
          onClick={toggleTheme}
          type="button"
        >
          <Icon name={theme === "dark" ? "sun" : "moon"} size={17} />
        </button>
        <span className="who">{email}</span>
        <button className="btn-ghost" onClick={logout}>
          Sign out
        </button>
      </header>
      {view === "dashboard" ? <Dashboard onOpenChat={() => setView("chat")} /> : <Chat />}
    </div>
  );
}
