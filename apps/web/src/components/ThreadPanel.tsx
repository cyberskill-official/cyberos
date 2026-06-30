import { useEffect, useRef, useState } from "react";
import type { Directory, Message } from "../lib/chat";
import { nameFor, timeOf } from "../lib/chat";
import { Attachment } from "./Attachment";

// Controlled thread panel: the parent owns `replies` (loads them and folds in live websocket replies), so
// this component just renders the root + replies and posts new ones through `onSend`.
export function ThreadPanel({
  token,
  me,
  dir,
  root,
  replies,
  onClose,
  onSend,
}: {
  token: string;
  me: string;
  dir: Directory;
  root: Message;
  replies: Message[];
  onClose(): void;
  onSend(text: string): Promise<void>;
}) {
  const [draft, setDraft] = useState("");
  const [busy, setBusy] = useState(false);
  const endRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    endRef.current?.scrollIntoView();
  }, [replies]);

  async function send() {
    const text = draft.trim();
    if (!text) return;
    setBusy(true);
    try {
      await onSend(text);
      setDraft("");
    } finally {
      setBusy(false);
    }
  }

  const render = (m: Message) =>
    m.attachment_id ? <Attachment token={token} id={m.attachment_id} /> : m.body;

  return (
    <aside className="thread">
      <div className="thread-head">
        <span>Thread</span>
        <button className="btn-mini" onClick={onClose} type="button" title="Close thread">
          ×
        </button>
      </div>
      <div className="thread-body">
        <div className="msg">
          <div className="meta">
            <span className="author">{nameFor(dir, me, root.sender_subject_id)}</span> {timeOf(root.created_at)}
          </div>
          <div className="bubble">{render(root)}</div>
        </div>
        <div className="thread-sep">
          {replies.length} repl{replies.length === 1 ? "y" : "ies"}
        </div>
        {replies.map((m) => (
          <div key={m.id} className={"msg" + (m.sender_subject_id === me ? " mine" : "")}>
            <div className="meta">
              <span className="author">{nameFor(dir, me, m.sender_subject_id)}</span> {timeOf(m.created_at)}
            </div>
            <div className="bubble">{render(m)}</div>
          </div>
        ))}
        <div ref={endRef} />
      </div>
      <div className="composer">
        <input
          value={draft}
          onChange={(e) => setDraft(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void send();
            }
          }}
          placeholder="Reply..."
        />
        <button onClick={() => void send()} disabled={busy || !draft.trim()} type="button">
          Reply
        </button>
      </div>
    </aside>
  );
}
