"""
STH signing + consolidate pipeline tests.

Verifies:
* STH signs + verifies + tamper-detects (3 fields).
* sign_and_publish chains via previous_sth.
* cyberos consolidate runs Walk → Compact → Sign → Publish end-to-end.
* Consolidation refuses to proceed over a failing Walk.
"""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

cryptography = pytest.importorskip("cryptography")
zstandard = pytest.importorskip("zstandard")

from cyberos.core.sth import (  # noqa: E402
    KeyPaths,
    ensure_key,
    key_id,
    latest_sth,
    load_public_key,
    sign_and_publish,
    sign_tree_head,
    verify_tree_head,
)


@pytest.fixture()
def keypaths(tmp_path: Path) -> KeyPaths:
    p = KeyPaths(
        private=tmp_path / "sth_signing_key",
        public=tmp_path / "sth_signing_key.pub",
    )
    ensure_key(p)
    return p


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos-memory"
    (s / "audit").mkdir(parents=True)
    (s / "manifest.json").write_text(
        json.dumps({"schema_version": 2}), encoding="utf-8",
    )
    return s


def test_sign_and_verify_roundtrip(keypaths: KeyPaths) -> None:
    sth = sign_tree_head(tree_size=42, root_hash_hex="a" * 64, paths=keypaths)
    assert verify_tree_head(sth, paths=keypaths)


def test_tamper_root_hash_detected(keypaths: KeyPaths) -> None:
    sth = sign_tree_head(tree_size=42, root_hash_hex="a" * 64, paths=keypaths)
    sth["root_hash"] = "b" * 64
    assert not verify_tree_head(sth, paths=keypaths)


def test_tamper_tree_size_detected(keypaths: KeyPaths) -> None:
    sth = sign_tree_head(tree_size=42, root_hash_hex="a" * 64, paths=keypaths)
    sth["tree_size"] = 99
    assert not verify_tree_head(sth, paths=keypaths)


def test_tamper_timestamp_detected(keypaths: KeyPaths) -> None:
    sth = sign_tree_head(tree_size=42, root_hash_hex="a" * 64, paths=keypaths)
    sth["timestamp"] = "1970-01-01T00:00:00Z"
    assert not verify_tree_head(sth, paths=keypaths)


def test_sign_and_publish_chains_previous(store: Path, keypaths: KeyPaths) -> None:
    a = sign_and_publish(store, tree_size=10, root_hash_hex="a" * 64, paths=keypaths)
    b = sign_and_publish(store, tree_size=20, root_hash_hex="b" * 64, paths=keypaths)
    assert a != b
    _, second = latest_sth(store)
    assert second["previous_sth"] is not None
    assert second["previous_sth"].endswith(".json")
    assert verify_tree_head(second, paths=keypaths)


def test_consolidate_runs_end_to_end(store: Path, keypaths: KeyPaths, monkeypatch) -> None:
    """Walk → Compact → Sign → Publish on a populated v2 store."""
    from cyberos.core.consolidate import run
    from cyberos.core.writer import AuditRecord, Writer

    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", "/")
    # Populate a few records so the MMR has something to sign.
    with Writer(store) as w:
        for i in range(5):
            w.submit(AuditRecord(
                op="view", path=f"memories/x{i}.md",
                actor="t", content_sha256=f"{i:064x}",
            ))

    # Point the STH machinery at the test-fixture key.
    monkeypatch.setattr(
        "cyberos.core.sth.KeyPaths.default",
        classmethod(lambda cls, base=None: keypaths),
    )

    report = run(store)
    assert report.ok, f"consolidation failed: {report.errors}"
    assert report.sth_path is not None
    assert report.leaf_count == 5

    # STH on disk verifies.
    _, sth_rec = latest_sth(store)
    assert verify_tree_head(sth_rec, paths=keypaths)

    # Manifest carries the new consolidation pointer.
    manifest = json.loads((store / "manifest.json").read_text(encoding="utf-8"))
    assert manifest["consolidation"]["last_mmr_root"] == sth_rec["root_hash"]
    assert manifest["consolidation"]["last_leaf_count"] == 5


def test_consolidate_refuses_over_failing_walk(store: Path, monkeypatch) -> None:
    """If Walk fails, Compact/Sign/Publish MUST NOT run."""
    from cyberos.core.consolidate import run

    monkeypatch.setenv("CYBEROS_HOST_MOUNT_PREFIX", "/")
    # Deliberately corrupt the manifest so the schema-version invariant fails.
    (store / "manifest.json").write_text("{}", encoding="utf-8")
    report = run(store)
    assert not report.ok
    assert report.sth_path is None  # Sign never ran
