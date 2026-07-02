import { useEffect, useRef, useState } from "react";
import type { CallApi } from "../lib/call";
import { t } from "../lib/i18n";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";

// The call surface: an incoming-call ring card, or the in-call stage (remote video/avatar, local PiP,
// and the control bar). Streams are attached to the <video> elements imperatively via srcObject.
export function CallOverlay({
  call,
  nameOf,
  avatarOf,
}: {
  call: CallApi;
  nameOf: (id: string) => string;
  avatarOf: (id: string) => string;
}) {
  const { state, localStream, remoteStream } = call;
  const localVideo = useRef<HTMLVideoElement | null>(null);
  const remoteVideo = useRef<HTMLVideoElement | null>(null);
  const [elapsed, setElapsed] = useState(0);

  useEffect(() => {
    if (localVideo.current) localVideo.current.srcObject = localStream;
  }, [localStream]);
  useEffect(() => {
    if (remoteVideo.current) remoteVideo.current.srcObject = remoteStream;
  }, [remoteStream, state.status]);
  useEffect(() => {
    if (state.status !== "connected") {
      setElapsed(0);
      return;
    }
    const iv = window.setInterval(() => setElapsed(Math.floor((Date.now() - state.startedAt) / 1000)), 1000);
    return () => window.clearInterval(iv);
  }, [state.status, state.startedAt]);

  if (state.status === "idle") return null;

  const name = nameOf(state.peerId);
  const mmss = (s: number) => `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;

  if (state.status === "incoming") {
    return (
      <div className="call-ring">
        <div className="ring-card">
          <Avatar id={state.peerId} name={name} size={76} src={avatarOf(state.peerId)} />
          <div className="ring-name">{name}</div>
          <div className="ring-sub">{state.video ? t("call.incomingVideo") : t("call.incomingVoice")}</div>
          <div className="ring-actions">
            <button className="ring-btn decline" onClick={call.decline} type="button" aria-label={t("call.decline")}>
              <Icon name="hangup" size={24} />
            </button>
            <button
              className="ring-btn accept"
              onClick={() => void call.accept()}
              type="button"
              aria-label={t("call.accept")}
            >
              <Icon name="phone" size={24} />
            </button>
          </div>
        </div>
      </div>
    );
  }

  const statusText =
    state.status === "calling" ? t("call.calling") : state.status === "connecting" ? t("call.connecting") : mmss(elapsed);
  const showRemote = state.video && !!remoteStream;

  return (
    <div className="call-stage">
      <div className="call-remote">
        {showRemote ? (
          <video ref={remoteVideo} autoPlay playsInline className="remote-video" />
        ) : (
          <div className="call-avatar">
            <Avatar id={state.peerId} name={name} size={132} src={avatarOf(state.peerId)} />
          </div>
        )}
        <div className="call-topbar">
          <span className="call-name">{name}</span>
          <span className="call-status">{statusText}</span>
        </div>
        {state.video && (
          <video ref={localVideo} autoPlay playsInline muted className={"local-video" + (state.camOff ? " hidden" : "")} />
        )}
      </div>
      <div className="call-controls">
        <button
          className={"cc-btn" + (state.muted ? " off" : "")}
          onClick={call.toggleMute}
          type="button"
          aria-label={state.muted ? t("call.unmute") : t("call.mute")}
        >
          <Icon name={state.muted ? "micOff" : "mic"} size={22} />
        </button>
        {state.video && (
          <button
            className={"cc-btn" + (state.camOff ? " off" : "")}
            onClick={call.toggleCam}
            type="button"
            aria-label={state.camOff ? t("call.camOn") : t("call.camOff")}
          >
            <Icon name={state.camOff ? "videoOff" : "video"} size={22} />
          </button>
        )}
        <button className="cc-btn hang" onClick={call.hangup} type="button" aria-label={t("call.hangup")}>
          <Icon name="hangup" size={22} />
        </button>
      </div>
    </div>
  );
}
