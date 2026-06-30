import { useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { apiFetch } from "../lib/api";
import { fileToAvatarDataUrl } from "../lib/chat";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";

// Edit your own display name + avatar (PATCH /v1/auth/me). The image is downscaled to a small square JPEG
// in the browser before upload, so the stored data URL stays well under the server's size cap.
export function ProfileEditor({
  token,
  me,
  initialName,
  initialAvatar,
  onClose,
  onSaved,
}: {
  token: string;
  me: string;
  initialName: string;
  initialAvatar: string;
  onClose(): void;
  onSaved(name: string, avatar: string): void;
}) {
  const [name, setName] = useState(initialName);
  const [avatar, setAvatar] = useState(initialAvatar);
  const [busy, setBusy] = useState(false);
  const [err, setErr] = useState("");
  const fileRef = useRef<HTMLInputElement | null>(null);

  async function onPick(e: ChangeEvent<HTMLInputElement>) {
    const f = e.target.files && e.target.files[0];
    e.target.value = "";
    if (!f) return;
    setErr("");
    try {
      setAvatar(await fileToAvatarDataUrl(f));
    } catch {
      setErr("Could not read that image.");
    }
  }

  async function save() {
    const n = name.trim();
    if (!n) {
      setErr("Display name cannot be empty.");
      return;
    }
    setBusy(true);
    setErr("");
    try {
      await apiFetch(token, "PATCH", "/v1/auth/me", { display_name: n, avatar });
      onSaved(n, avatar);
      onClose();
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="picker" style={{ width: 380 }}>
        <div className="picker-head">
          <span>Edit profile</span>
          <button className="icon-btn" onClick={onClose} type="button" title="Close">
            <Icon name="close" size={16} />
          </button>
        </div>
        <div className="profile-avatar-row">
          <Avatar id={me} name={name || "?"} size={72} src={avatar} />
          <div className="profile-avatar-actions">
            <button className="btn-ghost" onClick={() => fileRef.current?.click()} type="button">
              Upload photo
            </button>
            {avatar && (
              <button className="btn-ghost" onClick={() => setAvatar("")} type="button">
                Remove
              </button>
            )}
            <input ref={fileRef} type="file" accept="image/*" style={{ display: "none" }} onChange={onPick} />
          </div>
        </div>
        <div className="field">
          <label>Display name</label>
          <input value={name} onChange={(e) => setName(e.target.value)} maxLength={80} autoFocus />
        </div>
        <div className="err">{err}</div>
        <div className="picker-actions">
          <button className="btn-ghost" onClick={onClose} type="button">
            Cancel
          </button>
          <button className="btn-pill" onClick={() => void save()} disabled={busy} type="button">
            {busy ? "Saving..." : "Save"}
          </button>
        </div>
      </div>
    </div>
  );
}
