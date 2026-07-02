import { useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { apiFetch } from "../lib/api";
import { fileToAvatarDataUrl } from "../lib/chat";
import { t } from "../lib/i18n";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";
import { useModalA11y } from "./useModalA11y";

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
      setErr(t("profile.readError"));
    }
  }

  async function save() {
    const n = name.trim();
    if (!n) {
      setErr(t("profile.nameRequired"));
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

  const boxRef = useModalA11y(onClose);
  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div
        className="picker"
        style={{ width: 380 }}
        ref={boxRef}
        role="dialog"
        aria-modal="true"
        aria-label={t("profile.title")}
        tabIndex={-1}
      >
        <div className="picker-head">
          <span>{t("profile.title")}</span>
          <button className="icon-btn" onClick={onClose} type="button" title={t("common.close")}>
            <Icon name="close" size={16} />
          </button>
        </div>
        <div className="profile-avatar-row">
          <Avatar id={me} name={name || "?"} size={72} src={avatar} />
          <div className="profile-avatar-actions">
            <button className="btn-ghost" onClick={() => fileRef.current?.click()} type="button">
              {t("profile.uploadPhoto")}
            </button>
            {avatar && (
              <button className="btn-ghost" onClick={() => setAvatar("")} type="button">
                {t("profile.remove")}
              </button>
            )}
            <input ref={fileRef} type="file" accept="image/*" style={{ display: "none" }} onChange={onPick} />
          </div>
        </div>
        <div className="field">
          <label>{t("profile.displayName")}</label>
          <input value={name} onChange={(e) => setName(e.target.value)} maxLength={80} autoFocus />
        </div>
        <div className="err">{err}</div>
        <div className="picker-actions">
          <button className="btn-ghost" onClick={onClose} type="button">
            {t("common.cancel")}
          </button>
          <button className="btn-pill" onClick={() => void save()} disabled={busy} type="button">
            {busy ? t("profile.saving") : t("common.save")}
          </button>
        </div>
      </div>
    </div>
  );
}
