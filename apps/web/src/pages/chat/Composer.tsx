import type { RefObject } from "react";
import type { Channel, Directory } from "../../lib/chat";
import { channelLabel, formatBytes } from "../../lib/chat";
import { Icon } from "../../components/icons";

// The message composer: the staged-file preview strip (shown while a file is queued for the next send) plus the
// attach button, the growing textarea (Enter sends, Shift+Enter newline, paste stages a file), and the send
// button. All state lives in Chat; this renders it and calls back. DOM + classNames are copied verbatim.
export function Composer({
  active,
  directory,
  me,
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
  onPaste,
}: {
  active: Channel;
  directory: Directory;
  me: string;
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
  onPaste: (e: React.ClipboardEvent<HTMLTextAreaElement>) => void;
}) {
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
        <textarea
          ref={taRef}
          rows={1}
          value={draft}
          onChange={(e) => onDraftChange(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter" && !e.shiftKey) {
              e.preventDefault();
              void onSend();
            }
          }}
          onPaste={onPaste}
          placeholder={staged ? "Add a message or just send the file" : "Message " + channelLabel(directory, me, active)}
        />
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
