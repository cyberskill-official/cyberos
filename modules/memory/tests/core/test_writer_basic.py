"""Smoke tests for the group-commit Writer + chain integrity."""

from __future__ import annotations

import threading
from pathlib import Path

import pytest

from cyberos.core.writer import AuditRecord, Writer, WriterConfig
from cyberos.core.walker import MmapWalker, verify_segments


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos/memory/store"
    (s / "audit").mkdir(parents=True)
    return s


def test_single_append_round_trip(store: Path) -> None:
    with Writer(store) as writer:
        seq = writer.submit(
            AuditRecord(op="view", path="memories/x.md", actor="t", content_sha256="a" * 64)
        )
    assert seq == 1

    with MmapWalker(store / "audit" / "current.binlog") as w:
        records = list(w.iter_records())
    assert len(records) == 1
    _offset, rec = records[0]
    assert rec.op == "view"
    assert rec.path == "memories/x.md"
    assert rec.prev_chain == "0" * 64
    assert len(rec.chain) == 64
    assert rec.chain != "0" * 64


def test_group_commit_batches(store: Path) -> None:
    cfg = WriterConfig(coalesce_window_ms=20, coalesce_max_batch=64)
    with Writer(store, config=cfg) as writer:
        threads = []
        for i in range(8):
            t = threading.Thread(
                target=lambda i=i: [
                    writer.submit(
                        AuditRecord(
                            op="view",
                            path=f"memories/{i}/{j}.md",
                            actor=f"t{i}",
                            content_sha256="c" * 64,
                        )
                    )
                    for j in range(100)
                ]
            )
            threads.append(t)
            t.start()
        for t in threads:
            t.join()

    segments = [store / "audit" / "current.binlog"]
    n = verify_segments(segments)
    assert n == 800


def test_chain_link_invariant(store: Path) -> None:
    with Writer(store) as writer:
        for i in range(50):
            writer.submit(
                AuditRecord(
                    op="create",
                    path=f"memories/{i}.md",
                    actor="t",
                    content_sha256="d" * 64,
                )
            )
    n = verify_segments([store / "audit" / "current.binlog"])
    assert n == 50


def test_recovery_truncates_corrupt_tail(store: Path) -> None:
    """After a partial write, the tail frame is CRC-rejected on reopen."""
    with Writer(store) as writer:
        for i in range(5):
            writer.submit(
                AuditRecord(op="view", path=f"x{i}.md", actor="t", content_sha256="e" * 64)
            )

    binlog = store / "audit" / "current.binlog"
    original_size = binlog.stat().st_size
    # Corrupt the tail by appending garbage that won't pass CRC.
    with open(binlog, "ab") as fh:
        fh.write(b"\x00" * 64)  # half a fake frame header + body
    assert binlog.stat().st_size > original_size

    # Reopen — recovery should truncate the garbage and the new writer
    # should be able to append cleanly.
    with Writer(store) as writer:
        seq = writer.submit(
            AuditRecord(op="view", path="x-recovered.md", actor="t", content_sha256="f" * 64)
        )
    assert seq == 6
    # Chain still verifies end-to-end.
    n = verify_segments([binlog])
    assert n == 6


def test_head_advances_atomically(store: Path) -> None:
    """HEAD reflects last durably committed seq after each batch."""
    import struct

    head_path = store / "HEAD"
    with Writer(store) as writer:
        for i in range(10):
            seq = writer.submit(
                AuditRecord(op="view", path=f"a{i}.md", actor="t", content_sha256="0" * 64)
            )
            with open(head_path, "rb") as fh:
                head = struct.unpack("<Q", fh.read(8))[0]
            assert head >= seq, f"HEAD={head} < seq={seq}"
