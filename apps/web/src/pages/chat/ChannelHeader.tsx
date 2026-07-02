import { Fragment } from "react";
import type { Channel, Directory, Message } from "../../lib/chat";
import { channelLabel, timeOf } from "../../lib/chat";
import { Avatar } from "../../components/Avatar";
import { Icon } from "../../components/icons";

// The conversation header: identity (avatar/hash + title + subtitle) and the call / video / search / add-people
// actions, plus the collapsible channel search bar (a sibling of the header row, exactly as before). All state
// lives in Chat; this renders it and calls back.
export function ChannelHeader({
  active,
  directory,
  me,
  subtitle,
  presence,
  searchOpen,
  searchQ,
  searchResults,
  nameOf,
  avatarSrc,
  channelOf,
  onStartCall,
  onToggleSearch,
  onOpenAddPeople,
  onOpenSettings,
  onSearchQChange,
  onRunSearch,
  onPickResult,
}: {
  active: Channel;
  directory: Directory;
  me: string;
  subtitle: string;
  presence: Set<string>;
  searchOpen: boolean;
  searchQ: string;
  searchResults: Message[];
  nameOf: (id: string) => string;
  avatarSrc: (id: string) => string;
  /// The sidebar label of a result's channel ("" when unknown), so results say where they were found.
  channelOf: (m: Message) => string;
  onStartCall: (video: boolean) => void;
  onToggleSearch: () => void;
  onOpenAddPeople: () => void;
  onOpenSettings: () => void;
  onSearchQChange: (v: string) => void;
  onRunSearch: () => void;
  /// Jump to a result's message (switches channel when needed).
  onPickResult: (m: Message) => void;
}) {
  return (
    <Fragment>
      <div className="main-head">
        <div className="head-id">
          {active.kind === "direct" ? (
            <Avatar
              id={active.other_subject_id || active.id}
              name={nameOf(active.other_subject_id || "")}
              size={36}
              online={presence.has(active.other_subject_id || "")}
              src={avatarSrc(active.other_subject_id || "")}
            />
          ) : (
            <span className="head-hash">
              <Icon name="hash" size={20} />
            </span>
          )}
          <div className="head-text">
            <span className="chan-title">{channelLabel(directory, me, active)}</span>
            <span className="chan-sub">{subtitle}</span>
          </div>
        </div>
        <span className="spacer" />
        <button className="icon-btn" title="Voice call" onClick={() => onStartCall(false)} type="button">
          <Icon name="phone" />
        </button>
        <button className="icon-btn" title="Video call" onClick={() => onStartCall(true)} type="button">
          <Icon name="video" />
        </button>
        <button
          className={"icon-btn" + (searchOpen ? " on" : "")}
          title="Search all channels (Ctrl/Cmd+K)"
          onClick={onToggleSearch}
          type="button"
        >
          <Icon name="search" />
        </button>
        {active.kind !== "direct" && (
          <button className="icon-btn" title="Add people" onClick={onOpenAddPeople} type="button">
            <Icon name="users" />
          </button>
        )}
        {active.kind !== "direct" && (
          <button className="icon-btn" title="Channel settings" onClick={onOpenSettings} type="button">
            <Icon name="gear" />
          </button>
        )}
      </div>

      {searchOpen && (
        <div className="search-bar">
          <input
            value={searchQ}
            onChange={(e) => onSearchQChange(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                e.preventDefault();
                void onRunSearch();
              }
            }}
            placeholder="Search all channels"
            autoFocus
          />
          <button className="btn-pill" onClick={() => void onRunSearch()} type="button">
            Search
          </button>
          {searchResults.length > 0 && (
            <div className="search-results">
              {searchResults.map((m) => (
                <button key={m.id} className="search-row" onClick={() => onPickResult(m)} type="button">
                  {channelOf(m) && <span className="search-chan">{channelOf(m)}</span>}
                  <span className="author">{nameOf(m.sender_subject_id)}</span>{" "}
                  <span className="when">{timeOf(m.created_at)}</span>
                  <div className="snippet">{m.body || "[attachment]"}</div>
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </Fragment>
  );
}
