import { avatarColor, initialsOf } from "../lib/chat";

// An initials avatar with a deterministic color seeded by the subject id, and an optional presence dot.
export function Avatar({
  id,
  name,
  size = 36,
  online,
}: {
  id: string;
  name: string;
  size?: number;
  online?: boolean;
}) {
  return (
    <span className="avatar-wrap" style={{ width: size, height: size }}>
      <span
        className="avatar"
        style={{ background: avatarColor(id || name), fontSize: Math.round(size * 0.4) }}
      >
        {initialsOf(name)}
      </span>
      {online !== undefined && <span className={"av-dot" + (online ? " on" : "")} />}
    </span>
  );
}
