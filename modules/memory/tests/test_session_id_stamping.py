"""
§18.7 session-id stamping (TASK-MEMORY-303 §1.7, AC 7).

While ``sessions/.active`` names an active transcript session, the
canonical writer stamps ``extra.session_id`` on every put/move/delete
audit row; with no active session the key is absent. Stamping trusts the
marker file (a stale marker from a crashed session means rows carry a
dead session id — accepted for this task; lifecycle hygiene is
TASK-MEMORY-119's domain), and a caller-supplied session_id wins.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core.dream._audit_iter import iter_audit_rows
from cyberos.core.ops import delete, move, put
from cyberos.core.transcript import end, start
from cyberos.core.writer import Writer


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    """Exempt the test's tmp_path from AGENTS.md §0.1 sandbox-fragment check."""
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


def _init_store(tmp_path: Path) -> Path:
    store = tmp_path / ".cyberos/memory/store"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "manifest.json").write_text(json.dumps({"schema_version": 1}))
    # §18 anchor so the transcript lifecycle is permitted in this store.
    (store / "AGENTS.md").write_text(
        "# stub\n## §18  Session transcript ledger\n"
    )
    return store


def _rows_by_path(store: Path) -> dict[tuple[str, str], dict]:
    return {(r["op"], r["path"]): r for r in iter_audit_rows(store)}


def test_stamp_present_iff_active(tmp_path: Path) -> None:
    """AC 7 — put/move/delete carry extra.session_id iff a session is active."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        # Before any session — no stamp.
        put(w, "memories/facts/before.md", b"alpha", actor="t", kind="fact")

        start(w, session_id="sess-303")
        put(w, "memories/facts/during.md", b"beta", actor="t", kind="fact")
        move(
            w, "memories/facts/during.md", "memories/facts/moved.md",
            actor="t",
        )
        delete(w, "memories/facts/moved.md", actor="t")
        end(w, session_id="sess-303")

        # After the session ended — no stamp again.
        put(w, "memories/facts/after.md", b"delta", actor="t", kind="fact")

    rows = _rows_by_path(store)

    assert "session_id" not in rows[("put", "memories/facts/before.md")]["extra"]
    assert "session_id" not in rows[("put", "memories/facts/after.md")]["extra"]

    for key in (
        ("put", "memories/facts/during.md"),
        ("move", "memories/facts/during.md"),
        ("delete", "memories/facts/moved.md"),
    ):
        assert rows[key]["extra"].get("session_id") == "sess-303", (
            f"row {key} not stamped with the active session id: "
            f"{rows[key]['extra']}"
        )

    # Summary rows themselves are not re-stamped by the writer — they carry
    # session_id in their own §18.5 payload shape already.
    start_rows = [r for r in iter_audit_rows(store) if r["op"] == "session.start"]
    assert start_rows and start_rows[0]["extra"]["session_id"] == "sess-303"


def test_stale_marker_still_stamps(tmp_path: Path) -> None:
    """Documented §1.7 edge: stamping trusts sessions/.active blindly.

    A stale marker (crashed session — no session.start row, no binlog)
    still stamps rows with the dead id. Accepted for this task; the
    session-lifecycle walker invariant is what surfaces the hygiene issue.
    """
    store = _init_store(tmp_path)
    (store / "sessions").mkdir()
    (store / "sessions" / ".active").write_text("ghost-session")
    with Writer(store) as w:
        put(w, "memories/facts/x.md", b"x", actor="t", kind="fact")
    rows = _rows_by_path(store)
    assert rows[("put", "memories/facts/x.md")]["extra"].get(
        "session_id"
    ) == "ghost-session"


def test_caller_supplied_session_id_wins(tmp_path: Path) -> None:
    """A caller that already set extra.session_id is not overwritten."""
    store = _init_store(tmp_path)
    (store / "sessions").mkdir()
    (store / "sessions" / ".active").write_text("active-id")
    with Writer(store) as w:
        put(
            w, "memories/facts/y.md", b"y", actor="t", kind="fact",
            extra={"session_id": "explicit-id"},
        )
    rows = _rows_by_path(store)
    assert rows[("put", "memories/facts/y.md")]["extra"].get(
        "session_id"
    ) == "explicit-id"
