import { useState } from "react";
import type { RefObject } from "react";
import type { Channel, Directory, Person } from "../../lib/chat";
import { channelLabel, formatBytes } from "../../lib/chat";
import { Avatar } from "../../components/Avatar";
import { Icon } from "../../components/icons";

// The active @-token immediately before the caret: an "@" at a word boundary followed by up to 30 name chars.
const MENTION_RE = /(^|\s)@([\p{L}0-9._-]{0,30})$/u;

// The message composer: the staged-file preview strip (shown while a file is queued for the next send), the
// attach button, the growing textarea (Enter sends, Shift+Enter newline, paste stages a file) with an
// @-mention autocomplete popover, and the send button. Message + attachment state lives in Chat; picking a
// mention inserts "@Name " into the draft and reports the person up so Chat can send the resolved id.
export function Composer({
  active,
  directory,
  me,
  people,
  draft,
  staged,
  stagedPreview,
  uploading,
  sending,
  taRef,
  onDraftChange,
  onSend,
  onClearStaged,
  onOpenFilePicker,
  onOpenEmoji,
  onPaste,
  onMentionPicked,
}: {
  active: Channel;
  directory: Directory;
  me: string;
  people: Person[];
  draft: string;
  staged: File | null;
  stagedPreview: string;
  uploading: boolean;
  sending: boolean;
  taRef: RefObject<HTMLTextAreaElement>;
  onDraftChange: (v: string) => void;
  onSend: () => void;
  onClearStaged: () => void;
  onOpenFilePicker: () => void;
  onOpenEmoji: (rect: { top: number; left: number; bottom: number; right: number }) => void;
  onPaste: (e: React.ClipboardEvent<HTMLTextAreaElement>) => void;
  onMentionPicked: (p: Person) => void;
}) {
  const [mentionOpen, setMentionOpen] = useState(false);
  const [mentionQuery, setMentionQuery] = useState("");
  const [mentionIdx, setMentionIdx] = useState(0);

  const personLabel = (p: Person) => p.display_name || p.handle || (p.email || "").split("@")[0] || "user";

  const q = mentionQuery.toLowerCase();
  const candidates = mentionOpen
    ? people
        .filter((p) => p.subject_id !== me)
        .filter((p) => {
          if (!q) return true;
          const dn = (p.display_name || "").toLowerCase();
          const hn = (p.handle || "").toLowerCase();
          const em = (p.email || "").split("@")[0].toLowerCase();
          return dn.includes(q) || hn.includes(q) || em.includes(q);
        })
        .slice(0, 6)
    : [];

  // Open/refresh the popover from the text before the caret; close it when there is no active @-token.
  function refreshMention(value: string, caret: number) {
    const m = MENTION_RE.exec(value.slice(0, caret));
    if (m) {
      setMentionQuery(m[2]);
      setMentionOpen(true);
      setMentionIdx(0);
    } else {
      setMentionOpen(false);
      setMentionQuery("");
    }
  }

  // Replace the active @-token with "@Name " and report the person up so the send can resolve the id.
  function pick(p: Person) {
    const ta = taRef.current;
    const caret = ta ? ta.selectionStart ?? draft.length : draft.length;
    const rest = draft.slice(caret);
    const label = personLabel(p);
    const replaced = draft.slice(0, caret).replace(MENTION_RE, (_full, pre) => `${pre}@${label} `);
    onDraftChange(replaced + rest);
    onMentionPicked(p);
    setMentionOpen(false);
    setMentionQuery("");
    // Restore focus and place the caret just after the inserted mention.
    requestAnimationFrame(() => {
      const t = taRef.current;
      if (t) {
        t.focus();
        t.setSelectionRange(replaced.length, replaced.length);
      }
    });
  }

  return (
    <>
      {staged && (
        <div className="composer-attach">
          {stagedPreview ? (
            <img className="ca-thumb" src={stagedPreview} alt={staged.name} />
          ) : (
            <span className="ca-icon">
              <Icon name="paperclip" size={16} />
            </span>
          )}
          <span className="ca-meta">
            <span className="ca-name">{staged.name}</span>
            <span className="ca-size">{formatBytes(staged.size)}</span>
          </span>
          <button
            className="ca-x"
            title="Remove attachment"
            onClick={onClearStaged}
            disabled={uploading}
            type="button"
          >
            <Icon name="close" size={14} />
          </button>
        </div>
      )}

      <div className="composer">
        <button className="comp-btn" title="Attach a file" onClick={onOpenFilePicker} type="button">
          <Icon name="paperclip" />
        </button>
        <button
          className="comp-btn"
          title="Emoji"
          onClick={(e) => {
            const r = e.currentTarget.getBoundingClientRect();
            onOpenEmoji({ top: r.top, left: r.left, bottom: r.bottom, right: r.right });
          }}
          type="button"
        >
          <Icon name="smile" />
        </button>
        <div className="comp-field">
          {mentionOpen && candidates.length > 0 && (
            <div className="mention-pop">
              {candidates.map((p, i) => (
                <button
                  key={p.subject_id}
                  type="button"
                  className={"mention-item" + (i === mentionIdx ? " active" : "")}
                  // onMouseDown (not onClick) so the textarea does not blur before the insert runs.
                  onMouseDown={(e) => {
                    e.preventDefault();
                    pick(p);
                  }}
                >
                  <Avatar id={p.subject_id} name={personLabel(p)} size={22} src={p.avatar || ""} />
                  <span className="mention-name">{personLabel(p)}</span>
                  {p.handle && <span className="mention-handle">@{p.handle}</span>}
                </button>
              ))}
            </div>
          )}
          <textarea
            ref={taRef}
            rows={1}
            value={draft}
            onChange={(e) => {
              onDraftChange(e.target.value);
              refreshMention(e.target.value, e.target.selectionStart ?? e.target.value.length);
            }}
            onKeyUp={(e) => {
              const t = e.currentTarget;
              refreshMention(t.value, t.selectionStart ?? t.value.length);
            }}
            onClick={(e) => {
              const t = e.currentTarget;
              refreshMention(t.value, t.selectionStart ?? t.value.length);
            }}
            onKeyDown={(e) => {
              if (mentionOpen && candidates.length > 0) {
                if (e.key === "ArrowDown") {
                  e.preventDefault();
                  setMentionIdx((i) => (i + 1) % candidates.length);
                  return;
                }
                if (e.key === "ArrowUp") {
                  e.preventDefault();
                  setMentionIdx((i) => (i - 1 + candidates.length) % candidates.length);
                  return;
                }
                if (e.key === "Enter" || e.key === "Tab") {
                  e.preventDefault();
                  pick(candidates[Math.min(mentionIdx, candidates.length - 1)]);
                  return;
                }
                if (e.key === "Escape") {
                  e.preventDefault();
                  setMentionOpen(false);
                  return;
                }
              }
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                void onSend();
              }
            }}
            onPaste={onPaste}
            placeholder={
              staged ? "Add a message or just send the file" : "Message " + channelLabel(directory, me, active)
            }
          />
        </div>
        <button
          className="comp-send"
          onClick={() => void onSend()}
          disabled={sending || uploading || (!draft.trim() && !staged)}
          title="Send"
          type="button"
        >
          <Icon name={uploading ? "paperclip" : "send"} />
        </button>
      </div>
    </>
  );
}
