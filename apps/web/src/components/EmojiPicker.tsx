import { useEffect, useMemo, useRef, useState } from "react";
import { t } from "../lib/i18n";

// Full emoji picker (richer-messages cluster): search by name, category tabs, a global skin-tone selector,
// and a "frequently used" row - backed by the canonical unicode-emoji-json dataset, loaded lazily so the
// main bundle stays lean (Vite splits the dynamic import into its own chunk, fetched on first open and
// cached module-wide after that). One instance is rendered at a time (owned by Chat), positioned fixed from
// the trigger's rect and clamped to the viewport, so it never clips inside the scrolling message pane.

export interface AnchorRect {
  top: number;
  left: number;
  bottom: number;
  right: number;
}

interface EmojiEntry {
  emoji: string;
  skin_tone_support: boolean;
  name: string;
  slug: string;
}
interface EmojiGroup {
  name: string;
  slug: string;
  emojis: EmojiEntry[];
}

let cachedGroups: EmojiGroup[] | null = null;

async function loadGroups(): Promise<EmojiGroup[]> {
  if (cachedGroups) return cachedGroups;
  const mod = (await import("unicode-emoji-json/data-by-group.json")) as unknown as {
    default?: EmojiGroup[];
  };
  cachedGroups = (mod.default ?? (mod as unknown)) as EmojiGroup[];
  return cachedGroups;
}

// Fitzpatrick modifiers; index 0 = no tone. Applied after the base scalar (before any ZWJ tail), replacing a
// variation selector when present - the same rule emoji-mart applies.
const TONES = ["", "\u{1F3FB}", "\u{1F3FC}", "\u{1F3FD}", "\u{1F3FE}", "\u{1F3FF}"];

export function applyTone(emoji: string, toneIdx: number): string {
  const tone = TONES[toneIdx] || "";
  if (!tone) return emoji;
  // The tone modifier goes on the base scalar (before any ZWJ tail), replacing a variation selector.
  const ZWJ = String.fromCodePoint(0x200d);
  const VS16 = String.fromCodePoint(0xfe0f);
  const zwj = emoji.indexOf(ZWJ);
  const head = zwj === -1 ? emoji : emoji.slice(0, zwj);
  const tail = zwj === -1 ? "" : emoji.slice(zwj);
  const base = head.endsWith(VS16) ? head.slice(0, -1) : head;
  return base + tone + tail;
}

const RECENT_KEY = "cyberos.emojiRecent";
const TONE_KEY = "cyberos.emojiTone";
const RECENT_MAX = 24;

function loadRecent(): string[] {
  try {
    const v = JSON.parse(localStorage.getItem(RECENT_KEY) || "[]");
    return Array.isArray(v) ? v.filter((x) => typeof x === "string").slice(0, RECENT_MAX) : [];
  } catch {
    return [];
  }
}

const W = 324;
const H = 396;

export function EmojiPicker({
  anchor,
  onPick,
  onClose,
}: {
  anchor: AnchorRect;
  onPick: (emoji: string) => void;
  onClose: () => void;
}) {
  const [groups, setGroups] = useState<EmojiGroup[] | null>(cachedGroups);
  const [failed, setFailed] = useState(false);
  const [q, setQ] = useState("");
  const [tab, setTab] = useState(0);
  const [tone, setTone] = useState(() => {
    const t = parseInt(localStorage.getItem(TONE_KEY) || "0", 10);
    return Number.isInteger(t) && t >= 0 && t <= 5 ? t : 0;
  });
  const [recent, setRecent] = useState<string[]>(loadRecent);
  const ref = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    if (groups) return;
    let alive = true;
    loadGroups()
      .then((g) => {
        if (alive) setGroups(g);
      })
      .catch(() => {
        if (alive) setFailed(true);
      });
    return () => {
      alive = false;
    };
  }, [groups]);

  // Close on outside click or Escape (the picker floats over everything else).
  useEffect(() => {
    const onDown = (e: MouseEvent) => {
      const t = e.target as Node | null;
      if (t && ref.current && ref.current.contains(t)) return;
      onClose();
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [onClose]);

  // Fixed position from the trigger, clamped into the viewport; prefer opening upward.
  const vw = typeof window !== "undefined" ? window.innerWidth : 1280;
  const vh = typeof window !== "undefined" ? window.innerHeight : 800;
  const left = Math.min(Math.max(8, anchor.right - W), Math.max(8, vw - W - 8));
  let top = anchor.top - H - 8;
  if (top < 8) top = Math.min(anchor.bottom + 8, Math.max(8, vh - H - 8));

  const query = q.trim().toLowerCase();
  const results = useMemo(() => {
    if (!groups || !query) return [];
    const out: EmojiEntry[] = [];
    for (const g of groups) {
      for (const e of g.emojis) {
        if (e.name.toLowerCase().includes(query) || e.slug.includes(query)) {
          out.push(e);
          if (out.length >= 120) return out;
        }
      }
    }
    return out;
  }, [groups, query]);

  function pick(e: EmojiEntry) {
    const chosen = e.skin_tone_support ? applyTone(e.emoji, tone) : e.emoji;
    setRecent((prev) => {
      const next = [chosen, ...prev.filter((x) => x !== chosen)].slice(0, RECENT_MAX);
      try {
        localStorage.setItem(RECENT_KEY, JSON.stringify(next));
      } catch {
        /* storage full/blocked - recents just do not persist */
      }
      return next;
    });
    onPick(chosen);
  }

  function pickTone(i: number) {
    setTone(i);
    try {
      localStorage.setItem(TONE_KEY, String(i));
    } catch {
      /* best-effort */
    }
  }

  const cell = (e: EmojiEntry) => (
    <button
      key={e.slug}
      type="button"
      className="ef-cell"
      title={e.name}
      onClick={() => pick(e)}
    >
      {e.skin_tone_support ? applyTone(e.emoji, tone) : e.emoji}
    </button>
  );

  const active = groups && groups.length > 0 ? groups[Math.min(tab, groups.length - 1)] : null;

  return (
    <div className="emoji-full" ref={ref} style={{ left, top, width: W, height: H }}>
      <div className="ef-head">
        <input
          className="ef-search"
          placeholder={t("emoji.search")}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          autoFocus
        />
        <div className="ef-tones" title={t("emoji.skinTone")}>
          {TONES.map((_, i) => (
            <button
              key={i}
              type="button"
              className={"ef-tone" + (i === tone ? " on" : "")}
              onClick={() => pickTone(i)}
            >
              {applyTone("\u{270B}", i)}
            </button>
          ))}
        </div>
      </div>

      {!groups && !failed && <div className="ef-note">{t("emoji.loading")}</div>}
      {failed && <div className="ef-note">{t("emoji.loadFailed")}</div>}

      {groups && !query && (
        <div className="ef-tabs">
          {groups.map((g, i) => (
            <button
              key={g.slug}
              type="button"
              className={"ef-tab" + (i === tab ? " on" : "")}
              title={g.name}
              onClick={() => setTab(i)}
            >
              {g.emojis[0]?.emoji || "?"}
            </button>
          ))}
        </div>
      )}

      {groups && (
        <div className="ef-body">
          {query ? (
            results.length > 0 ? (
              <div className="ef-grid">{results.map(cell)}</div>
            ) : (
              <div className="ef-note">{t("emoji.noMatch", { q })}</div>
            )
          ) : (
            <>
              {recent.length > 0 && (
                <>
                  <div className="ef-sec">{t("emoji.frequent")}</div>
                  <div className="ef-grid">
                    {recent.map((r, i) => (
                      <button
                        key={`${r}.${i}`}
                        type="button"
                        className="ef-cell"
                        onClick={() => {
                          setRecent((prev) => {
                            const next = [r, ...prev.filter((x) => x !== r)].slice(0, RECENT_MAX);
                            try {
                              localStorage.setItem(RECENT_KEY, JSON.stringify(next));
                            } catch {
                              /* best-effort */
                            }
                            return next;
                          });
                          onPick(r);
                        }}
                      >
                        {r}
                      </button>
                    ))}
                  </div>
                </>
              )}
              {active && (
                <>
                  <div className="ef-sec">{active.name}</div>
                  <div className="ef-grid">{active.emojis.map(cell)}</div>
                </>
              )}
            </>
          )}
        </div>
      )}
    </div>
  );
}
