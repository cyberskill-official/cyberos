"""Tests for TASK-MEMORY-109 Claude Code hook capture."""

from __future__ import annotations

import io
import json
from pathlib import Path

import pytest

from cyberos.core.claude_code_hook import (
    SUPPORTED_EVENTS,
    HookCapture,
    capture_from_stdin,
    parse_event,
    write_to_memory,
)


def test_supported_events_is_closed_enum() -> None:
    assert "PostToolUse" in SUPPORTED_EVENTS
    assert "SessionEnd" in SUPPORTED_EVENTS
    assert len(SUPPORTED_EVENTS) == 6


def test_parse_event_rejects_unknown_kind() -> None:
    with pytest.raises(ValueError) as exc:
        parse_event("MadeUpEvent", {})
    assert "unsupported event kind" in str(exc.value)


def test_parse_event_extracts_session_and_cwd() -> None:
    cap = parse_event("PostToolUse", {
        "session_id": "sess-abc",
        "cwd": "/Users/x/repo",
        "tool_name": "Bash",
    })
    assert cap.session_id == "sess-abc"
    assert cap.cwd == "/Users/x/repo"
    assert cap.tool == "Bash"
    assert cap.event == "PostToolUse"


def test_parse_event_handles_camelcase_session_id() -> None:
    cap = parse_event("SessionEnd", {"sessionId": "sess-xyz", "cwd": "/tmp"})
    assert cap.session_id == "sess-xyz"


def test_parse_event_defaults_when_keys_missing() -> None:
    cap = parse_event("Stop", {})
    assert cap.session_id == "unknown"
    assert cap.cwd == ""
    assert cap.tool is None


def test_storage_path_buckets_by_date() -> None:
    cap = HookCapture(
        event="PostToolUse",
        session_id="s1",
        cwd="/tmp",
        tool="Bash",
        ts_ns=1_700_000_000_000_000_000,  # 2023-11-14
        raw_payload={},
    )
    path = cap.storage_path()
    assert path.startswith("memories/claude-code/2023-11-14/s1/PostToolUse-")
    assert path.endswith(".md")


def test_frontmatter_carries_required_fields() -> None:
    cap = parse_event("PreToolUse", {
        "session_id": "s2",
        "cwd": "/x",
        "tool_name": "Read",
    })
    fm = cap.frontmatter()
    assert fm["kind"] == "claude-code-event"
    assert fm["event"] == "PreToolUse"
    assert fm["tool"] == "Read"
    assert fm["sync_class"] == "private"
    assert fm["pii_policy"] == "redact"
    assert fm["source"] == "fr-memory-109"


def test_body_includes_payload_json() -> None:
    cap = parse_event("PostToolUse", {
        "session_id": "s3",
        "cwd": "/x",
        "tool_name": "Edit",
        "input": {"file_path": "/tmp/foo.py"},
    })
    body = cap.body()
    assert "Claude Code · PostToolUse" in body
    assert "**Tool:** `Edit`" in body
    assert "/tmp/foo.py" in body


def test_capture_from_stdin_round_trips() -> None:
    payload = {"session_id": "stdin-test", "cwd": "/t", "tool_name": "Read"}
    stdin = io.StringIO(json.dumps(payload))
    cap = capture_from_stdin("PostToolUse", stdin=stdin)
    assert cap.session_id == "stdin-test"
    assert cap.tool == "Read"


def test_capture_from_stdin_rejects_invalid_json() -> None:
    stdin = io.StringIO("{not valid json")
    with pytest.raises(ValueError):
        capture_from_stdin("SessionEnd", stdin=stdin)


def test_write_to_memory_writes_yaml_frontmatter(tmp_path: Path) -> None:
    cap = parse_event("SessionEnd", {"session_id": "wt", "cwd": "/t"})
    path = write_to_memory(cap, store_root=tmp_path)
    assert path.exists()
    text = path.read_text()
    assert text.startswith("---\n")
    assert 'kind: "claude-code-event"' in text
    assert 'event: "SessionEnd"' in text
    assert "Claude Code · SessionEnd" in text


def test_write_to_memory_via_writer_double(tmp_path: Path) -> None:
    captured = {}

    class FakeWriter:
        def put(self, path: str, *, body: str, frontmatter: dict) -> Path:
            captured["path"] = path
            captured["body_len"] = len(body)
            captured["fm_keys"] = sorted(frontmatter.keys())
            return tmp_path / "fake-rooted" / path

    cap = parse_event("PostToolUse", {"session_id": "fw", "cwd": "/x", "tool_name": "Bash"})
    out = write_to_memory(cap, store_root=tmp_path, writer=FakeWriter())
    assert captured["path"].startswith("memories/claude-code/")
    assert "kind" in captured["fm_keys"]
    assert "tool" in captured["fm_keys"]
    assert "fake-rooted" in str(out)
