import { Fragment, useEffect, useState } from "react";
import type { ReactNode } from "react";
import type { Channel, Directory, Message } from "../../lib/chat";
import { channelLabel, timeOf } from "../../lib/chat";
import { t } from "../../lib/i18n";
import { Avatar } from "../../components/Avatar";
import { Icon } from "../../components/icons";

// Wrap each case-insensitive occurrence of the query in <mark> so search hits stand out in the snippet.
function highlight(text: string, q: string): ReactNode {
  const query = q.trim();
  if (!query) return text;
  const lower = text.toLowerCase();
  const ql = query.toLowerCase();
  const out: ReactNode[] = [];
  let i = 0;
  let k = 0;
  while (i < text.length) {
    const idx = lower.indexOf(ql, i);
    if (idx === -1) {
      out.push(text.slice(i));
      break;
    }
    if (idx > i) out.push(text.slice(i, idx));
    out.push(<mark key={k++}>{text.slice(idx, idx + query.length)}</mark>);
    i = idx + query.length;
  }
  return out;
}

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
  onToggleAi,
  aiOpen,
  notifyMuted,
  onToggleMute,
  onOpenSidebar,
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
  onToggleAi: () => void;
  aiOpen: boolean;
  /// Whether the open channel is muted ("none" notify mode); the bell toggles it in one step.
  notifyMuted: boolean;
  onToggleMute: () => void;
  onOpenSidebar: () => void;
  onSearchQChange: (v: string) => void;
  onRunSearch: () => void;
  /// Jump to a result's message (switches channel when needed).
  onPickResult: (m: Message) => void;
}) {
  // Keyboard navigation of the results list: -1 means the input has focus and nothing is highlighted.
  const [selIdx, setSelIdx] = useState(-1);
  useEffect(() => {
    setSelIdx(-1);
  }, [searchResults]);

  return (
    <Fragment>
      <div className="main-head">
        <button className="icon-btn only-narrow" title={t("sidebar.channels")} onClick={onOpenSidebar} type="button">
          <Icon name="menu" size={18} />
        </button>
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
        <button
          className={"icon-btn" + (aiOpen ? " on" : "")}
          title={t("header.aiTooltip")}
          onClick={onToggleAi}
          type="button"
        >
          <Icon name="sparkle" />
        </button>
        <button className="icon-btn" title={t("header.voiceCall")} onClick={() => onStartCall(false)} type="button">
          <Icon name="phone" />
        </button>
        <button className="icon-btn" title={t("header.videoCall")} onClick={() => onStartCall(true)} type="button">
          <Icon name="video" />
        </button>
        <button
          className={"icon-btn" + (searchOpen ? " on" : "")}
          title={t("header.searchTooltip")}
          onClick={onToggleSearch}
          type="button"
        >
          <Icon name="search" />
        </button>
        <button
          className={"icon-btn" + (notifyMuted ? " on" : "")}
          title={notifyMuted ? t("header.unmute") : t("header.mute")}
          aria-pressed={notifyMuted}
          onClick={onToggleMute}
          type="button"
        >
          <Icon name={notifyMuted ? "bellOff" : "bell"} />
        </button>
        {active.kind !== "direct" && (
          <button className="icon-btn" title={t("header.addPeople")} onClick={onOpenAddPeople} type="button">
            <Icon name="users" />
          </button>
        )}
        {active.kind !== "direct" && (
          <button className="icon-btn" title={t("settings.title")} onClick={onOpenSettings} type="button">
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
                if (selIdx >= 0 && searchResults[selIdx]) onPickResult(searchResults[selIdx]);
                else void onRunSearch();
              } else if (e.key === "ArrowDown") {
                e.preventDefault();
                setSelIdx((i) => (searchResults.length ? Math.min(searchResults.length - 1, i + 1) : -1));
              } else if (e.key === "ArrowUp") {
                e.preventDefault();
                setSelIdx((i) => Math.max(-1, i - 1));
              } else if (e.key === "Escape") {
                e.preventDefault();
                onToggleSearch();
              }
            }}
            placeholder={t("header.searchPlaceholder")}
            aria-label={t("header.searchPlaceholder")}
            autoFocus
          />
          <button className="btn-pill" onClick={() => void onRunSearch()} type="button">
            {t("common.search")}
          </button>
          {searchResults.length > 0 && (
            <div className="search-results">
              <div className="search-count">{t("header.searchCount", { n: searchResults.length })}</div>
              {searchResults.map((m, i) => (
                <button
                  key={m.id}
                  className={"search-row" + (i === selIdx ? " active" : "")}
                  onMouseEnter={() => setSelIdx(i)}
                  onClick={() => onPickResult(m)}
                  type="button"
                >
                  {channelOf(m) && <span className="search-chan">{channelOf(m)}</span>}
                  <span className="author">{nameOf(m.sender_subject_id)}</span>{" "}
                  <span className="when">{timeOf(m.created_at)}</span>
                  <div className="snippet">
                    {m.body ? highlight(m.body, searchQ) : t("header.attachmentSnippet")}
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>
      )}
    </Fragment>
  );
}
