interface Mod {
  id: string;
  icon: string;
  name: string;
  desc: string;
  live: boolean;
}

// Chat is the live tile for the team rollout. The operator modules are defined but marked not-yet so the
// dashboard already shows where CyberOS is going without exposing half-wired surfaces.
const MODULES: Mod[] = [
  { id: "chat", icon: "💬", name: "Chat", desc: "Channels, messages, and live presence.", live: true },
  { id: "assistant", icon: "✨", name: "Assistant", desc: "Talk to your model via the AI gateway.", live: false },
  { id: "ai", icon: "🛠", name: "AI Ops", desc: "Provider routing, spend caps, residency.", live: false },
  { id: "mcp", icon: "🔌", name: "MCP Registry", desc: "Tools the MCP gateway is serving.", live: false },
  { id: "memory", icon: "🧾", name: "Memory & Audit", desc: "The tenant's hash-chained audit log.", live: false },
  { id: "cuo", icon: "🧠", name: "Workflows & GENIE", desc: "Dream-loop envelope and FR backlog.", live: false },
];

export function Dashboard({ onOpenChat }: { onOpenChat(): void }) {
  return (
    <main className="dash">
      <h2>Workspace</h2>
      <div className="hint">Pick a module to begin.</div>
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
            <div className={"badge" + (m.live ? " live" : "")}>{m.live ? "Open" : "Soon"}</div>
          </button>
        ))}
      </div>
    </main>
  );
}
