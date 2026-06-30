import { useEffect, useState } from "react";
import { apiFetch } from "../lib/api";
import { formatBytes, isImage } from "../lib/chat";
import { Icon } from "./icons";

interface Meta {
  content_type: string;
  filename: string;
  size_bytes?: number;
}

// Renders a message attachment by id: an inline image for image types, a download chip otherwise. The blob
// itself needs the bearer header, so it is fetched manually into an object URL (revoked on unmount).
export function Attachment({ token, id }: { token: string; id: string }) {
  const [meta, setMeta] = useState<Meta | null>(null);
  const [url, setUrl] = useState("");
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    let alive = true;
    let objectUrl = "";
    (async () => {
      try {
        const m = await apiFetch<Meta>(token, "GET", `/v1/chat/attachments/${id}/meta`);
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
    return <img className="att-img" src={url} alt={meta.filename} onClick={() => window.open(url, "_blank")} />;
  }
  const size = typeof meta.size_bytes === "number" ? formatBytes(meta.size_bytes) : "";
  return (
    <a className="att-chip" href={url || undefined} download={meta.filename}>
      <Icon name="paperclip" size={14} /> {meta.filename}
      {size && <span className="att-size">{size}</span>}
    </a>
  );
}
