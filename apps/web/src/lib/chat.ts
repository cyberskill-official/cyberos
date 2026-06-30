// Shared chat types + small pure helpers, kept out of the components so Chat.tsx stays readable.

export interface Person {
  subject_id: string;
  display_name?: string;
  handle?: string;
  email?: string;
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
