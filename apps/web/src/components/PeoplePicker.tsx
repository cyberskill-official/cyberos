import { useMemo, useState } from "react";
import type { Person } from "../lib/chat";
import { shortId } from "../lib/chat";

export type PickerMode = "dm" | "group" | "add";

// Directory-backed people picker. In "dm" mode a click immediately opens the DM; in "group"/"add" mode the
// user multi-selects and confirms. The directory is best-effort - an empty list shows a helpful note.
export function PeoplePicker({
  mode,
  people,
  me,
  onClose,
  onDm,
  onGroup,
  onAdd,
}: {
  mode: PickerMode;
  people: Person[];
  me: string;
  onClose(): void;
  onDm(subjectId: string): void;
  onGroup(name: string, ids: string[]): void;
  onAdd(ids: string[]): void;
}) {
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Record<string, boolean>>({});
  const [gname, setGname] = useState("");

  const list = useMemo(() => {
    const f = q.trim().toLowerCase();
    return people.filter((p) => {
      if (p.subject_id === me) return false;
      if (!f) return true;
      return ((p.display_name || "") + " " + (p.handle || "") + " " + (p.email || "")).toLowerCase().includes(f);
    });
  }, [people, me, q]);

  function toggle(id: string) {
    if (mode === "dm") {
      onClose();
      onDm(id);
      return;
    }
    setSel((s) => ({ ...s, [id]: !s[id] }));
  }

  function confirm() {
    const ids = Object.keys(sel).filter((k) => sel[k]);
    if (mode === "group") {
      if (!gname.trim() || ids.length === 0) return;
      onClose();
      onGroup(gname.trim(), ids);
    } else if (mode === "add") {
      if (ids.length === 0) return;
      onClose();
      onAdd(ids);
    }
  }

  const title = mode === "dm" ? "New direct message" : mode === "group" ? "New group channel" : "Add people";

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="picker">
        <div className="picker-head">{title}</div>
        {mode === "group" && (
          <input
            className="picker-input"
            placeholder="Channel name"
            value={gname}
            onChange={(e) => setGname(e.target.value)}
          />
        )}
        <input
          className="picker-input"
          placeholder="Search teammates"
          value={q}
          onChange={(e) => setQ(e.target.value)}
          autoFocus
        />
        <div className="picker-people">
          {list.length === 0 && (
            <div className="empty" style={{ padding: 14, fontSize: 13 }}>
              {people.length ? "No teammates match" : "Directory unavailable"}
            </div>
          )}
          {list.map((p) => (
            <div
              key={p.subject_id}
              className={"person" + (sel[p.subject_id] ? " sel" : "")}
              onClick={() => toggle(p.subject_id)}
            >
              <span className="pname">{p.display_name || p.handle || shortId(p.subject_id)}</span>
              <span className="psub">{p.email || p.handle || ""}</span>
            </div>
          ))}
        </div>
        <div className="picker-actions">
          <button className="btn-ghost" onClick={onClose} type="button">
            Cancel
          </button>
          {mode !== "dm" && (
            <button className="btn-pill" onClick={confirm} type="button">
              {mode === "add" ? "Add" : "Create"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
