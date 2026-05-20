"""backlog_reader — parse docs/feature-requests/BACKLOG.md into structured FR rows.

The BACKLOG table has shape:
    | FR-ID | Title | Pri | Status | Depends on | Effort |

This module exposes:
    parse_backlog(path)        → list[FrRow]
    next_eligible(rows, module, current_status="ready_to_implement")
                                → FrRow | None — first FR matching filter
                                  whose dep cone is all `done`.
    routed_back_count(fr_id, audit_dir)
                                → int — how many times this FR has been
                                  rework-routed in the current memory chain.

Used by the `cyberos-cuo drain` subcommand to walk module-scoped FRs.
Added 2026-05-19 (Phase 5 of supervisor build, post-STATUS-WAVE).
"""

from __future__ import annotations

import re
from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional


# FR IDs look like FR-<MODULE>-<NNN> — module slug is alphanumeric (no hyphens)
_FR_ROW_RE = re.compile(
    r"^\|\s*(?P<fr_id>FR-[A-Z]+-\d+)\s*\|"
    r"\s*(?P<title>[^|]+?)\s*\|"
    r"\s*(?P<priority>[^|]*?)\s*\|"
    r"\s*(?P<status>[^|]+?)\s*\|"
    r"\s*(?P<deps>[^|]*?)\s*\|"
    r"\s*(?P<effort>[^|]*?)\s*\|",
    re.MULTILINE,
)
_FR_ID_RE = re.compile(r"FR-[A-Z]+-\d+")


@dataclass
class FrRow:
    fr_id: str
    title: str
    priority: str
    status: str
    deps: list[str] = field(default_factory=list)
    effort: str = ""
    line_number: int = 0  # 1-indexed for the matching row in BACKLOG.md

    @property
    def module(self) -> str:
        """Module slug extracted from FR-<MODULE>-NNN."""
        m = re.match(r"FR-([A-Z]+)-\d+", self.fr_id)
        return m.group(1).lower() if m else ""

    def __repr__(self) -> str:
        return f"FrRow({self.fr_id} [{self.status}] {self.priority} deps={self.deps})"


def parse_backlog(backlog_path: Path) -> list[FrRow]:
    """Read BACKLOG.md and return every FR row as a structured FrRow.

    Skips header rows (where col 1 is literally "FR-ID" or column count < 6).
    Captures the 1-indexed line_number so callers can correlate back to file
    positions (e.g. for the backlog-state-update-author applier).
    """
    text = backlog_path.read_text(encoding="utf-8")
    rows: list[FrRow] = []
    for line_idx, line in enumerate(text.splitlines(), start=1):
        m = _FR_ROW_RE.match(line)
        if m is None:
            continue
        gd = m.groupdict()
        # Skip the header row template ("| FR-ID | Title | ...") — fr_id wouldn't
        # actually start with FR- though, so we additionally check.
        if not gd["fr_id"].startswith("FR-"):
            continue
        # Parse dependency cell — extract every FR-X-NNN occurrence.
        deps_raw = gd["deps"] or ""
        deps = _FR_ID_RE.findall(deps_raw)
        rows.append(FrRow(
            fr_id=gd["fr_id"].strip(),
            title=gd["title"].strip(),
            priority=gd["priority"].strip(),
            status=gd["status"].strip(),
            deps=deps,
            effort=gd["effort"].strip(),
            line_number=line_idx,
        ))
    return rows


def next_eligible(
    rows: list[FrRow],
    module: Optional[str] = None,
    current_status: str | list[str] | tuple[str, ...] | None = None,
    rework: bool = False,
) -> Optional[FrRow]:
    """Return the first FR in the matching status list whose deps are all `done`.

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

    done_ids = {r.fr_id for r in rows if r.status == "done"}
    for row in rows:
        if row.status not in statuses:
            continue
        if module and row.module != module.lower():
            continue
        if all(dep in done_ids for dep in row.deps):
            return row
    return None


def list_eligible(
    rows: list[FrRow],
    module: Optional[str] = None,
    current_status: str | list[str] | tuple[str, ...] | None = None,
    rework: bool = False,
) -> list[FrRow]:
    """List ALL eligible FRs (same filter as next_eligible) for visibility."""
    if current_status is None:
        statuses = ["ready_to_implement", "implementing", "ready_to_review", "reviewing", "ready_to_test", "testing"]
        if rework:
            statuses.append("done")
        statuses = tuple(statuses)
    elif isinstance(current_status, str):
        statuses = (current_status,)
    else:
        statuses = tuple(current_status)

    done_ids = {r.fr_id for r in rows if r.status == "done"}
    out = []
    for row in rows:
        if row.status not in statuses:
            continue
        if module and row.module != module.lower():
            continue
        if all(dep in done_ids for dep in row.deps):
            out.append(row)
    return out


def routed_back_count(fr_id: str, audit_dir: Path) -> int:
    """Count how many times `fr_id` has been rework-routed in the audit chain.

    Walks the latest binlog segments looking for memory.fr_routed_back rows
    whose payload.fr_id matches. Returns 0 if no such rows or audit_dir missing.

    For Phase 5 this is an approximation — production should scan all segments;
    here we just walk current.binlog + any *.binlog files in audit_dir.
    """
    if not audit_dir.is_dir():
        return 0
    count = 0
    # The binlog is binary; the simplest parse is to look for the FR ID and
    # event kind as raw bytes. The kind string `memory.fr_routed_back` will
    # appear verbatim near each instance.
    target = f'"fr_id":"{fr_id}"'.encode("utf-8")
    kind = b'memory.fr_routed_back'
    for binlog in audit_dir.glob("*.binlog"):
        try:
            data = binlog.read_bytes()
        except OSError:
            continue
        # Look for co-occurring kind + fr_id in proximity (within 256 bytes).
        idx = 0
        while True:
            k = data.find(kind, idx)
            if k < 0:
                break
            # Check whether target FR id appears in the next 256 bytes
            if target in data[k:k + 256]:
                count += 1
            idx = k + len(kind)
    return count
