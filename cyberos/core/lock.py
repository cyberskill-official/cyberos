"""
cyberos.core.lock — single .lock file with POSIX flock + monotonic-clock lease.

Replaces the legacy dual ``.lock.exclusive`` / ``.lock.shared`` files plus
the 5-minute stale-PID recovery heuristic. The old design:

* paid one extra ``open()`` per write for no semantic gain — POSIX ``flock``
  already supports ``LOCK_EX`` and ``LOCK_SH`` on the same fd;
* blocked competing agents for up to 5 minutes if a writer died with SIGKILL.

The new design:

* one ``.lock`` file holding a JSON lease record;
* ``LOCK_EX`` for writers, ``LOCK_SH`` for tools that need a consistent
  scan (read path doesn't take a lock — see :class:`cyberos.core.reader.Reader`
  which uses a HEAD seqlock);
* lease records ``{pid, host, monotonic_ns, expiry_ns, version}`` are
  written under the held flock; on acquire, expired leases are force-broken
  in O(microseconds), not 5 minutes;
* the writer renews the lease every ``_LEASE_RENEW_NS`` to cover long-running
  batches.

This matches LMDB's "wait-free readers, single serialised writer" model
(en.wikipedia.org/wiki/Lightning_Memory-Mapped_Database) adapted to a
filesystem-first store: we can't run a real MVCC engine on top of
human-readable Markdown files, but we can preserve the semantics.
"""

from __future__ import annotations

import json
import os
import socket
import sys
import time
from contextlib import contextmanager
from pathlib import Path
from typing import Final, Iterator

# 10s default lease; tuned so a writer killed -9 mid-batch unblocks
# competing agents in roughly one human-perceptible "moment", not in
# 5 minutes. Long-running writers (export, migration) override via
# ``lease_ttl_ns`` and call :meth:`StoreLock.renew` periodically.
_LEASE_TTL_NS: Final[int]   = 10 * 1_000_000_000
_LEASE_RENEW_NS: Final[int] = 3 * 1_000_000_000

_LEASE_VERSION: Final[int] = 1
_LEASE_MAX_BYTES: Final[int] = 4096

if sys.platform == "win32":
    import msvcrt  # type: ignore[import-not-found]
    _POSIX = False
else:
    import fcntl
    _POSIX = True


class LockContended(BlockingIOError):
    """Raised when a non-blocking lock acquisition fails."""


class LockBroken(RuntimeError):
    """Raised when the held lease has been broken by another process.

    Surfaces when :meth:`StoreLock.renew` finds the on-disk lease no
    longer matches the in-process lease — meaning another process saw the
    on-disk lease as expired and grabbed the lock. The caller must abort
    pending work; the audit ledger is no longer single-writer-safe from
    this process.
    """


class StoreLock:
    """Coordination primitive for a single ``.cyberos-memory/`` store.

    One instance per process per store. Re-entrant only through the
    ``shared`` context manager: explicit nested ``acquire_exclusive()``
    calls raise ``RuntimeError`` to keep callers honest about lock scope.

    Lifecycle::

        lock = StoreLock(store / ".lock")
        lock.acquire_exclusive()
        try:
            ...                          # batch of writes
            lock.renew()                 # periodically
        finally:
            lock.release()
    """

    def __init__(self, path: Path, *, lease_ttl_ns: int = _LEASE_TTL_NS):
        self.path = path
        self.lease_ttl_ns = lease_ttl_ns
        self._fd: int | None = None
        self._mode: str | None = None  # "ex" | "sh" | None
        self._issued_monotonic_ns: int | None = None

    # -- public API ----------------------------------------------------------

    def acquire_exclusive(self, *, blocking: bool = True) -> None:
        """Take the LOCK_EX lock and write a fresh lease record.

        If another process holds a valid (non-expired) lease, we wait for
        the kernel-level flock to release (the kernel handles this
        regardless of the lease record). If the OS-level lock is free
        but a stale lease record is on disk (e.g. a SIGKILL'd writer),
        we acquire and overwrite it — no 5-minute timer.
        """
        if self._mode is not None:
            raise RuntimeError(f"StoreLock {self.path!s} already held in mode={self._mode}")
        self._open()
        self._lock(exclusive=True, blocking=blocking)
        # We hold flock now. Inspect the on-disk lease; if it claims an
        # unexpired owner, we got the flock anyway which means the
        # previous owner released cleanly — proceed and overwrite.
        # If the lease is expired, we are correctly force-breaking it.
        self._read_existing_lease_for_diagnostics()
        self._write_lease()
        self._mode = "ex"

    def try_acquire_exclusive(self) -> bool:
        """Non-blocking variant. Returns False if the lock is contended."""
        try:
            self.acquire_exclusive(blocking=False)
            return True
        except (BlockingIOError, LockContended):
            return False

    @contextmanager
    def shared(self) -> Iterator[None]:
        """Take ``LOCK_SH`` for a consistent multi-file scan.

        Readers normally do NOT take this — they use the HEAD seqlock
        (see :class:`cyberos.core.reader.Reader`). Tools that need a
        consistent view across many files (export, full chain verify)
        use this to block concurrent writers for the duration of the scan.
        """
        if self._mode is not None:
            raise RuntimeError(f"StoreLock {self.path!s} already held in mode={self._mode}")
        self._open()
        self._lock(exclusive=False, blocking=True)
        self._mode = "sh"
        try:
            yield
        finally:
            self.release()

    def release(self) -> None:
        """Release the lock; clears the lease record by truncating to 0.

        Truncating instead of unlinking avoids a TOCTOU window where a
        competing acquirer ``open()``s the path after we unlink but
        before we close. The empty file is benign and treated as
        "no lease" by future acquirers.
        """
        if self._fd is None:
            return
        try:
            # Best-effort: clear the lease record before releasing.
            if self._mode == "ex":
                try:
                    os.ftruncate(self._fd, 0)
                except OSError:
                    pass
            if _POSIX:
                fcntl.flock(self._fd, fcntl.LOCK_UN)
            else:
                try:
                    msvcrt.locking(self._fd, msvcrt.LK_UNLCK, 1)  # type: ignore[attr-defined]
                except OSError:
                    pass
        finally:
            os.close(self._fd)
            self._fd = None
            self._mode = None
            self._issued_monotonic_ns = None

    def renew(self) -> None:
        """Refresh the lease for long-running writers.

        Must be called more frequently than ``lease_ttl_ns / 3`` to
        guarantee competing acquirers never see an expired lease while
        the writer is alive. The writer's commit-loop calls this every
        :data:`_LEASE_RENEW_NS`.

        Raises :class:`LockBroken` if the on-disk lease no longer matches
        our in-process lease — meaning another process broke us.
        """
        if self._mode != "ex" or self._fd is None:
            return
        # Detect a forced break: if the on-disk lease's monotonic_ns is
        # not the one we wrote, someone else took the lock.
        on_disk = self._read_lease()
        if on_disk is not None and on_disk.get("monotonic_ns") != self._issued_monotonic_ns:
            raise LockBroken(
                f"lease for {self.path!s} was broken by pid={on_disk.get('pid')} "
                f"host={on_disk.get('host')!r}"
            )
        self._write_lease()

    @property
    def held(self) -> bool:
        return self._mode is not None

    # -- internals -----------------------------------------------------------

    def _open(self) -> None:
        if self._fd is not None:
            return
        self.path.parent.mkdir(parents=True, exist_ok=True)
        # O_CLOEXEC where available so child processes don't inherit the
        # lock fd and accidentally keep the flock alive past parent exit.
        flags = os.O_RDWR | os.O_CREAT | getattr(os, "O_CLOEXEC", 0)
        self._fd = os.open(self.path, flags, 0o600)

    def _lock(self, *, exclusive: bool, blocking: bool) -> None:
        assert self._fd is not None
        if _POSIX:
            flag = fcntl.LOCK_EX if exclusive else fcntl.LOCK_SH
            if not blocking:
                flag |= fcntl.LOCK_NB
            try:
                fcntl.flock(self._fd, flag)
            except BlockingIOError as exc:
                raise LockContended(str(exc)) from exc
        else:
            # Windows: msvcrt.locking only supports byte-range exclusive
            # locks. We emulate shared semantics by always taking an
            # exclusive lock; concurrent readers serialise. Documented
            # limitation — POSIX is the supported deployment platform.
            mode = msvcrt.LK_LOCK if blocking else msvcrt.LK_NBLCK  # type: ignore[attr-defined]
            try:
                msvcrt.locking(self._fd, mode, 1)  # type: ignore[attr-defined]
            except OSError as exc:
                raise LockContended(str(exc)) from exc

    def _read_lease(self) -> dict | None:
        assert self._fd is not None
        try:
            os.lseek(self._fd, 0, os.SEEK_SET)
            raw = os.read(self._fd, _LEASE_MAX_BYTES)
            if not raw:
                return None
            return json.loads(raw.decode("utf-8"))
        except (OSError, ValueError, UnicodeDecodeError):
            return None

    def _read_existing_lease_for_diagnostics(self) -> None:
        # We hold flock — no one else is writing. If the lease is
        # expired, fine. If unexpired but we got flock, the previous
        # owner released cleanly. Diagnostic only; no action.
        existing = self._read_lease()
        if existing is None:
            return
        expiry = int(existing.get("expiry_ns", 0))
        if expiry > time.monotonic_ns():
            # Owner released the flock but the lease record still claims
            # they hold it. This is benign — flock is the source of truth
            # for the kernel-level mutex; the lease is for diagnostics
            # and stale-detection. Surface only on debug if requested.
            pass

    def _write_lease(self) -> None:
        assert self._fd is not None
        now = time.monotonic_ns()
        lease = {
            "pid": os.getpid(),
            "host": socket.gethostname(),
            "monotonic_ns": now,
            "expiry_ns": now + self.lease_ttl_ns,
            "version": _LEASE_VERSION,
        }
        body = json.dumps(lease, sort_keys=True, separators=(",", ":")).encode("utf-8")
        if len(body) > _LEASE_MAX_BYTES:
            raise RuntimeError(f"lease record exceeds {_LEASE_MAX_BYTES} bytes")
        os.lseek(self._fd, 0, os.SEEK_SET)
        os.ftruncate(self._fd, 0)
        os.write(self._fd, body)
        # Lease durability matters less than ledger durability — it's
        # recoverable on next session start. Plain fdatasync is fine.
        if hasattr(os, "fdatasync"):
            os.fdatasync(self._fd)
        else:  # pragma: no cover — Darwin/Windows fallback
            os.fsync(self._fd)
        self._issued_monotonic_ns = now


__all__ = [
    "StoreLock",
    "LockContended",
    "LockBroken",
]
