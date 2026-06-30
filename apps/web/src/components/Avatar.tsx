import { avatarColor, initialsOf } from "../lib/chat";

// An avatar: an uploaded image when one is set, else initials on a deterministic color. Optional presence dot.
export function Avatar({
  id,
  name,
  size = 36,
  online,
  src,
}: {
  id: string;
  name: string;
  size?: number;
  online?: boolean;
  src?: string | null;
}) {
  return (
    <span className="avatar-wrap" style={{ width: size, height: size }}>
      {src ? (
        <img className="avatar avatar-img" src={src} alt={name} />
      ) : (
        <span
          className="avatar"
          style={{ background: avatarColor(id || name), fontSize: Math.round(size * 0.4) }}
        >
          {initialsOf(name)}
        </span>
      )}
      {online !== undefined && <span className={"av-dot" + (online ? " on" : "")} />}
    </span>
  );
}
