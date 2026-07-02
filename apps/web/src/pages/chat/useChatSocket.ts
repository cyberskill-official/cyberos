import { useEffect, useRef, useState } from "react";
import type { MutableRefObject } from "react";
import { apiFetch } from "../../lib/api";
import type { Message, ReadMarker } from "../../lib/chat";
import { applyReaction } from "../../lib/chat";
import type { CallApi } from "../../lib/call";

// A websocket event: a message (Message shape) or one of the control frames the server emits.
export interface WsEvent extends Partial<Message> {
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

export interface ChatSocket {
  messages: Message[];
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>;
  threadRoot: Message | null;
  setThreadRoot: React.Dispatch<React.SetStateAction<Message | null>>;
  threadReplies: Message[];
  setThreadReplies: React.Dispatch<React.SetStateAction<Message[]>>;
  presence: Set<string>;
  setPresence: React.Dispatch<React.SetStateAction<Set<string>>>;
  typingSubject: string;
  setTypingSubject: React.Dispatch<React.SetStateAction<string>>;
  receipts: Record<string, string>;
  setReceipts: React.Dispatch<React.SetStateAction<Record<string, string>>>;
  lastActivity: Record<string, number>;
  setLastActivity: React.Dispatch<React.SetStateAction<Record<string, number>>>;
  wsRef: MutableRefObject<WebSocket | null>;
  typingSentAt: MutableRefObject<number>;
}

// Owns the per-channel timeline + presence + receipts state and the live websocket (messages, edits/deletes,
// reactions, presence, typing, read receipts, call signals). Extracted verbatim from Chat.tsx so behavior is
// unchanged: the parent passes the auth token, the active channel, the current subject id, a stable ref to the
// call engine, and `resetChannelUi` (run at channel switch, exactly as before). `setError` mirrors the parent's.
export function useChatSocket({
  token,
  activeId,
  me,
  callRef,
  setError,
  resetChannelUi,
}: {
  token: string | null;
  activeId: string;
  me: string;
  callRef: MutableRefObject<CallApi>;
  setError: (msg: string) => void;
  resetChannelUi: () => void;
}): ChatSocket {
  const [messages, setMessages] = useState<Message[]>([]);
  // Recent-activity timestamps keyed by channel id, used to sort the DM list. Pure client state: it is fed by
  // the unread poll (a channel with unread is treated as recently active) and by inbound `message` ws events.
  const [lastActivity, setLastActivity] = useState<Record<string, number>>({});
  const [receipts, setReceipts] = useState<Record<string, string>>({});

  const [presence, setPresence] = useState<Set<string>>(new Set());
  const [typingSubject, setTypingSubject] = useState("");

  const [threadRoot, setThreadRoot] = useState<Message | null>(null);
  const [threadReplies, setThreadReplies] = useState<Message[]>([]);
  const threadRootRef = useRef<Message | null>(null);
  useEffect(() => {
    threadRootRef.current = threadRoot;
  }, [threadRoot]);

  const wsRef = useRef<WebSocket | null>(null);
  const typingSentAt = useRef(0);
  const typingTimer = useRef<number | null>(null);

  // Keep the latest reset callback without retriggering the channel effect (its deps stay [token, activeId]).
  const resetRef = useRef(resetChannelUi);
  resetRef.current = resetChannelUi;
  const setErrorRef = useRef(setError);
  setErrorRef.current = setError;

  // Per-channel: timeline + presence + receipts, and the live websocket (messages, edits/deletes, presence,
  // typing, read receipts, call signals).
  useEffect(() => {
    if (!token || !activeId) {
      setMessages([]);
      return;
    }
    setThreadRoot(null);
    setThreadReplies([]);
    resetRef.current();
    setPresence(new Set());
    setTypingSubject("");
    setReceipts({});
    let alive = true;

    (async () => {
      try {
        const msgs = await apiFetch<Message[]>(token, "GET", `/v1/chat/channels/${activeId}/messages`);
        if (alive) setMessages((msgs || []).filter((m) => !m.parent_id));
      } catch (e) {
        if (alive) setErrorRef.current(e instanceof Error ? e.message : String(e));
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

  return {
    messages,
    setMessages,
    threadRoot,
    setThreadRoot,
    threadReplies,
    setThreadReplies,
    presence,
    setPresence,
    typingSubject,
    setTypingSubject,
    receipts,
    setReceipts,
    lastActivity,
    setLastActivity,
    wsRef,
    typingSentAt,
  };
}
