import { Fragment } from "react";
import type { Message } from "../../lib/chat";
import { formatDay, REACTION_EMOJIS, timeOf } from "../../lib/chat";
import { Avatar } from "../../components/Avatar";
import { Icon } from "../../components/icons";
import { Attachment } from "../../components/Attachment";

// One timeline entry: an optional day separator, the message row (gutter avatar/time, header, body or inline
// editor, reactions strip, inline translation), the hover action bar (react / translate / thread / edit /
// delete), and - when this is my most recent message - the "seen by" row. Every className and branch is copied
// from Chat.tsx verbatim; all state and handlers are passed in, so behavior is unchanged.
export function MessageRow({
  m,
  showDay,
  grouped,
  mine,
  token,
  editingId,
  editText,
  reactPickerId,
  translating,
  translations,
  translateError,
  isMyLast,
  seenBy,
  seenLabel,
  nameOf,
  avatarSrc,
  onEditTextChange,
  onSaveEdit,
  onCancelEdit,
  onToggleReaction,
  onSetReactPicker,
  onTranslate,
  onOpenThread,
  onStartEdit,
  onDelete,
}: {
  m: Message;
  showDay: boolean;
  grouped: boolean;
  mine: boolean;
  token: string | null;
  editingId: string;
  editText: string;
  reactPickerId: string;
  translating: Set<string>;
  translations: Record<string, string>;
  translateError: Set<string>;
  isMyLast: boolean;
  seenBy: string[];
  seenLabel: string;
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  onEditTextChange: (v: string) => void;
  onSaveEdit: (m: Message) => void;
  onCancelEdit: () => void;
  onToggleReaction: (m: Message, emoji: string) => void;
  onSetReactPicker: (updater: (id: string) => string) => void;
  onTranslate: (m: Message) => void;
  onOpenThread: (m: Message) => void;
  onStartEdit: (m: Message) => void;
  onDelete: (m: Message) => void;
}) {
  return (
    <Fragment>
      {showDay && (
        <div className="day-sep">
          <span>{formatDay(m.created_at)}</span>
        </div>
      )}
      <div className={"m-row" + (grouped ? " grouped" : "") + (mine ? " mine" : "")}>
        <div className="m-gutter">
          {grouped ? (
            <span className="m-time-hover">{timeOf(m.created_at)}</span>
          ) : (
            <Avatar
              id={m.sender_subject_id}
              name={nameOf(m.sender_subject_id)}
              size={36}
              src={avatarSrc(m.sender_subject_id)}
            />
          )}
        </div>
        <div className="m-content">
          {!grouped && (
            <div className="m-head">
              <span className="m-name">{nameOf(m.sender_subject_id)}</span>
              <span className="m-time">{timeOf(m.created_at)}</span>
              {m.edited_at && <span className="m-edited">edited</span>}
            </div>
          )}
          {editingId === m.id ? (
            <div className="edit-row">
              <input
                value={editText}
                onChange={(e) => onEditTextChange(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter") {
                    e.preventDefault();
                    void onSaveEdit(m);
                  } else if (e.key === "Escape") {
                    onCancelEdit();
                  }
                }}
                autoFocus
              />
              <button className="btn-pill" onClick={() => void onSaveEdit(m)} type="button">
                Save
              </button>
              <button className="btn-ghost" onClick={onCancelEdit} type="button">
                Cancel
              </button>
            </div>
          ) : (
            <div className="m-body">
              {m.attachment_id ? <Attachment token={token!} id={m.attachment_id} /> : m.body}
            </div>
          )}
          {m.reactions && m.reactions.length > 0 && (
            <div className="reactions">
              {m.reactions.map((r) => (
                <button
                  key={r.emoji}
                  className={"reaction" + (r.mine ? " mine" : "")}
                  onClick={() => void onToggleReaction(m, r.emoji)}
                  title={r.mine ? "Remove your reaction" : "React"}
                  type="button"
                >
                  <span className="re-emoji">{r.emoji}</span>
                  <span className="re-count">{r.count}</span>
                </button>
              ))}
            </div>
          )}
          {translating.has(m.id) && <div className="translation muted">Translating...</div>}
          {!translating.has(m.id) && translations[m.id] !== undefined && (
            <div className="translation">
              <span className="tr-label">English</span>
              <span className="tr-text">{translations[m.id]}</span>
            </div>
          )}
          {!translating.has(m.id) && translateError.has(m.id) && (
            <div className="translation muted">Translation unavailable</div>
          )}
        </div>
        {editingId !== m.id && (
          <div className="m-actions">
            <div className="react-wrap">
              <button
                title="Add reaction"
                onClick={() => onSetReactPicker((id) => (id === m.id ? "" : m.id))}
                type="button"
              >
                <Icon name="smile" size={15} />
              </button>
              {reactPickerId === m.id && (
                <div className="emoji-picker">
                  {REACTION_EMOJIS.map((e) => (
                    <button
                      key={e}
                      className="emoji-opt"
                      onClick={() => void onToggleReaction(m, e)}
                      type="button"
                    >
                      {e}
                    </button>
                  ))}
                </div>
              )}
            </div>
            {!m.attachment_id && m.body && (
              <button title="Translate to English" onClick={() => void onTranslate(m)} type="button">
                <Icon name="translate" size={15} />
              </button>
            )}
            <button title="Reply in thread" onClick={() => void onOpenThread(m)} type="button">
              <Icon name="thread" size={15} />
            </button>
            {mine && !m.attachment_id && (
              <button title="Edit" onClick={() => onStartEdit(m)} type="button">
                <Icon name="edit" size={15} />
              </button>
            )}
            {mine && (
              <button title="Delete" onClick={() => void onDelete(m)} type="button">
                <Icon name="trash" size={15} />
              </button>
            )}
          </div>
        )}
      </div>
      {isMyLast && seenBy.length > 0 && (
        <div className="seen-row">
          <Icon name="check" size={12} />
          <span>{seenLabel}</span>
        </div>
      )}
    </Fragment>
  );
}
