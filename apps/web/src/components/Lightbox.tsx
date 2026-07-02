import { useEffect } from "react";
import { t } from "../lib/i18n";
import { Icon } from "./icons";

// Minimal image lightbox for attachments: fixed overlay, the image at natural fit, filename + download +
// close in a top bar. Closes on Escape or a backdrop click. The url is an already-authorized object URL
// owned by the Attachment component that opened it.
export function Lightbox({ url, name, onClose }: { url: string; name: string; onClose: () => void }) {
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [onClose]);

  return (
    <div
      className="lightbox"
      onClick={(e) => {
        if (e.target === e.currentTarget) onClose();
      }}
    >
      <div className="lb-top">
        <span className="lb-name">{name}</span>
        <a className="icon-btn lb-btn" href={url} download={name} title={t("lightbox.download")}>
          <Icon name="paperclip" size={16} />
        </a>
        <button className="icon-btn lb-btn" onClick={onClose} type="button" title={t("common.close")}>
          <Icon name="close" size={16} />
        </button>
      </div>
      <img className="lb-img" src={url} alt={name} />
    </div>
  );
}
