"""
cyberos.core.fsync — the durability barrier abstraction.

Fixes a latent data-loss bug in the legacy writer: on macOS, plain
``os.fsync()`` does NOT flush the device write cache. Apple Developer Forums:
"F_FULLFSYNC asks the target device to flush its hardware cache" and
"fsync ... will report the data has been written to disk, but may remain in
the device cache". Apple's bundled SQLite quietly maps
``PRAGMA fullfsync=on`` to the cheaper ``F_BARRIERFSYNC`` (see
bonsaidb.io/blog/acid-on-apple, mjtsai.com 2025).

Per-platform strategy used by the group-commit writer:

  * Linux:   ``fdatasync(fd)`` per batch; optional ``io_uring`` linked
             ``WRITEV`` + ``FSYNC`` (see :mod:`cyberos.core.iouring`).
  * Darwin:  ``fcntl(F_BARRIERFSYNC)`` for the common per-batch path —
             ordering without paying the hardware-flush cost.
             ``F_FULLFSYNC`` reserved for Merkle-checkpoint flush, a
             true power-loss boundary the user asked for.
  * Windows: ``FlushFileBuffers`` via ``os.fsync``.

References
----------
* developer.apple.com/documentation/xcode/reducing-disk-writes
* sqlite.org/wal.html
* lwn.net/Articles/457667 ("ext4 and data loss")
* kernel.dk/io_uring.pdf §IOSQE_IO_LINK

Public API
----------
* :func:`durable_sync` — flush an open fd according to the chosen strategy.
* :func:`durable_dir_sync` — fsync a directory so a rename(2) is durable.
* :data:`F_BARRIERFSYNC`, :data:`F_FULLFSYNC` — Darwin fcntl constants
  (not exported by the stdlib ``fcntl`` module).
"""

from __future__ import annotations

import os
import sys
from pathlib import Path
from typing import Final

# Darwin fcntl constants. Not present in Python's fcntl module; values are
# stable across macOS releases and match <sys/fcntl.h>.
F_BARRIERFSYNC: Final[int] = 85
F_FULLFSYNC: Final[int]    = 51

# Strategy names accepted by :func:`durable_sync`.
STRATEGY_AUTO: Final[str]      = "auto"
STRATEGY_FDATASYNC: Final[str] = "fdatasync"
STRATEGY_FBARRIER: Final[str]  = "fbarrier"
STRATEGY_FFULL: Final[str]     = "ffull"

_PLATFORM: Final[str] = sys.platform


def _is_darwin() -> bool:
    return _PLATFORM == "darwin"


def _is_linux() -> bool:
    return _PLATFORM.startswith("linux")


def _is_windows() -> bool:
    return _PLATFORM == "win32"


def durable_sync(fd: int, *, strategy: str = STRATEGY_AUTO) -> None:
    """Flush ``fd`` durably according to ``strategy``.

    Parameters
    ----------
    fd:
        Open file descriptor (write end).
    strategy:
        One of:

        - ``"auto"`` — best per-platform default. Darwin → ``fbarrier``;
          everywhere else → ``fdatasync``. Use this for per-batch ledger
          appends.
        - ``"fdatasync"`` — Linux: ``os.fdatasync``; macOS/Windows: falls
          back to ``os.fsync`` (which on Darwin is NOT sufficient for
          power-loss durability — use ``fbarrier`` or ``ffull`` there).
        - ``"fbarrier"`` — Darwin: ``F_BARRIERFSYNC`` (ordering without
          hardware flush). Linux: ``fdatasync``. Windows: ``os.fsync``.
        - ``"ffull"`` — Darwin: ``F_FULLFSYNC`` (true power-loss
          durability). Everywhere else: ``os.fsync``. Use only on
          checkpoint flush, not per-batch — costly.

    Raises
    ------
    OSError
        If the underlying syscall fails. Callers must treat this as a
        commit failure; the writer aborts the batch on this path.
    """
    chosen = strategy
    if chosen == STRATEGY_AUTO:
        chosen = STRATEGY_FBARRIER if _is_darwin() else STRATEGY_FDATASYNC

    if chosen == STRATEGY_FFULL:
        if _is_darwin():
            import fcntl  # noqa: WPS433 — local import to keep cold paths cheap
            fcntl.fcntl(fd, F_FULLFSYNC)
            return
        os.fsync(fd)
        return

    if chosen == STRATEGY_FBARRIER:
        if _is_darwin():
            import fcntl  # noqa: WPS433
            fcntl.fcntl(fd, F_BARRIERFSYNC)
            return
        # Outside Darwin, F_BARRIERFSYNC has no analogue; fdatasync is the
        # closest cheap durable-ordering primitive.
        if hasattr(os, "fdatasync"):
            os.fdatasync(fd)
            return
        os.fsync(fd)
        return

    if chosen == STRATEGY_FDATASYNC:
        if hasattr(os, "fdatasync"):
            os.fdatasync(fd)
            return
        # Darwin and Windows do not expose fdatasync. NOTE the Darwin
        # caveat above: callers wanting *real* durability on macOS should
        # explicitly request ``fbarrier`` or ``ffull``.
        os.fsync(fd)
        return

    raise ValueError(f"unknown durable_sync strategy: {strategy!r}")


def durable_dir_sync(directory: Path) -> None:
    """Sync ``directory`` so a preceding ``rename(2)`` is durable across crashes.

    The classic ``tmp + fsync + rename + parent-fsync`` atomic-write pattern
    (lwn.net/Articles/457667) only delivers crash safety if the parent
    directory is itself fsynced AFTER the rename — otherwise the rename can
    be lost while the data is preserved, the opposite of what you want.

    Windows has no equivalent: ``FlushFileBuffers`` cannot target a
    directory handle; the NTFS journal handles directory durability
    transparently. This function is a no-op there.
    """
    if _is_windows():
        return
    fd = os.open(directory, os.O_DIRECTORY | os.O_RDONLY)
    try:
        # Use plain fsync here — on Darwin we accept that this is barrier
        # ordering only. The data-bearing fd was synced with the chosen
        # strategy already; this call is about making the rename durable
        # in the directory entry, where the cost vs F_FULLFSYNC tradeoff
        # is the same as for the data path.
        if _is_darwin():
            import fcntl  # noqa: WPS433
            fcntl.fcntl(fd, F_BARRIERFSYNC)
        else:
            os.fsync(fd)
    finally:
        os.close(fd)


def durable_rename(src: Path, dst: Path, *, strategy: str = STRATEGY_AUTO) -> None:
    """Atomic-rename helper: rename ``src`` over ``dst`` then sync the parent.

    Does NOT fsync ``src`` itself — callers must do that before invoking
    this helper so the data is on stable storage prior to the rename.
    See `lwn.net/Articles/457667 <https://lwn.net/Articles/457667>`_.
    """
    os.replace(src, dst)
    durable_dir_sync(dst.parent)


__all__ = [
    "F_BARRIERFSYNC",
    "F_FULLFSYNC",
    "STRATEGY_AUTO",
    "STRATEGY_FBARRIER",
    "STRATEGY_FDATASYNC",
    "STRATEGY_FFULL",
    "durable_sync",
    "durable_dir_sync",
    "durable_rename",
]
