"""Tests for TASK-MEMORY-119 — session transcript ledger.

Covers acceptance criteria from
`docs/tasks/memory/TASK-MEMORY-119-session-transcript-ledger/spec.md`:

* AC #1  — full lifecycle round-trip
* AC #2  — default classification is confidential
* AC #3  — restricted classification encrypts content
* AC #4  — `public` / `internal` classifications rejected
* AC #5  — append without start rejected
* AC #6  — append after end rejected
* AC #7  — double-end rejected
* AC #8  — two active sessions rejected
* AC #9  — session.start row on main chain
* AC #10 — session.end row on main chain
* AC #15 — read without --decrypt for restricted shows placeholder
* AC #16 — read with --decrypt for restricted shows plaintext
* AC #17 — `transcript list` enumerates sessions
* AC #18 — retention purge produces session.purged + tombstone
* AC #19 — purge --dry-run mutates nothing
* AC #20 — session.purged payload shape
* AC #22 — §18 anchor required → ProtocolAmendmentMissing
* AC #24 — storage date-partitioned at start-date
"""

from __future__ import annotations

import json
import os
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

from cyberos.core.transcript import (
    ProtocolAmendmentMissing,
    TranscriptError,
    active_session_id,
    append,
    end,
    list_sessions,
    purge_expired,
    read,
    start,
)
from cyberos.core.writer import Writer


# ---- fixtures --------------------------------------------------------------


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    """Exempt the test's tmp_path from AGENTS.md §0.1 sandbox-fragment check."""
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


def _init_store(tmp_path: Path, with_section_18: bool = True) -> Path:
    store = tmp_path / ".cyberos/memory/store"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "manifest.json").write_text(json.dumps({
        "schema_version": 1,
        "audit_chain_head": "sha256:" + "0" * 64,
        "last_updated_at": "2026-05-19T00:00:00Z",
        "timezone": "UTC",
    }))
    if with_section_18:
        (store / "AGENTS.md").write_text(
            "# stub\n## §18  Session transcript ledger\n\n"
            "§18.1 Sessions are an OPTIONAL turn-level audit trail...\n"
        )
    return store


# ---- §18 anchor gate -----------------------------------------------------


def test_start_requires_section_18(tmp_path: Path) -> None:
    """AC #22."""
    store = _init_store(tmp_path, with_section_18=False)
    with Writer(store) as w:
        with pytest.raises(ProtocolAmendmentMissing, match="P22 §18"):
            start(w, session_id="x")


# ---- happy-path lifecycle ------------------------------------------------


def test_full_lifecycle(tmp_path: Path) -> None:
    """AC #1 + #2 + #9 + #10 — start → 2 appends → end → read; chain has summary rows."""
    from cyberos.core.dream._audit_iter import iter_audit_rows
    store = _init_store(tmp_path)
    with Writer(store) as w:
        s = start(w, session_id="sess-1")
        assert s.classification == "confidential"   # AC #2
        seq0 = append(w, session_id="sess-1", role="user", content="hello")
        seq1 = append(w, session_id="sess-1", role="assistant", content="hi")
        s2 = end(w, session_id="sess-1", reason="test complete")
    assert seq0 == 0
    assert seq1 == 1
    # Round-trip read
    turns = read(store, "sess-1")
    assert len(turns) == 2
    assert turns[0]["role"] == "user"
    assert turns[0]["content"] == "hello"
    assert turns[1]["role"] == "assistant"
    # Chain has session.start + session.end summary rows
    rows = list(iter_audit_rows(store))
    starts = [r for r in rows if r["op"] == "session.start"]
    ends = [r for r in rows if r["op"] == "session.end"]
    assert len(starts) == 1 and starts[0]["extra"]["session_id"] == "sess-1"
    assert len(ends) == 1 and ends[0]["extra"]["session_id"] == "sess-1"


def test_storage_date_partitioned(tmp_path: Path) -> None:
    """AC #24."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        s = start(w, session_id="sess-dated")
    expected_dir = store / "sessions" / s.started_at.strftime("%Y-%m-%d")
    assert expected_dir.is_dir()
    assert (expected_dir / "sess-dated.binlog").exists()


# ---- restricted classification --------------------------------------------


def test_restricted_classification_encrypts(tmp_path: Path) -> None:
    """AC #3 + #15 + #16."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        start(w, session_id="sess-r", classification="restricted")
        append(w, session_id="sess-r", role="user", content="secret hello")
        end(w, session_id="sess-r")
    # Without decrypt → placeholder
    plain = read(store, "sess-r", decrypt=False)
    assert plain[0]["content"] == "[encrypted content; --decrypt to read]"
    # With decrypt → plaintext
    decrypted = read(store, "sess-r", decrypt=True)
    assert decrypted[0]["content"] == "secret hello"


def test_public_classification_rejected(tmp_path: Path) -> None:
    """AC #4."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        with pytest.raises(ValueError, match="classification"):
            start(w, session_id="bad", classification="public")  # type: ignore[arg-type]


def test_internal_classification_rejected(tmp_path: Path) -> None:
    """AC #4."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        with pytest.raises(ValueError, match="classification"):
            start(w, session_id="bad", classification="internal")  # type: ignore[arg-type]


# ---- lifecycle invariants ------------------------------------------------


def test_append_without_start_rejected(tmp_path: Path) -> None:
    """AC #5."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        with pytest.raises(TranscriptError, match="no active session"):
            append(w, session_id="ghost", role="user", content="x")


def test_append_after_end_rejected(tmp_path: Path) -> None:
    """AC #6."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        start(w, session_id="x")
        end(w, session_id="x")
        with pytest.raises(TranscriptError):
            append(w, session_id="x", role="user", content="late")


def test_double_end_rejected(tmp_path: Path) -> None:
    """AC #7."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        start(w, session_id="x")
        end(w, session_id="x")
        with pytest.raises(TranscriptError):
            end(w, session_id="x")


def test_two_active_sessions_rejected(tmp_path: Path) -> None:
    """AC #8."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        start(w, session_id="a")
        with pytest.raises(TranscriptError, match="already active"):
            start(w, session_id="b")


def test_active_session_id_helper(tmp_path: Path) -> None:
    """Sanity check: `active_session_id()` returns current pointer."""
    store = _init_store(tmp_path)
    assert active_session_id(store) is None
    with Writer(store) as w:
        start(w, session_id="x")
    assert active_session_id(store) == "x"
    with Writer(store) as w:
        end(w, session_id="x")
    assert active_session_id(store) is None


# ---- input validation ----------------------------------------------------


def test_empty_session_id_rejected(tmp_path: Path) -> None:
    store = _init_store(tmp_path)
    with Writer(store) as w:
        with pytest.raises(ValueError, match="session_id"):
            start(w, session_id="")


def test_bad_role_rejected(tmp_path: Path) -> None:
    store = _init_store(tmp_path)
    with Writer(store) as w:
        start(w, session_id="x")
        with pytest.raises(ValueError, match="role"):
            append(w, session_id="x", role="weird", content="x")  # type: ignore[arg-type]


def test_retention_days_must_be_positive(tmp_path: Path) -> None:
    store = _init_store(tmp_path)
    with Writer(store) as w:
        with pytest.raises(ValueError, match="retention_days"):
            start(w, session_id="x", retention_days=0)


# ---- list + purge --------------------------------------------------------


def test_list_sessions(tmp_path: Path) -> None:
    """AC #17."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        start(w, session_id="a"); end(w, session_id="a")
        start(w, session_id="b"); end(w, session_id="b")
    sessions = list_sessions(store)
    ids = {s["session_id"] for s in sessions}
    assert ids == {"a", "b"}
    assert all(s["state"] == "ended" for s in sessions)


def test_purge_expired_dry_run(tmp_path: Path) -> None:
    """AC #19."""
    store = _init_store(tmp_path)
    # Backdate a session dir by 40 days
    old_dir = store / "sessions" / "2026-04-09"
    old_dir.mkdir(parents=True, exist_ok=True)
    (old_dir / "old.binlog").write_text("{}")

    # Need a Writer to satisfy purge_expired's signature, even though
    # dry-run shouldn't emit rows.
    from cyberos.core.dream._audit_iter import iter_audit_rows
    with Writer(store) as w:
        before_rows = list(iter_audit_rows(store))
        result = purge_expired(w, retention_days=30, dry_run=True)
        after_rows = list(iter_audit_rows(store))
    assert result["dry_run"] is True
    assert result["purged_count"] >= 1
    # Body NOT replaced
    assert (old_dir / "old.binlog").read_text() == "{}"
    # No session.purged rows emitted
    new_purge_rows = len([r for r in after_rows if r["op"] == "session.purged"]) \
                    - len([r for r in before_rows if r["op"] == "session.purged"])
    assert new_purge_rows == 0


def test_purge_expired_actually_purges(tmp_path: Path) -> None:
    """AC #18 + #20 — session.purged row + tombstone body."""
    from cyberos.core.dream._audit_iter import iter_audit_rows
    store = _init_store(tmp_path)
    old_dir = store / "sessions" / "2026-04-09"
    old_dir.mkdir(parents=True, exist_ok=True)
    (old_dir / "old.binlog").write_text('{"some":"content"}')

    with Writer(store) as w:
        result = purge_expired(w, retention_days=30)
    assert result["purged_count"] == 1
    # Body replaced with tombstone marker
    body = json.loads((old_dir / "old.binlog").read_text())
    assert body["tombstone"] is True
    assert body["session_id"] == "old"
    # session.purged row on chain with correct payload shape
    rows = list(iter_audit_rows(store))
    purged = [r for r in rows if r["op"] == "session.purged"]
    assert len(purged) == 1
    payload = purged[0]["extra"]
    for key in ("session_id", "original_started_at", "purged_at", "reason"):
        assert key in payload
    assert payload["reason"] == "retention_expired"
    assert payload["session_id"] == "old"


# ---- read returns nothing for unknown session ---------------------------


def test_read_unknown_session_returns_empty(tmp_path: Path) -> None:
    store = _init_store(tmp_path)
    assert read(store, "no-such-session") == []
