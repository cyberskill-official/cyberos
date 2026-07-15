import { useEffect, useMemo, useState } from "react";
import { apiFetch } from "../lib/api";
import type { Channel } from "../lib/chat";
import { t } from "../lib/i18n";
import { Avatar } from "./Avatar";
import { ReportDialog } from "./ReportDialog";
import { ConfirmDialog } from "./ConfirmDialog";
import { Icon } from "./icons";
import { useModalA11y } from "./useModalA11y";

interface Member {
  channel_id: string;
  subject_id: string;
  role: string;
  joined_at?: string;
}

const ROLES = ["owner", "admin", "member"] as const;

// Channel settings modal (find-and-organize cluster): rename / topic / visibility (owner or admin), the
// member roster with role editing + removal (owner), leave (non-owners), and archive/unarchive (owner).
// Everything is enforced server-side; this UI just hides what the caller cannot do.
export function ChannelSettings({
  token,
  channel,
  me,
  nameOf,
  avatarSrc,
  notifyMode,
  onSetNotify,
  onClose,
  onChanged,
  onLeft,
}: {
  token: string;
  channel: Channel;
  me: string;
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  /// The caller's own notify mode for this channel ("all" | "mentions" | "none").
  notifyMode: string;
  onSetNotify(mode: string): void;
  onClose(): void;
  /// Fired after any successful change so the parent refreshes its channel list.
  onChanged(): void;
  /// Fired after the caller left the channel (the parent also deselects it).
  onLeft(): void;
}) {
  const [members, setMembers] = useState<Member[]>([]);
  const [name, setName] = useState(channel.name || "");
  const [topic, setTopic] = useState(channel.topic || "");
  const [visibility, setVisibility] = useState(channel.visibility || "private");
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  // TASK-CHAT-267 §1 #10 — the SECOND entry point: report a person, from the member list. Same dialog as the
  // message action bar. Reporting a person needs no shared channel on the server (§1 #3, the DM path), but
  // the member list is where you have a person in front of you, so it is where the affordance belongs.
  // TASK-CHAT-268 — the caller's OWN block list, and only ever their own. There is no surface anywhere that
  // lets the blocked person discover the block (§1 #2).
  const [blocked, setBlocked] = useState<Set<string>>(new Set());
  useEffect(() => {
    if (!token) return;
    void (async () => {
      try {
        const rows = await apiFetch<{ subject_id: string }[]>(token, "GET", "/v1/chat/blocks");
        setBlocked(new Set(rows.map((r) => r.subject_id)));
      } catch {
        /* a failed block-list read must not break the settings dialog */
      }
    })();
  }, [token]);

  async function toggleBlock(subject: string, on: boolean) {
    try {
      if (on) await apiFetch(token, "POST", "/v1/chat/blocks", { subject_id: subject });
      else await apiFetch(token, "DELETE", `/v1/chat/blocks/${subject}`);
      setBlocked((prev) => {
        const next = new Set(prev);
        if (on) next.add(subject);
        else next.delete(subject);
        return next;
      });
    } catch {
      setErr(t("blocked.failed"));
    }
  }

  const [reportSubject, setReportSubject] = useState("");
  const [reportDone, setReportDone] = useState(false);

  // The in-app confirm dialog (replaces window.confirm) for the destructive actions - archive, remove, leave.
  const [confirm, setConfirm] = useState<{ body: string; label: string; run: () => Promise<void> } | null>(null);
  const archived = !!channel.archived_at;

  const myRole = useMemo(
    () => members.find((m) => m.subject_id === me)?.role || "member",
    [members, me],
  );
  const isOwner = myRole === "owner";
  const isManager = isOwner || myRole === "admin";

  async function loadMembers() {
    try {
      const rows = await apiFetch<Member[]>(token, "GET", `/v1/chat/channels/${channel.id}/members`);
      setMembers(rows || []);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }
  useEffect(() => {
    void loadMembers();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [channel.id]);

  const dirty =
    (name.trim() && name.trim() !== (channel.name || "")) ||
    topic.trim() !== (channel.topic || "") ||
    visibility !== (channel.visibility || "private");

  async function saveMeta() {
    const payload: Record<string, unknown> = {};
    if (name.trim() && name.trim() !== (channel.name || "")) payload.name = name.trim();
    if (topic.trim() !== (channel.topic || "")) payload.topic = topic.trim();
    if (visibility !== (channel.visibility || "private")) payload.visibility = visibility;
    if (Object.keys(payload).length === 0) return;
    setBusy(true);
    setErr("");
    try {
      await apiFetch(token, "PATCH", `/v1/chat/channels/${channel.id}`, payload);
      onChanged();
      onClose();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  function requestArchive(a: boolean) {
    setConfirm({
      body: a ? t("settings.confirmArchive") : t("settings.confirmUnarchive"),
      label: a ? t("settings.archive") : t("settings.unarchive"),
      run: () => doArchive(a),
    });
  }
  async function doArchive(a: boolean) {
    setBusy(true);
    setErr("");
    try {
      await apiFetch(token, "PATCH", `/v1/chat/channels/${channel.id}`, { archived: a });
      onChanged();
      onClose();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function changeRole(subject: string, role: string) {
    setErr("");
    try {
      await apiFetch(token, "PATCH", `/v1/chat/channels/${channel.id}/members/${subject}`, { role });
      await loadMembers();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
      await loadMembers(); // snap the select back to the server truth
    }
  }

  function requestRemove(subject: string) {
    setConfirm({
      body: t("settings.confirmRemove", { name: nameOf(subject), channel: channel.name || t("settings.thisChannel") }),
      label: t("settings.removeFromChannel"),
      run: () => doRemove(subject),
    });
  }
  async function doRemove(subject: string) {
    setErr("");
    try {
      await apiFetch(token, "DELETE", `/v1/chat/channels/${channel.id}/members/${subject}`);
      await loadMembers();
      onChanged();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }

  function requestLeave() {
    setConfirm({
      body: t("settings.confirmLeave", { channel: channel.name || t("settings.thisChannel") }),
      label: t("settings.leaveChannel"),
      run: doLeave,
    });
  }
  async function doLeave() {
    setErr("");
    try {
      await apiFetch(token, "DELETE", `/v1/chat/channels/${channel.id}/members/${me}`);
      onClose();
      onLeft();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }

  const boxRef = useModalA11y(onClose);
  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div
        className="picker settings"
        ref={boxRef}
        role="dialog"
        aria-modal="true"
        aria-label={t("settings.title")}
        tabIndex={-1}
      >
        <div className="picker-head">
          <span>{t("settings.title")}</span>
          <button className="icon-btn" onClick={onClose} type="button" title={t("common.close")}>
            <Icon name="close" size={16} />
          </button>
        </div>

        {archived && <div className="cs-archived-note">{t("settings.archivedNote")}</div>}

        <label className="cs-label" htmlFor="cs-name">
          {t("settings.name")}
        </label>
        <input
          id="cs-name"
          className="picker-input"
          value={name}
          onChange={(e) => setName(e.target.value)}
          disabled={!isManager || busy}
        />
        <label className="cs-label" htmlFor="cs-topic">
          {t("settings.topic")}
        </label>
        <input
          id="cs-topic"
          className="picker-input"
          placeholder={t("settings.topicPlaceholder")}
          value={topic}
          onChange={(e) => setTopic(e.target.value)}
          disabled={!isManager || busy}
        />
        <label className="cs-label">{t("settings.visibility")}</label>
        <div className="cs-vis">
          <button
            className={"cs-vis-opt" + (visibility === "private" ? " on" : "")}
            onClick={() => isManager && setVisibility("private")}
            disabled={!isManager || busy}
            type="button"
          >
            {t("settings.private")}
            <span className="cs-vis-sub">{t("settings.privateSub")}</span>
          </button>
          <button
            className={"cs-vis-opt" + (visibility === "public" ? " on" : "")}
            onClick={() => isManager && setVisibility("public")}
            disabled={!isManager || busy}
            type="button"
          >
            {t("settings.public")}
            <span className="cs-vis-sub">{t("settings.publicSub")}</span>
          </button>
        </div>

        <label className="cs-label" htmlFor="cs-notify">
          {t("settings.notify")}
        </label>
        <select
          id="cs-notify"
          className="cs-role cs-notify"
          value={notifyMode}
          onChange={(e) => onSetNotify(e.target.value)}
        >
          <option value="all">{t("settings.notifyAll")}</option>
          <option value="mentions">{t("settings.notifyMentions")}</option>
          <option value="none">{t("settings.notifyNone")}</option>
        </select>

        <div className="cs-label cs-roster-label">{t("settings.members", { n: members.length })}</div>
        <div className="picker-people cs-roster">
          {members.map((m) => {
            const self = m.subject_id === me;
            return (
              <div key={m.subject_id} className="person cs-member">
                <Avatar id={m.subject_id} name={nameOf(m.subject_id)} size={30} src={avatarSrc(m.subject_id)} />
                <div className="person-meta">
                  <span className="pname">
                    {nameOf(m.subject_id)}
                    {self ? t("settings.youSuffix") : ""}
                  </span>
                </div>
                {isOwner && !self ? (
                  <select
                    className="cs-role"
                    value={m.role}
                    onChange={(e) => void changeRole(m.subject_id, e.target.value)}
                  >
                    {ROLES.map((r) => (
                      <option key={r} value={r}>
                        {t("role." + r)}
                      </option>
                    ))}
                  </select>
                ) : (
                  <span className="cs-role-tag">{t("role." + m.role)}</span>
                )}
                {/* TASK-CHAT-268 §1 #15 — Block / Unblock, confirmed. Not an owner power: blocking changes what
                    YOU see, and every member holds it over every other member. */}
                {!self && (
                  <button
                    className="icon-btn"
                    title={blocked.has(m.subject_id) ? t("blocked.unblockPerson") : t("blocked.blockPerson")}
                    onClick={() => {
                      const on = !blocked.has(m.subject_id);
                      setConfirm({
                        body: t(on ? "blocked.confirmBlock" : "blocked.confirmUnblock", {
                          name: nameOf(m.subject_id),
                        }),
                        label: t(on ? "blocked.block" : "blocked.unblock"),
                        run: () => toggleBlock(m.subject_id, on),
                      });
                    }}
                    type="button"
                  >
                    <Icon name={blocked.has(m.subject_id) ? "bell" : "bellOff"} size={14} />
                  </button>
                )}
                {/* Anyone may report anyone but themselves — reporting is not an owner power (§1 #10). */}
                {!self && (
                  <button
                    className="icon-btn"
                    title={t("report.reportPerson")}
                    onClick={() => setReportSubject(m.subject_id)}
                    type="button"
                  >
                    <Icon name="flag" size={14} />
                  </button>
                )}
                {isOwner && !self && (
                  <button
                    className="icon-btn cs-remove"
                    title={t("settings.removeFromChannel")}
                    onClick={() => requestRemove(m.subject_id)}
                    type="button"
                  >
                    <Icon name="close" size={14} />
                  </button>
                )}
              </div>
            );
          })}
        </div>

        {reportDone && <div className="banner">{t("report.sent")}</div>}
        {err && <div className="banner err">{err}</div>}

        {reportSubject && (
          <ReportDialog
            token={token}
            target={{ kind: "subject", id: reportSubject }}
            onClose={() => setReportSubject("")}
            onSent={() => setReportDone(true)}
          />
        )}

        <div className="picker-actions cs-actions">
          {!isOwner && (
            <button className="btn-ghost danger" onClick={requestLeave} disabled={busy} type="button">
              {t("settings.leaveChannel")}
            </button>
          )}
          {isOwner && (
            <button
              className="btn-ghost danger"
              onClick={() => requestArchive(!archived)}
              disabled={busy}
              type="button"
            >
              {archived ? t("settings.unarchive") : t("settings.archive")}
            </button>
          )}
          <span className="spacer" />
          <button className="btn-ghost" onClick={onClose} type="button">
            {t("common.cancel")}
          </button>
          {isManager && (
            <button className="btn-pill" onClick={() => void saveMeta()} disabled={busy || !dirty} type="button">
              {t("common.save")}
            </button>
          )}
        </div>
      </div>

      {confirm && (
        <ConfirmDialog
          body={confirm.body}
          confirmLabel={confirm.label}
          busy={busy}
          onCancel={() => setConfirm(null)}
          onConfirm={async () => {
            const run = confirm.run;
            await run();
            setConfirm(null);
          }}
        />
      )}
    </div>
  );
}
