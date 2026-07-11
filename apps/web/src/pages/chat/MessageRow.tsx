import { Fragment, useRef } from "react";
import type { Message } from "../../lib/chat";
import { formatDay, QUICK_REACTIONS, REACTION_EMOJIS, timeOf } from "../../lib/chat";
import { t } from "../../lib/i18n";
import type { MentionCandidate } from "../../lib/richtext";
import { RichText } from "../../lib/richtext-view";
import { Avatar } from "../../components/Avatar";
import { Icon } from "../../components/icons";
import { Attachment } from "../../components/Attachment";
import { BlockedMessage } from "../../components/BlockedMessage";

// One timeline entry: an optional day separator, the message row (gutter avatar/time, header, body or inline
// editor, reactions strip, inline translation), the hover action bar (react / translate / thread / edit /
// delete), and - when this is my most recent message - the "seen by" row. Every className and branch is copied
// from Chat.tsx verbatim; all state and handlers are passed in, so behavior is unchanged.
export function MessageRow({
  m,
  showDay,
  grouped,
  mine,
  highlighted,
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
  mentionNames,
  nameOf,
  avatarSrc,
  onOpenImage,
  onEditTextChange,
  onSaveEdit,
  onCancelEdit,
  onToggleReaction,
  onSetReactPicker,
  onOpenFullEmoji,
  onTranslate,
  onOpenThread,
  onStartEdit,
  onDelete,
  onRetry,
  onLongPress,
  onReport,
}: {
  m: Message;
  showDay: boolean;
  grouped: boolean;
  mine: boolean;
  highlighted: boolean;
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
  mentionNames: MentionCandidate[];
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  onOpenImage: (url: string, name: string) => void;
  onEditTextChange: (v: string) => void;
  onSaveEdit: (m: Message) => void;
  onCancelEdit: () => void;
  onToggleReaction: (m: Message, emoji: string) => void;
  onSetReactPicker: (updater: (id: string) => string) => void;
  onOpenFullEmoji: (m: Message, rect: { top: number; left: number; bottom: number; right: number }) => void;
  onTranslate: (m: Message) => void;
  onOpenThread: (m: Message) => void;
  onStartEdit: (m: Message) => void;
  onDelete: (m: Message) => void;
  onRetry?: (m: Message) => void;
  /// Long-press (touch) or right-click opens the mobile action sheet for this message.
  onLongPress?: (m: Message) => void;
  /// FR-CHAT-267 — open the report dialog for this message. Optional so a surface that has no reporting
  /// path (the thread panel's preview row) can omit it rather than pass a no-op.
  onReport?: (m: Message) => void;
}) {
  // Long-press detection for touch: a ~500ms hold with little movement opens the action sheet.
  const lpTimer = useRef<number | null>(null);
  const lpStart = useRef<{ x: number; y: number } | null>(null);
  const clearLp = () => {
    if (lpTimer.current) {
      window.clearTimeout(lpTimer.current);
      lpTimer.current = null;
    }
  };
  const canSheet = !!onLongPress && editingId !== m.id && !m.pending && !m.failed;
  return (
    <Fragment>
      {showDay && (
        <div className="day-sep">
          <span>{formatDay(m.created_at)}</span>
        </div>
      )}
      <div
        id={"m-" + m.id}
        className={
          "m-row" +
          (grouped ? " grouped" : "") +
          (mine ? " mine" : "") +
          (highlighted ? " flash" : "") +
          (m.pending ? " pending" : "") +
          (m.failed ? " failed" : "")
        }
        onTouchStart={
          canSheet
            ? (e) => {
                const tch = e.touches[0];
                lpStart.current = { x: tch.clientX, y: tch.clientY };
                clearLp();
                lpTimer.current = window.setTimeout(() => {
                  lpTimer.current = null;
                  onLongPress?.(m);
                }, 500);
              }
            : undefined
        }
        onTouchMove={
          canSheet
            ? (e) => {
                const s = lpStart.current;
                if (!s) return;
                const tch = e.touches[0];
                if (Math.abs(tch.clientX - s.x) > 10 || Math.abs(tch.clientY - s.y) > 10) clearLp();
              }
            : undefined
        }
        onTouchEnd={canSheet ? clearLp : undefined}
        onTouchCancel={canSheet ? clearLp : undefined}
        onContextMenu={
          canSheet
            ? (e) => {
                e.preventDefault();
                onLongPress?.(m);
              }
            : undefined
        }
      >
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
              {m.edited_at && <span className="m-edited">{t("message.edited")}</span>}
            </div>
          )}
          {editingId === m.id ? (
            <div className="edit-row">
              <textarea
                className="edit-input"
                value={editText}
                rows={1}
                ref={(el) => {
                  if (!el) return;
                  // Grow to fit the content (multi-line edits no longer collapse), capped so it never runs away.
                  el.style.height = "auto";
                  el.style.height = Math.min(el.scrollHeight, 220) + "px";
                  // Put the caret at the end on first mount (once), so editing continues where the text ends.
                  if (!el.dataset.caretSet) {
                    el.dataset.caretSet = "1";
                    const len = el.value.length;
                    try {
                      el.setSelectionRange(len, len);
                    } catch {
                      /* ignore */
                    }
                  }
                }}
                onChange={(e) => onEditTextChange(e.target.value)}
                onKeyDown={(e) => {
                  // Enter saves; Shift+Enter inserts a newline; Escape cancels.
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    void onSaveEdit(m);
                  } else if (e.key === "Escape") {
                    e.preventDefault();
                    onCancelEdit();
                  }
                }}
                autoFocus
              />
              <div className="edit-actions">
                <button className="btn-pill" onClick={() => void onSaveEdit(m)} type="button">
                  {t("common.save")}
                </button>
                <button className="btn-ghost" onClick={onCancelEdit} type="button">
                  {t("common.cancel")}
                </button>
                <span className="edit-hint">{t("message.editHint")}</span>
              </div>
            </div>
          ) : m.blocked_sender ? (
            /* FR-CHAT-268 §1 #5 — the sender is blocked and this is a group channel. The server already
               withheld the body, the attachments and the reactions; there is nothing here to leak. The row
               keeps its id and its position so the conversation around it still reads. */
            <div className="m-body">
              <BlockedMessage name={nameOf(m.sender_subject_id)} />
            </div>
          ) : (
            <div className="m-body">
              {/* Body and attachments both render: files sent with a caption show the caption too. */}
              {m.body && <RichText text={m.body} mentions={mentionNames} />}
              {m.attachments && m.attachments.length > 0 ? (
                <div className="att-group">
                  {m.attachments.map((a) => (
                    <Attachment key={a.id} token={token!} id={a.id} meta={a} onOpenImage={onOpenImage} />
                  ))}
                </div>
              ) : m.attachment_id ? (
                <Attachment token={token!} id={m.attachment_id} onOpenImage={onOpenImage} />
              ) : null}
            </div>
          )}
          {m.pending && <div className="m-status">{t("message.sending")}</div>}
          {m.failed && (
            <div className="m-status failed">
              <span>{t("message.failed")}</span>
              <button className="linkish" onClick={() => onRetry?.(m)} type="button">
                {t("message.retry")}
              </button>
            </div>
          )}
          {m.reactions && m.reactions.length > 0 && (
            <div className="reactions">
              {m.reactions.map((r) => (
                <button
                  key={r.emoji}
                  className={"reaction" + (r.mine ? " mine" : "")}
                  onClick={() => void onToggleReaction(m, r.emoji)}
                  title={r.mine ? t("message.removeReaction") : t("message.react")}
                  type="button"
                >
                  <span className="re-emoji">{r.emoji}</span>
                  <span className="re-count">{r.count}</span>
                </button>
              ))}
            </div>
          )}
          {!!m.reply_count && m.reply_count > 0 && (
            <button className="reply-chip" onClick={() => void onOpenThread(m)} type="button">
              <Icon name="thread" size={13} />
              <span>{t(m.reply_count === 1 ? "message.replyOne" : "message.replyMany", { n: m.reply_count })}</span>
            </button>
          )}
          {translating.has(m.id) && <div className="translation muted">{t("message.translating")}</div>}
          {!translating.has(m.id) && translations[m.id] !== undefined && (
            <div className="translation">
              <span className="tr-label">{t("message.translationLabel")}</span>
              <span className="tr-text">{translations[m.id]}</span>
            </div>
          )}
          {!translating.has(m.id) && translateError.has(m.id) && (
            <div className="translation muted">{t("message.translateUnavailable")}</div>
          )}
        </div>
        {/* No action bar on a collapsed row: there is no content to react to, translate, thread or
            report — the server never sent it. Reporting a blocked person still works from the member
            list, which is where you have the person rather than the message. */}
        {editingId !== m.id && !m.pending && !m.failed && !m.blocked_sender && (
          <div className="m-actions">
            {QUICK_REACTIONS.map((e) => (
              <button
                key={e}
                className="quick-react"
                title={t("message.react")}
                onClick={() => void onToggleReaction(m, e)}
                type="button"
              >
                {e}
              </button>
            ))}
            <div className="react-wrap">
              <button
                title={t("message.addReaction")}
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
                  <button
                    className="emoji-opt more"
                    title={t("message.allEmoji")}
                    onClick={(e) => {
                      const r = e.currentTarget.getBoundingClientRect();
                      onOpenFullEmoji(m, { top: r.top, left: r.left, bottom: r.bottom, right: r.right });
                    }}
                    type="button"
                  >
                    +
                  </button>
                </div>
              )}
            </div>
            {m.body && (
              <button title={t("message.translate")} onClick={() => void onTranslate(m)} type="button">
                <Icon name="translate" size={15} />
              </button>
            )}
            <button title={t("message.replyInThread")} onClick={() => void onOpenThread(m)} type="button">
              <Icon name="thread" size={15} />
            </button>
            {/* Any of my messages with text can be edited - including one that carries attachments. */}
            {mine && m.body && (
              <button title={t("message.edit")} onClick={() => onStartEdit(m)} type="button">
                <Icon name="edit" size={15} />
              </button>
            )}
            {mine && (
              <button title={t("message.delete")} onClick={() => void onDelete(m)} type="button">
                <Icon name="trash" size={15} />
              </button>
            )}
            {/* FR-CHAT-267 §1 #10 — the message entry point. Shown only on other people's messages: the
                server would accept a report of your own message, but offering it is noise, and the one thing
                you actually want for your own message (delete) is already right there. Reporting a PERSON
                lives in the member list, which is the other entry point. */}
            {!mine && onReport && (
              <button title={t("report.action")} onClick={() => onReport(m)} type="button">
                <Icon name="flag" size={15} />
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
