"""TASK-MEMORY-109 — Claude Code hook capture.

Claude Code's hook system fires JSON events on stdin at named lifecycle
points: ``PreToolUse``, ``PostToolUse``, ``SubagentStop``, ``SessionEnd``,
``UserPromptSubmit``, etc. This module converts those events into memory
``put`` rows under a canonical path layout so every Claude Code session
on every machine the user runs leaves a permanent audit trail in the
personal memory.

Path layout (DEC-095):

    memories/claude-code/<yyyy-mm-dd>/<session_id>/<event_kind>-<ts_ns>.md

Each row's frontmatter carries:

* ``kind: claude-code-event``
* ``event: <PreToolUse|PostToolUse|…>``
* ``session_id`` — Claude Code's session identifier
* ``tool`` — when the event refers to a tool (PostToolUse)
* ``cwd`` — the working directory at the time
* ``sync_class`` — defaults to ``private`` (set by AGENTS.md §11 default)
* ``pii_policy`` — defaults to ``redact`` for hook captures since they may
  contain raw paths / args / output snippets

The CLI entry point is ``cyberos hook capture --event <kind>`` which reads
the JSON event from stdin, builds the row, and calls into the existing
``cyberos.core.writer`` to persist it.

Install pattern (user-side, per AGENTS.md §11 hook-install playbook)::

    {
        "hooks": {
            "PostToolUse": "cyberos hook capture --event PostToolUse",
            "SessionEnd":  "cyberos hook capture --event SessionEnd"
        }
    }
"""

from __future__ import annotations

import json
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

# Closed set — adding a new event kind requires an ADR + matching test.
SUPPORTED_EVENTS: frozenset[str] = frozenset({
    "PreToolUse",
    "PostToolUse",
    "SubagentStop",
    "SessionEnd",
    "UserPromptSubmit",
    "Stop",
})


@dataclass(frozen=True)
class HookCapture:
    """Normalized view of a Claude Code hook event."""

    event: str
    session_id: str
    cwd: str
    tool: str | None
    ts_ns: int
    raw_payload: dict[str, Any]

    def storage_path(self) -> str:
        """Return the canonical ``memories/...`` path this capture should land at."""
        # YYYY-MM-DD bucketing keeps the daily-folder count bounded.
        date = time.strftime("%Y-%m-%d", time.gmtime(self.ts_ns / 1_000_000_000))
        # Include session_id + ts_ns so multiple events in the same session
        # don't clash + are time-orderable on disk.
        return (
            f"memories/claude-code/{date}/{self.session_id}/"
            f"{self.event}-{self.ts_ns}.md"
        )

    def frontmatter(self) -> dict[str, Any]:
        fm: dict[str, Any] = {
            "kind": "claude-code-event",
            "event": self.event,
            "session_id": self.session_id,
            "cwd": self.cwd,
            "ts_ns": self.ts_ns,
            "sync_class": "private",
            "pii_policy": "redact",
            "source": "task-memory-109",
        }
        if self.tool is not None:
            fm["tool"] = self.tool
        return fm

    def body(self) -> str:
        """Human-readable summary + raw payload appendix."""
        head = f"# Claude Code · {self.event}\n\n"
        meta = (
            f"- **Session:** `{self.session_id}`\n"
            f"- **Working dir:** `{self.cwd}`\n"
        )
        if self.tool:
            meta += f"- **Tool:** `{self.tool}`\n"
        # The payload is included verbatim so future searches can find it.
        payload_json = json.dumps(self.raw_payload, indent=2, sort_keys=True)
        return head + meta + "\n## Payload\n\n```json\n" + payload_json + "\n```\n"


def parse_event(event_kind: str, payload: dict[str, Any]) -> HookCapture:
    """Normalize a Claude Code hook payload into a :class:`HookCapture`.

    Raises ``ValueError`` if the event kind isn't in :data:`SUPPORTED_EVENTS`
    (closed-enum discipline mirrors AGENTS.md §3.6).
    """
    if event_kind not in SUPPORTED_EVENTS:
        raise ValueError(
            f"unsupported event kind {event_kind!r}; expected one of "
            f"{sorted(SUPPORTED_EVENTS)}"
        )
    session_id = str(payload.get("session_id") or payload.get("sessionId") or "unknown")
    cwd = str(payload.get("cwd") or payload.get("working_directory") or "")
    tool: str | None = None
    if event_kind in {"PreToolUse", "PostToolUse"}:
        # Claude Code sends 'tool_name' in the payload for tool-related events.
        tool = payload.get("tool_name") or payload.get("tool")
        if tool is not None:
            tool = str(tool)
    return HookCapture(
        event=event_kind,
        session_id=session_id,
        cwd=cwd,
        tool=tool,
        ts_ns=time.time_ns(),
        raw_payload=payload,
    )


def capture_from_stdin(event_kind: str, stdin: Any = None) -> HookCapture:
    """Read a hook payload from stdin (or the provided handle) and parse it.

    The CLI binds this to argv[--event] + actual sys.stdin.
    """
    stream = stdin if stdin is not None else sys.stdin
    raw = stream.read()
    if not raw.strip():
        payload: dict[str, Any] = {}
    else:
        try:
            payload = json.loads(raw)
        except json.JSONDecodeError as e:
            raise ValueError(f"hook payload is not valid JSON: {e}") from e
    return parse_event(event_kind, payload)


def write_to_memory(
    capture: HookCapture,
    *,
    store_root: Path,
    writer: Any = None,
) -> Path:
    """Persist a hook capture as a memory ``put`` row.

    `writer` is the application-supplied ``cyberos.core.writer.Writer`` (or
    a test double). The function returns the absolute path of the row file.

    Slice 1: if no writer is provided we fall back to writing the file
    directly under ``<store_root>/<capture.storage_path()>`` with a yaml
    frontmatter header. The production CLI wires the real writer so the
    audit chain advances correctly.
    """
    body = capture.body()
    frontmatter = capture.frontmatter()
    path = capture.storage_path()

    if writer is not None:
        return writer.put(path, body=body, frontmatter=frontmatter)

    # Fallback direct-write (development + offline mode).
    full_path = store_root / path
    full_path.parent.mkdir(parents=True, exist_ok=True)
    fm_yaml = "\n".join(f"{k}: {json.dumps(v)}" for k, v in frontmatter.items())
    full_path.write_text(f"---\n{fm_yaml}\n---\n\n{body}", encoding="utf-8")
    return full_path
