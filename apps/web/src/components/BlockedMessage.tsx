import { useState } from "react";
import { t } from "../lib/i18n";
import type { CatalogKey } from "../lib/i18n";

/** Every string the blocking surfaces render, checked against the catalog at COMPILE time — the same trick
 *  TASK-CHAT-267 uses for the report dialog. `satisfies readonly CatalogKey[]` makes `tsc --noEmit` fail if any
 *  of these is missing, and because a catalog entry is `{ en; vi }`, a key that type-checks necessarily
 *  carries both locales. TASK-CHAT-268 §1 #15 ("every string must ship in en and vi") is therefore a build
 *  failure when broken, not a string that quietly renders as its own name. */
export const BLOCK_KEYS = [
  "blocked.hidden",
  "blocked.revealed",
  "blocked.showAnyway",
  "blocked.block",
  "blocked.unblock",
  "blocked.blockPerson",
  "blocked.unblockPerson",
  "blocked.confirmBlock",
  "blocked.confirmUnblock",
  "blocked.failed",
] as const satisfies readonly CatalogKey[];

// TASK-CHAT-268 §1 #5 — the collapsed placeholder shown in a GROUP channel where a blocked person has posted.
//
// Why a placeholder and not a hole: removing the message outright would silently rewrite the channel's
// history for one participant. Replies to a vanished message become nonsense, thread counts stop matching,
// and the blocker ends up more confused than protected. The placeholder tells the truth — someone you
// blocked said something here — preserves the shape of the conversation, and leaves the choice to look with
// the person who made the block. It is *their* block; they are allowed to un-hide their own view.
//
// "Show anyway" is a purely local reveal. It reveals nothing, because there is nothing to reveal: the server
// withheld the body, and this client never had it. All the button does is stop hiding the fact that a
// message exists — it cannot un-withhold content it was never sent. If we ever want a real reveal it has to
// be a server round-trip, and that is a different task.
//
// This is deliberately NOT rendered in a DM: there, a blocked sender's messages are not returned at all
// (§1 #6). A column of these placeholders in a one-to-one thread is not context, it is a drip-feed of the
// harassment the person asked to stop.
export function BlockedMessage({ name }: { name: string }) {
  const [shown, setShown] = useState(false);

  return (
    <div className="blocked-msg">
      <span className="blocked-msg-text muted">
        {shown ? t("blocked.revealed", { name }) : t("blocked.hidden", { name })}
      </span>
      {!shown && (
        <button className="linkish" onClick={() => setShown(true)} type="button">
          {t("blocked.showAnyway")}
        </button>
      )}
    </div>
  );
}
