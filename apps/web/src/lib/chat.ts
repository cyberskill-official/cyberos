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
}
export interface ReadMarker {
  subject_id: string;
  last_read_message_id: string;
  last_read_at: string;
}
export type Directory = Record<string, Person>;

export const shortId = (id: string): string => (id ? id.slice(0, 8) : "?");

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
