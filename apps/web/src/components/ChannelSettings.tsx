import { useEffect, useMemo, useState } from "react";
import { apiFetch } from "../lib/api";
import type { Channel } from "../lib/chat";
import { t } from "../lib/i18n";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";

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

  async function setArchived(a: boolean) {
    const q = a ? t("settings.confirmArchive") : t("settings.confirmUnarchive");
    if (!window.confirm(q)) return;
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

  async function removeMember(subject: string) {
    if (
      !window.confirm(
        t("settings.confirmRemove", { name: nameOf(subject), channel: channel.name || t("settings.thisChannel") }),
      )
    )
      return;
    setErr("");
    try {
      await apiFetch(token, "DELETE", `/v1/chat/channels/${channel.id}/members/${subject}`);
      await loadMembers();
      onChanged();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }

  async function leave() {
    if (!window.confirm(t("settings.confirmLeave", { channel: channel.name || t("settings.thisChannel") }))) return;
    setErr("");
    try {
      await apiFetch(token, "DELETE", `/v1/chat/channels/${channel.id}/members/${me}`);
      onClose();
      onLeft();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="picker settings">
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
                        {r}
                      </option>
                    ))}
                  </select>
                ) : (
                  <span className="cs-role-tag">{m.role}</span>
                )}
                {isOwner && !self && (
                  <button
                    className="icon-btn cs-remove"
                    title={t("settings.removeFromChannel")}
                    onClick={() => void removeMember(m.subject_id)}
                    type="button"
                  >
                    <Icon name="close" size={14} />
                  </button>
                )}
              </div>
            );
          })}
        </div>

        {err && <div className="banner err">{err}</div>}

        <div className="picker-actions cs-actions">
          {!isOwner && (
            <button className="btn-ghost danger" onClick={() => void leave()} disabled={busy} type="button">
              {t("settings.leaveChannel")}
            </button>
          )}
          {isOwner && (
            <button
              className="btn-ghost danger"
              onClick={() => void setArchived(!archived)}
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
    </div>
  );
}
