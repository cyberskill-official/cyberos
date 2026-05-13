"""
Unit tests for ``runtime/lib/brain_writer_shim.py``.

Covers every branch of :func:`shim_dispatch`:

* v1 stores → returns ``None`` (legacy main runs)
* v2 stores → translates each verb correctly
* unknown verbs → returns ``None`` (legacy main handles --help, errors)
* deferred verbs (protocol-upgrade, self-audit) → returns 2 with clear msg
* malformed manifest → returns ``None`` (failsafe)

Run::

    python -m pytest tests/test_shim.py -v
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

# Ensure runtime/ is on sys.path so the shim's own re-import works correctly.
_REPO = Path(__file__).resolve().parent.parent
if str(_REPO) not in sys.path:
    sys.path.insert(0, str(_REPO))

from runtime.lib.brain_writer_shim import (  # noqa: E402
    _read_schema_version,
    shim_dispatch,
)


# --- fixtures ------------------------------------------------------------


@pytest.fixture()
def v1_store(tmp_path: Path) -> Path:
    """A schema-v1 store (legacy)."""
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    (store / "manifest.json").write_text(
        json.dumps({"schema_version": 1}), encoding="utf-8",
    )
    return store


@pytest.fixture()
def v2_store(tmp_path: Path) -> Path:
    """A schema-v2 store with a legacy chain bridge."""
    store = tmp_path / ".cyberos-memory"
    (store / "audit").mkdir(parents=True)
    (store / "manifest.json").write_text(
        json.dumps({
            "schema_version": 2,
            "migration": {"legacy_last_chain": "a" * 64},
        }),
        encoding="utf-8",
    )
    return store


@pytest.fixture()
def content_file(tmp_path: Path) -> Path:
    """A minimal JSON-frontmatter memory body."""
    body = (
        b"---\n"
        b'{"actor":"test","extra":{},"id":"X","kind":"fact","tags":[],"ts_ns":1}'
        b"\n---\n"
        b"# body\n"
    )
    p = tmp_path / "body.md"
    p.write_bytes(body)
    return p


# --- _read_schema_version --------------------------------------------------


def test_read_schema_version_v1(v1_store: Path) -> None:
    assert _read_schema_version(v1_store) == 1


def test_read_schema_version_v2(v2_store: Path) -> None:
    assert _read_schema_version(v2_store) == 2


def test_read_schema_version_missing(tmp_path: Path) -> None:
    # No manifest at all → default to 1 (failsafe: shim does not fire)
    assert _read_schema_version(tmp_path / "nope") == 1


def test_read_schema_version_malformed(tmp_path: Path) -> None:
    store = tmp_path / ".cyberos-memory"
    store.mkdir(parents=True)
    (store / "manifest.json").write_text("not json at all", encoding="utf-8")
    assert _read_schema_version(store) == 1


# --- v1 path: shim returns None for everything -----------------------------


@pytest.mark.parametrize("argv", [
    ["status"], ["write", "a", "p", "f"], ["str-replace", "a", "p", "f"],
    ["session-start", "a"], ["session-end", "a"], ["verify"],
    ["protocol-upgrade", "a", "old", "new"], ["self-audit", "a"],
])
def test_v1_store_shim_does_not_fire(v1_store: Path, argv: list[str]) -> None:
    """For schema_version=1, shim returns None and legacy main runs."""
    assert shim_dispatch(argv, v1_store) is None


# --- v2 path: verb-by-verb ---------------------------------------------------


def test_v2_status(v2_store: Path, capsys: pytest.CaptureFixture) -> None:
    rc = shim_dispatch(["status"], v2_store)
    assert rc == 0
    out = capsys.readouterr().out
    assert "schema_version  : 2" in out
    assert ("a" * 64) in out  # legacy_last_chain surfaced


def test_v2_write_creates_memory_and_audit_row(
    v2_store: Path, content_file: Path, capsys: pytest.CaptureFixture,
) -> None:
    rc = shim_dispatch(
        ["write", "agent:shim", "memories/facts/X.md", str(content_file)],
        v2_store,
    )
    assert rc == 0
    assert (v2_store / "memories" / "facts" / "X.md").is_file()
    assert (v2_store / "audit" / "current.binlog").stat().st_size > 0
    assert capsys.readouterr().out.startswith("seq=")


def test_v2_str_replace_overwrites(
    v2_store: Path, content_file: Path,
) -> None:
    """Legacy str-replace = full-file overwrite under v2."""
    target = "memories/facts/Y.md"
    rc1 = shim_dispatch(
        ["write", "agent:shim", target, str(content_file)], v2_store,
    )
    assert rc1 == 0
    # Now overwrite with new content via str-replace
    new_body = content_file.parent / "new.md"
    new_body.write_bytes(
        b"---\n"
        b'{"actor":"test","extra":{},"id":"Y2","kind":"fact","tags":[],"ts_ns":2}'
        b"\n---\n"
        b"# replaced\n"
    )
    rc2 = shim_dispatch(
        ["str-replace", "agent:shim", target, str(new_body)], v2_store,
    )
    assert rc2 == 0
    written = (v2_store / target).read_bytes()
    assert b"# replaced" in written


@pytest.mark.parametrize("kind", ["start", "end"])
def test_v2_session_boundary(
    v2_store: Path, kind: str, capsys: pytest.CaptureFixture,
) -> None:
    rc = shim_dispatch([f"session-{kind}", "agent:shim"], v2_store)
    assert rc == 0
    out = capsys.readouterr().out
    assert out.startswith("seq=")

    # Confirm the row was written with the right op.
    from cyberos.core.walker import MmapWalker
    with MmapWalker(v2_store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    assert any(r.op == f"session.{kind}" for _o, r in records)


def test_v2_verify_delegates(v2_store: Path) -> None:
    """verify is a subprocess delegation; it returns the subprocess's rc."""
    # Empty binlog → cyberos verify exits 0
    rc = shim_dispatch(["verify"], v2_store)
    assert rc == 0


# --- v2 path: deferred verbs -----------------------------------------------


@pytest.mark.parametrize("argv", [
    ["protocol-upgrade", "a", "sha256:" + "a" * 64, "sha256:" + "b" * 64],
    ["self-audit", "a"],
])
def test_v2_deferred_verbs_refuse(
    v2_store: Path, argv: list[str], capsys: pytest.CaptureFixture,
) -> None:
    rc = shim_dispatch(argv, v2_store)
    assert rc == 2
    err = capsys.readouterr().err
    assert "schema_v2 equivalent yet" in err
    assert argv[0] in err  # offending verb in message


# --- v2 path: unknown verb falls through -----------------------------------


def test_v2_unknown_verb_falls_through(v2_store: Path) -> None:
    """An unrecognised verb returns None so legacy main can print --help."""
    assert shim_dispatch(["bananas"], v2_store) is None
    assert shim_dispatch(["--help"], v2_store) is None
    assert shim_dispatch([], v2_store) is None


# --- v2 path: argument validation ------------------------------------------


def test_v2_write_with_wrong_arg_count(
    v2_store: Path, capsys: pytest.CaptureFixture,
) -> None:
    rc = shim_dispatch(["write", "only-actor"], v2_store)
    assert rc == 2
    assert "usage:" in capsys.readouterr().err


def test_v2_write_with_missing_content_file(
    v2_store: Path, tmp_path: Path, capsys: pytest.CaptureFixture,
) -> None:
    rc = shim_dispatch(
        ["write", "agent:shim", "memories/facts/X.md", str(tmp_path / "no.md")],
        v2_store,
    )
    assert rc == 2
    assert "content_file not found" in capsys.readouterr().err
