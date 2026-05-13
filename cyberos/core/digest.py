"""
cyberos.core.digest — deterministic daily summary (PROPOSAL.md P8).

Walks a slice of the audit ledger (default: last 24h) and answers four
questions for the operator:

  1. *What happened?* — counts by op (put / move / delete / etc.)
  2. *Who did it?*    — counts by actor (you / coding-agent / scheduled-task)
  3. *Where?*         — counts by top-level path prefix
                         (memories/, meta/, project/, etc.)
  4. *What stands out?* — the "interesting" change list:
       • decisions/ + drift/ + refinements/ writes (signal-heavy kinds)
       • purges (deletions with mode=purge — irreversible)
       • renames (path moves change downstream consumer URIs)

Determinism is the bedrock of P8: with no LLM in the loop, two invocations
over the same window MUST produce the same output. That makes the digest
safe for diff'd nightly logs and idempotent re-runs.

If the user opts in via ``--via-claude``, we hand the deterministic JSON
to a local Claude process and emit the prose summary it produces. The
deterministic JSON is still the primary artifact — the prose is a layer
ON TOP of it, never instead of it.
"""

from __future__ import annotations

import datetime as _dt
import json
import os
from collections import Counter, defaultdict
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterable

# ---------------------------------------------------------------------------
# config
# ---------------------------------------------------------------------------

# Path prefixes considered "signal-heavy" — writes here surface in the
# Highlights list. Aligned with AGENTS.md v2 §2 / memory kind taxonomy.
_HIGHLIGHT_KIND_PREFIXES: tuple[str, ...] = (
    "memories/decisions/",
    "memories/drift/",
    "memories/refinements/",
)


@dataclass
class DigestRow:
    """One audit row, projected down to the fields the digest cares about."""
    seq: int
    ts_ns: int
    op: str
    path: str
    actor: str
    extra: dict = field(default_factory=dict)


@dataclass
class Digest:
    """The deterministic summary object."""
    store: str
    window_start_ns: int
    window_end_ns: int
    total_rows: int
    op_counts: dict[str, int] = field(default_factory=dict)
    actor_counts: dict[str, int] = field(default_factory=dict)
    prefix_counts: dict[str, int] = field(default_factory=dict)
    highlights: list[DigestRow] = field(default_factory=list)

    def to_json(self, *, indent: int | None = 2) -> str:
        return json.dumps(asdict(self), indent=indent, sort_keys=True)


# ---------------------------------------------------------------------------
# scan
# ---------------------------------------------------------------------------


def _resolve_window(since: int | None, until: int | None) -> tuple[int, int]:
    """Compute [since, until) in ns from optional epoch-ns inputs.

    Default window: last 24 hours up to ``time.time_ns()``.
    """
    import time
    now_ns = time.time_ns()
    if until is None:
        until = now_ns
    if since is None:
        since = until - 24 * 3600 * 1_000_000_000
    if since >= until:
        raise ValueError(
            f"empty digest window: since={since} >= until={until}"
        )
    return since, until


def _top_prefix(path: str) -> str:
    """Project a memory path down to its top-level category bucket."""
    if not path:
        return "(unknown)"
    parts = path.split("/", 2)
    if len(parts) <= 1:
        return parts[0] + "/" if parts[0] else "(root)"
    if parts[0] == "memories" and len(parts) >= 2:
        return f"memories/{parts[1]}/"
    return parts[0] + "/"


def _is_highlight(rec_op: str, rec_path: str, rec_extra: dict) -> bool:
    """Decide whether a row deserves a slot in the Highlights list."""
    if rec_op == "delete":
        mode = rec_extra.get("mode")
        if mode == "purge":
            return True
    if rec_op in ("rename", "move"):
        return True
    if rec_op in ("put", "create", "str_replace", "insert"):
        for prefix in _HIGHLIGHT_KIND_PREFIXES:
            if rec_path.startswith(prefix):
                return True
    return False


def build(
    store: Path,
    *,
    since_ns: int | None = None,
    until_ns: int | None = None,
    highlight_cap: int = 50,
) -> Digest:
    """Walk the ledger in ``[since_ns, until_ns)`` and produce a Digest."""
    from cyberos.core.walker import MmapWalker  # noqa: WPS433 — lazy heavy import

    since_ns, until_ns = _resolve_window(since_ns, until_ns)

    audit = store / "audit"
    segs = sorted(p for p in audit.glob("*.binlog") if p.name != "current.binlog")
    current = audit / "current.binlog"
    if current.exists():
        segs.append(current)

    op_counts: Counter[str] = Counter()
    actor_counts: Counter[str] = Counter()
    prefix_counts: Counter[str] = Counter()
    highlights: list[DigestRow] = []
    total = 0

    for seg in segs:
        # Cheap pre-filter: sealed monthly segments outside the window can
        # be skipped wholesale. Filename pattern is "YYYY-MM.binlog".
        if seg.name != "current.binlog":
            stem = seg.stem
            try:
                yyyy, mm = stem.split("-")
                year, month = int(yyyy), int(mm)
                # Boundary check: any second of any day in this YYYY-MM.
                # If that range entirely precedes `since_ns` we can skip;
                # if it entirely follows `until_ns` we can skip.
                seg_start = _dt.datetime(year, month, 1, tzinfo=_dt.timezone.utc)
                if month == 12:
                    seg_end = _dt.datetime(year + 1, 1, 1, tzinfo=_dt.timezone.utc)
                else:
                    seg_end = _dt.datetime(year, month + 1, 1, tzinfo=_dt.timezone.utc)
                seg_start_ns = int(seg_start.timestamp() * 1_000_000_000)
                seg_end_ns = int(seg_end.timestamp() * 1_000_000_000)
                if seg_end_ns <= since_ns or seg_start_ns >= until_ns:
                    continue
            except ValueError:
                # Non-monthly filename; don't skip.
                pass

        with MmapWalker(seg) as walker:
            for _offset, rec in walker.iter_records():
                if rec.ts_ns < since_ns or rec.ts_ns >= until_ns:
                    continue
                total += 1
                op_counts[rec.op] += 1
                actor_counts[rec.actor] += 1
                prefix_counts[_top_prefix(rec.path)] += 1
                if _is_highlight(rec.op, rec.path, rec.extra) and len(highlights) < highlight_cap:
                    highlights.append(DigestRow(
                        seq=int(rec.extra.get("_seq", 0)),
                        ts_ns=rec.ts_ns,
                        op=rec.op,
                        path=rec.path,
                        actor=rec.actor,
                        extra={
                            k: v for k, v in rec.extra.items()
                            if k in ("mode", "kind", "reason")
                        },
                    ))

    return Digest(
        store=str(store),
        window_start_ns=since_ns,
        window_end_ns=until_ns,
        total_rows=total,
        op_counts=dict(sorted(op_counts.items())),
        actor_counts=dict(sorted(actor_counts.items())),
        prefix_counts=dict(sorted(prefix_counts.items())),
        highlights=highlights,
    )


# ---------------------------------------------------------------------------
# render
# ---------------------------------------------------------------------------


def _fmt_ts(ns: int) -> str:
    return _dt.datetime.fromtimestamp(ns / 1e9, tz=_dt.timezone.utc).strftime(
        "%Y-%m-%d %H:%M:%S UTC",
    )


def format_text(d: Digest) -> str:
    lines: list[str] = []
    lines.append(f"BRAIN digest — {d.store}")
    lines.append(f"  window : {_fmt_ts(d.window_start_ns)} → {_fmt_ts(d.window_end_ns)}")
    lines.append(f"  rows   : {d.total_rows}")
    lines.append("")
    if d.total_rows == 0:
        lines.append("  (no audit activity in this window)")
        return "\n".join(lines)

    lines.append("  by op:")
    for op, n in sorted(d.op_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        lines.append(f"    {n:>5}  {op}")
    lines.append("")

    lines.append("  by actor:")
    for actor, n in sorted(d.actor_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        lines.append(f"    {n:>5}  {actor}")
    lines.append("")

    lines.append("  by area:")
    for prefix, n in sorted(d.prefix_counts.items(), key=lambda kv: (-kv[1], kv[0])):
        lines.append(f"    {n:>5}  {prefix}")
    lines.append("")

    if d.highlights:
        lines.append(f"  highlights ({len(d.highlights)}):")
        for h in d.highlights:
            note = ""
            if h.extra.get("mode") == "purge":
                note = " [GDPR PURGE]"
            elif h.op in ("rename", "move"):
                note = " [path moved]"
            elif h.op in ("put", "create", "str_replace", "insert"):
                kind_marker = ""
                for prefix in _HIGHLIGHT_KIND_PREFIXES:
                    if h.path.startswith(prefix):
                        kind_marker = f" [{prefix.rstrip('/').rsplit('/', 1)[-1]}]"
                        break
                note = kind_marker
            lines.append(
                f"    {_fmt_ts(h.ts_ns)}  {h.op:<11}  {h.path}  by {h.actor}{note}"
            )

    return "\n".join(lines)


def format_markdown(d: Digest) -> str:
    lines: list[str] = []
    title_window = (
        f"{_fmt_ts(d.window_start_ns).split()[0]} → {_fmt_ts(d.window_end_ns).split()[0]}"
    )
    lines.append(f"# BRAIN digest — {title_window}")
    lines.append("")
    lines.append(f"Store: `{d.store}`  ·  rows in window: **{d.total_rows}**")
    lines.append("")
    if d.total_rows == 0:
        lines.append("_No audit activity in this window._")
        return "\n".join(lines)

    if d.op_counts:
        lines.append("## Activity by op")
        lines.append("")
        lines.append("| op | count |")
        lines.append("|---|---:|")
        for op, n in sorted(d.op_counts.items(), key=lambda kv: (-kv[1], kv[0])):
            lines.append(f"| `{op}` | {n} |")
        lines.append("")

    if d.actor_counts:
        lines.append("## Activity by actor")
        lines.append("")
        lines.append("| actor | count |")
        lines.append("|---|---:|")
        for actor, n in sorted(d.actor_counts.items(), key=lambda kv: (-kv[1], kv[0])):
            lines.append(f"| `{actor}` | {n} |")
        lines.append("")

    if d.prefix_counts:
        lines.append("## Activity by area")
        lines.append("")
        lines.append("| area | count |")
        lines.append("|---|---:|")
        for prefix, n in sorted(d.prefix_counts.items(), key=lambda kv: (-kv[1], kv[0])):
            lines.append(f"| `{prefix}` | {n} |")
        lines.append("")

    if d.highlights:
        lines.append(f"## Highlights ({len(d.highlights)})")
        lines.append("")
        for h in d.highlights:
            marker = ""
            if h.extra.get("mode") == "purge":
                marker = " — **GDPR purge**"
            elif h.op in ("rename", "move"):
                marker = " — path moved"
            lines.append(
                f"- `{_fmt_ts(h.ts_ns)}` · **{h.op}** `{h.path}` by `{h.actor}`{marker}"
            )

    return "\n".join(lines)


def claude_prose(d: Digest, *, model: str | None = None, timeout: float = 30.0) -> str:
    """Pipe the JSON digest to a local Claude process and return its prose.

    Optional. Skipped silently if the ``claude`` CLI is not on PATH — the
    deterministic text/markdown digest is always available as primary output.

    Heuristic: we shell out to ``claude --print --output-format=text`` and
    pass a fixed framing prompt. The model has full freedom on phrasing
    but the prompt pins the JSON it must reason from so the digest is
    grounded in deterministic facts, not invented detail.
    """
    import shutil
    import subprocess

    cmd_path = shutil.which("claude")
    if cmd_path is None:
        return "[claude CLI not on PATH — install Claude Code to enable prose summaries]"

    framing = (
        "You are an analyst summarising a developer's daily activity from a "
        "deterministic JSON audit digest. Write a tight 5-7 sentence prose "
        "summary in plain past tense — no bullet points, no headers, no "
        "preamble. Focus on the highlights list (decisions, drift, "
        "refinements, purges, renames) — those carry the meaning. "
        "Mention actor names verbatim. If the window is empty, say so in "
        "one sentence. The JSON follows.\n\n"
    )
    payload = framing + d.to_json()

    cmd = [cmd_path, "--print", "--output-format=text"]
    if model:
        cmd += ["--model", model]
    try:
        result = subprocess.run(
            cmd,
            input=payload, text=True,
            capture_output=True, timeout=timeout,
            check=False,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired) as exc:
        return f"[claude prose skipped: {type(exc).__name__}: {exc}]"
    if result.returncode != 0:
        return f"[claude prose failed: exit {result.returncode}; {result.stderr.strip()[:200]}]"
    return result.stdout.strip()


# ---------------------------------------------------------------------------
# parsing convenience for the CLI
# ---------------------------------------------------------------------------


def parse_human_duration(text: str) -> int:
    """Parse strings like ``24h``, ``7d``, ``30m``, ``2w`` → nanoseconds.

    Accepted units: s (seconds), m (minutes), h (hours), d (days), w (weeks).
    Multi-token strings (``1d 6h``) are not supported — keep CLI args atomic.
    """
    text = text.strip().lower()
    if not text:
        raise ValueError("empty duration")
    unit = text[-1]
    body = text[:-1]
    try:
        value = float(body) if "." in body else int(body)
    except ValueError as exc:
        raise ValueError(f"unparseable duration {text!r}: {exc}") from exc
    factor_ns = {
        "s": 1_000_000_000,
        "m": 60 * 1_000_000_000,
        "h": 3600 * 1_000_000_000,
        "d": 86_400 * 1_000_000_000,
        "w": 7 * 86_400 * 1_000_000_000,
    }.get(unit)
    if factor_ns is None:
        raise ValueError(f"unknown duration unit {unit!r}; use s|m|h|d|w")
    return int(value * factor_ns)


__all__ = [
    "Digest",
    "DigestRow",
    "build",
    "format_text",
    "format_markdown",
    "claude_prose",
    "parse_human_duration",
]
