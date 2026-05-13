"""
cyberos.core.session — multi-agent coordination (PROPOSAL.md P11).

The single-writer lock + actor-tagged audit chain already give us
serialisable writes. P11 adds the missing piece for *coordination across
agents*: a way for one agent to publish "I'm working on this subtree
right now" so other agents can see who's online and what they're touching
before stomping on the same file.

Mechanism:

* a session is a JSON file in ``<store>/meta/sessions/<id>.json``
  containing ``{actor, scope, started_at_ns, expires_at_ns, host}``;
* claiming a session bumps the audit chain with a ``session.start`` row
  (already part of the op enum); the audit row is the canonical truth,
  the JSON file is the discovery surface;
* ending a session writes ``session.end`` and removes the JSON;
* leases expire after TTL (default 4 hours) so a crashed agent doesn't
  block forever — the next listing GC sweeps stale files;
* ``scope`` is a list of POSIX path prefixes the agent intends to touch;
  another agent should warn if its scope overlaps an active session's.

This is *advisory*, not *enforcing*. The single-writer lock still
serialises the actual writes; sessions surface intent earlier so two
agents don't both start drafting the same decision and produce
conflicting bodies.
"""

from __future__ import annotations

import json
import os
import secrets
import socket
import time
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Iterable

_DEFAULT_TTL_NS = 4 * 3600 * 1_000_000_000  # 4 hours


def _sessions_dir(store: Path) -> Path:
    d = store / "meta" / "sessions"
    d.mkdir(parents=True, exist_ok=True)
    return d


# ---------------------------------------------------------------------------
# data types
# ---------------------------------------------------------------------------


@dataclass
class Session:
    """One active agent session."""
    id: str
    actor: str
    scope: list[str]
    started_at_ns: int
    expires_at_ns: int
    host: str
    note: str = ""

    @property
    def expired(self) -> bool:
        return time.time_ns() >= self.expires_at_ns

    def overlaps_scope(self, candidate_scope: Iterable[str]) -> list[str]:
        """Return scope prefixes that overlap between this and ``candidate_scope``."""
        out: list[str] = []
        for c in candidate_scope:
            for s in self.scope:
                if c.startswith(s) or s.startswith(c):
                    out.append(s)
        return sorted(set(out))


# ---------------------------------------------------------------------------
# start / end
# ---------------------------------------------------------------------------


def start_session(
    store: Path,
    *,
    actor: str,
    scope: list[str] | None = None,
    ttl_ns: int = _DEFAULT_TTL_NS,
    note: str = "",
) -> Session:
    """Open a new session. Writes ``session.start`` to the audit chain.

    Returns the populated Session; callers should keep ``session.id`` to
    end it later.
    """
    from cyberos.core.writer import AuditRecord, Writer  # noqa: WPS433

    session_id = f"sess-{secrets.token_hex(6)}"
    now = time.time_ns()
    expires = now + ttl_ns
    host = socket.gethostname()
    sess = Session(
        id=session_id, actor=actor,
        scope=list(scope or []),
        started_at_ns=now, expires_at_ns=expires,
        host=host, note=note,
    )

    sdir = _sessions_dir(store)
    json_path = sdir / f"{session_id}.json"
    json_path.write_text(
        json.dumps(asdict(sess), indent=2, sort_keys=True),
        encoding="utf-8",
    )

    with Writer(store) as writer:
        writer.submit(AuditRecord(
            op="session.start",
            path=f"meta/sessions/{session_id}.json",
            actor=actor,
            content_sha256="",
            extra={
                "session_id": session_id,
                "scope": list(scope or []),
                "host": host,
                "expires_at_ns": expires,
                "note": note,
            },
        ))
    return sess


def end_session(
    store: Path,
    session_id: str,
    *,
    actor: str | None = None,
) -> dict:
    """Close a session. Writes ``session.end`` and removes the JSON file.

    Returns a summary dict; raises FileNotFoundError if no such session.
    """
    from cyberos.core.writer import AuditRecord, Writer  # noqa: WPS433

    sdir = _sessions_dir(store)
    json_path = sdir / f"{session_id}.json"
    if not json_path.is_file():
        raise FileNotFoundError(f"no active session {session_id!r}")
    sess = json.loads(json_path.read_text(encoding="utf-8"))
    closing_actor = actor or sess["actor"]
    end_ns = time.time_ns()
    duration_ns = end_ns - sess["started_at_ns"]

    with Writer(store) as writer:
        writer.submit(AuditRecord(
            op="session.end",
            path=f"meta/sessions/{session_id}.json",
            actor=closing_actor,
            content_sha256="",
            extra={
                "session_id": session_id,
                "duration_ns": duration_ns,
            },
        ))
    json_path.unlink()
    return {
        "id": session_id,
        "actor": sess["actor"],
        "closing_actor": closing_actor,
        "duration_ns": duration_ns,
        "scope": sess.get("scope", []),
    }


# ---------------------------------------------------------------------------
# list + GC
# ---------------------------------------------------------------------------


def list_sessions(store: Path, *, include_expired: bool = False) -> list[Session]:
    """Enumerate active sessions on disk. Stale leases get GC'd silently."""
    sdir = _sessions_dir(store)
    out: list[Session] = []
    for path in sorted(sdir.glob("sess-*.json")):
        try:
            raw = json.loads(path.read_text(encoding="utf-8"))
            sess = Session(
                id=raw["id"], actor=raw["actor"],
                scope=raw.get("scope", []),
                started_at_ns=raw["started_at_ns"],
                expires_at_ns=raw["expires_at_ns"],
                host=raw.get("host", "unknown"),
                note=raw.get("note", ""),
            )
        except (OSError, ValueError, KeyError):
            # Corrupt session file — quietly remove it.
            try:
                path.unlink()
            except OSError:
                pass
            continue
        if sess.expired and not include_expired:
            # GC: write a session.end-style closure isn't appropriate
            # (we don't know who/why) — just drop the file. The original
            # session.start is already in the audit chain so the history
            # of "agent X opened a session at T" is preserved.
            try:
                path.unlink()
            except OSError:
                pass
            continue
        out.append(sess)
    return out


def find_scope_conflicts(
    store: Path,
    candidate_scope: list[str],
    *,
    exclude_session_id: str | None = None,
) -> list[tuple[Session, list[str]]]:
    """Return active sessions whose scopes overlap with ``candidate_scope``."""
    active = list_sessions(store)
    out: list[tuple[Session, list[str]]] = []
    for sess in active:
        if exclude_session_id and sess.id == exclude_session_id:
            continue
        overlaps = sess.overlaps_scope(candidate_scope)
        if overlaps:
            out.append((sess, overlaps))
    return out


# ---------------------------------------------------------------------------
# pretty-print
# ---------------------------------------------------------------------------


def format_sessions(sessions: list[Session]) -> str:
    if not sessions:
        return "  no active sessions"
    lines = [f"  {len(sessions)} active session(s):"]
    for s in sessions:
        remaining = max(0, s.expires_at_ns - time.time_ns()) / 1_000_000_000
        scope = ", ".join(s.scope) if s.scope else "(no scope)"
        note = f" — {s.note}" if s.note else ""
        lines.append(
            f"    {s.id}  actor={s.actor}  host={s.host}  "
            f"scope=[{scope}]  expires_in={remaining:.0f}s{note}"
        )
    return "\n".join(lines)


__all__ = [
    "Session",
    "start_session",
    "end_session",
    "list_sessions",
    "find_scope_conflicts",
    "format_sessions",
]
