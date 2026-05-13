"""StoreLock — leased single-lock tests."""

from __future__ import annotations

import sys
import threading
import time
from pathlib import Path

import pytest

from cyberos.core.lock import LockContended, StoreLock


@pytest.fixture()
def lock_path(tmp_path: Path) -> Path:
    return tmp_path / ".lock"


def test_exclusive_acquire_and_release(lock_path: Path) -> None:
    lock = StoreLock(lock_path)
    lock.acquire_exclusive()
    assert lock.held
    lock.release()
    assert not lock.held


def test_try_acquire_when_held_returns_false(lock_path: Path) -> None:
    if sys.platform == "win32":
        pytest.skip("Windows lock model differs; documented limitation")
    holder = StoreLock(lock_path)
    holder.acquire_exclusive()
    try:
        challenger = StoreLock(lock_path)
        # try_acquire is non-blocking; should immediately fail.
        assert challenger.try_acquire_exclusive() is False
    finally:
        holder.release()


def test_shared_is_reentrant_via_context_manager(lock_path: Path) -> None:
    lock = StoreLock(lock_path)
    with lock.shared():
        assert lock.held
    assert not lock.held


def test_lease_record_is_written(lock_path: Path) -> None:
    import json

    lock = StoreLock(lock_path)
    lock.acquire_exclusive()
    try:
        body = lock_path.read_bytes()
        lease = json.loads(body.decode("utf-8"))
        assert "pid" in lease
        assert "expiry_ns" in lease
        assert lease["expiry_ns"] > lease["monotonic_ns"]
    finally:
        lock.release()


def test_lease_cleared_on_release(lock_path: Path) -> None:
    lock = StoreLock(lock_path)
    lock.acquire_exclusive()
    lock.release()
    assert lock_path.stat().st_size == 0
