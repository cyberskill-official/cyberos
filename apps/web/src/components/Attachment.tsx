import { useEffect, useState } from "react";
import { apiFetch } from "../lib/api";
import type { AttachmentMeta } from "../lib/chat";
import { formatBytes, isImage } from "../lib/chat";
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

  useEffect(() => {
    let alive = true;
    let objectUrl = "";
    (async () => {
      try {
        let m = givenMeta || null;
        if (!m) {
          m = await apiFetch<AttachmentMeta>(token, "GET", `/v1/chat/attachments/${id}/meta`);
        }
        if (!alive) return;
        setMeta(m);
        const res = await fetch(`/v1/chat/attachments/${id}`, {
          headers: { Authorization: "Bearer " + token },
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
    // givenMeta is stable per message render; id identifies the fetch.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, id]);

  if (failed)
    return (
      <span className="att-chip">
        <Icon name="paperclip" size={14} /> attachment unavailable
      </span>
    );
  if (!meta)
    return (
      <span className="att-chip">
        <Icon name="paperclip" size={14} /> loading...
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
