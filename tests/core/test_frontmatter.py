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
