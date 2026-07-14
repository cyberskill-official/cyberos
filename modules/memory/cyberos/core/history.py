"""
cyberos.core.history — read-only per-path history projection
(TASK-MEMORY-120, AGENTS.md §13).

Pure projection over the existing audit chain. Emits zero new rows. The
core function ``walk(store, target_path, ...)`` iterates the chain in
reverse, collects rows that touch ``target_path`` (and its prior names
when ``follow_moves=True``), and returns a list of :class:`HistoryEntry`
records with derived ``frontmatter_diff`` and ``body_diff`` annotations
where possible.

What makes a row "touch" a path:

* ``op == "put"`` and ``payload.path == path`` — primary
* ``op == "delete"`` and ``payload.path == path`` — tombstone / purge
* ``op == "move"`` and either ``payload.src == path`` or
  ``payload.dst == path``. When ``follow_moves=True`` (default), a move
  row encountered with ``dst == current-target`` adds ``src`` to the
  tracked-paths set so subsequent older rows under that prior name are
  also included.
* Any other audit-row kind whose ``path`` field equals ``target_path`` —
  this catches ``memory.importance_scored`` (TASK-MEMORY-114),
  ``dream.proposal_applied`` (TASK-MEMORY-115), ``memory.acl_denied``
  (TASK-MEMORY-117), ``memory.precondition_failed`` (TASK-MEMORY-118),
  ``episode.logged`` (TASK-MEMORY-112).

The CLI surfaces a human-readable view; the function returns structured
``HistoryEntry`` records that the REST endpoint and other consumers can
project as JSON.
"""

from __future__ import annotations

import difflib
import json
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterator, Optional


# ────────────────────────────────────────────────────────────────────
# HistoryEntry
# ────────────────────────────────────────────────────────────────────


@dataclass
class HistoryEntry:
    """One row touching the target path, projected with optional diffs."""

    seq: int
    ts: datetime
    kind: str
    actor: str
    body_hash: Optional[str] = None
    frontmatter_diff: Optional[dict] = None
    body_diff: Optional[str] = None
    extra: dict = field(default_factory=dict)
    path: str = ""

    def to_dict(self) -> dict:
        return {
            "seq": self.seq,
            "ts": self.ts.isoformat() if isinstance(self.ts, datetime) else str(self.ts),
            "kind": self.kind,
            "actor": self.actor,
            "body_hash": self.body_hash,
            "frontmatter_diff": self.frontmatter_diff,
            "body_diff": self.body_diff,
            "extra": dict(self.extra),
            "path": self.path,
        }


# ────────────────────────────────────────────────────────────────────
# core walk
# ────────────────────────────────────────────────────────────────────


def _parse_iso(ts: str | int | None) -> datetime:
    """Robust ISO / ns-epoch → datetime conversion."""
    if isinstance(ts, int):
        # Treat as ns since epoch
        return datetime.fromtimestamp(ts / 1e9, tz=timezone.utc)
    if isinstance(ts, str):
        try:
            return datetime.fromisoformat(ts.replace("Z", "+00:00"))
        except ValueError:
            return datetime.now(timezone.utc)
    return datetime.now(timezone.utc)


def walk(
    store: Path,
    target_path: str,
    *,
    follow_moves: bool = True,
    since: Optional[datetime] = None,
    limit: Optional[int] = None,
    show_body: bool = False,
) -> list[HistoryEntry]:
    """Project the audit chain into a list of HistoryEntry records for
    ``target_path``. Returns most-recent-first by default.
    """
    from cyberos.core.dream._audit_iter import iter_audit_rows  # lazy

    # Collect rows in chain order (oldest first), filter to our paths,
    # then reverse at the end for most-recent-first output.
    all_rows = list(iter_audit_rows(store))
    # First pass: figure out the path-set considering moves
    tracked_paths: set[str] = {target_path}
    if follow_moves:
        # Forward sweep — when a move's dst == current target, the src
        # was the prior name. Scan all rows once and union.
        for row in all_rows:
            if row.get("op") != "move":
                continue
            payload = row.get("extra") or {}
            src = payload.get("src")
            dst = payload.get("dst") or row.get("path")
            # The walker's row puts src/dst in extra; older writers may
            # surface src as `path`. We handle both.
            if isinstance(dst, str) and dst in tracked_paths and isinstance(src, str):
                tracked_paths.add(src)
            if isinstance(src, str) and src in tracked_paths and isinstance(dst, str):
                tracked_paths.add(dst)

    # Second pass: collect rows that touch any tracked path
    touched: list[dict] = []
    for row in all_rows:
        if _row_touches_paths(row, tracked_paths):
            if since is not None:
                ts = _parse_iso(row.get("ts_ns"))
                if ts < since:
                    continue
            touched.append(row)

    # Build HistoryEntry list (most-recent-first)
    entries: list[HistoryEntry] = []
    # Track body_hash across put rows to compute diffs
    prev_body_hash_by_path: dict[str, Optional[str]] = {}
    # Build a sha→bytes map by re-reading on disk where possible
    for row in touched:
        op = row.get("op", "?")
        payload = row.get("extra") or {}
        # Resolve the "path" associated with this row
        rel_path = row.get("path") or payload.get("path") or ""
        body_hash: Optional[str] = row.get("content_sha256") or None
        frontmatter_diff: Optional[dict] = None
        body_diff: Optional[str] = None

        # Compute frontmatter diff against previous put on same path
        if op == "put":
            prev_hash = prev_body_hash_by_path.get(rel_path)
            if prev_hash and prev_hash != body_hash:
                # Try to reconstruct prev + current bodies from extras
                # The chain stores hashes; bodies live on disk. The
                # "current" body is the latest write at rel_path; the
                # "previous" is harder to reconstruct without snapshot,
                # so we fall back to whatever extras the writer left.
                if show_body and (store / rel_path).exists():
                    # Show diff between THIS put's hash and the previous —
                    # we only have the current body; expose that as a
                    # one-sided context for the operator.
                    try:
                        current = (store / rel_path).read_bytes().decode("utf-8", errors="replace")
                        body_diff = (
                            f"# (only current body available; prev sha={prev_hash[:12]}…)\n"
                            + "\n".join(f"+ {line}" for line in current.splitlines()[:40])
                        )
                    except Exception:
                        body_diff = None
            prev_body_hash_by_path[rel_path] = body_hash
        elif op == "delete":
            mode = payload.get("mode")
            if mode == "purge":
                # Body redacted by §3.6
                body_diff = None
                body_hash = body_hash  # preserved
        # For aux-row kinds, leave diffs None

        entries.append(HistoryEntry(
            seq=int(payload.get("_seq") or 0),
            ts=_parse_iso(row.get("ts_ns")),
            kind=op,
            actor=row.get("actor", "?"),
            body_hash=body_hash if body_hash else None,
            frontmatter_diff=frontmatter_diff,
            body_diff=body_diff,
            extra={k: v for k, v in payload.items() if not k.startswith("_")},
            path=rel_path,
        ))

    # Apply limit + reverse to most-recent-first
    entries.reverse()
    if limit is not None and limit > 0:
        entries = entries[:limit]
    return entries


def _row_touches_paths(row: dict, paths: set[str]) -> bool:
    """True iff this audit row touches any path in the tracked set."""
    direct = row.get("path")
    if isinstance(direct, str) and direct in paths:
        return True
    payload = row.get("extra") or {}
    if isinstance(payload.get("path"), str) and payload["path"] in paths:
        return True
    if isinstance(payload.get("affected_paths"), list):
        for ap in payload["affected_paths"]:
            if ap in paths:
                return True
    # move rows: src/dst either in payload.extra or as the row's path
    if row.get("op") == "move":
        if isinstance(payload.get("src"), str) and payload["src"] in paths:
            return True
        if isinstance(payload.get("dst"), str) and payload["dst"] in paths:
            return True
    return False


# ────────────────────────────────────────────────────────────────────
# annotation rendering (TASK-MEMORY-120 §1 #4)
# ────────────────────────────────────────────────────────────────────


def render_annotations(extra: dict) -> str:
    """Render the recognised provenance annotations as a compact suffix."""
    parts: list[str] = []
    if "dream_id" in extra and isinstance(extra["dream_id"], str):
        parts.append(f"via dream {extra['dream_id'][:8]}…")
    if "proposal_id" in extra:
        parts.append(f"(proposal {extra['proposal_id']})")
    if "session_id" in extra:
        parts.append(f"during session {extra['session_id']}")
    if "invocation" in extra:
        parts.append(f"via {extra['invocation']}")
    if "imported_from" in extra:
        parts.append(f"imported from {extra['imported_from']}")
    if "merged_into" in extra:
        parts.append(f"merged into {extra['merged_into']}")
    if extra.get("warn_only") is True:
        parts.append("WARN-ONLY")
    return " " + " ".join(parts) if parts else ""


def render_human(entry: HistoryEntry, *, show_body: bool = False) -> str:
    """Render one entry as a one- or multi-line human-readable string."""
    annot = render_annotations(entry.extra)
    body_hash_short = (entry.body_hash[:8] + "…") if entry.body_hash else "—"
    header = (
        f"[{entry.seq:>6}] {entry.ts.isoformat()} {entry.kind:<28} "
        f"{entry.actor:<16} body={body_hash_short}{annot}"
    )
    out = [header]
    if entry.frontmatter_diff:
        for op, fields in entry.frontmatter_diff.items():
            for fld, val in fields.items():
                marker = {"added": "+", "removed": "-", "changed": "~"}.get(op, "?")
                out.append(f"         {marker} {fld}: {val!r}")
    if show_body and entry.body_diff:
        for line in entry.body_diff.splitlines():
            out.append(f"         {line}")
    return "\n".join(out)
