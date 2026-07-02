// Shared chat types + small pure helpers, kept out of the components so the pages stay readable.

export interface Person {
  subject_id: string;
  display_name?: string;
  handle?: string;
  email?: string;
  avatar?: string | null;
}
export interface Channel {
  id: string;
  name?: string;
  kind?: string;
  other_subject_id?: string;
  /// Channel management (find-and-organize cluster): purpose line, private|public, archived marker.
  topic?: string;
  visibility?: string;
  archived_at?: string | null;
}
// A folded reaction on a message: the emoji, how many people used it, and whether I am one of them.
export interface ReactionSummary {
  emoji: string;
  count: number;
  mine: boolean;
}
// Attachment metadata as folded into a message by the server (multi-file), so rendering needs no extra
// round-trip; the bytes themselves are fetched on demand by id.
export interface AttachmentMeta {
  id: string;
  filename: string;
  content_type: string;
  size_bytes: number;
}
export interface Message {
  id: string;
  channel_id: string;
  sender_subject_id: string;
  body: string;
  parent_id?: string | null;
  attachment_id?: string | null;
  edited_at?: string | null;
  deleted_at?: string | null;
  created_at?: string;
  reactions?: ReactionSummary[];
  attachments?: AttachmentMeta[];
}
export interface ReadMarker {
  subject_id: string;
  last_read_message_id: string;
  last_read_at: string;
}
export type Directory = Record<string, Person>;

export const shortId = (id: string): string => (id ? id.slice(0, 8) : "?");

// The server returns message pages newest-first (DESC); the timeline renders oldest-first and appends live
// messages at the end. Every fetch goes through this so history is never rendered reversed.
export function sortMessagesAsc(list: Message[]): Message[] {
  return [...list].sort((a, b) => Date.parse(a.created_at || "") - Date.parse(b.created_at || ""));
}

// The fixed reaction set the picker offers. Kept small and self-contained (no external emoji library).
export const REACTION_EMOJIS = ["\u{1F44D}", "❤️", "\u{1F602}", "\u{1F389}", "✅", "\u{1F440}"];

// Fold a reaction change (one subject added/removed one emoji) into a message's reaction list, from the
// caller's point of view. `isMe` says whether the acting subject is the current user, so `mine` stays correct.
// Pure and order-stable: a new emoji is appended; an emoji whose count hits zero is dropped.
export function applyReaction(
  list: ReactionSummary[] | undefined,
  emoji: string,
  added: boolean,
  isMe: boolean,
): ReactionSummary[] {
  const next = (list || []).map((r) => ({ ...r }));
  const i = next.findIndex((r) => r.emoji === emoji);
  if (added) {
    if (i === -1) {
      next.push({ emoji, count: 1, mine: isMe });
    } else {
      next[i].count += 1;
      if (isMe) next[i].mine = true;
    }
  } else if (i !== -1) {
    next[i].count -= 1;
    if (isMe) next[i].mine = false;
    if (next[i].count <= 0) next.splice(i, 1);
  }
  return next;
}

// Display name from the directory: a real name/handle when known, else a short id. "You" for self.
export function nameFor(dir: Directory, me: string, id: string): string {
  if (!id) return "?";
  if (id === me) return "You";
  const p = dir[id];
  return (p && (p.display_name || p.handle)) || shortId(id);
}

// A channel's label: a direct message shows the other person's name; a group shows its name.
export function channelLabel(dir: Directory, me: string, c: Channel): string {
  if (c.kind === "direct") {
    return c.other_subject_id ? nameFor(dir, me, c.other_subject_id) : "Direct message";
  }
  return c.name || shortId(c.id);
}

export function timeOf(iso?: string): string {
  if (!iso) return "";
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return "";
  return new Date(t).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

const ymd = (d: Date): string => `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;

// A stable per-day key (used to detect day boundaries between messages).
export function dayKey(iso?: string): string {
  if (!iso) return "";
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return "";
  return ymd(new Date(t));
}

// "Today" / "Yesterday" / "Mon, Jun 30" for the day separators in the timeline.
export function formatDay(iso?: string): string {
  if (!iso) return "";
  const t = Date.parse(iso);
  if (Number.isNaN(t)) return "";
  const d = new Date(t);
  const today = new Date();
  const yest = new Date();
  yest.setDate(today.getDate() - 1);
  if (ymd(d) === ymd(today)) return "Today";
  if (ymd(d) === ymd(yest)) return "Yesterday";
  return d.toLocaleDateString([], { weekday: "short", month: "short", day: "numeric" });
}

// One or two initials from a display name (or handle/id), for avatars.
export function initialsOf(name: string): string {
  const n = (name || "").trim();
  if (!n || n === "?") return "?";
  const parts = n.replace(/^@/, "").split(/\s+/).filter(Boolean);
  if (parts.length >= 2) return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  const w = parts[0] || "?";
  return (w.length >= 2 ? w.slice(0, 2) : w).toUpperCase();
}

// Deterministic avatar color from a stable seed (the subject id), tuned to sit on the dark theme.
export function avatarColor(seed: string): string {
  let h = 0;
  for (let i = 0; i < seed.length; i++) h = (Math.imul(h, 31) + seed.charCodeAt(i)) >>> 0;
  return `hsl(${h % 360}, 52%, 42%)`;
}

export const isImage = (ct: string): boolean => /^image\//.test(ct);

// Human-readable byte size, e.g. 820 B / 14.6 KB / 4.7 MB. Kept simple and locale-neutral.
export function formatBytes(n: number): string {
  if (!Number.isFinite(n) || n < 0) return "";
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v < 10 ? v.toFixed(1) : Math.round(v)} ${units[i]}`;
}

// Read a File as raw base64 (no data: prefix), the shape the attachments endpoint wants.
export function fileToBase64(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const r = new FileReader();
    r.onload = () => resolve(String(r.result).split(",")[1] || "");
    r.onerror = () => reject(new Error("read failed"));
    r.readAsDataURL(file);
  });
}

// Downscale + center-crop an image File to a square avatar, returning a small JPEG data URL.
export function fileToAvatarDataUrl(file: File, size = 256): Promise<string> {
  return new Promise((resolve, reject) => {
    const url = URL.createObjectURL(file);
    const img = new Image();
    img.onload = () => {
      URL.revokeObjectURL(url);
      const canvas = document.createElement("canvas");
      canvas.width = size;
      canvas.height = size;
      const ctx = canvas.getContext("2d");
      if (!ctx) {
        reject(new Error("canvas unavailable"));
        return;
      }
      const s = Math.min(img.width, img.height);
      const sx = (img.width - s) / 2;
      const sy = (img.height - s) / 2;
      ctx.drawImage(img, sx, sy, s, s, 0, 0, size, size);
      resolve(canvas.toDataURL("image/jpeg", 0.85));
    };
    img.onerror = () => {
      URL.revokeObjectURL(url);
      reject(new Error("image load failed"));
    };
    img.src = url;
  });
}
