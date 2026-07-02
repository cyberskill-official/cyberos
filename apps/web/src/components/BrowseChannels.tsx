import { useEffect, useMemo, useState } from "react";
import { apiFetch } from "../lib/api";
import { Icon } from "./icons";

interface BrowseRow {
  id: string;
  name: string;
  topic: string;
  member_count: number;
  is_member: boolean;
}

// The channel browser (find-and-organize cluster): every public, live channel on the team, with member
// counts and one-click join. Private channels never appear here - that is what private means.
export function BrowseChannels({
  token,
  onClose,
  onJoined,
  onOpen,
}: {
  token: string;
  onClose(): void;
  /// Fired after a successful join (the parent refreshes its list and opens the channel).
  onJoined(channelId: string): void;
  /// Open a channel the caller already belongs to.
  onOpen(channelId: string): void;
}) {
  const [rows, setRows] = useState<BrowseRow[] | null>(null);
  const [q, setQ] = useState("");
  const [err, setErr] = useState("");
  const [joining, setJoining] = useState("");

  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const r = await apiFetch<BrowseRow[]>(token, "GET", "/v1/chat/channels/browse");
        if (alive) setRows(r || []);
      } catch (e) {
        if (alive) {
          setErr(e instanceof Error ? e.message : String(e));
          setRows([]);
        }
      }
    })();
    return () => {
      alive = false;
    };
  }, [token]);

  const list = useMemo(() => {
    const f = q.trim().toLowerCase();
    if (!rows) return [];
    if (!f) return rows;
    return rows.filter((r) => (r.name + " " + r.topic).toLowerCase().includes(f));
  }, [rows, q]);

  async function join(id: string) {
    setJoining(id);
    setErr("");
    try {
      await apiFetch(token, "POST", `/v1/chat/channels/${id}/join`);
      onClose();
      onJoined(id);
    } catch (e) {
      setErr(e instanceof Error ? e.message : String(e));
    } finally {
      setJoining("");
    }
  }

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="picker">
        <div className="picker-head">
          <span>Browse channels</span>
          <button className="icon-btn" onClick={onClose} type="button" title="Close">
            <Icon name="close" size={16} />
          </button>
        </div>
        <input
          className="picker-input"
          placeholder="Filter channels"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          autoFocus
        />
        <div className="picker-people">
          {rows === null && <div className="side-empty" style={{ padding: 16 }}>Loading...</div>}
          {rows !== null && list.length === 0 && (
            <div className="side-empty" style={{ padding: 16 }}>
              {rows.length === 0 ? "No public channels yet" : "No channels match"}
            </div>
          )}
          {list.map((r) => (
            <div key={r.id} className="person browse-row">
              <span className="chan-hash">
                <Icon name="hash" size={16} />
              </span>
              <div className="person-meta">
                <span className="pname">{r.name}</span>
                <span className="psub">
                  {r.member_count} member{r.member_count === 1 ? "" : "s"}
                  {r.topic ? ` · ${r.topic}` : ""}
                </span>
              </div>
              {r.is_member ? (
                <button className="btn-ghost" onClick={() => (onClose(), onOpen(r.id))} type="button">
                  Open
                </button>
              ) : (
                <button
                  className="btn-pill"
                  onClick={() => void join(r.id)}
                  disabled={joining === r.id}
                  type="button"
                >
                  Join
                </button>
              )}
            </div>
          ))}
        </div>
        {err && <div className="banner err">{err}</div>}
      </div>
    </div>
  );
}
