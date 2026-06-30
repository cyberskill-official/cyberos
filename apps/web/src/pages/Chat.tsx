import { useEffect, useMemo, useRef, useState } from "react";
import { useAuth } from "../lib/auth";
import { apiFetch, decodeJwt } from "../lib/api";

interface Channel {
  id: string;
  name?: string;
  kind?: string;
}
interface Message {
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
// A live websocket frame: a tagged event with the message fields flattened onto it.
type WsEvent = Partial<Message> & { type: string; subject?: string; status?: string };

const shortId = (id: string) => (id ? id.slice(0, 8) : "?");

function timeOf(m: Message): string {
  const t = m.created_at ? Date.parse(m.created_at) : NaN;
  if (Number.isNaN(t)) return "";
  return new Date(t).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

export function Chat() {
  const { token } = useAuth();
  const me = useMemo(() => {
    const c = token ? decodeJwt(token) : null;
    return c && typeof c.sub === "string" ? c.sub : "";
  }, [token]);

  const [channels, setChannels] = useState<Channel[]>([]);
  const [activeId, setActiveId] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [health, setHealth] = useState<"unknown" | "ok" | "bad">("unknown");
  const [error, setError] = useState("");
  const [sending, setSending] = useState(false);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  const active = channels.find((c) => c.id === activeId) || null;

  // Load the channel list once signed in; open the first channel by default.
  useEffect(() => {
    if (!token) return;
    (async () => {
      try {
        const list = await apiFetch<Channel[]>(token, "GET", "/v1/chat/channels");
        setChannels(list || []);
        setActiveId((cur) => cur || (list && list.length ? list[0].id : ""));
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [token]);

  // Reachability dot (the /healthz route Caddy proxies to chat).
  useEffect(() => {
    let alive = true;
    const ping = async () => {
      try {
        const r = await fetch("/healthz");
        if (alive) setHealth(r.ok ? "ok" : "bad");
      } catch {
        if (alive) setHealth("bad");
      }
    };
    void ping();
    const iv = window.setInterval(ping, 20000);
    return () => {
      alive = false;
      window.clearInterval(iv);
    };
  }, []);

  // On channel change: load the main timeline and (re)connect the live websocket. Thread replies (those
  // with a parent_id) are excluded from the main list for this MVP.
  useEffect(() => {
    if (!token || !activeId) {
      setMessages([]);
      return;
    }
    let alive = true;
    (async () => {
      try {
        const msgs = await apiFetch<Message[]>(token, "GET", `/v1/chat/channels/${activeId}/messages`);
        if (alive) setMessages((msgs || []).filter((m) => !m.parent_id));
      } catch (e) {
        if (alive) setError(e instanceof Error ? e.message : String(e));
      }
    })();

    let stopped = false;
    let sock: WebSocket | null = null;
    const connect = () => {
      if (stopped) return;
      const url =
        location.origin.replace(/^http/, "ws") +
        `/v1/chat/ws?channel=${encodeURIComponent(activeId)}&access_token=${encodeURIComponent(token)}`;
      sock = new WebSocket(url);
      sock.onmessage = (ev) => {
        let data: WsEvent;
        try {
          data = JSON.parse(ev.data as string) as WsEvent;
        } catch {
          return;
        }
        if (data.type === "message" && data.id && !data.parent_id) {
          const msg = data as Message;
          setMessages((prev) => (prev.some((m) => m.id === msg.id) ? prev : [...prev, msg]));
        } else if (data.type === "message_edited" && data.id) {
          setMessages((prev) =>
            prev.map((m) => (m.id === data.id ? { ...m, body: data.body ?? m.body, edited_at: data.edited_at } : m)),
          );
        } else if (data.type === "message_deleted" && data.id) {
          setMessages((prev) => prev.filter((m) => m.id !== data.id));
        }
      };
      sock.onclose = () => {
        if (!stopped) window.setTimeout(connect, 1500);
      };
    };
    connect();
    return () => {
      stopped = true;
      if (sock) {
        try {
          sock.close();
        } catch {
          /* already closed */
        }
      }
    };
  }, [token, activeId]);

  // Keep the timeline pinned to the newest message.
  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  async function send() {
    const text = draft.trim();
    if (!text || !active || !token) return;
    setSending(true);
    setError("");
    try {
      const m = await apiFetch<Message>(token, "POST", `/v1/chat/channels/${active.id}/messages`, { body: text });
      setMessages((prev) => (prev.some((x) => x.id === m.id) ? prev : [...prev, m]));
      setDraft("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSending(false);
    }
  }

  async function createChannel() {
    const name = window.prompt("New channel name");
    if (!name || !name.trim() || !token) return;
    try {
      const c = await apiFetch<Channel>(token, "POST", "/v1/chat/channels", { name: name.trim() });
      setChannels((prev) => [...prev, c]);
      setActiveId(c.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="chat">
      <aside className="sidebar">
        <div className="side-head">
          <span className="title">Channels</span>
          <span className="add">
            <button className="btn-mini" title="New channel" onClick={createChannel} type="button">
              +
            </button>
          </span>
        </div>
        <div className="channels">
          {channels.length === 0 && <div className="empty" style={{ fontSize: 13, padding: 16 }}>No channels yet</div>}
          {channels.map((c) => (
            <div
              key={c.id}
              className={"channel-item" + (c.id === activeId ? " active" : "")}
              onClick={() => setActiveId(c.id)}
            >
              <span className="hash">{c.kind === "direct" ? "@" : "#"}</span>
              <span className="cname">{c.name || shortId(c.id)}</span>
            </div>
          ))}
        </div>
      </aside>

      <section className="main">
        <div className="main-head">
          <span className="chan-title">
            {active ? (active.kind === "direct" ? "@" : "#") + (active.name || shortId(active.id)) : "Chat"}
          </span>
          <span className="spacer" />
          <span className={"dot " + (health === "ok" ? "ok" : health === "bad" ? "bad" : "")} />
          <span className="health">
            {health === "ok" ? "connected" : health === "bad" ? "unreachable" : "..."}
          </span>
        </div>

        {!active ? (
          <div className="empty">Select or create a channel to start.</div>
        ) : (
          <>
            <div className="messages" ref={scrollRef}>
              {messages.map((m) => (
                <div key={m.id} className={"msg" + (m.sender_subject_id === me ? " mine" : "")}>
                  <div className="meta">
                    <span className="author">{m.sender_subject_id === me ? "You" : shortId(m.sender_subject_id)}</span>{" "}
                    {timeOf(m)} {m.edited_at ? "(edited)" : ""}
                  </div>
                  <div className="bubble">{m.body}</div>
                </div>
              ))}
              {messages.length === 0 && <div className="empty">No messages yet. Say hello.</div>}
            </div>

            {error && <div className="msg"><div className="bubble sys">{error}</div></div>}

            <div className="composer">
              <input
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" && !e.shiftKey) {
                    e.preventDefault();
                    void send();
                  }
                }}
                placeholder={"Message " + (active.name ? "#" + active.name : "channel")}
              />
              <button onClick={() => void send()} disabled={sending || !draft.trim()} type="button">
                Send
              </button>
            </div>
          </>
        )}
      </section>
    </div>
  );
}
