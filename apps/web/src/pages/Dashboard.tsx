import { t } from "../lib/i18n";

interface Mod {
  id: string;
  icon: string;
  name: string;
  desc: string;
  live: boolean;
}

// Chat is the live tile for the team rollout. The operator modules are defined but marked not-yet so the
// dashboard already shows where CyberOS is going without exposing half-wired surfaces. The language is fixed
// per page load, so resolving t() once at module init is safe.
const MODULES: Mod[] = [
  { id: "chat", icon: "💬", name: t("dash.mod.chat.name"), desc: t("dash.mod.chat.desc"), live: true },
  { id: "assistant", icon: "✨", name: t("dash.mod.assistant.name"), desc: t("dash.mod.assistant.desc"), live: false },
  { id: "ai", icon: "🛠", name: t("dash.mod.ai.name"), desc: t("dash.mod.ai.desc"), live: false },
  { id: "mcp", icon: "🔌", name: t("dash.mod.mcp.name"), desc: t("dash.mod.mcp.desc"), live: false },
  { id: "memory", icon: "🧾", name: t("dash.mod.memory.name"), desc: t("dash.mod.memory.desc"), live: false },
  { id: "cuo", icon: "🧠", name: t("dash.mod.cuo.name"), desc: t("dash.mod.cuo.desc"), live: false },
];

export function Dashboard({ onOpenChat }: { onOpenChat(): void }) {
  return (
    <main className="dash">
      <h2>{t("dash.title")}</h2>
      <div className="hint">{t("dash.hint")}</div>
      <div className="tiles">
        {MODULES.map((m) => (
          <button
            key={m.id}
            className={"tile" + (m.live ? "" : " soon")}
            onClick={m.id === "chat" ? onOpenChat : undefined}
            disabled={!m.live}
            type="button"
          >
            <div className="icon">{m.icon}</div>
            <div className="name">{m.name}</div>
            <div className="desc">{m.desc}</div>
            <div className={"badge" + (m.live ? " live" : "")}>{m.live ? t("common.open") : t("dash.soon")}</div>
          </button>
        ))}
      </div>
    </main>
  );
}
