import type { Message } from "../lib/chat";
import { QUICK_REACTIONS } from "../lib/chat";
import { t } from "../lib/i18n";
import { Icon } from "./icons";
import { useModalA11y } from "./useModalA11y";

// A bottom action sheet for a single message, opened by a long-press (touch) or right-click. Gives phones the
// message actions the hover-only bar cannot: quick reactions plus reply / translate / edit / delete. Reuses
// the shared modal a11y (focus trap, Escape, focus restore); each action runs then closes the sheet.
export function MessageActionSheet({
  m,
  me,
  onReact,
  onReply,
  onTranslate,
  onEdit,
  onDelete,
  onClose,
}: {
  m: Message;
  me: string;
  onReact: (emoji: string) => void;
  onReply: () => void;
  onTranslate: () => void;
  onEdit: () => void;
  onDelete: () => void;
  onClose: () => void;
}) {
  const boxRef = useModalA11y<HTMLDivElement>(onClose);
  const mine = m.sender_subject_id === me;
  const run = (fn: () => void) => () => {
    fn();
    onClose();
  };
  return (
    <div className="sheet-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="sheet" ref={boxRef} role="dialog" aria-modal="true" aria-label={t("sheet.title")} tabIndex={-1}>
        <div className="sheet-react">
          {QUICK_REACTIONS.map((emoji) => (
            <button key={emoji} className="sheet-emoji" onClick={run(() => onReact(emoji))} type="button">
              {emoji}
            </button>
          ))}
        </div>
        <button className="sheet-item" onClick={run(onReply)} type="button">
          <Icon name="thread" size={18} />
          {t("message.replyInThread")}
        </button>
        {m.body && (
          <button className="sheet-item" onClick={run(onTranslate)} type="button">
            <Icon name="translate" size={18} />
            {t("message.translate")}
          </button>
        )}
        {mine && m.body && (
          <button className="sheet-item" onClick={run(onEdit)} type="button">
            <Icon name="edit" size={18} />
            {t("message.edit")}
          </button>
        )}
        {mine && (
          <button className="sheet-item danger" onClick={run(onDelete)} type="button">
            <Icon name="trash" size={18} />
            {t("message.delete")}
          </button>
        )}
        <button className="sheet-item cancel" onClick={onClose} type="button">
          {t("common.cancel")}
        </button>
      </div>
    </div>
  );
}
