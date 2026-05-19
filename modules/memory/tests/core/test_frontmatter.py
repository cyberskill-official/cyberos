"""msgspec frontmatter parser tests."""

from __future__ import annotations

import json
import time

import pytest

from cyberos.core.frontmatter import (
    Frontmatter,
    looks_like_yaml,
    parse,
    parse_legacy_yaml,
    serialize,
)


def _build(raw_fm: dict, body: bytes = b"# body\n") -> bytes:
    return b"---\n" + json.dumps(raw_fm).encode("utf-8") + b"\n---\n" + body


def test_round_trip() -> None:
    fm = Frontmatter(
        id="DEC-001",
        kind="decision",
        ts_ns=time.time_ns(),
        actor="stephen",
        tags=["arch", "writer"],
        extra={"version": 2},
    )
    body = b"# Title\n\nSome body text.\n"
    raw = serialize(fm, body)
    fm2, body2 = parse(raw)
    assert fm2 == fm
    assert body2 == body


def test_serialise_is_deterministic() -> None:
    fm = Frontmatter(id="x", kind="fact", ts_ns=1, actor="a", tags=["b", "a"], extra={"z": 1})
    body = b"body"
    assert serialize(fm, body) == serialize(fm, body)


def test_missing_delimiter() -> None:
    with pytest.raises(ValueError):
        parse(b"no delimiter here\nbody")


def test_missing_trailing_delimiter() -> None:
    with pytest.raises(ValueError):
        parse(b"---\n{}\nno trailing\n")


def test_legacy_yaml_dispatches() -> None:
    pytest.importorskip("yaml")
    raw = b"---\nid: REF-001\nkind: refinement\nts_ns: 1\nactor: ops\n---\n# body\n"
    assert looks_like_yaml(raw) is True
    fm, body = parse_legacy_yaml(raw)
    assert fm.id == "REF-001"
    assert fm.kind == "refinement"
    assert body == b"# body\n"


def test_looks_like_yaml_for_json_is_false() -> None:
    raw = _build({"id": "x", "kind": "fact", "ts_ns": 1, "actor": "a"})
    assert looks_like_yaml(raw) is False


def test_legacy_yaml_v0_workbench_alias() -> None:
    """v0 workbench frontmatter (memory_id / scope / created_by / created_at)
    must alias cleanly onto the v1 (id / kind / actor / ts_ns) schema so the
    708 memories imported on 2026-05-19 are readable via cyberos view.

    v0 scope=meta is not in the v1 kind enum, so kind="unknown" + the original
    scope is preserved in extra.v0_scope (see _V0_SCOPE_TO_V1_KIND).
    """
    pytest.importorskip("yaml")
    raw = (
        b"---\n"
        b"memory_id: mem_019df384-4d80-74a9-a613-4b841e911a29\n"
        b"scope: meta\n"
        b"classification: operational\n"
        b"created_at: 2026-05-04T22:03:47+07:00\n"
        b"created_by: agent:claude-opus-4-7\n"
        b"---\n"
        b"# REF-001 body\n"
    )
    fm, body = parse_legacy_yaml(raw)
    assert fm.id == "mem_019df384-4d80-74a9-a613-4b841e911a29"
    assert fm.kind == "unknown"  # v0 scope=meta → v1 kind=unknown
    assert fm.extra.get("v0_scope") == "meta"
    assert fm.actor == "agent:claude-opus-4-7"
    # ts_ns derived from created_at via ISO 8601 → epoch nanoseconds
    from datetime import datetime
    expected_ts_ns = int(
        datetime.fromisoformat("2026-05-04T22:03:47+07:00").timestamp() * 1_000_000_000
    )
    assert fm.ts_ns == expected_ts_ns
    assert body == b"# REF-001 body\n"


def test_legacy_yaml_v1_wins_over_v0_aliases() -> None:
    """If both v0 and v1 names are present, v1 wins (alias never clobbers)."""
    pytest.importorskip("yaml")
    raw = (
        b"---\n"
        b"id: DEC-EXPLICIT\n"
        b"kind: decision\n"
        b"ts_ns: 42\n"
        b"actor: stephen\n"
        b"memory_id: mem_should_be_ignored\n"
        b"scope: should_be_ignored\n"
        b"created_by: should_be_ignored\n"
        b"created_at: 2026-01-01T00:00:00+00:00\n"
        b"---\n"
        b"body\n"
    )
    fm, _ = parse_legacy_yaml(raw)
    assert fm.id == "DEC-EXPLICIT"
    assert fm.kind == "decision"
    assert fm.actor == "stephen"
    assert fm.ts_ns == 42


def test_legacy_yaml_v0_scope_meta_maps_to_unknown() -> None:
    """v0 scope=meta has no v1 kind equivalent → kind="unknown" + extra.v0_scope.

    Without this remap, `cyberos validate` rejects v0-imported files because
    'meta' is not in the v1 kind enum.
    """
    pytest.importorskip("yaml")
    raw = (
        b"---\n"
        b"memory_id: mem_x\n"
        b"scope: meta\n"
        b"created_at: 2026-05-04T22:03:47+07:00\n"
        b"created_by: stephen\n"
        b"---\n"
        b"body\n"
    )
    fm, _ = parse_legacy_yaml(raw)
    assert fm.kind == "unknown"
    assert fm.extra.get("v0_scope") == "meta"
    # original v0 metadata preserved
    assert fm.extra.get("v0_memory_id") == "mem_x"
    assert fm.extra.get("v0_created_by") == "stephen"


def test_legacy_yaml_v0_scope_decision_passes_through() -> None:
    """v0 scope=decision (a value that IS in v1 kind enum) maps cleanly."""
    pytest.importorskip("yaml")
    raw = (
        b"---\n"
        b"memory_id: DEC-100\n"
        b"scope: decision\n"
        b"created_at: 2026-01-01T00:00:00+00:00\n"
        b"created_by: stephen\n"
        b"---\n"
        b"body\n"
    )
    fm, _ = parse_legacy_yaml(raw)
    assert fm.kind == "decision"
    assert fm.extra.get("v0_scope") == "decision"
