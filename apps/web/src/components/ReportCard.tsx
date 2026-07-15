import { t } from "../lib/i18n";
import type { CatalogKey } from "../lib/i18n";

// TASK-CHAT-269 — one entry in the moderation queue, and the detail panel for the selected one.
//
// Everything an attacker controls is rendered as TEXT (§1 #20). `snapshot_body`, `detail` and `note` are
// strings a reporter or a reported person authored, displayed in an administrator's browser. React escapes
// by default, so this is safe as long as nobody reaches for `dangerouslySetInnerHTML` or a markdown renderer
// — which is exactly why neither appears anywhere in this file, and must not.

export type QueueEntry = {
  lead_report_id: string;
  target_kind: string;
  target_message_id?: string | null;
  target_subject_id?: string | null;
  report_count: number;
  severity: number;
  reasons: string[];
  last_reported_at: string;
};

export type ReportDetail = {
  id: string;
  reason: string;
  detail?: string | null;
  reported_at: string;
  reporter_subject_id: string;
  report_count: number;
  target_kind: string;
  target_message_id?: string | null;
  channel_id?: string | null;
  snapshot_body?: string | null;
  snapshot_filename?: string | null;
  snapshot_sender_id?: string | null;
  snapshot_taken_at: string;
  original_present: boolean;
  status: string;
  resolution?: string | null;
  context: { id: string; sender_subject_id: string; body: string; created_at?: string }[];
};

/** Every string this surface renders, checked against the catalog at compile time — same guarantee as
 *  TASK-CHAT-267's REPORT_DIALOG_KEYS. A missing key is a build failure, so §1 #21 (en + vi) cannot silently
 *  regress. */
export const MODERATION_KEYS = [
  "mod.title",
  "mod.empty",
  "mod.reportCount",
  "mod.reportedBy",
  "mod.evidence",
  "mod.originalGone",
  "mod.originalPresent",
  "mod.context",
  "mod.noContext",
  "mod.noContextDm",
  "mod.note",
  "mod.notePlaceholder",
  "mod.dismiss",
  "mod.deleteMessage",
  "mod.removeMember",
  "mod.resolved",
  "mod.failed",
  "mod.contentPolicy",
  "mod.severity",
] as const satisfies readonly CatalogKey[];

/** Severity 0 is the worst. Mirrors severity_rank in services/chat/src/moderation.rs. */
function severityClass(sev: number): string {
  if (sev <= 1) return "sev-critical"; // self_harm, illegal
  if (sev <= 3) return "sev-high"; // violence, sexual
  if (sev <= 5) return "sev-medium"; // hate, harassment
  return "sev-low"; // spam, other
}

export function ReportCard({
  entry,
  selected,
  onSelect,
  nameOf,
}: {
  entry: QueueEntry;
  selected: boolean;
  onSelect(): void;
  nameOf: (id: string) => string;
}) {
  return (
    <button
      className={"report-card" + (selected ? " on" : "")}
      onClick={onSelect}
      type="button"
      aria-current={selected}
    >
      <span className={"sev-dot " + severityClass(entry.severity)} aria-hidden="true" />
      <span className="report-card-main">
        {/* Reasons are a closed enum, so they resolve to a translated label rather than raw text. */}
        <span className="report-card-reasons">
          {entry.reasons.map((r) => t(`report.reason.${r}`)).join(", ")}
        </span>
        <span className="report-card-meta muted">
          {entry.target_kind === "subject"
            ? nameOf(entry.target_subject_id || "")
            : t("mod.reportCount", { n: entry.report_count })}
        </span>
      </span>
      {/* §1 #4 — three people reporting one message is ONE entry carrying a count, not three entries. */}
      {entry.report_count > 1 && <span className="report-card-count">{entry.report_count}</span>}
    </button>
  );
}
