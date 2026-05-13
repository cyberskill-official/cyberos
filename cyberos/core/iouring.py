"""
cyberos.core.iouring — optional Linux fast path for the writer's flush.

Imported lazily by :mod:`cyberos.core.writer`. On Linux ≥ 5.6 with the
``uring`` Python binding installed, the writer's batch flush becomes a
single linked SQE chain::

    WRITEV(binlog_fd, frames)
       │
       │ IOSQE_IO_LINK — "the next sqe will not be started before the
       │                  previous sqe has completed successfully"
       ▼
    FSYNC(binlog_fd, IORING_FSYNC_DATASYNC)
       │
       │ IOSQE_IO_LINK
       ▼
    WRITEV(head_fd, head_bytes, offset=0)
       │
       │ IOSQE_IO_LINK
       ▼
    FSYNC(head_fd, IORING_FSYNC_DATASYNC)

Source for the linked-SQE semantics:
`kernel.dk/io_uring.pdf <https://kernel.dk/io_uring.pdf>`_ §IOSQE_IO_LINK.

Removes 3 syscalls and 3 context switches per batch versus the
``writev → fdatasync → write → fdatasync`` syscall path on the standard
flush path.

If the binding is unavailable, :func:`available` returns False and the
writer transparently falls back to :mod:`cyberos.core.fsync` with no
behavioural change. The fallback is the production path; this module is
strictly an opt-in optimisation.
"""

from __future__ import annotations

import os
import sys
from typing import Final, Sequence

_AVAILABLE: bool = False
_RING = None  # type: ignore[var-annotated]
_uring = None  # type: ignore[var-annotated]


def _try_init() -> None:
    """Probe for the ``uring`` binding; populate module globals on success.

    Called lazily on first use, not at import time, so cold paths that
    don't actually flush via io_uring don't pay for the import.
    """
    global _AVAILABLE, _RING, _uring
    if _AVAILABLE or not sys.platform.startswith("linux"):
        return
    try:
        import uring  # type: ignore[import-not-found]
        _uring = uring
        _RING = uring.Ring(entries=64, flags=0)
        _AVAILABLE = True
    except ImportError:
        _AVAILABLE = False
    except OSError:  # pragma: no cover — kernel too old, ENOSYS, etc.
        _AVAILABLE = False


def available() -> bool:
    """Return True if the io_uring fast path can be used on this host."""
    _try_init()
    return _AVAILABLE


def flush_batch_linked(
    binlog_fd: int,
    frames: Sequence[bytes],
    head_fd: int,
    head_bytes: bytes,
) -> None:
    """Submit a linked WRITEV+FSYNC chain for binlog and HEAD as one ring op.

    Equivalent in durability to the userspace ``writev → fdatasync →
    write → fdatasync`` sequence, but issued as a single submit-and-wait
    call. The linked SQEs guarantee strict ordering — FSYNC(binlog) only
    fires after WRITEV(binlog) returns success, and WRITEV(head) only
    after FSYNC(binlog) — which preserves the writer's crash-safety
    invariant.

    Caller MUST have invoked :func:`available` and got True. If io_uring
    is unavailable this function raises ``RuntimeError``.
    """
    if not available():
        raise RuntimeError("io_uring not available on this host")
    assert _RING is not None and _uring is not None

    sqe = _RING.get_sqe()
    sqe.prep_writev(binlog_fd, list(frames), offset=-1)
    sqe.set_flags(_uring.IOSQE_IO_LINK)

    sqe2 = _RING.get_sqe()
    sqe2.prep_fsync(binlog_fd, flags=_uring.IORING_FSYNC_DATASYNC)
    sqe2.set_flags(_uring.IOSQE_IO_LINK)

    sqe3 = _RING.get_sqe()
    sqe3.prep_writev(head_fd, [head_bytes], offset=0)
    sqe3.set_flags(_uring.IOSQE_IO_LINK)

    sqe4 = _RING.get_sqe()
    sqe4.prep_fsync(head_fd, flags=_uring.IORING_FSYNC_DATASYNC)

    _RING.submit_and_wait(4)
    for cqe in _RING.peek_cqes(4):
        if cqe.res < 0:
            raise OSError(-cqe.res, os.strerror(-cqe.res))
        _RING.cqe_seen(cqe)


__all__ = ["available", "flush_batch_linked"]
