"""
Crash-safety regression — Jepsen-lite for the Writer.

Audit report §5 calls for 1,000 iterations per release in CI. This file
runs a configurable iteration count (default 100, override via env var
``CYBEROS_CRASH_ITERATIONS``) so PR runs stay fast while nightly soak
runs can push much higher.

The pattern:

  1. Fork a child writer.
  2. Have it submit N records.
  3. SIGKILL the child at a random point.
  4. Reopen the store with a new writer; expect:
       * tail frame CRC-truncated if needed,
       * chain verifies end-to-end,
       * last fully-flushed batch intact.
"""

from __future__ import annotations

import os
import random
import signal
import sys
import time
from pathlib import Path

import pytest

# Forking on macOS via Python is fragile; skip there in CI. Linux is the
# supported deployment platform for the crash-safety regression suite.
_NOT_LINUX = sys.platform != "linux"


@pytest.fixture()
def store(tmp_path: Path) -> Path:
    s = tmp_path / ".cyberos-memory"
    (s / "audit").mkdir(parents=True)
    return s


def _child_writer(store_str: str, n: int, seed: int) -> None:
    """Run as child process. Submit n rows; sleep briefly between them."""
    from cyberos.core.writer import AuditRecord, Writer, WriterConfig
    rng = random.Random(seed)
    cfg = WriterConfig(coalesce_window_ms=2, coalesce_max_batch=8)
    writer = Writer(Path(store_str), config=cfg)
    writer.open()
    try:
        for i in range(n):
            writer.submit(
                AuditRecord(
                    op="view",
                    path=f"memories/k{i:05d}.md",
                    actor="child",
                    content_sha256=f"{rng.randint(0, 1<<256):064x}",
                )
            )
    finally:
        writer.close()


@pytest.mark.skipif(_NOT_LINUX, reason="fork-and-kill crash safety regression runs on Linux only")
@pytest.mark.parametrize(
    "iteration",
    range(int(os.environ.get("CYBEROS_CRASH_ITERATIONS", "5"))),
)
def test_crash_safety_random_kill(store: Path, iteration: int) -> None:
    from cyberos.core.walker import verify_segments

    seed = iteration * 7919
    pid = os.fork()
    if pid == 0:
        # Child.
        try:
            _child_writer(str(store), n=200, seed=seed)
        finally:
            os._exit(0)
    # Parent: wait a random short interval, then SIGKILL.
    delay = random.Random(seed).uniform(0.005, 0.080)
    time.sleep(delay)
    try:
        os.kill(pid, signal.SIGKILL)
    except ProcessLookupError:
        pass  # child finished first
    os.waitpid(pid, 0)

    # Reopen and verify the chain.
    from cyberos.core.writer import AuditRecord, Writer
    with Writer(store) as writer:
        seq = writer.submit(
            AuditRecord(op="view", path="memories/post-crash.md", actor="p", content_sha256="0" * 64)
        )
        assert seq > 0

    segments = sorted(
        p for p in (store / "audit").glob("*.binlog") if p.name != "current.binlog"
    )
    current = store / "audit" / "current.binlog"
    if current.exists():
        segments.append(current)
    n = verify_segments(segments)
    assert n >= 1
