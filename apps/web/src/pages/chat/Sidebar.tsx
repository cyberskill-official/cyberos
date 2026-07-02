import type { Channel, Directory } from "../../lib/chat";
import { channelLabel } from "../../lib/chat";
import { t } from "../../lib/i18n";
import { Avatar } from "../../components/Avatar";
import { Icon } from "../../components/icons";
import type { PickerMode } from "../../components/PeoplePicker";

// The left rail: the workspace header (self avatar + profile trigger), the Channels and Direct messages
// sections (each row a channel button with unread badge + presence dot), the + triggers that open the people
// picker, and the connection status footer. Pure presentation over props lifted from Chat.
export function Sidebar({
  me,
  email,
  selfName,
  myAvatar,
  directory,
  groups,
  archived,
  dms,
  activeId,
  unread,
  mentions,
  notifyPrefs,
  open,
  presence,
  health,
  nameOf,
  avatarSrc,
  onOpenProfile,
  onSelectChannel,
  onOpenPicker,
  onOpenBrowse,
}: {
  me: string;
  email: string;
  selfName: string;
  myAvatar: string;
  directory: Directory;
  groups: Channel[];
  archived: Channel[];
  dms: Channel[];
  activeId: string;
  unread: Record<string, number>;
  mentions: Record<string, number>;
  /// channel -> "mentions" | "none" overrides (absent = all): muted rows dim + quiet their unread badge.
  notifyPrefs: Record<string, string>;
  /// Narrow-viewport drawer state (ignored on wide screens, where the sidebar is always visible).
  open: boolean;
  presence: Set<string>;
  health: "unknown" | "ok" | "bad";
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  onOpenProfile: () => void;
  onSelectChannel: (id: string) => void;
  onOpenPicker: (mode: PickerMode) => void;
  onOpenBrowse: () => void;
}) {
  const renderRow = (c: Channel) => {
    const u = unread[c.id] || 0;
    const mAt = mentions[c.id] || 0;
    const isActive = c.id === activeId;
    const dm = c.kind === "direct";
    const other = c.other_subject_id || "";
    // Muted ("none") and mentions-only channels stay quiet: no unread emphasis or badge; a real @-mention
    // still surfaces its red badge (except full mute, which shows nothing - matching server delivery).
    const mode = notifyPrefs[c.id] || "all";
    const quiet = mode !== "all";
    const showMention = mAt > 0 && mode !== "none";
    const showUnread = u > 0 && !quiet;
    return (
      <button
        key={c.id}
        className={
          "chan-row" +
          (isActive ? " active" : "") +
          (showUnread && !isActive ? " unread" : "") +
          (mode === "none" ? " muted" : "")
        }
        onClick={() => onSelectChannel(c.id)}
        type="button"
      >
        {dm ? (
          // Presence is per-open-socket today: we only learn a subject is online while we hold a socket on a
          // channel they are in. So this dot reflects presence only while that DM's own socket is held (i.e.
          // when it has been opened this session). A correct always-on presence needs a per-user socket
          // independent of the open channel - a known follow-up. We do not fake presence here.
          <Avatar
            id={other || c.id}
            name={nameOf(other)}
            size={26}
            online={presence.has(other)}
            src={avatarSrc(other)}
          />
        ) : (
          <span className="chan-hash">
            <Icon name="hash" size={16} />
          </span>
        )}
        <span className="chan-name">{channelLabel(directory, me, c)}</span>
        {!isActive &&
          (showMention ? (
            <span
              className="chan-badge mention"
              title={mAt > 1 ? t("sidebar.mentionCount_other", { n: mAt }) : t("sidebar.mentionCount_one")}
            >
              @{mAt > 99 ? "99+" : mAt}
            </span>
          ) : (
            showUnread && <span className="chan-badge">{u > 99 ? "99+" : u}</span>
          ))}
      </button>
    );
  };

  return (
    <aside className={"sidebar" + (open ? " open" : "")}>
      <button className="ws-head" onClick={onOpenProfile} type="button" title={t("sidebar.editProfile")}>
        <Avatar id={me} name={selfName} size={34} src={myAvatar} />
        <div className="ws-meta">
          <span className="ws-name">{selfName}</span>
          <span className="ws-sub">{email}</span>
        </div>
        <span className="ws-edit">
          <Icon name="edit" size={14} />
        </span>
      </button>
      <div className="side-scroll">
        <div className="side-section">
          <div className="side-label">
            <span>{t("sidebar.channels")}</span>
            <button className="side-add" title={t("sidebar.browseChannels")} onClick={onOpenBrowse} type="button">
              <Icon name="search" size={13} />
            </button>
            <button className="side-add" title={t("sidebar.newChannel")} onClick={() => onOpenPicker("group")} type="button">
              <Icon name="plus" size={14} />
            </button>
          </div>
          {groups.map(renderRow)}
          {groups.length === 0 && <div className="side-empty">{t("sidebar.noChannels")}</div>}
        </div>
        {archived.length > 0 && (
          <div className="side-section archived">
            <div className="side-label">
              <span>{t("sidebar.archived")}</span>
            </div>
            {archived.map(renderRow)}
          </div>
        )}
        <div className="side-section">
          <div className="side-label">
            <span>{t("sidebar.dms")}</span>
            <button className="side-add" title={t("sidebar.newDm")} onClick={() => onOpenPicker("dm")} type="button">
              <Icon name="plus" size={14} />
            </button>
          </div>
          {dms.map(renderRow)}
          {dms.length === 0 && <div className="side-empty">{t("sidebar.noDms")}</div>}
        </div>
      </div>
      <div className="side-foot">
        <span className={"dot " + (health === "ok" ? "ok" : health === "bad" ? "bad" : "")} />
        <span>
          {health === "ok" ? t("sidebar.connected") : health === "bad" ? t("sidebar.reconnecting") : t("sidebar.connecting")}
        </span>
      </div>
    </aside>
  );
}
