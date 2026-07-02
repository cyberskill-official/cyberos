import { useEffect, useState } from "react";
import { apiFetch, ApiError } from "../lib/api";
import type { Channel } from "../lib/chat";
import { t } from "../lib/i18n";
import { RichText } from "../lib/richtext-view";
import { Icon } from "./icons";

// The AI assistant side panel (AI cluster): "Catch me up" (a bullet summary of the recent conversation) and
// "Action items", both computed server-side by the chat service against the ai-gateway. The transcript never
// reaches the browser's network - the client only sends a subject->name map so the prompt can label
// speakers. When the gateway is not deployed the endpoints 502 and this panel degrades to a quiet note,
// exactly like inline translation.
export function AiPanel({
  token,
  channel,
  names,
  onClose,
}: {
  token: string;
  channel: Channel;
  names: Record<string, string>;
  onClose(): void;
}) {
  const [mode, setMode] = useState<"summary" | "actions">("summary");
  const [busy, setBusy] = useState(false);
  const [text, setText] = useState("");
  const [count, setCount] = useState(0);
  const [note, setNote] = useState("");

  async function run(m: "summary" | "actions") {
    setMode(m);
    setBusy(true);
    setNote("");
    setText("");
    try {
      const r = await apiFetch<{ text: string; message_count: number }>(
        token,
        "POST",
        `/v1/chat/channels/${channel.id}/ai/${m === "summary" ? "summarize" : "actions"}`,
        { names },
      );
      setText(r.text);
      setCount(r.message_count);
    } catch (e) {
      if (e instanceof ApiError && e.status === 502) {
        setNote(t("ai.unavailable"));
      } else {
        setNote(e instanceof Error ? e.message : String(e));
      }
    } finally {
      setBusy(false);
    }
  }

  // Fetch the summary on open - "catch me up" is the reason this panel exists.
  useEffect(() => {
    void run("summary");
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [channel.id]);

  return (
    <aside className="thread ai-panel">
      <div className="thread-head">
        <span className="ai-title">
          <Icon name="sparkle" size={16} /> {t("ai.title")}
        </span>
        <button className="icon-btn" onClick={onClose} type="button" title={t("common.close")}>
          <Icon name="close" size={16} />
        </button>
      </div>
      <div className="ai-tabs">
        <button
          className={"ai-tab" + (mode === "summary" ? " on" : "")}
          onClick={() => void run("summary")}
          disabled={busy}
          type="button"
        >
          {t("ai.catchMeUp")}
        </button>
        <button
          className={"ai-tab" + (mode === "actions" ? " on" : "")}
          onClick={() => void run("actions")}
          disabled={busy}
          type="button"
        >
          {t("ai.actionItems")}
        </button>
      </div>
      <div className="thread-body ai-body">
        {busy && <div className="ai-note">{t("ai.thinking")}</div>}
        {!busy && note && <div className="ai-note">{note}</div>}
        {!busy && !note && text && (
          <>
            <div className="m-body ai-text">
              <RichText text={text} />
            </div>
            <div className="ai-meta">{t("ai.meta", { n: count })}</div>
          </>
        )}
      </div>
      <div className="ai-foot">
        <button className="btn-ghost" onClick={() => void run(mode)} disabled={busy} type="button">
          {t("common.refresh")}
        </button>
      </div>
    </aside>
  );
}
