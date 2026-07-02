import type { Channel, Directory } from "../../lib/chat";
import { channelLabel } from "../../lib/chat";
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
  dms,
  activeId,
  unread,
  presence,
  health,
  nameOf,
  avatarSrc,
  onOpenProfile,
  onSelectChannel,
  onOpenPicker,
}: {
  me: string;
  email: string;
  selfName: string;
  myAvatar: string;
  directory: Directory;
  groups: Channel[];
  dms: Channel[];
  activeId: string;
  unread: Record<string, number>;
  presence: Set<string>;
  health: "unknown" | "ok" | "bad";
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  onOpenProfile: () => void;
  onSelectChannel: (id: string) => void;
  onOpenPicker: (mode: PickerMode) => void;
}) {
  const renderRow = (c: Channel) => {
    const u = unread[c.id] || 0;
    const isActive = c.id === activeId;
    const dm = c.kind === "direct";
    const other = c.other_subject_id || "";
    return (
      <button
        key={c.id}
        className={"chan-row" + (isActive ? " active" : "") + (u > 0 && !isActive ? " unread" : "")}
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
        {u > 0 && !isActive && <span className="chan-badge">{u > 99 ? "99+" : u}</span>}
      </button>
    );
  };

  return (
    <aside className="sidebar">
      <button className="ws-head" onClick={onOpenProfile} type="button" title="Edit your profile">
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
            <span>Channels</span>
            <button className="side-add" title="New channel" onClick={() => onOpenPicker("group")} type="button">
              <Icon name="plus" size={14} />
            </button>
          </div>
          {groups.map(renderRow)}
          {groups.length === 0 && <div className="side-empty">No channels yet</div>}
        </div>
        <div className="side-section">
          <div className="side-label">
            <span>Direct messages</span>
            <button className="side-add" title="New direct message" onClick={() => onOpenPicker("dm")} type="button">
              <Icon name="plus" size={14} />
            </button>
          </div>
          {dms.map(renderRow)}
          {dms.length === 0 && <div className="side-empty">No direct messages</div>}
        </div>
      </div>
      <div className="side-foot">
        <span className={"dot " + (health === "ok" ? "ok" : health === "bad" ? "bad" : "")} />
        <span>{health === "ok" ? "Connected" : health === "bad" ? "Reconnecting..." : "Connecting..."}</span>
      </div>
    </aside>
  );
}
