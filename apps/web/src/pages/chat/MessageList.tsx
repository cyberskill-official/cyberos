import type { RefObject } from "react";
import type { Message } from "../../lib/chat";
import { t } from "../../lib/i18n";
import type { MentionCandidate } from "../../lib/richtext";
import { MessageRow } from "./MessageRow";

// The scrolling message pane: drag / drop / paste to stage a file, the empty-state note, and the grouped rows.
// The parent owns the scroll ref, the computed `rows`, and every handler; this is the container plus the map.
export function MessageList({
  rows,
  messages,
  me,
  token,
  scrollRef,
  dragOver,
  highlightId,
  showJumpLatest,
  onJumpLatest,
  onScrollPane,
  editingId,
  editText,
  reactPickerId,
  translating,
  translations,
  translateError,
  myLastId,
  seenBy,
  seenLabel,
  mentionNames,
  nameOf,
  avatarSrc,
  onDragOver,
  onDragLeave,
  onDrop,
  onPaste,
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
}: {
  rows: { m: Message; showDay: boolean; grouped: boolean }[];
  messages: Message[];
  me: string;
  token: string | null;
  scrollRef: RefObject<HTMLDivElement>;
  dragOver: boolean;
  highlightId: string;
  showJumpLatest: boolean;
  onJumpLatest: () => void;
  onScrollPane: () => void;
  editingId: string;
  editText: string;
  reactPickerId: string;
  translating: Set<string>;
  translations: Record<string, string>;
  translateError: Set<string>;
  myLastId: string;
  seenBy: string[];
  seenLabel: string;
  mentionNames: MentionCandidate[];
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  onDragOver: (e: React.DragEvent<HTMLDivElement>) => void;
  onDragLeave: (e: React.DragEvent<HTMLDivElement>) => void;
  onDrop: (e: React.DragEvent<HTMLDivElement>) => void;
  onPaste: (e: React.ClipboardEvent<HTMLDivElement>) => void;
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
}) {
  return (
    <div
      className={"messages" + (dragOver ? " drag-over" : "")}
      ref={scrollRef}
      onScroll={onScrollPane}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onPaste={onPaste}
    >
      {messages.length === 0 && (
        <div className="empty">
          <div className="empty-sub">{t("chat.noMessages")}</div>
        </div>
      )}
      {rows.map(({ m, showDay, grouped }) => (
        <MessageRow
          key={m.id}
          m={m}
          showDay={showDay}
          grouped={grouped}
          mine={m.sender_subject_id === me}
          token={token}
          editingId={editingId}
          editText={editText}
          reactPickerId={reactPickerId}
          translating={translating}
          translations={translations}
          translateError={translateError}
          highlighted={m.id === highlightId}
          isMyLast={m.id === myLastId}
          seenBy={seenBy}
          seenLabel={seenLabel}
          mentionNames={mentionNames}
          nameOf={nameOf}
          avatarSrc={avatarSrc}
          onOpenImage={onOpenImage}
          onEditTextChange={onEditTextChange}
          onSaveEdit={onSaveEdit}
          onCancelEdit={onCancelEdit}
          onToggleReaction={onToggleReaction}
          onSetReactPicker={onSetReactPicker}
          onOpenFullEmoji={onOpenFullEmoji}
          onTranslate={onTranslate}
          onOpenThread={onOpenThread}
          onStartEdit={onStartEdit}
          onDelete={onDelete}
          onRetry={onRetry}
        />
      ))}
      {showJumpLatest && (
        <button className="jump-pill" onClick={onJumpLatest} type="button">
          {t("chat.jumpLatest")}
        </button>
      )}
    </div>
  );
}
