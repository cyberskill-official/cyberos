import type { RefObject } from "react";
import type { Message } from "../../lib/chat";
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
  rows: { m: Message; showDay: boolean; grouped: boolean }[];
  messages: Message[];
  me: string;
  token: string | null;
  scrollRef: RefObject<HTMLDivElement>;
  dragOver: boolean;
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
    <div
      className={"messages" + (dragOver ? " drag-over" : "")}
      ref={scrollRef}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      onPaste={onPaste}
    >
      {messages.length === 0 && (
        <div className="empty">
          <div className="empty-sub">No messages yet. Say hello.</div>
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
          isMyLast={m.id === myLastId}
          seenBy={seenBy}
          seenLabel={seenLabel}
          mentionNames={mentionNames}
          nameOf={nameOf}
          avatarSrc={avatarSrc}
          onEditTextChange={onEditTextChange}
          onSaveEdit={onSaveEdit}
          onCancelEdit={onCancelEdit}
          onToggleReaction={onToggleReaction}
          onSetReactPicker={onSetReactPicker}
          onTranslate={onTranslate}
          onOpenThread={onOpenThread}
          onStartEdit={onStartEdit}
          onDelete={onDelete}
        />
      ))}
    </div>
  );
}
