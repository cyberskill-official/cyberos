"""Tests for cyberos.core.session (PROPOSAL.md P11)."""

from __future__ import annotations

import json
import time
from pathlib import Path

import pytest

from cyberos.core import session as session_mod


def _store(tmp_path) -> Path:
    s = tmp_path / ".cyberos-memory"
    s.mkdir()
    (s / "audit").mkdir()
    (s / "manifest.json").write_text('{}', encoding="utf-8")
    return s


# ---------------------------------------------------------------------------
# start / end roundtrip
# ---------------------------------------------------------------------------


def test_start_session_creates_json_and_audit_row(tmp_path):
    store = _store(tmp_path)
    sess = session_mod.start_session(
        store,
        actor="stephen",
        scope=["memories/decisions/"],
        note="working on the auth ADR",
    )

    # JSON file exists at meta/sessions/<id>.json
    json_path = store / "meta" / "sessions" / f"{sess.id}.json"
    assert json_path.is_file()
    raw = json.loads(json_path.read_text(encoding="utf-8"))
    assert raw["actor"] == "stephen"
    assert raw["scope"] == ["memories/decisions/"]
    assert raw["note"] == "working on the auth ADR"

    # Audit chain has the session.start row
    from cyberos.core.walker import MmapWalker
    current = store / "audit" / "current.binlog"
    assert current.is_file()
    with MmapWalker(current) as walker:
        ops = [r.op for _o, r in walker.iter_records()]
    assert "session.start" in ops


def test_start_session_id_is_unique(tmp_path):
    store = _store(tmp_path)
    s1 = session_mod.start_session(store, actor="a", scope=[])
    s2 = session_mod.start_session(store, actor="b", scope=[])
    assert s1.id != s2.id


def test_end_session_removes_json_and_writes_audit_row(tmp_path):
    store = _store(tmp_path)
    sess = session_mod.start_session(store, actor="stephen", scope=[])
    json_path = store / "meta" / "sessions" / f"{sess.id}.json"
    assert json_path.is_file()

    summary = session_mod.end_session(store, sess.id)
    assert not json_path.exists()
    assert summary["id"] == sess.id
    assert summary["duration_ns"] >= 0

    from cyberos.core.walker import MmapWalker
    current = store / "audit" / "current.binlog"
    with MmapWalker(current) as walker:
        ops = [r.op for _o, r in walker.iter_records()]
    assert ops.count("session.end") == 1


def test_end_session_nonexistent(tmp_path):
    store = _store(tmp_path)
    with pytest.raises(FileNotFoundError):
        session_mod.end_session(store, "sess-nope")


# ---------------------------------------------------------------------------
# listing + GC
# ---------------------------------------------------------------------------


def test_list_sessions_includes_active(tmp_path):
    store = _store(tmp_path)
    s1 = session_mod.start_session(store, actor="a", scope=["memories/facts/"])
    s2 = session_mod.start_session(store, actor="b", scope=["meta/"])
    active = session_mod.list_sessions(store)
    ids = {s.id for s in active}
    assert s1.id in ids
    assert s2.id in ids


def test_list_sessions_gcs_expired(tmp_path):
    store = _store(tmp_path)
    # Build an expired lease by hand-writing the JSON
    sdir = store / "meta" / "sessions"
    sdir.mkdir(parents=True, exist_ok=True)
    expired = {
        "id": "sess-old",
        "actor": "zombie",
        "scope": [],
        "started_at_ns": 1,
        "expires_at_ns": 2,  # well in the past
        "host": "lab",
        "note": "",
    }
    (sdir / "sess-old.json").write_text(json.dumps(expired), encoding="utf-8")

    active = session_mod.list_sessions(store)
    assert all(s.id != "sess-old" for s in active)
    # GC actually removed the file
    assert not (sdir / "sess-old.json").exists()


def test_list_sessions_includes_expired_when_asked(tmp_path):
    store = _store(tmp_path)
    sdir = store / "meta" / "sessions"
    sdir.mkdir(parents=True, exist_ok=True)
    expired = {
        "id": "sess-old",
        "actor": "zombie",
        "scope": [],
        "started_at_ns": 1,
        "expires_at_ns": 2,
        "host": "lab",
        "note": "",
    }
    (sdir / "sess-old.json").write_text(json.dumps(expired), encoding="utf-8")
    active = session_mod.list_sessions(store, include_expired=True)
    assert any(s.id == "sess-old" for s in active)


def test_list_sessions_skips_corrupt(tmp_path):
    store = _store(tmp_path)
    sdir = store / "meta" / "sessions"
    sdir.mkdir(parents=True, exist_ok=True)
    (sdir / "sess-corrupt.json").write_text("not valid json", encoding="utf-8")
    active = session_mod.list_sessions(store)
    assert all(s.id != "sess-corrupt" for s in active)
    # Corrupt file is removed by GC
    assert not (sdir / "sess-corrupt.json").exists()


# ---------------------------------------------------------------------------
# scope overlap detection
# ---------------------------------------------------------------------------


def test_overlaps_scope_via_prefix(tmp_path):
    store = _store(tmp_path)
    s = session_mod.start_session(
        store, actor="a", scope=["memories/decisions/"],
    )
    # Exact match
    assert s.overlaps_scope(["memories/decisions/"]) == ["memories/decisions/"]
    # Candidate is a subpath of the existing scope
    assert s.overlaps_scope(["memories/decisions/sub/"]) == ["memories/decisions/"]
    # Candidate is a superpath of the existing scope
    assert s.overlaps_scope(["memories/"]) == ["memories/decisions/"]
    # No overlap
    assert s.overlaps_scope(["meta/"]) == []


def test_find_scope_conflicts_returns_matching_sessions(tmp_path):
    store = _store(tmp_path)
    s1 = session_mod.start_session(store, actor="a", scope=["memories/decisions/"])
    s2 = session_mod.start_session(store, actor="b", scope=["meta/"])

    # Conflicts with s1, NOT with s2
    conflicts = session_mod.find_scope_conflicts(store, ["memories/decisions/x/"])
    conflict_ids = {sess.id for (sess, _ov) in conflicts}
    assert s1.id in conflict_ids
    assert s2.id not in conflict_ids


def test_find_scope_conflicts_exclude_self(tmp_path):
    store = _store(tmp_path)
    s1 = session_mod.start_session(store, actor="a", scope=["memories/decisions/"])
    conflicts = session_mod.find_scope_conflicts(
        store, ["memories/decisions/"],
        exclude_session_id=s1.id,
    )
    assert conflicts == []


def test_find_scope_conflicts_empty_when_no_active(tmp_path):
    store = _store(tmp_path)
    assert session_mod.find_scope_conflicts(store, ["memories/"]) == []


# ---------------------------------------------------------------------------
# format_sessions
# ---------------------------------------------------------------------------


def test_format_sessions_empty(tmp_path):
    rendered = session_mod.format_sessions([])
    assert "no active sessions" in rendered


def test_format_sessions_populated(tmp_path):
    store = _store(tmp_path)
    s = session_mod.start_session(store, actor="stephen",
                                  scope=["memories/decisions/"],
                                  note="ADR draft")
    rendered = session_mod.format_sessions(session_mod.list_sessions(store))
    assert s.id in rendered
    assert "stephen" in rendered
    assert "memories/decisions/" in rendered
    assert "ADR draft" in rendered


# ---------------------------------------------------------------------------
# expired property
# ---------------------------------------------------------------------------


def test_expired_returns_false_for_future():
    sess = session_mod.Session(
        id="x", actor="a", scope=[], host="h",
        started_at_ns=time.time_ns(),
        expires_at_ns=time.time_ns() + 10**12,
    )
    assert sess.expired is False


def test_expired_returns_true_for_past():
    sess = session_mod.Session(
        id="x", actor="a", scope=[], host="h",
        started_at_ns=1, expires_at_ns=2,
    )
    assert sess.expired is True
