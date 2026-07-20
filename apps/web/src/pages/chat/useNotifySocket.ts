import { useEffect, useRef } from "react";
import { wsOrigin } from "../../lib/api";

// A cross-channel notification pushed over the per-user socket (server: notify.rs NotifyEvent). It carries
// only enough to bump an unread badge, float the channel, and raise a desktop notification - the full message
// arrives over the per-channel socket when that channel is opened.
export interface NotifyEvent {
  type: string;
  channel_id?: string;
  message_id?: string;
  sender?: string;
  channel_kind?: string;
  preview?: string;
  mention?: boolean;
  created_at?: string;
}

// One persistent per-user websocket to /v1/chat/notify, independent of the open channel. It reconnects on a
// drop and re-runs when the token changes (so a refreshed token gets a fresh socket) - mirroring useChatSocket.
// The latest onNotify is kept in a ref, so the effect's only dependency is the token: passing a new callback
// each render never tears down the socket.
export function useNotifySocket({
  token,
  onNotify,
}: {
  token: string | null;
  onNotify: (e: NotifyEvent) => void;
}) {
  const cbRef = useRef(onNotify);
  cbRef.current = onNotify;

  useEffect(() => {
    if (!token) return;
    let stopped = false;
    let sock: WebSocket | null = null;
    const connect = () => {
      if (stopped) return;
      const url = wsOrigin() + `/v1/chat/notify?access_token=${encodeURIComponent(token)}`;
      sock = new WebSocket(url);
      sock.onmessage = (ev) => {
        try {
          const e = JSON.parse(ev.data as string) as NotifyEvent;
          if (e && e.type === "message") cbRef.current(e);
        } catch {
          /* ignore a malformed frame */
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
  }, [token]);
}
