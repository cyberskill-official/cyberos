import { useMemo, useState } from "react";
import type { Person } from "../lib/chat";
import { shortId } from "../lib/chat";
import { t } from "../lib/i18n";
import { Avatar } from "./Avatar";
import { Icon } from "./icons";

export type PickerMode = "dm" | "group" | "add" | "call";

// Directory-backed people picker. In "dm" and "call" mode a click acts immediately; in "group"/"add" mode
// the user multi-selects and confirms. The directory is best-effort - an empty list shows a helpful note.
export function PeoplePicker({
  mode,
  people,
  me,
  onClose,
  onDm,
  onGroup,
  onAdd,
  onCall,
}: {
  mode: PickerMode;
  people: Person[];
  me: string;
  onClose(): void;
  onDm(subjectId: string): void;
  onGroup(name: string, ids: string[], visibility: "private" | "public"): void;
  onAdd(ids: string[]): void;
  onCall(subjectId: string): void;
}) {
  const [q, setQ] = useState("");
  const [sel, setSel] = useState<Record<string, boolean>>({});
  const [gname, setGname] = useState("");
  const [gvis, setGvis] = useState<"private" | "public">("private");
  const multi = mode === "group" || mode === "add";

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
    if (mode === "call") {
      onClose();
      onCall(id);
      return;
    }
    setSel((s) => ({ ...s, [id]: !s[id] }));
  }

  function confirm() {
    const ids = Object.keys(sel).filter((k) => sel[k]);
    if (mode === "group") {
      if (!gname.trim() || ids.length === 0) return;
      onClose();
      onGroup(gname.trim(), ids, gvis);
    } else if (mode === "add") {
      if (ids.length === 0) return;
      onClose();
      onAdd(ids);
    }
  }

  const title =
    mode === "dm"
      ? t("sidebar.newDm")
      : mode === "call"
        ? t("picker.startCall")
        : mode === "group"
          ? t("picker.newGroup")
          : t("header.addPeople");

  return (
    <div className="picker-bg" onClick={(e) => e.target === e.currentTarget && onClose()}>
      <div className="picker">
        <div className="picker-head">
          <span>{title}</span>
          <button className="icon-btn" onClick={onClose} type="button" title={t("common.close")}>
            <Icon name="close" size={16} />
          </button>
        </div>
        {mode === "group" && (
          <>
            <input
              className="picker-input"
              placeholder={t("picker.channelName")}
              value={gname}
              onChange={(e) => setGname(e.target.value)}
            />
            <div className="cs-vis">
              <button
                className={"cs-vis-opt" + (gvis === "private" ? " on" : "")}
                onClick={() => setGvis("private")}
                type="button"
              >
                {t("settings.private")}
                <span className="cs-vis-sub">{t("settings.privateSub")}</span>
              </button>
              <button
                className={"cs-vis-opt" + (gvis === "public" ? " on" : "")}
                onClick={() => setGvis("public")}
                type="button"
              >
                {t("settings.public")}
                <span className="cs-vis-sub">{t("settings.publicSub")}</span>
              </button>
            </div>
          </>
        )}
        <input
          className="picker-input"
          placeholder={t("picker.searchTeammates")}
          value={q}
          onChange={(e) => setQ(e.target.value)}
          autoFocus
        />
        <div className="picker-people">
          {list.length === 0 && (
            <div className="side-empty" style={{ padding: 16 }}>
              {people.length ? t("picker.noMatch") : t("picker.directoryUnavailable")}
            </div>
          )}
          {list.map((p) => {
            const label = p.display_name || p.handle || shortId(p.subject_id);
            const on = !!sel[p.subject_id];
            return (
              <div
                key={p.subject_id}
                className={"person" + (on ? " sel" : "")}
                onClick={() => toggle(p.subject_id)}
              >
                <Avatar id={p.subject_id} name={label} size={34} src={p.avatar || undefined} />
                <div className="person-meta">
                  <span className="pname">{label}</span>
                  <span className="psub">{p.email || p.handle || ""}</span>
                </div>
                {multi && <span className={"pcheck" + (on ? " on" : "")}>{on && <Icon name="check" size={14} />}</span>}
              </div>
            );
          })}
        </div>
        <div className="picker-actions">
          <button className="btn-ghost" onClick={onClose} type="button">
            {t("common.cancel")}
          </button>
          {multi && (
            <button className="btn-pill" onClick={confirm} type="button">
              {mode === "add" ? t("common.add") : t("common.create")}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
