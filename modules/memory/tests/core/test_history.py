"""Tests for FR-MEMORY-120 — `cyberos history <path>`.

Covers acceptance criteria from
`docs/feature-requests/memory/FR-MEMORY-120-cyberos-history/spec.md`:

* AC #1  — single-write history → 1 entry
* AC #2  — multi-write history → N entries most-recent-first
* AC #3  — chronological=True flips order
* AC #4  — limit caps result count
* AC #5  — --since filters by time
* AC #7  — JSON output produces valid structured list
* AC #10 — dream annotations rendered (dream_id / proposal_id / merged_into)
* AC #13 — tombstone row appears in history
* AC #16 — multi-kind events appear (put + delete + aux)
* AC #20 — path never existed → empty list
* AC #21 — read-only: head_seq unchanged before/after walk
"""

from __future__ import annotations

import json
import time
from datetime import datetime, timedelta, timezone
from pathlib import Path

import pytest

from cyberos.core.frontmatter import Frontmatter, serialize
from cyberos.core.history import HistoryEntry, render_annotations, render_human, walk
from cyberos.core.ops import put as canonical_put, delete as canonical_delete
from cyberos.core.writer import Writer


# ---- fixtures --------------------------------------------------------------


@pytest.fixture(autouse=True)
def _exempt_sandbox_path(monkeypatch, tmp_path):
    """Exempt the test's tmp_path from AGENTS.md §0.1 sandbox-fragment check."""
    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", str(tmp_path))


def _init_store(tmp_path: Path) -> Path:
    store = tmp_path / ".cyberos/memory/store"
    (store / "audit").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "manifest.json").write_text(json.dumps({
        "schema_version": 1,
        "audit_chain_head": "sha256:" + "0" * 64,
        "last_updated_at": "2026-05-19T00:00:00Z",
        "timezone": "UTC",
    }))
    return store


def _body(content: str = "v1") -> bytes:
    fm = Frontmatter(id="F-1", kind="fact", ts_ns=time.time_ns(),
                     actor="t", tags=[], extra={})
    return serialize(fm, content.encode("utf-8"))


# ---- single + multi write -------------------------------------------------


def test_single_write_one_entry(tmp_path: Path) -> None:
    """AC #1."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body("v1"),
                      actor="stephen", kind="fact")
    entries = walk(store, "memories/facts/x.md")
    assert len(entries) == 1
    assert entries[0].kind == "put"
    assert entries[0].actor == "stephen"


def test_multi_write_most_recent_first(tmp_path: Path) -> None:
    """AC #2."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body("v1"),
                      actor="stephen", kind="fact")
        canonical_put(w, "memories/facts/x.md", _body("v2"),
                      actor="stephen", kind="fact")
        canonical_put(w, "memories/facts/x.md", _body("v3"),
                      actor="stephen", kind="fact")
    entries = walk(store, "memories/facts/x.md")
    assert len(entries) == 3
    # Most-recent-first → seqs should descend
    assert entries[0].seq > entries[1].seq > entries[2].seq


def test_chronological_flips_order(tmp_path: Path) -> None:
    """AC #3 — caller reverses; the walker returns most-recent-first by default."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body("v1"),
                      actor="t", kind="fact")
        canonical_put(w, "memories/facts/x.md", _body("v2"),
                      actor="t", kind="fact")
    entries = walk(store, "memories/facts/x.md")
    entries_chrono = list(reversed(entries))
    assert entries[0].seq > entries[-1].seq
    assert entries_chrono[0].seq < entries_chrono[-1].seq


# ---- filters --------------------------------------------------------------


def test_limit_caps(tmp_path: Path) -> None:
    """AC #4."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        for _ in range(5):
            canonical_put(w, "memories/facts/x.md", _body(),
                          actor="t", kind="fact")
    entries = walk(store, "memories/facts/x.md", limit=2)
    assert len(entries) == 2


def test_since_filters(tmp_path: Path) -> None:
    """AC #5."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
    future = datetime.now(timezone.utc) + timedelta(hours=1)
    # since cut-off in the future → no entries
    entries = walk(store, "memories/facts/x.md", since=future)
    assert entries == []
    # since=0h-ago (epoch start) → all entries
    entries_all = walk(store, "memories/facts/x.md",
                       since=datetime(2020, 1, 1, tzinfo=timezone.utc))
    assert len(entries_all) == 1


# ---- multi-kind history ---------------------------------------------------


def test_multi_kind_events_appear(tmp_path: Path) -> None:
    """AC #16 — put + delete on the same path produce ≥ 2 entries with both kinds."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
        canonical_delete(w, "memories/facts/x.md",
                         actor="dream-applier", mode="tombstone",
                         reason="merge", extra={"dream_id": "abc", "proposal_id": "P1234ABCD"})
    entries = walk(store, "memories/facts/x.md")
    kinds = {e.kind for e in entries}
    assert "put" in kinds
    assert "delete" in kinds


def test_tombstone_renders(tmp_path: Path) -> None:
    """AC #13."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
        canonical_delete(w, "memories/facts/x.md",
                         actor="t", mode="tombstone", reason="cleanup")
    entries = walk(store, "memories/facts/x.md")
    delete_entry = next(e for e in entries if e.kind == "delete")
    assert delete_entry.extra.get("mode") == "tombstone"
    assert delete_entry.extra.get("reason") == "cleanup"


def test_dream_annotations_in_extras(tmp_path: Path) -> None:
    """AC #10 — dream provenance carried through to history.extra."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
        canonical_delete(w, "memories/facts/x.md",
                         actor="dream-applier", mode="tombstone",
                         reason="merge",
                         extra={"dream_id": "01HJ8XVK9P0M7N5G4F3E2D1C0B",
                                "proposal_id": "P3FQ8K2X",
                                "merged_into": "memories/facts/canonical.md"})
    entries = walk(store, "memories/facts/x.md")
    delete_entry = next(e for e in entries if e.kind == "delete")
    assert delete_entry.extra["dream_id"].startswith("01HJ")
    assert delete_entry.extra["proposal_id"] == "P3FQ8K2X"
    assert delete_entry.extra["merged_into"] == "memories/facts/canonical.md"


# ---- annotation rendering -------------------------------------------------


def test_render_annotations_dream() -> None:
    """AC #10 — human render captures all recognised tags."""
    out = render_annotations({
        "dream_id": "01HJ8XVK9P0M7N5G4F3E2D1C0B",
        "proposal_id": "P3FQ8K2X",
        "merged_into": "memories/facts/canonical.md",
    })
    assert "dream" in out
    assert "01HJ8XVK" in out
    assert "P3FQ8K2X" in out
    assert "merged into" in out


def test_render_annotations_session() -> None:
    out = render_annotations({"session_id": "sess-1"})
    assert "during session sess-1" in out


def test_render_annotations_invocation() -> None:
    out = render_annotations({"invocation": "consolidate"})
    assert "via consolidate" in out


def test_render_annotations_warn_only() -> None:
    out = render_annotations({"warn_only": True})
    assert "WARN-ONLY" in out


def test_render_annotations_empty() -> None:
    assert render_annotations({}) == ""


# ---- never-existed path ---------------------------------------------------


def test_never_existed_path_returns_empty(tmp_path: Path) -> None:
    """AC #20."""
    store = _init_store(tmp_path)
    entries = walk(store, "memories/facts/never.md")
    assert entries == []


# ---- read-only invariant --------------------------------------------------


def test_history_is_read_only(tmp_path: Path) -> None:
    """AC #21 — walking history MUST NOT emit new audit rows."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
        head_before = w.head_seq
    # Walk
    walk(store, "memories/facts/x.md")
    # Re-check head
    with Writer(store) as w:
        head_after = w.head_seq
    assert head_after == head_before


# ---- HistoryEntry.to_dict + JSON projection ------------------------------


def test_history_entry_to_dict(tmp_path: Path) -> None:
    """AC #7 — JSON projection has all the documented fields."""
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
    entries = walk(store, "memories/facts/x.md")
    d = entries[0].to_dict()
    for key in ("seq", "ts", "kind", "actor", "body_hash",
                "frontmatter_diff", "body_diff", "extra", "path"):
        assert key in d


# ---- render_human format check ------------------------------------------


def test_render_human_includes_seq_and_kind(tmp_path: Path) -> None:
    store = _init_store(tmp_path)
    with Writer(store) as w:
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="stephen", kind="fact")
    entries = walk(store, "memories/facts/x.md")
    line = render_human(entries[0])
    assert "[" in line and "put" in line and "stephen" in line


# ---- multi-touch path -----------------------------------------------------


def test_aux_rows_on_path_appear_in_history(tmp_path: Path) -> None:
    """AC #16 — non-put audit kinds (like memory.acl_denied) appear when
    their `path` field equals target."""
    from cyberos.core.writer import AuditRecord
    store = _init_store(tmp_path)
    with Writer(store) as w:
        # Put then a synthetic aux row pointing at the same path
        canonical_put(w, "memories/facts/x.md", _body(),
                      actor="t", kind="fact")
        w.submit(AuditRecord(
            op="memory.importance_scored",
            path="memories/facts/x.md",
            actor="t",
            extra={
                "score": 0.72, "model": "mock",
                "outcome": "ok", "cache_hit": False,
            },
        ))
    entries = walk(store, "memories/facts/x.md")
    kinds = [e.kind for e in entries]
    assert "memory.importance_scored" in kinds
