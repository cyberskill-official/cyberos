"""
Chain-bridge tests — verify the legacy→v2 continuity invariant.

The migration's chain-bridge model says: when the binlog is empty AND the
manifest carries ``migration.legacy_last_chain``, the first new record's
``prev_chain`` must equal that legacy chain (with ``sha256:`` prefix
stripped). This test pins that invariant.
"""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberos.core.walker import MmapWalker
from cyberos.core.writer import AuditRecord, Writer, _GENESIS_CHAIN


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos-memory"
    (s / "audit").mkdir(parents=True)
    return s


def _write_manifest(store: Path, manifest: dict) -> None:
    (store / "manifest.json").write_text(
        json.dumps(manifest, sort_keys=True), encoding="utf-8"
    )


def test_empty_store_uses_genesis(store: Path) -> None:
    """No manifest → first row's prev_chain is GENESIS (64 zeros)."""
    with Writer(store) as writer:
        writer.submit(AuditRecord(op="view", path="x.md", actor="t", content_sha256="0" * 64))
    with MmapWalker(store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    assert records[0][1].prev_chain == _GENESIS_CHAIN


def test_manifest_bridge_picked_up(store: Path) -> None:
    """manifest.migration.legacy_last_chain → first record's prev_chain."""
    legacy_chain = "a" * 64
    _write_manifest(
        store,
        {"schema_version": 2, "migration": {"legacy_last_chain": legacy_chain}},
    )
    with Writer(store) as writer:
        writer.submit(AuditRecord(op="view", path="x.md", actor="t", content_sha256="0" * 64))
    with MmapWalker(store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    assert records[0][1].prev_chain == legacy_chain


def test_manifest_bridge_strips_sha256_prefix(store: Path) -> None:
    """Legacy writer emits 'sha256:<hex>' — Writer must strip the prefix."""
    legacy_hex = "b" * 64
    _write_manifest(
        store,
        {
            "schema_version": 2,
            "migration": {"legacy_last_chain": f"sha256:{legacy_hex}"},
        },
    )
    with Writer(store) as writer:
        writer.submit(AuditRecord(op="view", path="x.md", actor="t", content_sha256="0" * 64))
    with MmapWalker(store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    assert records[0][1].prev_chain == legacy_hex


def test_explicit_override_beats_manifest(store: Path) -> None:
    """Constructor's initial_chain kwarg wins over manifest (test ergonomics)."""
    _write_manifest(
        store,
        {"migration": {"legacy_last_chain": "a" * 64}},
    )
    override = "c" * 64
    with Writer(store, initial_chain=override) as writer:
        writer.submit(AuditRecord(op="view", path="x.md", actor="t", content_sha256="0" * 64))
    with MmapWalker(store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    assert records[0][1].prev_chain == override


def test_malformed_legacy_chain_raises(store: Path) -> None:
    """A bridge value that isn't 64 hex must fail loudly, not silently fall through."""
    _write_manifest(
        store,
        {"migration": {"legacy_last_chain": "not-hex"}},
    )
    with pytest.raises(ValueError):
        # ValueError surfaces when _resolve_initial_chain() is called from
        # open() → _recover_tail() on an empty binlog.
        with Writer(store) as writer:
            writer.submit(AuditRecord(op="view", path="x.md", actor="t", content_sha256="0" * 64))


def test_existing_binlog_ignores_bridge(store: Path) -> None:
    """Once the binlog has records, the bridge is irrelevant — chain came from disk."""
    # First session: no manifest bridge, genesis start.
    with Writer(store) as writer:
        writer.submit(AuditRecord(op="view", path="x.md", actor="t", content_sha256="0" * 64))

    # Add a bridge AFTER records exist — must be ignored.
    _write_manifest(
        store,
        {"migration": {"legacy_last_chain": "d" * 64}},
    )
    with Writer(store) as writer:
        writer.submit(AuditRecord(op="view", path="y.md", actor="t", content_sha256="0" * 64))

    with MmapWalker(store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    # First record still chains from genesis; second chains from first.
    assert records[0][1].prev_chain == _GENESIS_CHAIN
    assert records[1][1].prev_chain == records[0][1].chain
    # Bridge value never appears in the chain.
    assert "d" * 64 not in (r.prev_chain for _, r in records)
