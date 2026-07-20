import { useEffect, useRef, useState } from "react";
import type { MutableRefObject } from "react";
import { apiFetch, wsOrigin } from "../../lib/api";
import type { Message, ReadMarker } from "../../lib/chat";
import { applyReaction, sortMessagesAsc } from "../../lib/chat";
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
  count?: number;
}

export interface ChatSocket {
  messages: Message[];
  setMessages: React.Dispatch<React.SetStateAction<Message[]>>;
  /// True while the open channel's first page is being fetched, so the pane can show skeletons instead of the
  /// empty-state note.
  loading: boolean;
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
  jumpRef,
  onJumped,
  pausedRef,
}: {
  token: string | null;
  activeId: string;
  me: string;
  callRef: MutableRefObject<CallApi>;
  setError: (msg: string) => void;
  resetChannelUi: () => void;
  /// A pending jump-to-message (from global search). When set for the channel being opened, the initial
  /// fetch loads an ?around= window instead of the latest page, then the ref is cleared.
  jumpRef?: MutableRefObject<{ channelId: string; messageId: string } | null>;
  /// Called once after a jump window landed, with the target message id (Chat scrolls + flashes it).
  onJumped?: (messageId: string) => void;
  /// True while the timeline is showing a history window (not the live tail); the reconnect refetch is skipped
  /// then so it does not inject the latest page into a jump/search view.
  pausedRef?: MutableRefObject<boolean>;
}): ChatSocket {
  const [messages, setMessages] = useState<Message[]>([]);
  const [loading, setLoading] = useState(false);
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
  const onJumpedRef = useRef(onJumped);
  onJumpedRef.current = onJumped;

  // Per-channel: timeline + presence + receipts, and the live websocket (messages, edits/deletes, presence,
  // typing, read receipts, call signals).
  useEffect(() => {
    if (!token || !activeId) {
      setMessages([]);
      setLoading(false);
      return;
    }
    setThreadRoot(null);
    setThreadReplies([]);
    resetRef.current();
    setPresence(new Set());
    setTypingSubject("");
    setReceipts({});
    setMessages([]); // clear the previous channel's timeline so the loading skeletons show on every open
    setLoading(true);
    let alive = true;

    (async () => {
      // A pending jump into this channel loads a window around the target instead of the latest page.
      const jump = jumpRef && jumpRef.current && jumpRef.current.channelId === activeId ? jumpRef.current : null;
      if (jump && jumpRef) jumpRef.current = null;
      try {
        const path = jump
          ? `/v1/chat/channels/${activeId}/messages?around=${encodeURIComponent(jump.messageId)}&limit=80`
          : `/v1/chat/channels/${activeId}/messages`;
        const msgs = await apiFetch<Message[]>(token, "GET", path);
        if (alive) {
          setMessages(sortMessagesAsc((msgs || []).filter((m) => !m.parent_id)));
          if (jump) onJumpedRef.current?.(jump.messageId);
        }
      } catch (e) {
        if (alive) setErrorRef.current(e instanceof Error ? e.message : String(e));
      } finally {
        if (alive) setLoading(false);
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
    let firstOpen = true;
    // On every RECONNECT (not the first open), re-fetch the live tail and merge it in, so messages, edits, and
    // reaction changes that landed during the drop are recovered instead of lost until the user switches away.
    const refetchTail = async () => {
      if (!token) return;
      try {
        const msgs = await apiFetch<Message[]>(token, "GET", `/v1/chat/channels/${activeId}/messages`);
        if (!alive) return;
        const fresh = sortMessagesAsc((msgs || []).filter((m) => !m.parent_id));
        const freshIds = new Set(fresh.map((m) => m.id));
        setMessages((prev) => sortMessagesAsc([...prev.filter((m) => !freshIds.has(m.id)), ...fresh]));
      } catch {
        /* best-effort; the next reconnect retries */
      }
    };
    const connect = () => {
      if (stopped) return;
      const url =
        wsOrigin() +
        `/v1/chat/ws?channel=${encodeURIComponent(activeId)}&access_token=${encodeURIComponent(token)}`;
      sock = new WebSocket(url);
      wsRef.current = sock;
      sock.onopen = () => {
        const wasFirst = firstOpen;
        firstOpen = false;
        // The first open is already covered by the channel effect's initial fetch; only reconnects backfill,
        // and never while a history/jump window is showing (pausedRef).
        if (!wasFirst && !pausedRef?.current) void refetchTail();
      };
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
            const pid = msg.parent_id;
            // Bump the parent's reply-count chip live, whether or not its thread is currently open.
            setMessages((prev) =>
              prev.map((m) => (m.id === pid ? { ...m, reply_count: (m.reply_count || 0) + 1 } : m)),
            );
            const root = threadRootRef.current;
            if (root && pid === root.id) {
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
          const count = typeof data.count === "number" ? data.count : added ? 1 : 0;
          const patch = (m: Message): Message =>
            m.id === mid ? { ...m, reactions: applyReaction(m.reactions, emoji, added, isMe, count) } : m;
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
    loading,
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
