import { useEffect, useState } from "react";
import { apiFetch } from "../lib/api";
import { isImage } from "../lib/chat";

interface Meta {
  content_type: string;
  filename: string;
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

  if (failed) return <span className="att-chip">📎 attachment unavailable</span>;
  if (!meta) return <span className="att-chip">📎 loading...</span>;
  if (isImage(meta.content_type) && url) {
    return (
      <img className="att-img" src={url} alt={meta.filename} onClick={() => window.open(url, "_blank")} />
    );
  }
  return (
    <a className="att-chip" href={url || undefined} download={meta.filename}>
      📎 {meta.filename}
    </a>
  );
}
