import { useEffect, useRef, useState } from "react";
import type { Message } from "../lib/chat";
import { timeOf } from "../lib/chat";
import { Attachment } from "./Attachment";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";

// Controlled thread panel: the parent owns `replies` (loads them and folds in live websocket replies), so
// this component just renders the root + replies and posts new ones through `onSend`.
export function ThreadPanel({
  token,
  nameOf,
  root,
  replies,
  onClose,
  onSend,
}: {
  token: string;
  nameOf: (id: string) => string;
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

  const bubble = (m: Message) => (
    <div className="t-msg" key={m.id}>
      <Avatar id={m.sender_subject_id} name={nameOf(m.sender_subject_id)} size={30} />
      <div className="t-body">
        <div className="t-head">
          <span className="t-name">{nameOf(m.sender_subject_id)}</span>
          <span className="t-time">{timeOf(m.created_at)}</span>
        </div>
        <div className="m-body">{m.attachment_id ? <Attachment token={token} id={m.attachment_id} /> : m.body}</div>
      </div>
    </div>
  );

  return (
    <aside className="thread">
      <div className="thread-head">
        <span>Thread</span>
        <button className="icon-btn" onClick={onClose} type="button" title="Close thread">
          <Icon name="close" size={16} />
        </button>
      </div>
      <div className="thread-body">
        {bubble(root)}
        <div className="thread-sep">
          {replies.length} repl{replies.length === 1 ? "y" : "ies"}
        </div>
        {replies.map(bubble)}
        <div ref={endRef} />
      </div>
      <div className="composer thread-composer">
        <textarea
          rows={1}
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
        <button className="comp-send" onClick={() => void send()} disabled={busy || !draft.trim()} title="Reply" type="button">
          <Icon name="send" />
        </button>
      </div>
    </aside>
  );
}
