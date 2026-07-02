import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import type { ChangeEvent } from "react";
import { useAuth } from "../lib/auth";
import { apiFetch, decodeJwt } from "../lib/api";
import type { Channel, Directory, Message, Person } from "../lib/chat";
import { channelLabel, dayKey, fileToBase64, formatBytes, isImage, shortId } from "../lib/chat";
import type { MentionCandidate } from "../lib/richtext";
import { Icon } from "../components/icons";
import { EmojiPicker } from "../components/EmojiPicker";
import type { AnchorRect } from "../components/EmojiPicker";
import { PeoplePicker } from "../components/PeoplePicker";
import type { PickerMode } from "../components/PeoplePicker";
import { ThreadPanel } from "../components/ThreadPanel";
import { CallOverlay } from "../components/CallOverlay";
import { useCall } from "../lib/call";
import { ProfileEditor } from "../components/ProfileEditor";
import { useChatSocket } from "./chat/useChatSocket";
import { useNotifySocket } from "./chat/useNotifySocket";
import { Sidebar } from "./chat/Sidebar";
import { ChannelHeader } from "./chat/ChannelHeader";
import { MessageList } from "./chat/MessageList";
import { Composer } from "./chat/Composer";

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

  // Every name a person can be @-mentioned as in a rendered body (display name, handle, email local-part),
  // flagged when the name is mine so "mentions me" gets the stronger tint. Deduped (me wins), longest first
  // so "Anna Vu" is tried before "Anna". Feeds the RichText mention highlighter.
  const mentionNames = useMemo<MentionCandidate[]>(() => {
    const seen = new Map<string, MentionCandidate>();
    const add = (name: string | null | undefined, isMe: boolean) => {
      const n = (name || "").trim();
      if (!n) return;
      const k = n.toLowerCase();
      const prev = seen.get(k);
      if (!prev || (isMe && !prev.me)) seen.set(k, { name: n, me: isMe });
    };
    for (const p of dirList) {
      const isMe = p.subject_id === me;
      add(p.display_name, isMe);
      add(p.handle, isMe);
      add((p.email || "").split("@")[0], isMe);
    }
    add(selfName, true);
    return [...seen.values()].sort((a, b) => b.name.length - a.name.length);
  }, [dirList, me, selfName]);

  const [channels, setChannels] = useState<Channel[]>([]);
  const [activeId, setActiveId] = useState("");
  const [unread, setUnread] = useState<Record<string, number>>({});
  // Unread @-mentions per channel (a subset of unread), for the distinct mention badge. Seeded from the
  // summary and bumped live by the notify socket.
  const [mentions, setMentions] = useState<Record<string, number>>({});
  const [draft, setDraft] = useState("");
  // Mentions picked from the composer autocomplete this compose cycle ({id, name}). At send we keep only
  // those whose "@name" still appears in the draft, so deleting the text also drops the mention.
  const [pickedMentions, setPickedMentions] = useState<{ id: string; name: string }[]>([]);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState("");
  const [health, setHealth] = useState<"unknown" | "ok" | "bad">("unknown");
  const [picker, setPicker] = useState<PickerMode | null>(null);
  const [pendingVideo, setPendingVideo] = useState(false);

  const [editingId, setEditingId] = useState("");
  const [editText, setEditText] = useState("");

  // Emoji reactions: which message's picker is open (only one at a time). Translations: a per-message cache of
  // the inline result, and the set of message ids whose translation is in flight. A second translate click
  // hides the cached result (removes the key).
  const [reactPickerId, setReactPickerId] = useState("");
  // The one full emoji picker instance: opened either from a message's reaction bar ("+") or from the
  // composer's emoji button; anchored to the trigger's rect and rendered fixed at the page root.
  const [emojiFor, setEmojiFor] = useState<
    { kind: "reaction"; m: Message; rect: AnchorRect } | { kind: "composer"; rect: AnchorRect } | null
  >(null);
  const [translations, setTranslations] = useState<Record<string, string>>({});
  const [translating, setTranslating] = useState<Set<string>>(new Set());
  const [translateError, setTranslateError] = useState<Set<string>>(new Set());

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

  const active = channels.find((c) => c.id === activeId) || null;

  // The latest active channel id, readable from callbacks (the unread poll, the notify handler) that are
  // captured once and would otherwise close over a stale value.
  const activeIdRef = useRef(activeId);
  useEffect(() => {
    activeIdRef.current = activeId;
  }, [activeId]);
  // Ask for desktop-notification permission once, lazily, on the first channel selection (a real user
  // gesture, which browsers require). If denied, badges and the tab title still work; we just never notify.
  const askedNotifyRef = useRef(false);
  function selectChannel(id: string) {
    if (!askedNotifyRef.current) {
      askedNotifyRef.current = true;
      if (typeof Notification !== "undefined" && Notification.permission === "default") {
        void Notification.requestPermission().catch(() => {});
      }
    }
    setActiveId(id);
  }

  // Reset the per-channel UI bits Chat owns when the active channel changes (timeline/presence/receipts reset
  // lives in useChatSocket). Kept as a stable callback so the socket effect deps stay [token, activeId].
  const resetChannelUi = useCallback(() => {
    setSearchOpen(false);
    setEditingId("");
    setReactPickerId("");
    setEmojiFor(null);
    setTranslations({});
    setTranslating(new Set());
    setTranslateError(new Set());
  }, []);

  // The chat websocket + the per-channel timeline / presence / receipts / recent-activity state it owns.
  // `callRef` is wired below (declared after the engine); the hook only reads it inside the ws handler, so the
  // forward reference is safe.
  const callRef = useRef<ReturnType<typeof useCall> | null>(null);
  const socket = useChatSocket({
    token,
    activeId,
    me,
    // Non-null after mount; the hook dereferences it lazily inside onmessage, never during render.
    callRef: callRef as React.MutableRefObject<ReturnType<typeof useCall>>,
    setError,
    resetChannelUi,
  });
  const {
    messages,
    setMessages,
    threadRoot,
    setThreadRoot,
    threadReplies,
    setThreadReplies,
    presence,
    typingSubject,
    receipts,
    setLastActivity,
    wsRef,
    typingSentAt,
  } = socket;

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
  callRef.current = call;

  // The per-user notification socket: live cross-channel activity, independent of the open channel. Bump the
  // unread badge + recent-activity for any channel that is not the open one, and (only when the tab is hidden
  // and permission was granted) raise a desktop notification. The open channel is served by its own socket,
  // which also auto-marks it read, so we skip it here.
  useNotifySocket({
    token,
    onNotify: (e) => {
      const chan = e.channel_id;
      if (!chan) return;
      const when = Date.parse(e.created_at || "") || Date.now();
      setLastActivity((prev) => ({ ...prev, [chan]: when }));
      if (chan === activeIdRef.current) return;
      setUnread((prev) => ({ ...prev, [chan]: (prev[chan] || 0) + 1 }));
      if (e.mention) setMentions((prev) => ({ ...prev, [chan]: (prev[chan] || 0) + 1 }));
      if (
        typeof document !== "undefined" &&
        document.hidden &&
        typeof Notification !== "undefined" &&
        Notification.permission === "granted"
      ) {
        const ch = channels.find((c) => c.id === chan);
        const who = nameOf(e.sender || "");
        const title = ch && ch.kind !== "direct" ? `${who} in ${channelLabel(directory, me, ch)}` : who;
        const body = e.mention ? `mentioned you: ${e.preview || ""}` : e.preview || "New message";
        try {
          new Notification(title, { body });
        } catch {
          /* Notification construction can throw on some platforms; ignore */
        }
      }
    },
  });

  // Seed and reconcile unread badges for every channel in one request (GET /v1/chat/unread). The live notify
  // socket keeps counts current between polls; this corrects any drift (a missed event, another device).
  async function refreshUnread() {
    if (!token) return;
    try {
      const rows = await apiFetch<{ channel_id: string; unread: number; mentions: number }[]>(
        token,
        "GET",
        "/v1/chat/unread",
      );
      setUnread((prev) => {
        const next: Record<string, number> = { ...prev };
        for (const r of rows || []) next[r.channel_id] = r.channel_id === activeIdRef.current ? 0 : r.unread;
        return next;
      });
      setMentions((prev) => {
        const next: Record<string, number> = { ...prev };
        for (const r of rows || []) next[r.channel_id] = r.channel_id === activeIdRef.current ? 0 : r.mentions || 0;
        return next;
      });
      // A channel with unread messages is treated as recently active so the DM list floats it up; do not
      // clobber a precise timestamp a live ws event already set.
      const now = Date.now();
      setLastActivity((prev) => {
        const next = { ...prev };
        for (const r of rows || []) if (r.unread > 0 && !next[r.channel_id]) next[r.channel_id] = now;
        return next;
      });
    } catch {
      /* summary endpoint may predate this deploy - the live socket still keeps counts current */
    }
  }

  async function reloadChannels(selectId?: string) {
    if (!token) return;
    try {
      const list = await apiFetch<Channel[]>(token, "GET", "/v1/chat/channels");
      setChannels(list || []);
      if (selectId) setActiveId(selectId);
      void refreshUnread();
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
        void refreshUnread();
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
    // A slow reconciler; the notify socket makes counts feel live, so this only corrects drift.
    const iv = window.setInterval(() => {
      void refreshUnread();
    }, 30000);
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

  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  // Reflect total unread (across channels, excluding the open one) in the tab title, so a background tab
  // still signals new activity even when the window is not focused.
  useEffect(() => {
    const total = Object.entries(unread).reduce((n, [id, c]) => n + (id === activeId ? 0 : c || 0), 0);
    const base = "CyberOS Chat";
    document.title = total > 0 ? `(${total}) ${base}` : base;
    return () => {
      document.title = base;
    };
  }, [unread, activeId]);

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

  // Drop any staged file and pending mentions when switching channels, so neither posts to the wrong place.
  useEffect(() => {
    setStaged(null);
    setPickedMentions([]);
  }, [activeId]);

  // Auto-mark the active channel read (debounced) when its timeline changes, and clear its badge.
  useEffect(() => {
    if (!token || !activeId || messages.length === 0) return;
    const last = messages[messages.length - 1];
    const tid = window.setTimeout(() => {
      void apiFetch(token, "POST", `/v1/chat/channels/${activeId}/read`, { message_id: last.id }).catch(() => {});
      setUnread((u) => ({ ...u, [activeId]: 0 }));
      setMentions((mn) => ({ ...mn, [activeId]: 0 }));
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

  async function postMessage(body: string, attachmentId?: string, mentionIds?: string[]) {
    if (!active || !token) return;
    const payload: Record<string, unknown> = { body };
    if (attachmentId) payload.attachment_id = attachmentId;
    if (mentionIds && mentionIds.length) payload.mentions = mentionIds;
    const m = await apiFetch<Message>(token, "POST", `/v1/chat/channels/${active.id}/messages`, payload);
    setMessages((prev) => (prev.some((x) => x.id === m.id) ? prev : [...prev, m]));
  }

  // Keep only the picked mentions whose "@name" still appears in the outgoing text as a whole token (bounded
  // by start/space before and space/end after), so a name that is a prefix of another (An vs Anna) never
  // mis-resolves. Deduped ids.
  function resolveMentions(text: string): string[] {
    const ids = new Set<string>();
    for (const pm of pickedMentions) {
      if (!pm.name) continue;
      const esc = pm.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      if (new RegExp("(^|\\s)@" + esc + "(\\s|$)").test(text)) ids.add(pm.id);
    }
    return [...ids];
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
    const mentionIds = resolveMentions(draft);
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
        await postMessage(text, attId, mentionIds);
        setStaged(null);
      } else {
        await postMessage(text, undefined, mentionIds);
      }
      setDraft("");
      setPickedMentions([]);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSending(false);
    }
  }

  // Insert an emoji into the draft at the caret (from the composer's emoji button) and restore focus.
  function insertEmoji(emoji: string) {
    const ta = taRef.current;
    const pos = ta ? (ta.selectionStart ?? draft.length) : draft.length;
    const next = draft.slice(0, pos) + emoji + draft.slice(pos);
    setDraft(next);
    requestAnimationFrame(() => {
      const t = taRef.current;
      if (t) {
        t.focus();
        const p = pos + emoji.length;
        t.setSelectionRange(p, p);
      }
    });
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
      .sort((a, b) => (socket.lastActivity[b.id] || 0) - (socket.lastActivity[a.id] || 0));
  }, [channels, socket.lastActivity]);

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

      <Sidebar
        me={me}
        email={email}
        selfName={selfName}
        myAvatar={myAvatar}
        directory={directory}
        groups={groups}
        dms={dms}
        activeId={activeId}
        unread={unread}
        mentions={mentions}
        presence={presence}
        health={health}
        nameOf={nameOf}
        avatarSrc={avatarSrc}
        onOpenProfile={() => setProfileOpen(true)}
        onSelectChannel={selectChannel}
        onOpenPicker={setPicker}
      />

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
            <ChannelHeader
              active={active}
              directory={directory}
              me={me}
              subtitle={subtitle}
              presence={presence}
              searchOpen={searchOpen}
              searchQ={searchQ}
              searchResults={searchResults}
              nameOf={nameOf}
              avatarSrc={avatarSrc}
              onStartCall={startCallWith}
              onToggleSearch={() => setSearchOpen((s) => !s)}
              onOpenAddPeople={() => setPicker("add")}
              onSearchQChange={setSearchQ}
              onRunSearch={runSearch}
            />

            <div className="main-row">
              <div className="main-col">
                <MessageList
                  rows={rows}
                  messages={messages}
                  me={me}
                  token={token}
                  scrollRef={scrollRef}
                  dragOver={dragOver}
                  editingId={editingId}
                  editText={editText}
                  reactPickerId={reactPickerId}
                  translating={translating}
                  translations={translations}
                  translateError={translateError}
                  myLastId={myLastId}
                  seenBy={seenBy}
                  seenLabel={seenLabel}
                  mentionNames={mentionNames}
                  nameOf={nameOf}
                  avatarSrc={avatarSrc}
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
                  onEditTextChange={setEditText}
                  onSaveEdit={saveEdit}
                  onCancelEdit={() => {
                    setEditingId("");
                    setEditText("");
                  }}
                  onToggleReaction={toggleReaction}
                  onSetReactPicker={(updater) => setReactPickerId(updater)}
                  onOpenFullEmoji={(m, rect) => {
                    setReactPickerId("");
                    setEmojiFor({ kind: "reaction", m, rect });
                  }}
                  onTranslate={translateMessage}
                  onOpenThread={openThread}
                  onStartEdit={(m) => {
                    setEditingId(m.id);
                    setEditText(m.body);
                  }}
                  onDelete={deleteMessage}
                />

                <div className="typing">
                  {typingSubject && typingSubject !== me ? `${nameOf(typingSubject)} is typing...` : ""}
                </div>

                {(error || call.error) && <div className="banner err">{call.error || error}</div>}

                <Composer
                  active={active}
                  directory={directory}
                  me={me}
                  people={dirList}
                  draft={draft}
                  staged={staged}
                  stagedPreview={stagedPreview}
                  uploading={uploading}
                  sending={sending}
                  taRef={taRef}
                  onDraftChange={onDraftChange}
                  onSend={send}
                  onMentionPicked={(p) =>
                    setPickedMentions((prev) =>
                      prev.some((x) => x.id === p.subject_id)
                        ? prev
                        : [
                            ...prev,
                            {
                              id: p.subject_id,
                              name: p.display_name || p.handle || (p.email || "").split("@")[0] || "",
                            },
                          ],
                    )
                  }
                  onClearStaged={() => setStaged(null)}
                  onOpenFilePicker={() => fileRef.current?.click()}
                  onOpenEmoji={(rect) => setEmojiFor({ kind: "composer", rect })}
                  onPaste={(e) => {
                    const f = e.clipboardData.files && e.clipboardData.files[0];
                    if (f) {
                      e.preventDefault();
                      stageFile(f);
                    }
                  }}
                />
              </div>

              {threadRoot && token && (
                <ThreadPanel
                  token={token}
                  nameOf={nameOf}
                  avatarOf={avatarSrc}
                  mentionNames={mentionNames}
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

      {emojiFor && (
        <EmojiPicker
          anchor={emojiFor.rect}
          onClose={() => setEmojiFor(null)}
          onPick={(em) => {
            if (emojiFor.kind === "reaction") void toggleReaction(emojiFor.m, em);
            else insertEmoji(em);
            setEmojiFor(null);
          }}
        />
      )}

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
