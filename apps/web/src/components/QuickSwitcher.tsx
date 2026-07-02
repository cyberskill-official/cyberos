import { useEffect, useMemo, useState } from "react";
import type { Channel, Directory, Person } from "../lib/chat";
import { channelLabel } from "../lib/chat";
import { t } from "../lib/i18n";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";

type Item =
  | { kind: "channel"; id: string; label: string; dm: boolean; otherId: string }
  | { kind: "person"; subjectId: string; label: string; handle: string };

// Cmd/Ctrl+K quick switcher: type to jump to any channel, DM, or teammate (picking a teammate opens or
// starts their DM). Keyboard-first - arrows move, Enter selects, Escape closes - so switching is one gesture.
export function QuickSwitcher({
  channels,
  directory,
  me,
  people,
  onClose,
  onSelectChannel,
  onStartDm,
}: {
  channels: Channel[];
  directory: Directory;
  me: string;
  people: Person[];
  onClose(): void;
  onSelectChannel(id: string): void;
  onStartDm(subjectId: string): void;
}) {
  const [q, setQ] = useState("");
  const [idx, setIdx] = useState(0);

  const items = useMemo<Item[]>(() => {
    const chans: Item[] = channels.map((c) => ({
      kind: "channel",
      id: c.id,
      dm: c.kind === "direct",
      otherId: c.other_subject_id || "",
      label: channelLabel(directory, me, c),
    }));
    const persons: Item[] = people
      .filter((p) => p.subject_id !== me)
      .map((p) => ({
        kind: "person",
        subjectId: p.subject_id,
        handle: p.handle || "",
        label: p.display_name || p.handle || "",
      }));
    const all = [...chans, ...persons].filter((it) => it.label);
    const f = q.trim().toLowerCase();
    if (!f) return all.slice(0, 8);
    const scored = all
      .map((it) => {
        const l = it.label.toLowerCase();
        const h = it.kind === "person" ? it.handle.toLowerCase() : "";
        let score = -1;
        if (l.startsWith(f) || (h && h.startsWith(f))) score = 0;
        else if (l.includes(f) || (h && h.includes(f))) score = 1;
        return { it, score };
      })
      .filter((x) => x.score >= 0)
      .sort((a, b) => a.score - b.score);
    return scored.slice(0, 8).map((x) => x.it);
  }, [channels, people, directory, me, q]);

  useEffect(() => {
    setIdx(0);
  }, [q]);

  function choose(it: Item) {
    if (it.kind === "channel") onSelectChannel(it.id);
    else onStartDm(it.subjectId);
    onClose();
  }

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="picker switcher" role="dialog" aria-modal="true" aria-label={t("switch.title")}>
        <input
          className="picker-input"
          placeholder={t("switch.placeholder")}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "ArrowDown") {
              e.preventDefault();
              setIdx((i) => (items.length ? (i + 1) % items.length : 0));
            } else if (e.key === "ArrowUp") {
              e.preventDefault();
              setIdx((i) => (items.length ? (i - 1 + items.length) % items.length : 0));
            } else if (e.key === "Enter") {
              e.preventDefault();
              if (items[idx]) choose(items[idx]);
            } else if (e.key === "Escape") {
              e.preventDefault();
              onClose();
            }
          }}
          autoFocus
        />
        <div className="picker-people switcher-list">
          {items.length === 0 && (
            <div className="side-empty" style={{ padding: 14 }}>
              {t("switch.noMatch")}
            </div>
          )}
          {items.map((it, i) => {
            const avatarId = it.kind === "channel" ? it.otherId : it.subjectId;
            return (
              <button
                key={it.kind === "channel" ? "c" + it.id : "p" + it.subjectId}
                className={"switch-row" + (i === idx ? " active" : "")}
                onMouseEnter={() => setIdx(i)}
                onClick={() => choose(it)}
                type="button"
              >
                {it.kind === "channel" && !it.dm ? (
                  <span className="chan-hash">
                    <Icon name="hash" size={16} />
                  </span>
                ) : (
                  <Avatar id={avatarId} name={it.label} size={24} src={directory[avatarId]?.avatar || ""} />
                )}
                <span className="switch-label">{it.label}</span>
                {it.kind === "person" && <span className="switch-hint">{t("switch.dmHint")}</span>}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
