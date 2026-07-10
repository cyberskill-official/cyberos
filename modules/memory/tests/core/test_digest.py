"""Tests for cyberos.core.digest (PROPOSAL.md P8)."""

from __future__ import annotations

import json
import time

import pytest

from cyberos.core import digest as digest_mod
from cyberos.core.ops import delete, move, put
from cyberos.core.writer import Writer


def _build_test_store(tmp_path) -> "Path":
    from pathlib import Path
    store = tmp_path / ".cyberos/memory/store"
    store.mkdir(parents=True, exist_ok=True)
    (store / "audit").mkdir(parents=True, exist_ok=True)
    (store / "memories" / "decisions").mkdir(parents=True)
    (store / "memories" / "drift").mkdir(parents=True)
    (store / "memories" / "facts").mkdir(parents=True)
    (store / "meta").mkdir(parents=True, exist_ok=True)
    return store


# ---------------------------------------------------------------------------
# duration parsing
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("text,expected_ns", [
    ("1s", 1_000_000_000),
    ("60s", 60_000_000_000),
    ("1m", 60_000_000_000),
    ("1h", 3600 * 1_000_000_000),
    ("24h", 24 * 3600 * 1_000_000_000),
    ("7d", 7 * 86_400 * 1_000_000_000),
    ("2w", 14 * 86_400 * 1_000_000_000),
    ("0.5h", 30 * 60 * 1_000_000_000),
])
def test_parse_human_duration(text, expected_ns):
    assert digest_mod.parse_human_duration(text) == expected_ns


@pytest.mark.parametrize("bad", ["", "1y", "abc", "10", "h", " "])
def test_parse_human_duration_rejects(bad):
    with pytest.raises(ValueError):
        digest_mod.parse_human_duration(bad)


# ---------------------------------------------------------------------------
# build over an empty store
# ---------------------------------------------------------------------------


def test_build_empty_store(tmp_path):
    store = _build_test_store(tmp_path)
    d = digest_mod.build(store)
    assert d.total_rows == 0
    assert d.op_counts == {}
    assert d.actor_counts == {}
    assert d.prefix_counts == {}
    assert d.highlights == []


def test_build_window_validates(tmp_path):
    store = _build_test_store(tmp_path)
    now = time.time_ns()
    with pytest.raises(ValueError, match="empty"):
        digest_mod.build(store, since_ns=now, until_ns=now)


# ---------------------------------------------------------------------------
# build over a store with some activity
# ---------------------------------------------------------------------------


def test_build_with_activity_counts_by_op_actor_prefix(tmp_path):
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
        put(w, "memories/facts/f1.md", b"a", actor="stephen", kind="fact")
        put(w, "memories/drift/x1.md", b"a", actor="coding-agent", kind="drift")
        put(w, "memories/facts/f1.md", b"ab", actor="stephen", kind="fact")
        move(w, "memories/facts/f1.md", "memories/facts/f2.md", actor="stephen")
        delete(
            w, "memories/facts/f2.md", actor="stephen",
            mode="purge", reason="cleanup",
            approval_phrase="APPROVE protocol change P4 §3.6",
        )

    d = digest_mod.build(store)
    assert d.total_rows == 6
    assert sum(d.op_counts.values()) == 6
    # Actors
    assert d.actor_counts.get("stephen", 0) == 5
    assert d.actor_counts.get("coding-agent", 0) == 1
    # Prefixes — every path was under memories/{decisions,facts,drift}/
    assert "memories/decisions/" in d.prefix_counts
    assert "memories/facts/" in d.prefix_counts
    assert "memories/drift/" in d.prefix_counts


def test_build_highlights_include_decisions_drift_purges_renames(tmp_path):
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        # Highlights: put under decisions/, put under drift/, rename, purge
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
        put(w, "memories/drift/x1.md", b"a", actor="stephen", kind="drift")
        put(w, "memories/facts/f1.md", b"a", actor="stephen", kind="fact")
        move(w, "memories/facts/f1.md", "memories/facts/f2.md", actor="stephen")
        delete(
            w, "memories/facts/f2.md", actor="stephen",
            mode="purge", reason="cleanup",
            approval_phrase="APPROVE protocol change P4 §3.6",
        )

    d = digest_mod.build(store)
    # The decision + drift puts MUST be highlighted; rename + purge MUST too.
    paths_in_highlights = {h.path for h in d.highlights}
    assert "memories/decisions/d1.md" in paths_in_highlights
    assert "memories/drift/x1.md" in paths_in_highlights

    # The plain `put` to memories/facts/f1.md MUST NOT highlight as a put,
    # but f1.md MAY appear as the SOURCE path of the subsequent move row.
    # Assert that intent precisely: no `put` highlight under facts/.
    put_highlights = [
        h for h in d.highlights
        if h.op == "put"
    ]
    fact_put_highlights = [
        h for h in put_highlights if h.path.startswith("memories/facts/")
    ]
    assert fact_put_highlights == [], (
        f"plain put under facts/ should not highlight: {fact_put_highlights}"
    )

    # Move / purge appear regardless of path prefix.
    move_present = any(h.op == "move" for h in d.highlights)
    purge_present = any(h.extra.get("mode") == "purge" for h in d.highlights)
    assert move_present
    assert purge_present


def test_build_highlight_cap_respected(tmp_path):
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        for i in range(10):
            put(w, f"memories/decisions/d{i}.md", b"a", actor="stephen", kind="decision")
    d = digest_mod.build(store, highlight_cap=3)
    assert len(d.highlights) == 3


def test_build_window_filters_old_rows(tmp_path):
    """A future-anchored window should see zero rows from earlier writes."""
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
    # Window starts 1h in the future
    future = time.time_ns() + 3600 * 1_000_000_000
    d = digest_mod.build(
        store,
        since_ns=future,
        until_ns=future + 3_600_000_000_000,
    )
    assert d.total_rows == 0


# ---------------------------------------------------------------------------
# determinism
# ---------------------------------------------------------------------------


def test_build_is_deterministic(tmp_path):
    """Two invocations over the exact same window MUST produce identical JSON."""
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
        put(w, "memories/facts/f1.md", b"a", actor="stephen", kind="fact")
        move(w, "memories/facts/f1.md", "memories/facts/f2.md", actor="stephen")

    # Pin the window so both runs see the same bounds.
    until_ns = time.time_ns() + 60 * 1_000_000_000
    since_ns = until_ns - 86_400 * 1_000_000_000

    a = digest_mod.build(store, since_ns=since_ns, until_ns=until_ns)
    b = digest_mod.build(store, since_ns=since_ns, until_ns=until_ns)
    assert a.to_json() == b.to_json()


# ---------------------------------------------------------------------------
# formatters
# ---------------------------------------------------------------------------


def test_format_text_includes_section_headers(tmp_path):
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
    d = digest_mod.build(store)
    rendered = digest_mod.format_text(d)
    assert "memory digest" in rendered
    assert "by op:" in rendered
    assert "by actor:" in rendered
    assert "by area:" in rendered
    assert "highlights" in rendered


def test_format_text_empty_window_message(tmp_path):
    store = _build_test_store(tmp_path)
    d = digest_mod.build(store)
    rendered = digest_mod.format_text(d)
    assert "no audit activity" in rendered.lower()


def test_format_markdown_includes_tables(tmp_path):
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
    d = digest_mod.build(store)
    md = digest_mod.format_markdown(d)
    assert md.startswith("# memory digest")
    assert "| op | count |" in md
    assert "| actor | count |" in md


def test_json_round_trips(tmp_path):
    store = _build_test_store(tmp_path)
    with Writer(store) as w:
        put(w, "memories/decisions/d1.md", b"a", actor="stephen", kind="decision")
    d = digest_mod.build(store)
    parsed = json.loads(d.to_json())
    assert parsed["total_rows"] == 1
    assert parsed["actor_counts"]["stephen"] == 1


# ---------------------------------------------------------------------------
# private helpers
# ---------------------------------------------------------------------------


@pytest.mark.parametrize("path,expected", [
    ("memories/decisions/d1.md", "memories/decisions/"),
    ("memories/facts/sub/dir/x.md", "memories/facts/"),
    ("project/p1.md", "project/"),
    ("meta/x.md", "meta/"),
    ("README.md", "README.md/"),
    ("", "(unknown)"),
])
def test_top_prefix(path, expected):
    assert digest_mod._top_prefix(path) == expected
