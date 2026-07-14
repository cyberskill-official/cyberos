import { useState } from "react";
import { apiFetch } from "../lib/api";
import { t, currentLang } from "../lib/i18n";
import type { CatalogKey } from "../lib/i18n";
import { useModalA11y } from "./useModalA11y";

// TASK-CHAT-267 — the one report dialog, opened from two places: a message's action bar (report the message,
// or its attachment) and the member list (report a person). Both entry points render THIS component (§1 #10).
//
// Three deliberate behaviours:
//
//  * No reason is pre-selected, and Submit stays disabled until one is chosen (§11). A pre-selected default
//    is what an upset person submits when clicking fast, and it poisons the moderation queue's ordering —
//    which sorts by reason.
//  * Nothing about the outcome is shown back (§1 #14). No report id, no status, no "you already reported
//    this". The response carries only an id and we deliberately drop it: any downstream state rendered here
//    would be an oracle telling a reporter whether a prior report exists.
//  * Accessibility is not decoration (§1 #11). `useModalA11y` moves focus in on open, traps Tab, closes on
//    Escape, and returns focus to the invoking control on close. A moderation control that only a mouse
//    user can reach is not a moderation control.

/** The closed set from §1 #2. Order is the order the radios render in — roughly worst-first, so the
 *  categories a distressed person is most likely to want are not buried at the bottom. */
export const REPORT_REASONS = [
  "harassment",
  "hate",
  "sexual",
  "violence",
  "self_harm",
  "illegal",
  "spam",
  "other",
] as const;

export type ReportReason = (typeof REPORT_REASONS)[number];

/** What is being reported. Mirrors the server's `target_kind` + the one populated target id. */
export type ReportTarget =
  | { kind: "message"; id: string }
  | { kind: "attachment"; id: string }
  | { kind: "subject"; id: string };

/** Every i18n key this dialog renders.
 *
 *  AC 16 ("every string resolves in en and vi; no key falls back to its own name") is enforced HERE, by the
 *  typechecker, rather than by a runtime test — apps/web has no test runner, and a compile-time proof is
 *  strictly stronger than one anyway. `satisfies readonly CatalogKey[]` makes `tsc --noEmit` fail if any of
 *  these keys is absent from the catalog, and because a catalog Entry is `{ en; vi }`, a key that exists
 *  necessarily has both locales. Delete "report.privacyNote" from lib/i18n.ts and the build breaks.
 *
 *  The reason keys are spelled out rather than mapped from REPORT_REASONS, because a `.map()` produces
 *  `string[]` and erases exactly the literal types the check depends on. */
export const REPORT_DIALOG_KEYS = [
  "report.title",
  "report.subtitleMessage",
  "report.subtitleAttachment",
  "report.subtitleSubject",
  "report.reasonLegend",
  "report.detailLabel",
  "report.detailPlaceholder",
  "report.submit",
  "report.submitting",
  "report.sent",
  "report.failed",
  "report.privacyNote",
  "report.reason.harassment",
  "report.reason.hate",
  "report.reason.sexual",
  "report.reason.violence",
  "report.reason.self_harm",
  "report.reason.illegal",
  "report.reason.spam",
  "report.reason.other",
] as const satisfies readonly CatalogKey[];

const DETAIL_MAX = 1000; // §1 #2 — must match DETAIL_MAX_CHARS in services/chat/src/reports.rs.

function subtitleKey(kind: ReportTarget["kind"]): string {
  if (kind === "message") return "report.subtitleMessage";
  if (kind === "attachment") return "report.subtitleAttachment";
  return "report.subtitleSubject";
}

export function ReportDialog({
  token,
  target,
  onClose,
  onSent,
}: {
  token: string;
  target: ReportTarget;
  onClose(): void;
  /** Fired after the server accepts. The caller shows a non-blocking toast (§1 #14); we pass nothing,
   *  because there is nothing about the report the reporter is allowed to learn. */
  onSent(): void;
}) {
  const [reason, setReason] = useState<ReportReason | "">("");
  const [detail, setDetail] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const boxRef = useModalA11y<HTMLDivElement>(onClose);

  async function submit() {
    if (!reason || busy) return;
    setBusy(true);
    setError("");
    try {
      // The server takes exactly one target id, matching target_kind (the DB's chat_reports_target_shape
      // CHECK enforces the same rule).
      await apiFetch(token, "POST", "/v1/chat/reports", {
        target_kind: target.kind,
        ...(target.kind === "message" ? { target_message_id: target.id } : {}),
        ...(target.kind === "attachment" ? { target_attachment_id: target.id } : {}),
        ...(target.kind === "subject" ? { target_subject_id: target.id } : {}),
        reason,
        ...(detail.trim() ? { detail: detail.trim() } : {}),
      });
      // A duplicate comes back 200 rather than 409 and lands here too — by design (§1 #6). The reporter
      // sees the same confirmation either way; pressing Report twice is not an error, and a different
      // outcome would tell them a prior report exists.
      onSent();
      onClose();
    } catch {
      setError(t("report.failed"));
      setBusy(false);
    }
  }

  const tooLong = detail.length > DETAIL_MAX;

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && !busy && onClose()}>
      <div
        className="picker report-box"
        role="dialog"
        aria-modal="true"
        aria-label={t("report.title")}
        tabIndex={-1}
        ref={boxRef}
      >
        <div className="report-head">
          <h2 className="report-title">{t("report.title")}</h2>
          <p className="report-sub muted">{t(subtitleKey(target.kind))}</p>
        </div>

        <fieldset className="report-reasons">
          <legend>{t("report.reasonLegend")}</legend>
          {REPORT_REASONS.map((r) => (
            <label key={r} className="report-reason">
              <input
                type="radio"
                name="report-reason"
                value={r}
                checked={reason === r}
                onChange={() => setReason(r)}
                disabled={busy}
              />
              <span>{t(`report.reason.${r}`)}</span>
            </label>
          ))}
        </fieldset>

        <label className="report-detail">
          <span>{t("report.detailLabel")}</span>
          <textarea
            value={detail}
            onChange={(e) => setDetail(e.target.value)}
            placeholder={t("report.detailPlaceholder")}
            maxLength={DETAIL_MAX}
            rows={3}
            disabled={busy}
          />
        </label>

        <p className="report-privacy muted">
          {t("report.privacyNote")}{" "}
          {/* TASK-CHAT-269 §1 #19 — the report dialog carries the same content-policy link Settings does.
              Google Play requires the policy to exist AND to be reachable; a policy nobody can find is not
              a policy. Right here is where someone actually wants to know what the rules are. */}
          <a
            className="linkish"
            href={`https://cyberskill.world/${currentLang()}/cyberos/content-policy`}
            target="_blank"
            rel="noreferrer"
          >
            {t("mod.contentPolicy")}
          </a>
        </p>

        {/* aria-live so a screen reader hears the failure without moving focus (§1 #11). */}
        {error && (
          <p className="report-error" role="alert" aria-live="assertive">
            {error}
          </p>
        )}

        <div className="confirm-actions">
          <button className="btn-ghost" onClick={onClose} disabled={busy} type="button">
            {t("common.cancel")}
          </button>
          <button
            className="btn-pill danger"
            onClick={() => void submit()}
            // §11 — disabled until a reason is chosen. No default is offered on purpose.
            disabled={!reason || busy || tooLong}
            type="button"
          >
            {busy ? t("report.submitting") : t("report.submit")}
          </button>
        </div>
      </div>
    </div>
  );
}
