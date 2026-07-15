"""backlog_reader — parse docs/tasks/BACKLOG.md into structured task rows.

The BACKLOG table has shape:
    | TASK-ID | Title | Pri | Status | Depends on | Effort |

This module exposes:
    parse_backlog(path)        → list[TaskRow]
    next_eligible(rows, module, current_status="ready_to_implement")
                                → TaskRow | None — first task matching filter
                                  whose dep cone is all `done`.
    routed_back_count(task_id, audit_dir)
                                → int — how many times this task has been
                                  rework-routed in the current memory chain.

Used by the `cyberos-cuo drain` subcommand to walk module-scoped tasks.
Added 2026-05-19 (Phase 5 of supervisor build, post-STATUS-WAVE).
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


# Task IDs look like TASK-<MODULE>-<NNN> — module slug is alphanumeric (no hyphens)
# Note: TASK-IDs may be wrapped in ** markdown bold markers.
_TASK_ROW_RE = re.compile(
    r"^\|\s*\*{0,2}(?P<task_id>TASK-[A-Z]+-\d+)\*{0,2}\s*\|"
    r"\s*(?P<title>[^|]+?)\s*\|"
    r"\s*(?P<priority>[^|]*?)\s*\|"
    r"\s*(?P<status>[^|]+?)\s*\|"
    r"\s*(?P<deps>[^|]*?)\s*\|"
    r"\s*(?P<effort>[^|]*?)\s*\|",
    re.MULTILINE,
)
_TASK_ID_RE = re.compile(r"TASK-[A-Z]+-\d+")


@dataclass
class TaskRow:
    task_id: str
    title: str
    priority: str
    status: str
    deps: list[str] = field(default_factory=list)
    effort: str = ""
    line_number: int = 0  # 1-indexed row in BACKLOG.md (table mode only; 0 from specs)
    spec_path: Optional[Path] = None  # set in spec mode — the frontmatter IS the truth

    @property
    def module(self) -> str:
        """Module slug extracted from TASK-<MODULE>-NNN."""
        m = re.match(r"TASK-([A-Z]+)-\d+", self.task_id)
        return m.group(1).lower() if m else ""

    def __repr__(self) -> str:
        return f"TaskRow({self.task_id} [{self.status}] {self.priority} deps={self.deps})"


# ── Spec-frontmatter mode ────────────────────────────────────────────────────
#
# BACKLOG.md is an ORPHAN. Three facts, none of them caused by the fr->task rename:
#
#   1. Its own header says so: "Source of truth = task frontmatter. This file lists
#      ONLY remaining work."
#   2. `docs/status` already absorbed it. status-app.js:
#          LEGACY = { roadmap: "board", backlog: "table", changelog: "timeline" }
#      and render-status-hub.mjs declares its inputs as "task frontmatter,
#      CHANGELOG.md version sections, VERSION". It never opens BACKLOG.md.
#   3. Nothing generates the table shape any more. The file on disk is 357 bullets
#      carrying no priority, no depends_on and no effort — so the table regex below
#      matched 0 rows, next_eligible() returned None, and the applier's
#      `line.startswith("|")` guard silently skipped every write.
#
# Net effect: `ship-tasks` could neither read nor write the queue, and reported
# "no eligible task" forever. It failed silently because an empty parse is not an
# error.
#
# Fix: read what the status hub reads. The 507 spec.md frontmatters are the single
# source of truth for id / title / status / priority / depends_on. The table path
# is kept for back-compat (tests, and any repo whose BACKLOG really is a table).
_FM_RE = re.compile(r"\A---\r?\n(.*?)\r?\n---", re.DOTALL)
_FM_SCALAR = re.compile(r"^(?P<k>[a-z_]+):\s*(?P<v>.*?)\s*$", re.MULTILINE)


def _strip_comment(v: str) -> str:
    """Drop a trailing YAML `# ...` comment, unless the value is quoted.

    Real example that bit us — a status value carrying its own history:
        status: on_hold   # was "blocked" (not a valid status per STATUS-REFERENCE §1)
    Without this, `status` becomes the whole line and no enum check ever matches.
    """
    v = v.strip()
    if v[:1] in ('"', "'"):
        return v
    return v.split("#", 1)[0].strip()


def _frontmatter(text: str) -> dict[str, str]:
    m = _FM_RE.match(text)
    if not m:
        return {}
    return {mm.group("k"): _strip_comment(mm.group("v"))
            for mm in _FM_SCALAR.finditer(m.group(1))}


def parse_specs(tasks_root: Path) -> list[TaskRow]:
    """Hydrate the queue from `docs/tasks/<module>/TASK-*/spec.md` frontmatter.

    Same input set render-status-hub.mjs uses, so the CLI and the status page can
    never disagree about what is eligible.
    """
    rows: list[TaskRow] = []
    if not tasks_root.is_dir():
        return rows
    for mod in sorted(p for p in tasks_root.iterdir() if p.is_dir()):
        if mod.name.startswith((".", "_")):
            continue
        for d in sorted(p for p in mod.iterdir() if p.is_dir()):
            if not d.name.startswith("TASK-"):
                continue
            spec = d / "spec.md"
            if not spec.is_file():
                continue
            fm = _frontmatter(spec.read_text(encoding="utf-8"))
            tid = (fm.get("id") or "").strip().strip('"\'')
            if not tid.startswith("TASK-"):
                continue
            rows.append(TaskRow(
                task_id=tid,
                title=(fm.get("title") or "").strip().strip('"\''),
                priority=(fm.get("priority") or "").strip(),
                status=(fm.get("status") or "").strip(),
                deps=_TASK_ID_RE.findall(fm.get("depends_on") or ""),
                effort=(fm.get("effort_hours") or "").strip(),
                spec_path=spec,
            ))
    return rows


def parse_backlog(backlog_path: Path) -> list[TaskRow]:
    """Return every task as a structured TaskRow.

    Table mode: if BACKLOG.md contains table rows, parse them and record the
    1-indexed line_number so the applier can do its optimistic-concurrency write.

    Spec mode (the live path): if the table yields nothing, fall back to the real
    source of truth — the spec.md frontmatters next to this file. See the note
    above on why BACKLOG.md is an orphan.
    """
    text = backlog_path.read_text(encoding="utf-8")
    rows: list[TaskRow] = []
    for line_idx, line in enumerate(text.splitlines(), start=1):
        m = _TASK_ROW_RE.match(line)
        if m is None:
            continue
        gd = m.groupdict()
        # Skip the header row template ("| TASK-ID | Title | ...") — task_id wouldn't
        # actually start with TASK- though, so we additionally check.
        if not gd["task_id"].startswith("TASK-"):
            continue
        # Parse dependency cell — extract every TASK-X-NNN occurrence.
        deps_raw = gd["deps"] or ""
        deps = _TASK_ID_RE.findall(deps_raw)
        rows.append(TaskRow(
            task_id=gd["task_id"].strip(),
            title=gd["title"].strip(),
            priority=gd["priority"].strip(),
            status=gd["status"].strip(),
            deps=deps,
            effort=gd["effort"].strip(),
            line_number=line_idx,
        ))
    if rows:
        return rows
    return parse_specs(backlog_path.parent)


def next_eligible(
    rows: list[TaskRow],
    module: Optional[str] = None,
    current_status: str | list[str] | tuple[str, ...] | None = None,
    rework: bool = False,
    skip_task_ids: set[str] | None = None,
) -> Optional[TaskRow]:
    """Return the first task in the matching status list whose deps are all `done`.

    If `current_status` is None, defaults to all active statuses:
    ("ready_to_implement", "implementing", "ready_to_review", "reviewing", "ready_to_test", "testing").
    If `rework` is True, "done" is also added to the active status set.
    """
    if current_status is None:
        statuses = ["ready_to_implement", "implementing", "ready_to_review", "reviewing", "ready_to_test", "testing"]
        if rework:
            statuses.append("done")
        statuses = tuple(statuses)
    elif isinstance(current_status, str):
        statuses = (current_status,)
    else:
        statuses = tuple(current_status)

    done_ids = {r.task_id for r in rows if r.status == "done"}
    for row in rows:
        if row.status not in statuses:
            continue
        if skip_task_ids and row.task_id in skip_task_ids:
            continue
        if module and row.module != module.lower():
            continue
        if all(dep in done_ids for dep in row.deps):
            return row
    return None


def list_eligible(
    rows: list[TaskRow],
    module: Optional[str] = None,
    current_status: str | list[str] | tuple[str, ...] | None = None,
    rework: bool = False,
) -> list[TaskRow]:
    """List ALL eligible tasks (same filter as next_eligible) for visibility."""
    if current_status is None:
        statuses = ["ready_to_implement", "implementing", "ready_to_review", "reviewing", "ready_to_test", "testing"]
        if rework:
            statuses.append("done")
        statuses = tuple(statuses)
    elif isinstance(current_status, str):
        statuses = (current_status,)
    else:
        statuses = tuple(current_status)

    done_ids = {r.task_id for r in rows if r.status == "done"}
    out = []
    for row in rows:
        if row.status not in statuses:
            continue
        if module and row.module != module.lower():
            continue
        if all(dep in done_ids for dep in row.deps):
            out.append(row)
    return out


def routed_back_count(task_id: str, audit_dir: Path) -> int:
    """Count how many times `task_id` has been rework-routed in the audit chain.

    Walks the latest binlog segments looking for memory.task_routed_back rows
    whose payload.task_id matches. Returns 0 if no such rows or audit_dir missing.

    For Phase 5 this is an approximation — production should scan all segments;
    here we just walk current.binlog + any *.binlog files in audit_dir.
    """
    if not audit_dir.is_dir():
        return 0
    count = 0
    # The binlog is binary; the simplest parse is to look for the task ID and
    # event kind as raw bytes. The kind string `memory.task_routed_back` will
    # appear verbatim near each instance.
    target = f'"task_id":"{task_id}"'.encode("utf-8")
    kind = b'memory.task_routed_back'
    for binlog in audit_dir.glob("*.binlog"):
        try:
            data = binlog.read_bytes()
        except OSError:
            continue
        # Look for co-occurring kind + task_id in proximity (within 256 bytes).
        idx = 0
        while True:
            k = data.find(kind, idx)
            if k < 0:
                break
            # Check whether target task id appears in the next 256 bytes
            if target in data[k:k + 256]:
                count += 1
            idx = k + len(kind)
    return count
