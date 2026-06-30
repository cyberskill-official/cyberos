import { useEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { useAuth } from "../lib/auth";
import { apiFetch, decodeJwt } from "../lib/api";
import type { Channel, Directory, Message, Person } from "../lib/chat";
import { channelLabel, fileToBase64, nameFor, timeOf } from "../lib/chat";
import { Attachment } from "../components/Attachment";
import { PeoplePicker } from "../components/PeoplePicker";
import type { PickerMode } from "../components/PeoplePicker";
import { ThreadPanel } from "../components/ThreadPanel";

interface WsEvent extends Partial<Message> {
  type: string;
  subject?: string;
  status?: string;
}

export function Chat() {
  const { token } = useAuth();
  const me = useMemo(() => {
    const c = token ? decodeJwt(token) : null;
    return c && typeof c.sub === "string" ? c.sub : "";
  }, [token]);

  const [dirList, setDirList] = useState<Person[]>([]);
  const directory = useMemo<Directory>(() => {
    const d: Directory = {};
    for (const p of dirList) d[p.subject_id] = p;
    return d;
  }, [dirList]);

  const [channels, setChannels] = useState<Channel[]>([]);
  const [activeId, setActiveId] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const [health, setHealth] = useState<"unknown" | "ok" | "bad">("unknown");
  const [picker, setPicker] = useState<PickerMode | null>(null);

  const [presence, setPresence] = useState<Set<string>>(new Set());
  const [typingSubject, setTypingSubject] = useState("");
  const [editingId, setEditingId] = useState("");
  const [editText, setEditText] = useState("");

  const [threadRoot, setThreadRoot] = useState<Message | null>(null);
  const [threadReplies, setThreadReplies] = useState<Message[]>([]);
  const threadRootRef = useRef<Message | null>(null);
  useEffect(() => {
    threadRootRef.current = threadRoot;
  }, [threadRoot]);

  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQ, setSearchQ] = useState("");
  const [searchResults, setSearchResults] = useState<Message[]>([]);

  const scrollRef = useRef<HTMLDivElement | null>(null);
  const fileRef = useRef<HTMLInputElement | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const typingSentAt = useRef(0);
  const typingTimer = useRef<number | null>(null);

  const active = channels.find((c) => c.id === activeId) || null;

  async function reloadChannels(selectId?: string) {
    if (!token) return;
    try {
      const list = await apiFetch<Channel[]>(token, "GET", "/v1/chat/channels");
      setChannels(list || []);
      if (selectId) setActiveId(selectId);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  // Directory (best-effort: names + the people-picker) and the channel list.
  useEffect(() => {
    if (!token) return;
    (async () => {
      try {
        const d = await apiFetch<{ items?: Person[] }>(token, "GET", "/v1/auth/directory");
        setDirList(d.items || []);
      } catch {
        /* directory is best-effort */
      }
    })();
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

  // Reachability dot.
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

  // Per-channel: load timeline + presence, connect the live websocket (messages, edits/deletes, presence,
  // typing). Thread replies route to the open thread.
  useEffect(() => {
    if (!token || !activeId) {
      setMessages([]);
      return;
    }
    setThreadRoot(null);
    setThreadReplies([]);
    setSearchOpen(false);
    setPresence(new Set());
    setTypingSubject("");
    setEditingId("");
    let alive = true;

    (async () => {
      try {
        const msgs = await apiFetch<Message[]>(token, "GET", `/v1/chat/channels/${activeId}/messages`);
        if (alive) setMessages((msgs || []).filter((m) => !m.parent_id));
      } catch (e) {
        if (alive) setError(e instanceof Error ? e.message : String(e));
      }
    })();
    (async () => {
      try {
        const on = await apiFetch<unknown[]>(token, "GET", `/v1/chat/channels/${activeId}/presence`);
        const ids = (on || [])
          .map((x) =>
            typeof x === "string" ? x : ((x as Record<string, string>).subject_id || (x as Record<string, string>).subject || ""),
          )
          .filter(Boolean);
        if (alive) setPresence(new Set(ids));
      } catch {
        /* presence is best-effort */
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
      wsRef.current = sock;
      sock.onmessage = (ev) => {
        let data: WsEvent;
        try {
          data = JSON.parse(ev.data as string) as WsEvent;
        } catch {
          return;
        }
        if (data.type === "message" && data.id) {
          const msg = data as Message;
          if (msg.parent_id) {
            const root = threadRootRef.current;
            if (root && msg.parent_id === root.id) {
              setThreadReplies((prev) => (prev.some((m) => m.id === msg.id) ? prev : [...prev, msg]));
            }
          } else {
            setMessages((prev) => (prev.some((m) => m.id === msg.id) ? prev : [...prev, msg]));
          }
        } else if (data.type === "message_edited" && data.id) {
          const patch = (m: Message): Message =>
            m.id === data.id ? { ...m, body: data.body ?? m.body, edited_at: data.edited_at } : m;
          setMessages((prev) => prev.map(patch));
          setThreadReplies((prev) => prev.map(patch));
        } else if (data.type === "message_deleted" && data.id) {
          setMessages((prev) => prev.filter((m) => m.id !== data.id));
          setThreadReplies((prev) => prev.filter((m) => m.id !== data.id));
        } else if (data.type === "presence" && data.subject) {
          const sub = data.subject;
          setPresence((prev) => {
            const next = new Set(prev);
            if (data.status === "online") next.add(sub);
            else next.delete(sub);
            return next;
          });
        } else if (data.type === "typing" && data.subject) {
          setTypingSubject(data.subject);
          if (typingTimer.current) window.clearTimeout(typingTimer.current);
          typingTimer.current = window.setTimeout(() => setTypingSubject(""), 2500);
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
      wsRef.current = null;
    };
  }, [token, activeId]);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  function onDraftChange(v: string) {
    setDraft(v);
    const ws = wsRef.current;
    const now = Date.now();
    if (ws && ws.readyState === 1 && now - typingSentAt.current > 1500) {
      typingSentAt.current = now;
      try {
        ws.send(JSON.stringify({ type: "typing" }));
      } catch {
        /* socket closing */
      }
    }
  }

  async function postMessage(body: string, attachmentId?: string) {
    if (!active || !token) return;
    const payload: Record<string, unknown> = { body };
    if (attachmentId) payload.attachment_id = attachmentId;
    const m = await apiFetch<Message>(token, "POST", `/v1/chat/channels/${active.id}/messages`, payload);
    setMessages((prev) => (prev.some((x) => x.id === m.id) ? prev : [...prev, m]));
  }

  async function send() {
    const text = draft.trim();
    if (!text || sending) return;
    setSending(true);
    setError("");
    try {
      await postMessage(text);
      setDraft("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSending(false);
    }
  }

  async function onPickFile(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files && e.target.files[0];
    e.target.value = "";
    if (!file || !active || !token) return;
    setError("");
    try {
      const b64 = await fileToBase64(file);
      const att = await apiFetch<{ id: string }>(token, "POST", `/v1/chat/channels/${active.id}/attachments`, {
        filename: file.name,
        content_type: file.type || "application/octet-stream",
        data_base64: b64,
      });
      await postMessage("", att.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function saveEdit(m: Message) {
    const body = editText.trim();
    if (!body || !token) return;
    try {
      const updated = await apiFetch<Message>(
        token,
        "PATCH",
        `/v1/chat/channels/${m.channel_id}/messages/${m.id}`,
        { body },
      );
      setMessages((prev) =>
        prev.map((x) => (x.id === m.id ? { ...x, body: updated.body ?? body, edited_at: updated.edited_at } : x)),
      );
      setEditingId("");
      setEditText("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function deleteMessage(m: Message) {
    if (!token || !window.confirm("Delete this message?")) return;
    try {
      await apiFetch(token, "DELETE", `/v1/chat/channels/${m.channel_id}/messages/${m.id}`);
      setMessages((prev) => prev.filter((x) => x.id !== m.id));
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function createDm(subjectId: string) {
    if (!token) return;
    try {
      const c = await apiFetch<Channel>(token, "POST", "/v1/chat/dms", { subject_id: subjectId });
      await reloadChannels(c.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function createGroup(name: string, ids: string[]) {
    if (!token) return;
    try {
      const c = await apiFetch<Channel>(token, "POST", "/v1/chat/channels", { name });
      for (const id of ids) {
        try {
          await apiFetch(token, "POST", `/v1/chat/channels/${c.id}/members`, { subject_id: id, role: "member" });
        } catch {
          /* best-effort per member */
        }
      }
      await reloadChannels(c.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function addPeople(ids: string[]) {
    if (!active || !token) return;
    for (const id of ids) {
      try {
        await apiFetch(token, "POST", `/v1/chat/channels/${active.id}/members`, { subject_id: id, role: "member" });
      } catch {
        /* best-effort per member */
      }
    }
  }

  async function openThread(m: Message) {
    setThreadRoot(m);
    setThreadReplies([]);
    if (!token) return;
    try {
      const r = await apiFetch<Message[]>(
        token,
        "GET",
        `/v1/chat/channels/${m.channel_id}/messages?parent_id=${m.id}`,
      );
      setThreadReplies(r || []);
    } catch {
      /* leave the panel open with just the root */
    }
  }

  async function threadSend(text: string) {
    if (!threadRoot || !token) return;
    const m = await apiFetch<Message>(token, "POST", `/v1/chat/channels/${threadRoot.channel_id}/messages`, {
      body: text,
      parent_id: threadRoot.id,
    });
    setThreadReplies((prev) => (prev.some((x) => x.id === m.id) ? prev : [...prev, m]));
  }

  async function runSearch() {
    const q = searchQ.trim();
    if (!q || !active || !token) {
      setSearchResults([]);
      return;
    }
    try {
      const rows = await apiFetch<Message[]>(
        token,
        "GET",
        `/v1/chat/channels/${active.id}/search?q=${encodeURIComponent(q)}`,
      );
      setSearchResults(rows || []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  return (
    <div className="chat">
      <input ref={fileRef} type="file" style={{ display: "none" }} onChange={onPickFile} />

      <aside className="sidebar">
        <div className="side-head">
          <span className="title">Channels</span>
          <span className="add">
            <button className="btn-mini" title="New direct message" onClick={() => setPicker("dm")} type="button">
              @
            </button>
            <button className="btn-mini" title="New group channel" onClick={() => setPicker("group")} type="button">
              +
            </button>
          </span>
        </div>
        <div className="channels">
          {channels.length === 0 && (
            <div className="empty" style={{ fontSize: 13, padding: 16 }}>
              No channels yet
            </div>
          )}
          {channels.map((c) => (
            <div
              key={c.id}
              className={"channel-item" + (c.id === activeId ? " active" : "")}
              onClick={() => setActiveId(c.id)}
            >
              <span className="hash">{c.kind === "direct" ? "@" : "#"}</span>
              <span className="cname">{channelLabel(directory, me, c)}</span>
            </div>
          ))}
        </div>
      </aside>

      <section className="main">
        <div className="main-head">
          <span className="chan-title">
            {active ? (active.kind === "direct" ? "@" : "#") + channelLabel(directory, me, active) : "Chat"}
          </span>
          {active && active.kind !== "direct" && presence.size > 0 && (
            <span className="presence" title={[...presence].map((id) => nameFor(directory, me, id)).join(", ")}>
              ● {presence.size} online
            </span>
          )}
          <span className="spacer" />
          {active && (
            <>
              <button className="btn-mini" title="Search this channel" onClick={() => setSearchOpen((s) => !s)} type="button">
                🔍
              </button>
              {active.kind !== "direct" && (
                <button className="btn-mini" title="Add people" onClick={() => setPicker("add")} type="button">
                  ＋
                </button>
              )}
            </>
          )}
          <span className={"dot " + (health === "ok" ? "ok" : health === "bad" ? "bad" : "")} />
          <span className="health">
            {health === "ok" ? "connected" : health === "bad" ? "unreachable" : "..."}
          </span>
        </div>

        {searchOpen && active && (
          <div className="search-bar">
            <input
              value={searchQ}
              onChange={(e) => setSearchQ(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") {
                  e.preventDefault();
                  void runSearch();
                }
              }}
              placeholder="Search messages in this channel"
              autoFocus
            />
            <button className="btn-pill" onClick={() => void runSearch()} type="button">
              Search
            </button>
            {searchResults.length > 0 && (
              <div className="search-results">
                {searchResults.map((m) => (
                  <div key={m.id} className="search-row">
                    <span className="author">{nameFor(directory, me, m.sender_subject_id)}</span>{" "}
                    <span className="when">{timeOf(m.created_at)}</span>
                    <div className="snippet">{m.body || "[attachment]"}</div>
                  </div>
                ))}
              </div>
            )}
          </div>
        )}

        {!active ? (
          <div className="empty">Select or start a conversation.</div>
        ) : (
          <div className="main-row">
            <div className="main-col">
              <div className="messages" ref={scrollRef}>
                {messages.length === 0 && <div className="empty">No messages yet. Say hello.</div>}
                {messages.map((m) => {
                  const mine = m.sender_subject_id === me;
                  return (
                    <div key={m.id} className={"msg" + (mine ? " mine" : "")}>
                      <div className="meta">
                        <span className="author">{nameFor(directory, me, m.sender_subject_id)}</span>{" "}
                        {timeOf(m.created_at)} {m.edited_at ? "(edited)" : ""}
                        <span className="msg-actions">
                          <button className="reply-link" onClick={() => void openThread(m)} type="button">
                            Reply
                          </button>
                          {mine && !m.attachment_id && (
                            <button
                              className="reply-link"
                              onClick={() => {
                                setEditingId(m.id);
                                setEditText(m.body);
                              }}
                              type="button"
                            >
                              Edit
                            </button>
                          )}
                          {mine && (
                            <button className="reply-link" onClick={() => void deleteMessage(m)} type="button">
                              Delete
                            </button>
                          )}
                        </span>
                      </div>
                      {editingId === m.id ? (
                        <div className="edit-row">
                          <input
                            value={editText}
                            onChange={(e) => setEditText(e.target.value)}
                            onKeyDown={(e) => {
                              if (e.key === "Enter") {
                                e.preventDefault();
                                void saveEdit(m);
                              } else if (e.key === "Escape") {
                                setEditingId("");
                                setEditText("");
                              }
                            }}
                            autoFocus
                          />
                          <button className="btn-pill" onClick={() => void saveEdit(m)} type="button">
                            Save
                          </button>
                          <button
                            className="btn-ghost"
                            onClick={() => {
                              setEditingId("");
                              setEditText("");
                            }}
                            type="button"
                          >
                            Cancel
                          </button>
                        </div>
                      ) : (
                        <div className="bubble">
                          {m.attachment_id ? <Attachment token={token!} id={m.attachment_id} /> : m.body}
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>

              {typingSubject && typingSubject !== me && (
                <div className="typing">{nameFor(directory, me, typingSubject)} is typing...</div>
              )}

              {error && (
                <div className="msg">
                  <div className="bubble sys">{error}</div>
                </div>
              )}

              <div className="composer">
                <button className="btn-mini" title="Attach a file" onClick={() => fileRef.current?.click()} type="button">
                  📎
                </button>
                <input
                  value={draft}
                  onChange={(e) => onDraftChange(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && !e.shiftKey) {
                      e.preventDefault();
                      void send();
                    }
                  }}
                  placeholder={"Message " + (active ? channelLabel(directory, me, active) : "")}
                />
                <button onClick={() => void send()} disabled={sending || !draft.trim()} type="button">
                  Send
                </button>
              </div>
            </div>

            {threadRoot && token && (
              <ThreadPanel
                token={token}
                me={me}
                dir={directory}
                root={threadRoot}
                replies={threadReplies}
                onClose={() => setThreadRoot(null)}
                onSend={threadSend}
              />
            )}
          </div>
        )}
      </section>

      {picker && token && (
        <PeoplePicker
          mode={picker}
          people={dirList}
          me={me}
          onClose={() => setPicker(null)}
          onDm={createDm}
          onGroup={createGroup}
          onAdd={addPeople}
        />
      )}
    </div>
  );
}
