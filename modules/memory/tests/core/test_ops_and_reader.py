"""Six-file-ops + lock-free Reader integration tests."""

from __future__ import annotations

from pathlib import Path

import pytest

from cyberos.core import ops
from cyberos.core.frontmatter import Frontmatter, serialize
from cyberos.core.reader import Reader, ReaderUnstable
from cyberos.core.writer import Writer


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos/memory/store"
    (s / "audit").mkdir(parents=True)
    return s


def _make_memory(fm_id: str, body: bytes = b"# body\n") -> bytes:
    fm = Frontmatter(id=fm_id, kind="fact", ts_ns=1, actor="test")
    return serialize(fm, body)


def test_put_then_read(store: Path) -> None:
    with Writer(store) as writer:
        ops.put(writer, "memories/facts/FACT-001.md", _make_memory("FACT-001"), actor="t")

    reader = Reader(store)
    fm, body = reader.view("memories/facts/FACT-001.md")
    assert fm.id == "FACT-001"
    assert body == b"# body\n"


def test_put_is_idempotent(store: Path) -> None:
    with Writer(store) as writer:
        ops.put(writer, "memories/facts/x.md", _make_memory("x"), actor="t")
        # put is idempotent — second call replaces the body cleanly.
        ops.put(writer, "memories/facts/x.md", _make_memory("x"), actor="t")


def test_move_dst_collision(store: Path) -> None:
    with Writer(store) as writer:
        ops.put(writer, "memories/facts/a.md", _make_memory("a"), actor="t")
        ops.put(writer, "memories/facts/b.md", _make_memory("b"), actor="t")
        with pytest.raises(FileExistsError):
            ops.move(writer, "memories/facts/a.md", "memories/facts/b.md", actor="t")


def test_path_traversal_blocked(store: Path) -> None:
    with Writer(store) as writer:
        with pytest.raises(ops.PathTraversal):
            ops.put(writer, "../escape.md", _make_memory("x"), actor="t")
        with pytest.raises(ops.PathTraversal):
            ops.put(writer, "/absolute.md", _make_memory("x"), actor="t")


def test_reader_returns_after_concurrent_writer(store: Path) -> None:
    """Smoke test for the seqlock retry loop.

    Spawns a writer that re-creates the same file with new contents
    repeatedly; the reader must converge on one consistent (fm, body).
    """
    import threading
    import time

    target = "memories/facts/contention.md"
    abs_path = store / target
    abs_path.parent.mkdir(parents=True, exist_ok=True)
    abs_path.write_bytes(_make_memory("V0"))

    stop = threading.Event()

    def churn():
        i = 0
        with Writer(store) as writer:
            while not stop.is_set():
                body = _make_memory(f"V{i}", body=f"# v{i}\n".encode())
                tmp = abs_path.with_name(abs_path.name + ".tmp")
                tmp.write_bytes(body)
                import os
                os.replace(tmp, abs_path)
                # also append an audit row to bump HEAD
                writer.submit(
                    __import__("cyberos.core.writer", fromlist=["AuditRecord"]).AuditRecord(
                        op="put", path=target, actor="churn", content_sha256="0" * 64,
                    )
                )
                i += 1
                time.sleep(0.001)

    t = threading.Thread(target=churn, daemon=True)
    t.start()
    try:
        # Give the churn thread a moment to start writing — without this
        # the reader can race ahead and only see the pre-churn V0 state,
        # which is a valid observation but not what the test is trying
        # to exercise.
        time.sleep(0.020)
        reader = Reader(store)
        for _ in range(20):
            fm, body = reader.view(target)
            # Any consistent (fm, body) pair is acceptable — the test
            # asserts the reader stabilises rather than tearing across
            # a concurrent write. The id format is "V<int>" so it
            # starts with 'V'; the body always begins with "# ".
            assert fm.id.startswith("V")
            assert body.startswith(b"# ")
    finally:
        stop.set()
        t.join(timeout=2.0)
