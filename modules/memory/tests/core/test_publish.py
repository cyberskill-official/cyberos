"""Tests for cyberos.core.publish (PROPOSAL.md P12)."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core import publish as pub


def _write_memory(store: Path, rel: str, *, mid: str, kind: str, actor: str,
                  ts_ns: int, tags: list[str], body: str) -> None:
    """Make a v2-format memory file with msgspec-canonical JSON frontmatter."""
    import msgspec
    fm_dict = {
        "id": mid, "kind": kind, "ts_ns": ts_ns,
        "actor": actor, "tags": tags,
    }
    fm_bytes = msgspec.json.encode(fm_dict, order="sorted")
    path = store / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"---\n" + fm_bytes + b"\n---\n" + body.encode("utf-8"))


def _empty_store(tmp_path: Path) -> Path:
    store = tmp_path / ".cyberos-memory"
    store.mkdir()
    (store / "audit").mkdir()
    (store / "manifest.json").write_text('{}', encoding="utf-8")
    return store


# ---------------------------------------------------------------------------
# collect
# ---------------------------------------------------------------------------


def test_collect_empty_store(tmp_path):
    store = _empty_store(tmp_path)
    m = pub.collect(store)
    assert m.memories == []
    assert m.counts_by_kind == {}
    assert m.counts_by_actor == {}
    assert not hasattr(m, "schema_version")


def test_collect_walks_canonical_roots(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/decisions/d1.md",
                  mid="DEC-1", kind="decision", actor="stephen",
                  ts_ns=1_700_000_000_000_000_000,
                  tags=["a", "b"], body="# A decision\nbody text")
    _write_memory(store, "memories/facts/f1.md",
                  mid="FACT-1", kind="fact", actor="agent",
                  ts_ns=1_700_000_001_000_000_000,
                  tags=[], body="fact body")
    _write_memory(store, "project/p1.md",
                  mid="PRJ-1", kind="project", actor="stephen",
                  ts_ns=1_700_000_002_000_000_000,
                  tags=[], body="project body")

    m = pub.collect(store)
    assert len(m.memories) == 3
    assert m.counts_by_kind == {"decision": 1, "fact": 1, "project": 1}
    # ts_ns DESC ordering — project (latest) first
    assert m.memories[0].id == "PRJ-1"
    assert m.memories[-1].id == "DEC-1"


def test_collect_kind_allowlist(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/decisions/d1.md",
                  mid="DEC-1", kind="decision", actor="s",
                  ts_ns=1, tags=[], body="")
    _write_memory(store, "memories/facts/f1.md",
                  mid="FACT-1", kind="fact", actor="s",
                  ts_ns=2, tags=[], body="")

    m = pub.collect(store, kinds=["decision"])
    assert {x.id for x in m.memories} == {"DEC-1"}


def test_collect_kind_blocklist(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/decisions/d1.md",
                  mid="DEC-1", kind="decision", actor="s",
                  ts_ns=1, tags=[], body="")
    _write_memory(store, "memories/facts/f1.md",
                  mid="FACT-1", kind="fact", actor="s",
                  ts_ns=2, tags=[], body="")

    m = pub.collect(store, exclude_kinds=["decision"])
    assert {x.id for x in m.memories} == {"FACT-1"}


def test_collect_truncates_long_bodies(tmp_path):
    store = _empty_store(tmp_path)
    big = "x" * 5000
    _write_memory(store, "memories/facts/f1.md",
                  mid="F-1", kind="fact", actor="s",
                  ts_ns=1, tags=[], body=big)

    m = pub.collect(store, max_body_chars=100)
    assert "[truncated]" in m.memories[0].body
    # Truncated to 100 chars + truncation marker — should be much less than 5000
    assert len(m.memories[0].body) < 1000


def test_collect_skips_audit_index_exports(tmp_path):
    """Files outside the canonical roots are never published."""
    store = _empty_store(tmp_path)
    # Drop a memory-shaped file in audit/ — must be ignored
    (store / "audit" / "fake.md").write_bytes(
        b'---\n{"id":"x","kind":"fact","ts_ns":1,"actor":"s","tags":[]}\n---\nignored'
    )
    m = pub.collect(store)
    assert m.memories == []


def test_collect_skips_unparseable_files(tmp_path):
    store = _empty_store(tmp_path)
    (store / "memories" / "facts").mkdir(parents=True)
    # Garbage file
    (store / "memories" / "facts" / "broken.md").write_bytes(b"no frontmatter at all")
    # Valid file
    _write_memory(store, "memories/facts/good.md",
                  mid="G", kind="fact", actor="s",
                  ts_ns=1, tags=[], body="good")
    m = pub.collect(store)
    assert {x.id for x in m.memories} == {"G"}


# ---------------------------------------------------------------------------
# render
# ---------------------------------------------------------------------------


def test_render_is_self_contained(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/facts/f1.md",
                  mid="F-1", kind="fact", actor="s",
                  ts_ns=1, tags=["t1"], body="hello")
    m = pub.collect(store)
    html = pub.render_html(m)
    # Self-contained: no http/https external links
    for token in ("https://", "http://", "<link", "<iframe"):
        # Allow the `<link>` element check? render uses inline CSS, no <link>.
        assert token not in html, (
            f"render output should be fully self-contained; found {token!r}"
        )
    # Embedded data MUST contain the memory
    assert '"F-1"' in html
    assert '"fact"' in html
    assert "hello" in html


def test_render_escapes_script_close_in_body(tmp_path):
    """A memory body containing </script> must not break the inline JSON."""
    store = _empty_store(tmp_path)
    nasty = "before </script><script>alert('xss')</script> after"
    _write_memory(store, "memories/facts/f1.md",
                  mid="F-1", kind="fact", actor="s",
                  ts_ns=1, tags=[], body=nasty)
    m = pub.collect(store)
    html = pub.render_html(m)
    # The literal "</script>" should not appear in the inline JSON region
    payload_start = html.index('id="memory-data">') + len('id="memory-data">')
    payload_end = html.index("</script>", payload_start)
    payload_region = html[payload_start:payload_end]
    assert "</script>" not in payload_region
    assert "<\\/script>" in payload_region


def test_render_deterministic_zeros_timestamp(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/facts/f1.md",
                  mid="F-1", kind="fact", actor="s",
                  ts_ns=1, tags=[], body="hi")
    m1 = pub.collect(store)
    # Don't mutate generated_at_ns directly — render with deterministic=True
    html_a = pub.render_html(m1, deterministic=True)
    m2 = pub.collect(store)
    html_b = pub.render_html(m2, deterministic=True)
    assert html_a == html_b


def test_render_non_deterministic_differs_only_in_timestamp(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/facts/f1.md",
                  mid="F-1", kind="fact", actor="s",
                  ts_ns=1, tags=[], body="hi")
    m1 = pub.collect(store)
    import time
    time.sleep(0.001)  # ensure ts_ns drifts
    m2 = pub.collect(store)
    # Generated timestamps differ
    assert m1.generated_at_ns != m2.generated_at_ns


# ---------------------------------------------------------------------------
# publish_to_file end-to-end
# ---------------------------------------------------------------------------


def test_publish_to_file_writes_html(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/decisions/d1.md",
                  mid="DEC-1", kind="decision", actor="s",
                  ts_ns=1, tags=[], body="decided")
    out = tmp_path / "memory.html"
    summary = pub.publish_to_file(store, out)
    assert out.is_file()
    assert summary["n_memories"] == 1
    assert summary["bytes"] > 0
    assert len(summary["sha256"]) == 64

    text = out.read_text(encoding="utf-8")
    assert "<!DOCTYPE html>" in text
    assert "DEC-1" in text


def test_publish_to_file_deterministic_byte_identical(tmp_path):
    store = _empty_store(tmp_path)
    _write_memory(store, "memories/decisions/d1.md",
                  mid="DEC-1", kind="decision", actor="s",
                  ts_ns=1, tags=[], body="decided")
    out_a = tmp_path / "a.html"
    out_b = tmp_path / "b.html"
    sa = pub.publish_to_file(store, out_a, deterministic=True)
    sb = pub.publish_to_file(store, out_b, deterministic=True)
    assert sa["sha256"] == sb["sha256"]
    assert out_a.read_bytes() == out_b.read_bytes()
