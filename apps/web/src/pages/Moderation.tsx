import { useCallback, useEffect, useState } from "react";
import { useAuth } from "../lib/auth";
import { apiFetch } from "../lib/api";
import { t, currentLang } from "../lib/i18n";
import { isModerator } from "../lib/roles";
import { ReportCard } from "../components/ReportCard";
import type { QueueEntry, ReportDetail } from "../components/ReportCard";

// FR-CHAT-269 — the workspace moderation queue.
//
// This page is reachable only for an administrator (App.tsx renders neither the route nor the nav entry
// otherwise, §1 #18), but that is a UX decision, not a security one: all three endpoints re-check the role
// server-side and fail closed. Editing `roles` in devtools gets you a 403, not a queue.
//
// Nothing here is rendered as markup. `snapshot_body`, `detail` and `note` are attacker-controlled strings
// shown in an admin's browser (§1 #20); React escapes them, and there is deliberately no markdown renderer
// and no dangerouslySetInnerHTML anywhere in this file.

const CONTENT_POLICY = (): string => `https://cyberskill.world/${currentLang()}/cyberos/content-policy`;

export function Moderation({ onBack }: { onBack: () => void }) {
  const { token } = useAuth();
  const [entries, setEntries] = useState<QueueEntry[]>([]);
  const [selected, setSelected] = useState<ReportDetail | null>(null);
  const [note, setNote] = useState("");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const [loading, setLoading] = useState(true);

  const loadQueue = useCallback(async () => {
    if (!token) return;
    setLoading(true);
    try {
      const page = await apiFetch<{ entries: QueueEntry[] }>(token, "GET", "/v1/chat/admin/reports");
      setEntries(page.entries || []);
    } catch {
      setErr(t("mod.failed"));
    } finally {
      setLoading(false);
    }
  }, [token]);

  useEffect(() => {
    void loadQueue();
  }, [loadQueue]);

  async function open(id: string) {
    if (!token) return;
    setNote("");
    setErr("");
    try {
      setSelected(await apiFetch<ReportDetail>(token, "GET", `/v1/chat/admin/reports/${id}`));
    } catch {
      setErr(t("mod.failed"));
    }
  }

  async function act(action: "dismiss" | "delete_message" | "remove_member") {
    if (!token || !selected || busy) return;
    setBusy(true);
    setErr("");
    try {
      await apiFetch(token, "POST", `/v1/chat/admin/reports/${selected.id}/resolve`, {
        action,
        ...(note.trim() ? { note: note.trim() } : {}),
      });
      setSelected(null);
      await loadQueue();
    } catch {
      setErr(t("mod.failed"));
    } finally {
      setBusy(false);
    }
  }

  // Defence in depth. App.tsx does not route here for a non-admin, but if it ever did, this renders nothing
  // rather than an empty queue that looks like "no reports".
  if (!isModerator(token)) return null;

  const nameOf = (id: string) => (id ? id.slice(0, 8) : "");

  return (
    <div className="moderation">
      <div className="mod-head">
        <h1 className="mod-title">{t("mod.title")}</h1>
        <div className="mod-head-right">
          {/* §1 #19 — the published content policy must be reachable. Play requires it to exist AND to be
              findable; a policy nobody can find is not a policy. */}
          <a className="linkish" href={CONTENT_POLICY()} target="_blank" rel="noreferrer">
            {t("mod.contentPolicy")}
          </a>
          <button className="btn-ghost" onClick={onBack} type="button">
            {t("top.backToChat")}
          </button>
        </div>
      </div>

      {err && <div className="banner err">{err}</div>}

      <div className="mod-body">
        <div className="mod-queue">
          {loading ? (
            <div className="muted">{t("common.loading")}</div>
          ) : entries.length === 0 ? (
            <div className="muted">{t("mod.empty")}</div>
          ) : (
            entries.map((e) => (
              <ReportCard
                key={e.lead_report_id}
                entry={e}
                selected={selected?.id === e.lead_report_id}
                onSelect={() => void open(e.lead_report_id)}
                nameOf={nameOf}
              />
            ))
          )}
        </div>

        <div className="mod-detail">
          {!selected ? (
            <div className="muted">{t("mod.empty")}</div>
          ) : (
            <>
              <div className="mod-meta muted">
                <span>{t("report.reason." + selected.reason)}</span>
                <span>{t("mod.reportedBy", { name: nameOf(selected.reporter_subject_id) })}</span>
                <span>{t("mod.reportCount", { n: selected.report_count })}</span>
              </div>

              <h2 className="mod-h2">{t("mod.evidence")}</h2>
              {/* The immutable snapshot from FR-CHAT-267 §1 #4. Rendered as TEXT. */}
              <blockquote className="mod-snapshot" data-testid="snapshot">
                {selected.snapshot_body || selected.snapshot_filename || ""}
              </blockquote>
              {/* §1 #7 — "they said this" vs "they said this and have since deleted it". The second is
                  itself evidence, and the reviewer must be able to tell. */}
              <p className={"mod-original " + (selected.original_present ? "" : "gone")}>
                {selected.original_present ? t("mod.originalPresent") : t("mod.originalGone")}
              </p>

              {selected.detail && (
                <blockquote className="mod-detail-text" data-testid="detail">
                  {selected.detail}
                </blockquote>
              )}

              <h2 className="mod-h2">{t("mod.context")}</h2>
              {selected.context.length === 0 ? (
                // §1 #9 — say WHY there is no context, plainly. A silent empty panel invites someone to
                // "fix" it by fetching the DM thread, which is the one thing this FR exists to prevent.
                <p className="muted">
                  {selected.target_kind === "message" && selected.channel_id
                    ? t("mod.noContextDm")
                    : t("mod.noContext")}
                </p>
              ) : (
                <ul className="mod-context">
                  {selected.context.map((m) => (
                    <li key={m.id}>
                      <span className="mod-ctx-who">{nameOf(m.sender_subject_id)}</span>
                      <span className="mod-ctx-body">{m.body}</span>
                    </li>
                  ))}
                </ul>
              )}

              <label className="mod-note">
                <span>{t("mod.note")}</span>
                <textarea
                  value={note}
                  onChange={(e) => setNote(e.target.value)}
                  placeholder={t("mod.notePlaceholder")}
                  maxLength={1000}
                  rows={2}
                  disabled={busy}
                />
              </label>

              <div className="confirm-actions">
                <button className="btn-ghost" onClick={() => void act("dismiss")} disabled={busy} type="button">
                  {t("mod.dismiss")}
                </button>
                {selected.target_message_id && (
                  <button
                    className="btn-pill danger"
                    onClick={() => void act("delete_message")}
                    disabled={busy}
                    type="button"
                  >
                    {t("mod.deleteMessage")}
                  </button>
                )}
                {selected.channel_id && (
                  <button
                    className="btn-pill danger"
                    onClick={() => void act("remove_member")}
                    disabled={busy}
                    type="button"
                  >
                    {t("mod.removeMember")}
                  </button>
                )}
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
