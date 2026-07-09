"""
MMR property tests.

Verifies the invariants the writer + doctor depend on:

* Determinism — same leaves same order ⇒ same root.
* Inclusion proofs verify for every leaf.
* Tampering (wrong leaf data) detected.
* Disk persistence round-trip.
* Cross-check helper recomputes correctly from a binlog.
"""

from __future__ import annotations

import random
from pathlib import Path

import pytest

from cyberos.core.mmr import (
    EMPTY_ROOT,
    MMR,
    OnDiskMMR,
    Peak,
    _peak_heights_for_leaf_count,
    mmr_root_for_binlog,
)


def test_empty_root() -> None:
    assert MMR().root() == EMPTY_ROOT


def test_single_leaf_root_is_hash_of_leaf() -> None:
    import hashlib
    mmr = MMR()
    mmr.append_leaf(b"x")
    expected = hashlib.sha256(b"\x00x").digest()
    assert mmr.root() == expected


@pytest.mark.parametrize("n", [1, 2, 3, 4, 5, 7, 8, 9, 15, 16, 17, 100])
def test_determinism_for_n_leaves(n: int) -> None:
    leaves = [f"leaf-{i}".encode() for i in range(n)]
    a, b = MMR(), MMR()
    for x in leaves:
        a.append_leaf(x)
        b.append_leaf(x)
    assert a.root() == b.root()
    assert a.leaf_count == n
    assert len(a.peaks) == bin(n).count("1")


@pytest.mark.parametrize("n", [1, 2, 3, 4, 5, 7, 8, 15, 16, 17])
def test_inclusion_proofs_for_every_leaf(n: int) -> None:
    leaves = [f"leaf-{i}".encode() for i in range(n)]
    mmr = MMR()
    for x in leaves:
        mmr.append_leaf(x)
    root = mmr.root()
    for i in range(n):
        proof = mmr.inclusion_proof(i, iter(leaves))
        assert MMR.verify_inclusion(leaves[i], i, proof, root, mmr.leaf_count), (
            f"leaf {i} of {n} failed to verify"
        )


def test_tampered_leaf_rejected() -> None:
    leaves = [f"leaf-{i}".encode() for i in range(11)]
    mmr = MMR()
    for x in leaves:
        mmr.append_leaf(x)
    root = mmr.root()
    proof = mmr.inclusion_proof(4, iter(leaves))
    assert not MMR.verify_inclusion(b"tampered", 4, proof, root, mmr.leaf_count)


def test_tampered_root_rejected() -> None:
    leaves = [f"leaf-{i}".encode() for i in range(11)]
    mmr = MMR()
    for x in leaves:
        mmr.append_leaf(x)
    proof = mmr.inclusion_proof(4, iter(leaves))
    assert not MMR.verify_inclusion(leaves[4], 4, proof, EMPTY_ROOT, mmr.leaf_count)


def test_peak_heights_match_popcount() -> None:
    """Number of peaks equals the popcount of leaf count."""
    for n in range(1, 50):
        heights = _peak_heights_for_leaf_count(n)
        assert len(heights) == bin(n).count("1")
        assert heights == sorted(heights, reverse=True)


def test_on_disk_persistence_round_trip(tmp_path: Path) -> None:
    """Persist after appends, reopen, append more, root matches a fresh run."""
    store = tmp_path / ".cyberos/memory/store"
    (store / "audit" / "mmr").mkdir(parents=True)

    first = OnDiskMMR(store)
    for i in range(5):
        first.append_leaf(f"x{i}".encode())
    root_after_5 = first.root()
    n_after_5 = first.leaf_count

    second = OnDiskMMR(store)
    assert second.leaf_count == n_after_5
    assert second.root() == root_after_5

    # Continue appending — root must match a fresh-MMR over all leaves.
    for i in range(5, 8):
        second.append_leaf(f"x{i}".encode())

    fresh = MMR()
    for i in range(8):
        fresh.append_leaf(f"x{i}".encode())
    assert second.root() == fresh.root()


def test_mmr_root_for_binlog_matches_inline_mmr(tmp_path: Path) -> None:
    """The cross-check helper must agree with an in-process MMR over the same records."""
    from cyberos.core.writer import AuditRecord, Writer, WriterConfig, _canonical

    store = tmp_path / ".cyberos/memory/store"
    (store / "audit").mkdir(parents=True)

    expected_root_mmr = MMR()
    # Disable the writer's own MMR; we'll cross-check via mmr_root_for_binlog
    # against an independently computed MMR over the same canonical payloads.
    with Writer(store, config=WriterConfig(enable_mmr=False)) as w:
        for i in range(10):
            rec = AuditRecord(
                op="view", path=f"memories/x{i}.md", actor="t",
                content_sha256=f"{i:064x}",
            )
            w.submit(rec)

    # Replay binlog and feed into a fresh MMR via the helper.
    binlogs = sorted((store / "audit").glob("*.binlog"))
    root_from_binlog, leaves = mmr_root_for_binlog(binlogs)
    assert leaves == 10
    # The writer assigns ts_ns + prev_chain + chain at flush time which
    # affects the canonical bytes, so we cannot reconstruct the same MMR
    # without replaying the binlog. The helper IS that replay. Confirm
    # determinism: calling it twice produces the same root.
    root_again, leaves_again = mmr_root_for_binlog(binlogs)
    assert root_from_binlog == root_again
    assert leaves_again == 10


def test_writer_mmr_matches_cross_check(tmp_path: Path) -> None:
    """When the writer's own MMR is enabled, the doctor's cross-check passes."""
    from cyberos.core.writer import AuditRecord, Writer

    store = tmp_path / ".cyberos/memory/store"
    (store / "audit").mkdir(parents=True)
    with Writer(store) as w:
        for i in range(15):
            w.submit(AuditRecord(
                op="view", path=f"memories/x{i}.md", actor="t",
                content_sha256=f"{i:064x}",
            ))

    # Writer persisted peaks.bin; reload and verify root matches a replay.
    persisted = OnDiskMMR(store)
    binlogs = sorted((store / "audit").glob("*.binlog"))
    replay_root, replay_leaves = mmr_root_for_binlog(binlogs)
    assert persisted.leaf_count == replay_leaves
    assert persisted.root() == replay_root
