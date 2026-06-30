// 1:1 WebRTC calling over the chat signaling relay.
//
// The chat websocket already relays {type:"signal", to, data} frames privately to the addressed subject
// (server: ChatEvent::Signal). This hook drives getUserMedia + RTCPeerConnection and exchanges offer /
// answer / ICE inside those `data` payloads. Media flows peer to peer (public STUN); a TURN server, needed
// for some restrictive networks, is a later addition. Signaling rides the active channel's socket, so the
// callee must have that conversation open to receive the ring - background ringing needs a per-user socket.

import { useEffect, useRef, useState } from "react";

const RTC_CONFIG: RTCConfiguration = {
  iceServers: [
    { urls: "stun:stun.l.google.com:19302" },
    { urls: "stun:stun1.l.google.com:19302" },
  ],
};

export type CallStatus = "idle" | "calling" | "incoming" | "connecting" | "connected";

export interface CallState {
  status: CallStatus;
  peerId: string;
  video: boolean;
  muted: boolean;
  camOff: boolean;
  startedAt: number;
}

interface SignalData {
  kind: "offer" | "answer" | "ice" | "hangup" | "decline";
  callId?: string;
  video?: boolean;
  sdp?: RTCSessionDescriptionInit;
  candidate?: RTCIceCandidateInit;
}

const IDLE: CallState = {
  status: "idle",
  peerId: "",
  video: false,
  muted: false,
  camOff: false,
  startedAt: 0,
};

export interface CallApi {
  state: CallState;
  localStream: MediaStream | null;
  remoteStream: MediaStream | null;
  error: string;
  startCall(peerId: string, video: boolean): Promise<void>;
  accept(): Promise<void>;
  decline(): void;
  hangup(): void;
  toggleMute(): void;
  toggleCam(): void;
  handleSignal(from: string, data: unknown): void;
}

export function useCall(send: (to: string, data: SignalData) => void): CallApi {
  const [state, setState] = useState<CallState>(IDLE);
  const [localStream, setLocalStream] = useState<MediaStream | null>(null);
  const [remoteStream, setRemoteStream] = useState<MediaStream | null>(null);
  const [error, setError] = useState("");

  const pcRef = useRef<RTCPeerConnection | null>(null);
  const localRef = useRef<MediaStream | null>(null);
  const peerRef = useRef("");
  const callIdRef = useRef("");
  const offerRef = useRef<RTCSessionDescriptionInit | null>(null);
  const pendingIce = useRef<RTCIceCandidateInit[]>([]);
  const sendRef = useRef(send);
  sendRef.current = send;
  const statusRef = useRef<CallStatus>("idle");
  statusRef.current = state.status;
  const videoRef = useRef(false);
  videoRef.current = state.video;

  function patch(p: Partial<CallState>) {
    setState((s) => ({ ...s, ...p }));
  }

  function teardown(notify?: "hangup" | "decline") {
    const peer = peerRef.current;
    if (notify && peer) {
      try {
        sendRef.current(peer, { kind: notify, callId: callIdRef.current });
      } catch {
        /* socket already gone */
      }
    }
    if (pcRef.current) {
      try {
        pcRef.current.close();
      } catch {
        /* already closed */
      }
      pcRef.current = null;
    }
    if (localRef.current) {
      localRef.current.getTracks().forEach((t) => t.stop());
      localRef.current = null;
    }
    pendingIce.current = [];
    offerRef.current = null;
    peerRef.current = "";
    callIdRef.current = "";
    setLocalStream(null);
    setRemoteStream(null);
    setState(IDLE);
  }

  function newPc(peerId: string): RTCPeerConnection {
    const pc = new RTCPeerConnection(RTC_CONFIG);
    pc.onicecandidate = (e) => {
      if (e.candidate) {
        sendRef.current(peerId, {
          kind: "ice",
          callId: callIdRef.current,
          candidate: e.candidate.toJSON(),
        });
      }
    };
    pc.ontrack = (e) => {
      if (e.streams[0]) setRemoteStream(e.streams[0]);
    };
    pc.onconnectionstatechange = () => {
      const st = pc.connectionState;
      if (st === "connected") {
        setState((s) => (s.status === "connected" ? s : { ...s, status: "connected", startedAt: Date.now() }));
      } else if (st === "failed" || st === "disconnected" || st === "closed") {
        teardown();
      }
    };
    pcRef.current = pc;
    return pc;
  }

  async function getMedia(video: boolean): Promise<MediaStream> {
    const stream = await navigator.mediaDevices.getUserMedia({ audio: true, video });
    localRef.current = stream;
    setLocalStream(stream);
    return stream;
  }

  async function startCall(peerId: string, video: boolean) {
    if (statusRef.current !== "idle" || !peerId) return;
    setError("");
    const callId = Math.random().toString(36).slice(2);
    callIdRef.current = callId;
    peerRef.current = peerId;
    setState({ status: "calling", peerId, video, muted: false, camOff: false, startedAt: 0 });
    try {
      const stream = await getMedia(video);
      const pc = newPc(peerId);
      stream.getTracks().forEach((t) => pc.addTrack(t, stream));
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);
      sendRef.current(peerId, { kind: "offer", callId, video, sdp: offer });
    } catch (e) {
      setError(e instanceof Error ? e.message : "could not start the call");
      teardown();
    }
  }

  async function accept() {
    const peerId = peerRef.current;
    const offer = offerRef.current;
    if (!peerId || !offer) return;
    const video = videoRef.current;
    patch({ status: "connecting" });
    try {
      const stream = await getMedia(video);
      const pc = newPc(peerId);
      stream.getTracks().forEach((t) => pc.addTrack(t, stream));
      await pc.setRemoteDescription(offer);
      for (const c of pendingIce.current) {
        try {
          await pc.addIceCandidate(c);
        } catch {
          /* stale candidate */
        }
      }
      pendingIce.current = [];
      const answer = await pc.createAnswer();
      await pc.setLocalDescription(answer);
      sendRef.current(peerId, { kind: "answer", callId: callIdRef.current, sdp: answer });
    } catch (e) {
      setError(e instanceof Error ? e.message : "could not join the call");
      teardown();
    }
  }

  function decline() {
    teardown("decline");
  }
  function hangup() {
    teardown("hangup");
  }

  function toggleMute() {
    const stream = localRef.current;
    if (!stream) return;
    const on = stream.getAudioTracks().some((t) => t.enabled);
    stream.getAudioTracks().forEach((t) => (t.enabled = !on));
    patch({ muted: on });
  }
  function toggleCam() {
    const stream = localRef.current;
    if (!stream) return;
    const on = stream.getVideoTracks().some((t) => t.enabled);
    stream.getVideoTracks().forEach((t) => (t.enabled = !on));
    patch({ camOff: on });
  }

  function handleSignal(from: string, raw: unknown) {
    const data = raw as SignalData;
    if (!data || typeof data.kind !== "string") return;
    if (data.kind === "offer") {
      if (statusRef.current !== "idle") {
        try {
          sendRef.current(from, { kind: "decline", callId: data.callId });
        } catch {
          /* socket gone */
        }
        return;
      }
      peerRef.current = from;
      callIdRef.current = data.callId || "";
      offerRef.current = data.sdp || null;
      pendingIce.current = [];
      setState({ status: "incoming", peerId: from, video: !!data.video, muted: false, camOff: false, startedAt: 0 });
    } else if (data.kind === "answer") {
      const pc = pcRef.current;
      if (pc && data.sdp) {
        const sdp = data.sdp;
        void (async () => {
          try {
            await pc.setRemoteDescription(sdp);
            for (const c of pendingIce.current) {
              try {
                await pc.addIceCandidate(c);
              } catch {
                /* stale candidate */
              }
            }
            pendingIce.current = [];
          } catch {
            /* renegotiation race */
          }
        })();
        patch({ status: "connecting" });
      }
    } else if (data.kind === "ice") {
      if (!data.candidate) return;
      const pc = pcRef.current;
      if (pc && pc.remoteDescription) {
        void pc.addIceCandidate(data.candidate).catch(() => {});
      } else {
        pendingIce.current.push(data.candidate);
      }
    } else if (data.kind === "hangup" || data.kind === "decline") {
      if (from === peerRef.current) teardown();
    }
  }

  // Stop media + close the connection if the chat unmounts mid-call (refs hold the live objects).
  useEffect(() => {
    return () => {
      if (pcRef.current) {
        try {
          pcRef.current.close();
        } catch {
          /* already closed */
        }
      }
      if (localRef.current) localRef.current.getTracks().forEach((t) => t.stop());
    };
  }, []);

  return {
    state,
    localStream,
    remoteStream,
    error,
    startCall,
    accept,
    decline,
    hangup,
    toggleMute,
    toggleCam,
    handleSignal,
  };
}
