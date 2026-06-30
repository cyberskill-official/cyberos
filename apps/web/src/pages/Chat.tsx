import { Fragment, useEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { useAuth } from "../lib/auth";
import { apiFetch, decodeJwt } from "../lib/api";
import type { Channel, Directory, Message, Person, ReadMarker } from "../lib/chat";
import { applyReaction, channelLabel, dayKey, fileToBase64, formatBytes, formatDay, isImage, nameFor, REACTION_EMOJIS, shortId, timeOf } from "../lib/chat";
import { Avatar } from "../components/Avatar";
import { Icon } from "../components/icons";
import { Attachment } from "../components/Attachment";
import { PeoplePicker } from "../components/PeoplePicker";
import type { PickerMode } from "../components/PeoplePicker";
import { ThreadPanel } from "../components/ThreadPanel";
import { CallOverlay } from "../components/CallOverlay";
import { useCall } from "../lib/call";
import { ProfileEditor } from "../components/ProfileEditor";

interface WsEvent extends Partial<Message> {
  type: string;
  subject?: string;
  status?: string;
  from?: string;
  to?: string;
  data?: unknown;
  last_read_message_id?: string;
  // reaction_changed
  message_id?: string;
  emoji?: string;
  added?: boolean;
}

const GROUP_WINDOW_MS = 5 * 60 * 1000;
// Client-side guard mirroring the server's attachment cap (5 MB). Keep this in sync with the service.
const MAX_ATTACH_BYTES = 5 * 1024 * 1024;

export function Chat() {
  const { token, email } = useAuth();
  const me = useMemo(() => {
    const c = token ? decodeJwt(token) : null;
    return c && typeof c.sub === "string" ? c.sub : "";
  }, [token]);

  // My own editable profile (display name + avatar) from GET /v1/auth/me.
  const [meProfile, setMeProfile] = useState<{ display_name?: string | null; avatar?: string | null } | null>(null);
  const [profileOpen, setProfileOpen] = useState(false);

  // A friendly name for self: the saved display name, else the email local-part.
  const selfName = useMemo(() => {
    const fromProfile = (meProfile?.display_name || "").trim();
    if (fromProfile) return fromProfile;
    const local = (email || "").split("@")[0];
    const pretty = local
      .split(/[._-]+/)
      .filter(Boolean)
      .map((w) => w[0].toUpperCase() + w.slice(1))
      .join(" ");
    return pretty || "You";
  }, [email, meProfile]);
  const myAvatar = meProfile?.avatar || "";

  const [dirList, setDirList] = useState<Person[]>([]);
  const directory = useMemo<Directory>(() => {
    const d: Directory = {};
    for (const p of dirList) d[p.subject_id] = p;
    return d;
  }, [dirList]);

  // Resolve any subject id to a display name (self -> selfName, else directory, else short id).
  const nameOf = useMemo(() => {
    return (id: string): string => {
      if (id && id === me) return selfName;
      const p = directory[id];
      return (p && (p.display_name || p.handle)) || shortId(id);
    };
  }, [directory, me, selfName]);

  // Avatar image for a subject: my own from my profile, others from the directory ("" -> initials).
  const avatarSrc = useMemo(() => {
    return (id: string): string => (id && id === me ? myAvatar : directory[id]?.avatar || "");
  }, [directory, me, myAvatar]);

  const [channels, setChannels] = useState<Channel[]>([]);
  const [activeId, setActiveId] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [unread, setUnread] = useState<Record<string, number>>({});
  // Recent-activity timestamps keyed by channel id, used to sort the DM list. Pure client state: it is fed by
  // the unread poll (a channel with unread is treated as recently active) and by inbound `message` ws events.
  const [lastActivity, setLastActivity] = useState<Record<string, number>>({});
  const [receipts, setReceipts] = useState<Record<string, string>>({});
  const [draft, setDraft] = useState("");
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const [health, setHealth] = useState<"unknown" | "ok" | "bad">("unknown");
  const [picker, setPicker] = useState<PickerMode | null>(null);
  const [pendingVideo, setPendingVideo] = useState(false);

  const [presence, setPresence] = useState<Set<string>>(new Set());
  const [typingSubject, setTypingSubject] = useState("");
  const [editingId, setEditingId] = useState("");
  const [editText, setEditText] = useState("");

  // Emoji reactions: which message's picker is open (only one at a time). Translations: a per-message cache of
  // the inline result, and the set of message ids whose translation is in flight. A second translate click
  // hides the cached result (removes the key).
  const [reactPickerId, setReactPickerId] = useState("");
  const [translations, setTranslations] = useState<Record<string, string>>({});
  const [translating, setTranslating] = useState<Set<string>>(new Set());
  const [translateError, setTranslateError] = useState<Set<string>>(new Set());

  const [threadRoot, setThreadRoot] = useState<Message | null>(null);
  const [threadReplies, setThreadReplies] = useState<Message[]>([]);
  const threadRootRef = useRef<Message | null>(null);
  useEffect(() => {
    threadRootRef.current = threadRoot;
  }, [threadRoot]);

  const [searchOpen, setSearchOpen] = useState(false);
  const [searchQ, setSearchQ] = useState("");
  const [searchResults, setSearchResults] = useState<Message[]>([]);

  // Composer attachment staging: a file is held here (with an image preview URL) until the user presses Send,
  // at which point it is uploaded and posted. `uploading` shows the in-flight state; `dragOver` highlights
  // the message pane during a drag.
  const [staged, setStaged] = useState<File | null>(null);
  const [stagedPreview, setStagedPreview] = useState("");
  const [uploading, setUploading] = useState(false);
  const [dragOver, setDragOver] = useState(false);

  const scrollRef = useRef<HTMLDivElement | null>(null);
  const fileRef = useRef<HTMLInputElement | null>(null);
  const taRef = useRef<HTMLTextAreaElement | null>(null);
  const wsRef = useRef<WebSocket | null>(null);
  const typingSentAt = useRef(0);
  const typingTimer = useRef<number | null>(null);

  const active = channels.find((c) => c.id === activeId) || null;

  // Calls: send signaling over the active channel websocket; route inbound signal events into the engine.
  const sendSignal = (to: string, data: unknown) => {
    const ws = wsRef.current;
    if (ws && ws.readyState === 1) {
      try {
        ws.send(JSON.stringify({ type: "signal", to, data }));
      } catch {
        /* socket closing */
      }
    }
  };
  const call = useCall(sendSignal);
  const callRef = useRef(call);
  callRef.current = call;

  async function refreshUnread(list: Channel[]) {
    if (!token) return;
    const entries = await Promise.all(
      list.map(async (c) => {
        try {
          const u = await apiFetch<{ unread: number }>(token, "GET", `/v1/chat/channels/${c.id}/unread`);
          return [c.id, u.unread] as const;
        } catch {
          return [c.id, 0] as const;
        }
      }),
    );
    setUnread((prev) => {
      const next: Record<string, number> = { ...prev };
      for (const [id, n] of entries) next[id] = id === activeId ? 0 : n;
      return next;
    });
    // Treat a channel with unread messages as recently active so the DM list can float it up. This is a
    // coarse signal (it has no per-message timestamp), but it keeps conversations with new messages near
    // the top until the live ws event gives a precise bump.
    const now = Date.now();
    setLastActivity((prev) => {
      const next = { ...prev };
      for (const [id, n] of entries) if (n > 0) next[id] = now;
      return next;
    });
  }

  async function reloadChannels(selectId?: string) {
    if (!token) return;
    try {
      const list = await apiFetch<Channel[]>(token, "GET", "/v1/chat/channels");
      setChannels(list || []);
      if (selectId) setActiveId(selectId);
      void refreshUnread(list || []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function reloadDirectory() {
    if (!token) return;
    try {
      const d = await apiFetch<{ items?: Person[] }>(token, "GET", "/v1/auth/directory");
      setDirList(d.items || []);
    } catch {
      /* best-effort */
    }
  }

  // Directory + channel list on sign-in; then poll unread counts so sidebar badges stay roughly current.
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
        const p = await apiFetch<{ display_name?: string | null; avatar?: string | null }>(token, "GET", "/v1/auth/me");
        setMeProfile(p);
      } catch {
        /* profile is best-effort (endpoint may predate this deploy) */
      }
    })();
    (async () => {
      try {
        const list = await apiFetch<Channel[]>(token, "GET", "/v1/chat/channels");
        setChannels(list || []);
        setActiveId((cur) => cur || (list && list.length ? list[0].id : ""));
        void refreshUnread(list || []);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
    const iv = window.setInterval(() => {
      setChannels((cur) => {
        void refreshUnread(cur);
        return cur;
      });
    }, 15000);
    return () => window.clearInterval(iv);
    // eslint-disable-next-line react-hooks/exhaustive-deps
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

  // Per-channel: timeline + presence + receipts, and the live websocket (messages, edits/deletes, presence,
  // typing, read receipts, call signals).
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
    setReceipts({});
    setReactPickerId("");
    setTranslations({});
    setTranslating(new Set());
    setTranslateError(new Set());
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
            typeof x === "string"
              ? x
              : ((x as Record<string, string>).subject_id || (x as Record<string, string>).subject || ""),
          )
          .filter(Boolean);
        if (alive) setPresence(new Set(ids));
      } catch {
        /* presence is best-effort */
      }
    })();
    (async () => {
      try {
        const r = await apiFetch<ReadMarker[]>(token, "GET", `/v1/chat/channels/${activeId}/receipts`);
        const map: Record<string, string> = {};
        for (const m of r || []) map[m.subject_id] = m.last_read_message_id;
        if (alive) setReceipts(map);
      } catch {
        /* receipts endpoint may predate this deploy - degrade quietly */
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
          // Bump recent-activity for the message's channel so the DM list re-sorts live.
          const chanId = msg.channel_id || activeId;
          if (chanId) setLastActivity((prev) => ({ ...prev, [chanId]: Date.now() }));
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
        } else if (data.type === "reaction_changed" && data.message_id && data.emoji) {
          const mid = data.message_id;
          const emoji = data.emoji;
          const added = !!data.added;
          const isMe = !!data.subject && data.subject === me;
          const patch = (m: Message): Message =>
            m.id === mid ? { ...m, reactions: applyReaction(m.reactions, emoji, added, isMe) } : m;
          setMessages((prev) => prev.map(patch));
          setThreadReplies((prev) => prev.map(patch));
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
        } else if (data.type === "read" && data.subject && data.last_read_message_id) {
          const sub = data.subject;
          const last = data.last_read_message_id;
          setReceipts((prev) => ({ ...prev, [sub]: last }));
        } else if (data.type === "signal" && data.from) {
          callRef.current.handleSignal(data.from, data.data);
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
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [token, activeId]);

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  // Close the emoji reaction picker on any outside click or Escape (it is a small floating popover).
  useEffect(() => {
    if (!reactPickerId) return;
    const onDown = (e: MouseEvent) => {
      const t = e.target as HTMLElement | null;
      if (t && t.closest(".react-wrap")) return;
      setReactPickerId("");
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setReactPickerId("");
    };
    document.addEventListener("mousedown", onDown);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDown);
      document.removeEventListener("keydown", onKey);
    };
  }, [reactPickerId]);

  // Grow the composer with its content (Enter sends; Shift+Enter is a newline).
  useEffect(() => {
    const ta = taRef.current;
    if (!ta) return;
    ta.style.height = "auto";
    ta.style.height = Math.min(ta.scrollHeight, 140) + "px";
  }, [draft]);

  // Make (and revoke) an object URL for the staged file when it is an image, so the preview strip can show
  // a thumbnail without re-reading the file.
  useEffect(() => {
    if (staged && isImage(staged.type)) {
      const u = URL.createObjectURL(staged);
      setStagedPreview(u);
      return () => URL.revokeObjectURL(u);
    }
    setStagedPreview("");
  }, [staged]);

  // Drop any staged file when switching channels, so it cannot post to the wrong place.
  useEffect(() => {
    setStaged(null);
  }, [activeId]);

  // Auto-mark the active channel read (debounced) when its timeline changes, and clear its badge.
  useEffect(() => {
    if (!token || !activeId || messages.length === 0) return;
    const last = messages[messages.length - 1];
    const tid = window.setTimeout(() => {
      void apiFetch(token, "POST", `/v1/chat/channels/${activeId}/read`, { message_id: last.id }).catch(() => {});
      setUnread((u) => ({ ...u, [activeId]: 0 }));
    }, 500);
    return () => window.clearTimeout(tid);
  }, [token, activeId, messages]);

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

  // Upload a staged file and return its attachment id (reuses the exact attachments endpoint).
  async function uploadStaged(file: File): Promise<string> {
    if (!active || !token) throw new Error("no active channel");
    const b64 = await fileToBase64(file);
    const att = await apiFetch<{ id: string }>(token, "POST", `/v1/chat/channels/${active.id}/attachments`, {
      filename: file.name,
      content_type: file.type || "application/octet-stream",
      data_base64: b64,
    });
    return att.id;
  }

  async function send() {
    const text = draft.trim();
    // Send when there is text or a staged file; nothing to do otherwise, or while a send is in flight.
    if ((!text && !staged) || sending || uploading) return;
    setError("");
    setSending(true);
    try {
      if (staged) {
        setUploading(true);
        let attId: string;
        try {
          attId = await uploadStaged(staged);
        } finally {
          setUploading(false);
        }
        await postMessage(text, attId);
        setStaged(null);
      } else {
        await postMessage(text);
      }
      setDraft("");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSending(false);
    }
  }

  // Stage a file for the next Send (does not upload yet). Guards the server's 5 MB cap up front so an
  // oversize file is rejected with a friendly message before any upload.
  function stageFile(file: File | null | undefined) {
    if (!file) return;
    if (file.size > MAX_ATTACH_BYTES) {
      setError(`"${file.name}" is ${formatBytes(file.size)}, over the ${formatBytes(MAX_ATTACH_BYTES)} limit.`);
      return;
    }
    setError("");
    setStaged(file);
  }

  function onPickFile(e: ChangeEvent<HTMLInputElement>) {
    const file = e.target.files && e.target.files[0];
    e.target.value = "";
    stageFile(file);
  }

  async function saveEdit(m: Message) {
    const body = editText.trim();
    if (!body || !token) return;
    try {
      const updated = await apiFetch<Message>(token, "PATCH", `/v1/chat/channels/${m.channel_id}/messages/${m.id}`, {
        body,
      });
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

  // Add or remove my reaction. Idempotent on the server; the live `reaction_changed` ws event (which the
  // server echoes to the originator too) is what patches the count, so we do not optimistically apply here to
  // avoid double-counting. Mirrors how edits rely on the echoed event.
  async function toggleReaction(m: Message, emoji: string) {
    if (!token) return;
    setReactPickerId("");
    const has = (m.reactions || []).some((r) => r.emoji === emoji && r.mine);
    const path = `/v1/chat/channels/${m.channel_id}/messages/${m.id}/reactions`;
    try {
      if (has) {
        await apiFetch(token, "DELETE", `${path}/${encodeURIComponent(emoji)}`);
      } else {
        await apiFetch(token, "POST", path, { emoji });
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  // Translate a message inline. A second click hides the cached result. The endpoint depends on the ai-gateway,
  // which is not deployed yet, so a failure shows a small "unavailable" note rather than an error banner.
  async function translateMessage(m: Message) {
    if (!token) return;
    // Toggle off if already shown.
    if (translations[m.id] !== undefined) {
      setTranslations((prev) => {
        const next = { ...prev };
        delete next[m.id];
        return next;
      });
      return;
    }
    const text = (m.body || "").trim();
    if (!text) return;
    setTranslateError((prev) => {
      const next = new Set(prev);
      next.delete(m.id);
      return next;
    });
    setTranslating((prev) => new Set(prev).add(m.id));
    try {
      const r = await apiFetch<{ translated: string }>(token, "POST", "/v1/chat/translate", {
        text,
        target_lang: "English",
      });
      setTranslations((prev) => ({ ...prev, [m.id]: r.translated }));
    } catch {
      // Expected until the gateway is deployed: flag this message so the row can show the unavailable note.
      setTranslateError((prev) => new Set(prev).add(m.id));
    } finally {
      setTranslating((prev) => {
        const next = new Set(prev);
        next.delete(m.id);
        return next;
      });
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

  function startCallWith(video: boolean) {
    if (!active) return;
    setError("");
    if (active.kind === "direct" && active.other_subject_id) {
      void call.startCall(active.other_subject_id, video);
    } else {
      setPendingVideo(video);
      setPicker("call");
    }
  }

  async function openThread(m: Message) {
    setThreadRoot(m);
    setThreadReplies([]);
    if (!token) return;
    try {
      const r = await apiFetch<Message[]>(token, "GET", `/v1/chat/channels/${m.channel_id}/messages?parent_id=${m.id}`);
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

  // Group consecutive same-sender messages and mark day boundaries.
  const rows = useMemo(() => {
    const out: { m: Message; showDay: boolean; grouped: boolean }[] = [];
    let prev: Message | null = null;
    for (const m of messages) {
      const showDay = !prev || dayKey(m.created_at) !== dayKey(prev.created_at);
      const grouped =
        !!prev &&
        !showDay &&
        prev.sender_subject_id === m.sender_subject_id &&
        Date.parse(m.created_at || "") - Date.parse(prev.created_at || "") < GROUP_WINDOW_MS;
      out.push({ m, showDay, grouped });
      prev = m;
    }
    return out;
  }, [messages]);

  // Read receipts: which others have read up to (or past) my most recent message.
  const idxOf = useMemo(() => {
    const map = new Map<string, number>();
    messages.forEach((m, i) => map.set(m.id, i));
    return map;
  }, [messages]);
  const myLastId = useMemo(() => {
    for (let i = messages.length - 1; i >= 0; i--) if (messages[i].sender_subject_id === me) return messages[i].id;
    return "";
  }, [messages, me]);
  const seenBy = useMemo(() => {
    if (!myLastId) return [] as string[];
    const myPos = idxOf.get(myLastId);
    if (myPos === undefined) return [];
    const readers: string[] = [];
    for (const [sub, lastId] of Object.entries(receipts)) {
      if (sub === me) continue;
      const p = idxOf.get(lastId);
      if (p !== undefined && p >= myPos) readers.push(sub);
    }
    return readers;
  }, [receipts, myLastId, idxOf, me]);

  const groups = channels.filter((c) => c.kind !== "direct");
  // Direct messages sorted by recent activity (most recent first). Channels with no recorded activity keep
  // their server order at the bottom (Array.sort is stable). Pure client-side ordering.
  const dms = useMemo(() => {
    return channels
      .filter((c) => c.kind === "direct")
      .sort((a, b) => (lastActivity[b.id] || 0) - (lastActivity[a.id] || 0));
  }, [channels, lastActivity]);

  const renderRow = (c: Channel) => {
    const u = unread[c.id] || 0;
    const isActive = c.id === activeId;
    const dm = c.kind === "direct";
    const other = c.other_subject_id || "";
    return (
      <button
        key={c.id}
        className={"chan-row" + (isActive ? " active" : "") + (u > 0 && !isActive ? " unread" : "")}
        onClick={() => setActiveId(c.id)}
        type="button"
      >
        {dm ? (
          // Presence is per-open-socket today: we only learn a subject is online while we hold a socket on a
          // channel they are in. So this dot reflects presence only while that DM's own socket is held (i.e.
          // when it has been opened this session). A correct always-on presence needs a per-user socket
          // independent of the open channel - a known follow-up. We do not fake presence here.
          <Avatar
            id={other || c.id}
            name={nameOf(other)}
            size={26}
            online={presence.has(other)}
            src={avatarSrc(other)}
          />
        ) : (
          <span className="chan-hash">
            <Icon name="hash" size={16} />
          </span>
        )}
        <span className="chan-name">{channelLabel(directory, me, c)}</span>
        {u > 0 && !isActive && <span className="chan-badge">{u > 99 ? "99+" : u}</span>}
      </button>
    );
  };

  const subtitle = active
    ? active.kind === "direct"
      ? presence.has(active.other_subject_id || "")
        ? "Active now"
        : "Direct message"
      : presence.size > 0
        ? `${presence.size} online`
        : "Channel"
    : "";

  const seenLabel =
    active && active.kind === "direct"
      ? "Seen"
      : seenBy.length <= 3
        ? "Seen by " + seenBy.map(nameOf).join(", ")
        : `Seen by ${seenBy.length}`;

  return (
    <div className="chat">
      <input ref={fileRef} type="file" style={{ display: "none" }} onChange={onPickFile} />

      <aside className="sidebar">
        <button className="ws-head" onClick={() => setProfileOpen(true)} type="button" title="Edit your profile">
          <Avatar id={me} name={selfName} size={34} src={myAvatar} />
          <div className="ws-meta">
            <span className="ws-name">{selfName}</span>
            <span className="ws-sub">{email}</span>
          </div>
          <span className="ws-edit">
            <Icon name="edit" size={14} />
          </span>
        </button>
        <div className="side-scroll">
          <div className="side-section">
            <div className="side-label">
              <span>Channels</span>
              <button className="side-add" title="New channel" onClick={() => setPicker("group")} type="button">
                <Icon name="plus" size={14} />
              </button>
            </div>
            {groups.map(renderRow)}
            {groups.length === 0 && <div className="side-empty">No channels yet</div>}
          </div>
          <div className="side-section">
            <div className="side-label">
              <span>Direct messages</span>
              <button className="side-add" title="New direct message" onClick={() => setPicker("dm")} type="button">
                <Icon name="plus" size={14} />
              </button>
            </div>
            {dms.map(renderRow)}
            {dms.length === 0 && <div className="side-empty">No direct messages</div>}
          </div>
        </div>
        <div className="side-foot">
          <span className={"dot " + (health === "ok" ? "ok" : health === "bad" ? "bad" : "")} />
          <span>{health === "ok" ? "Connected" : health === "bad" ? "Reconnecting..." : "Connecting..."}</span>
        </div>
      </aside>

      <section className="main">
        {!active ? (
          <div className="empty big">
            <div className="empty-mark">
              <Icon name="at" size={30} />
            </div>
            <div className="empty-title">Welcome to CyberOS Chat</div>
            <div className="empty-sub">Pick a channel or start a direct message to begin.</div>
          </div>
        ) : (
          <>
            <div className="main-head">
              <div className="head-id">
                {active.kind === "direct" ? (
                  <Avatar
                    id={active.other_subject_id || active.id}
                    name={nameOf(active.other_subject_id || "")}
                    size={36}
                    online={presence.has(active.other_subject_id || "")}
                    src={avatarSrc(active.other_subject_id || "")}
                  />
                ) : (
                  <span className="head-hash">
                    <Icon name="hash" size={20} />
                  </span>
                )}
                <div className="head-text">
                  <span className="chan-title">{channelLabel(directory, me, active)}</span>
                  <span className="chan-sub">{subtitle}</span>
                </div>
              </div>
              <span className="spacer" />
              <button className="icon-btn" title="Voice call" onClick={() => startCallWith(false)} type="button">
                <Icon name="phone" />
              </button>
              <button className="icon-btn" title="Video call" onClick={() => startCallWith(true)} type="button">
                <Icon name="video" />
              </button>
              <button
                className={"icon-btn" + (searchOpen ? " on" : "")}
                title="Search this channel"
                onClick={() => setSearchOpen((s) => !s)}
                type="button"
              >
                <Icon name="search" />
              </button>
              {active.kind !== "direct" && (
                <button className="icon-btn" title="Add people" onClick={() => setPicker("add")} type="button">
                  <Icon name="users" />
                </button>
              )}
            </div>

            {searchOpen && (
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
                        <span className="author">{nameOf(m.sender_subject_id)}</span>{" "}
                        <span className="when">{timeOf(m.created_at)}</span>
                        <div className="snippet">{m.body || "[attachment]"}</div>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            )}

            <div className="main-row">
              <div className="main-col">
                <div
                  className={"messages" + (dragOver ? " drag-over" : "")}
                  ref={scrollRef}
                  onDragOver={(e) => {
                    e.preventDefault();
                    if (!dragOver) setDragOver(true);
                  }}
                  onDragLeave={(e) => {
                    // Only clear when the pointer actually leaves the pane, not when crossing a child.
                    if (e.currentTarget === e.target) setDragOver(false);
                  }}
                  onDrop={(e) => {
                    e.preventDefault();
                    setDragOver(false);
                    stageFile(e.dataTransfer.files && e.dataTransfer.files[0]);
                  }}
                  onPaste={(e) => {
                    const f = e.clipboardData.files && e.clipboardData.files[0];
                    if (f) {
                      e.preventDefault();
                      stageFile(f);
                    }
                  }}
                >
                  {messages.length === 0 && (
                    <div className="empty">
                      <div className="empty-sub">No messages yet. Say hello.</div>
                    </div>
                  )}
                  {rows.map(({ m, showDay, grouped }) => {
                    const mine = m.sender_subject_id === me;
                    return (
                      <Fragment key={m.id}>
                        {showDay && (
                          <div className="day-sep">
                            <span>{formatDay(m.created_at)}</span>
                          </div>
                        )}
                        <div className={"m-row" + (grouped ? " grouped" : "") + (mine ? " mine" : "")}>
                          <div className="m-gutter">
                            {grouped ? (
                              <span className="m-time-hover">{timeOf(m.created_at)}</span>
                            ) : (
                              <Avatar
                                id={m.sender_subject_id}
                                name={nameOf(m.sender_subject_id)}
                                size={36}
                                src={avatarSrc(m.sender_subject_id)}
                              />
                            )}
                          </div>
                          <div className="m-content">
                            {!grouped && (
                              <div className="m-head">
                                <span className="m-name">{nameOf(m.sender_subject_id)}</span>
                                <span className="m-time">{timeOf(m.created_at)}</span>
                                {m.edited_at && <span className="m-edited">edited</span>}
                              </div>
                            )}
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
                              <div className="m-body">
                                {m.attachment_id ? <Attachment token={token!} id={m.attachment_id} /> : m.body}
                              </div>
                            )}
                            {m.reactions && m.reactions.length > 0 && (
                              <div className="reactions">
                                {m.reactions.map((r) => (
                                  <button
                                    key={r.emoji}
                                    className={"reaction" + (r.mine ? " mine" : "")}
                                    onClick={() => void toggleReaction(m, r.emoji)}
                                    title={r.mine ? "Remove your reaction" : "React"}
                                    type="button"
                                  >
                                    <span className="re-emoji">{r.emoji}</span>
                                    <span className="re-count">{r.count}</span>
                                  </button>
                                ))}
                              </div>
                            )}
                            {translating.has(m.id) && <div className="translation muted">Translating...</div>}
                            {!translating.has(m.id) && translations[m.id] !== undefined && (
                              <div className="translation">
                                <span className="tr-label">English</span>
                                <span className="tr-text">{translations[m.id]}</span>
                              </div>
                            )}
                            {!translating.has(m.id) && translateError.has(m.id) && (
                              <div className="translation muted">Translation unavailable</div>
                            )}
                          </div>
                          {editingId !== m.id && (
                            <div className="m-actions">
                              <div className="react-wrap">
                                <button
                                  title="Add reaction"
                                  onClick={() => setReactPickerId((id) => (id === m.id ? "" : m.id))}
                                  type="button"
                                >
                                  <Icon name="smile" size={15} />
                                </button>
                                {reactPickerId === m.id && (
                                  <div className="emoji-picker">
                                    {REACTION_EMOJIS.map((e) => (
                                      <button
                                        key={e}
                                        className="emoji-opt"
                                        onClick={() => void toggleReaction(m, e)}
                                        type="button"
                                      >
                                        {e}
                                      </button>
                                    ))}
                                  </div>
                                )}
                              </div>
                              {!m.attachment_id && m.body && (
                                <button title="Translate to English" onClick={() => void translateMessage(m)} type="button">
                                  <Icon name="translate" size={15} />
                                </button>
                              )}
                              <button title="Reply in thread" onClick={() => void openThread(m)} type="button">
                                <Icon name="thread" size={15} />
                              </button>
                              {mine && !m.attachment_id && (
                                <button
                                  title="Edit"
                                  onClick={() => {
                                    setEditingId(m.id);
                                    setEditText(m.body);
                                  }}
                                  type="button"
                                >
                                  <Icon name="edit" size={15} />
                                </button>
                              )}
                              {mine && (
                                <button title="Delete" onClick={() => void deleteMessage(m)} type="button">
                                  <Icon name="trash" size={15} />
                                </button>
                              )}
                            </div>
                          )}
                        </div>
                        {m.id === myLastId && seenBy.length > 0 && (
                          <div className="seen-row">
                            <Icon name="check" size={12} />
                            <span>{seenLabel}</span>
                          </div>
                        )}
                      </Fragment>
                    );
                  })}
                </div>

                <div className="typing">
                  {typingSubject && typingSubject !== me ? `${nameOf(typingSubject)} is typing...` : ""}
                </div>

                {(error || call.error) && <div className="banner err">{call.error || error}</div>}

                {staged && (
                  <div className="composer-attach">
                    {stagedPreview ? (
                      <img className="ca-thumb" src={stagedPreview} alt={staged.name} />
                    ) : (
                      <span className="ca-icon">
                        <Icon name="paperclip" size={16} />
                      </span>
                    )}
                    <span className="ca-meta">
                      <span className="ca-name">{staged.name}</span>
                      <span className="ca-size">{formatBytes(staged.size)}</span>
                    </span>
                    <button
                      className="ca-x"
                      title="Remove attachment"
                      onClick={() => setStaged(null)}
                      disabled={uploading}
                      type="button"
                    >
                      <Icon name="close" size={14} />
                    </button>
                  </div>
                )}

                <div className="composer">
                  <button className="comp-btn" title="Attach a file" onClick={() => fileRef.current?.click()} type="button">
                    <Icon name="paperclip" />
                  </button>
                  <textarea
                    ref={taRef}
                    rows={1}
                    value={draft}
                    onChange={(e) => onDraftChange(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter" && !e.shiftKey) {
                        e.preventDefault();
                        void send();
                      }
                    }}
                    onPaste={(e) => {
                      const f = e.clipboardData.files && e.clipboardData.files[0];
                      if (f) {
                        e.preventDefault();
                        stageFile(f);
                      }
                    }}
                    placeholder={staged ? "Add a message or just send the file" : "Message " + channelLabel(directory, me, active)}
                  />
                  <button
                    className="comp-send"
                    onClick={() => void send()}
                    disabled={sending || uploading || (!draft.trim() && !staged)}
                    title="Send"
                    type="button"
                  >
                    <Icon name={uploading ? "paperclip" : "send"} />
                  </button>
                </div>
              </div>

              {threadRoot && token && (
                <ThreadPanel
                  token={token}
                  nameOf={nameOf}
                  avatarOf={avatarSrc}
                  root={threadRoot}
                  replies={threadReplies}
                  onClose={() => setThreadRoot(null)}
                  onSend={threadSend}
                />
              )}
            </div>
          </>
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
          onCall={(id) => void call.startCall(id, pendingVideo)}
        />
      )}

      <CallOverlay call={call} nameOf={nameOf} avatarOf={avatarSrc} />

      {profileOpen && token && (
        <ProfileEditor
          token={token}
          me={me}
          initialName={selfName}
          initialAvatar={myAvatar}
          onClose={() => setProfileOpen(false)}
          onSaved={(n, a) => {
            setMeProfile((p) => ({ ...(p || {}), display_name: n, avatar: a }));
            void reloadDirectory();
          }}
        />
      )}
    </div>
  );
}
