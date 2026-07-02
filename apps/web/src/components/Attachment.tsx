import { useEffect, useRef, useState } from "react";
import { apiFetch } from "../lib/api";
import type { AttachmentMeta } from "../lib/chat";
import { formatBytes, isImage } from "../lib/chat";
import { t } from "../lib/i18n";
import { Icon } from "./icons";

// Renders a message attachment: an inline image for image types, a download chip otherwise. New messages
// carry their metadata folded in (`meta`), so only the bytes are fetched; legacy messages (single
// attachment_id, no meta) fall back to the /meta endpoint. The blob needs the bearer header, so it is
// fetched manually into an object URL (revoked on unmount). Clicking an image opens the lightbox when the
// parent provides one.
export function Attachment({
  token,
  id,
  meta: givenMeta,
  onOpenImage,
}: {
  token: string;
  id: string;
  meta?: AttachmentMeta;
  onOpenImage?: (url: string, name: string) => void;
}) {
  const [meta, setMeta] = useState<AttachmentMeta | null>(givenMeta || null);
  const [url, setUrl] = useState("");
  const [failed, setFailed] = useState(false);
  // The bearer changes on the hourly refresh; read it via a ref so a refresh does NOT re-run the fetch (which
  // would swap the object URL out from under an open lightbox). The blob only needs re-fetching when `id` changes.
  const tokenRef = useRef(token);
  tokenRef.current = token;

  useEffect(() => {
    let alive = true;
    let objectUrl = "";
    (async () => {
      try {
        let m = givenMeta || null;
        if (!m) {
          m = await apiFetch<AttachmentMeta>(tokenRef.current, "GET", `/v1/chat/attachments/${id}/meta`);
        }
        if (!alive) return;
        setMeta(m);
        const res = await fetch(`/v1/chat/attachments/${id}`, {
          headers: { Authorization: "Bearer " + tokenRef.current },
        });
        if (!res.ok) throw new Error("attachment " + res.status);
        objectUrl = URL.createObjectURL(await res.blob());
        if (alive) setUrl(objectUrl);
      } catch {
        if (alive) setFailed(true);
      }
    })();
    return () => {
      alive = false;
      if (objectUrl) URL.revokeObjectURL(objectUrl);
    };
    // givenMeta is stable per message render; id identifies the fetch. Token is read via ref (see above).
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [id]);

  if (failed)
    return (
      <span className="att-chip">
        <Icon name="paperclip" size={14} /> {t("attachment.unavailable")}
      </span>
    );
  if (!meta)
    return (
      <span className="att-chip">
        <Icon name="paperclip" size={14} /> {t("attachment.loading")}
      </span>
    );
  if (isImage(meta.content_type) && url) {
    return (
      <img
        className="att-img"
        src={url}
        alt={meta.filename}
        onClick={() => {
          if (onOpenImage) onOpenImage(url, meta.filename);
          else window.open(url, "_blank");
        }}
      />
    );
  }
  const size = typeof meta.size_bytes === "number" ? formatBytes(meta.size_bytes) : "";
  return (
    <a className="att-chip" href={url || undefined} download={meta.filename}>
      <Icon name="paperclip" size={14} /> {meta.filename}
      {size && <span className="att-size">{size}</span>}
    </a>
  );
}
