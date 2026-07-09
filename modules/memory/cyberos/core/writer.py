"""
cyberos.core.writer — the only writer for the Layer-1 audit ledger.

Implements audit report §4 CODE BLOCK 1. The hot path is:

    submit(record)         # N producer threads enqueue
        └─ commit-loop     # 1 thread drains every <coalesce_window_ms>
            ├─ assign seq, compute chain, msgspec-encode payload
            ├─ ONE writev(binlog_fd, frames)
            ├─ ONE durable_sync(binlog_fd, strategy=fbarrier|fdatasync)
            ├─ ONE atomic HEAD update (tmp+sync+rename+parent-sync)
            └─ wake all producers in batch

This is the same group-commit pattern as PostgreSQL / InnoDB / Pebble /
RocksDB. Reduces per-commit fsync syscalls by ~N (batch size), turning
~250 appends/sec on consumer NVMe into ~6,000–9,000/sec on the same
hardware with *stronger* durability than the legacy writer (which used
plain ``os.fsync()`` — non-durable on macOS).

Invariants preserved (audit report §3.C):

* Single writer per store via :class:`cyberos.core.lock.StoreLock`.
* Append-only ledger; no record mutated after the next record is written.
* Merkle LINK invariant: ``chain = SHA-256(canonical_json(rec_minus_chain) || prev_chain)``.
* Atomic record visibility: a record is "committed" only after BOTH the
  binlog ``durable_sync`` AND the HEAD atomic update have returned.
* The canonical file ops (view / put / move / delete) decompose into
  :class:`AuditRecord` variants; nothing else is permitted to append to
  the ledger. Historical binlog rows may carry earlier op names verbatim.
"""

from __future__ import annotations

import hashlib
import os
import struct
import sys
import threading
import time
from collections import deque
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Final

try:
    import msgspec
except ImportError:  # pragma: no cover
    sys.stderr.write(
        "FATAL: msgspec is not installed. Run:\n"
        "  pip install -r cyberos/requirements.txt --break-system-packages\n"
        "or, minimally:\n"
        "  pip install msgspec --break-system-packages\n"
    )
    raise

from cyberos.core.fsync import (
    STRATEGY_AUTO,
    STRATEGY_FFULL,
    durable_dir_sync,
    durable_sync,
)
from cyberos.core.lock import StoreLock

# --- on-disk record framing ------------------------------------------------
#
# Each binlog frame:
#
#   ┌──────────────────────┬──────────────────────┬─────────────┬──────────────┐
#   │ length (u32 BE)      │ crc32c (u32 BE)      │ seq (u64 BE)│ ts_ns (u64 BE)│  ← 24 bytes header
#   ├──────────────────────┴──────────────────────┴─────────────┴──────────────┤
#   │ payload — msgspec canonical-JSON of AuditRecord (length bytes)            │
#   └───────────────────────────────────────────────────────────────────────────┘
#
# Length-prefix framing lets the walker skip records without decoding
# (Bitcask's "review only the last record or two written and verify CRC"
# recovery rule — riak.com/assets/bitcask-intro.pdf).

_FRAME_HDR: Final[struct.Struct] = struct.Struct(">IIQQ")  # 24 bytes
_HEAD_FMT: Final[struct.Struct] = struct.Struct("<Q")      # LE u64 for cheap atomic read

_GENESIS_CHAIN: Final[str] = "0" * 64
_FLUSH_TIMEOUT_S: Final[float] = 30.0
_THREAD_JOIN_TIMEOUT_S: Final[float] = 5.0


class AuditRecord(msgspec.Struct, kw_only=True, frozen=True, omit_defaults=False):
    """One row in the append-only Merkle-chained audit ledger.

    Schema is frozen for the lifetime of the store version. Forward-compat
    additions land in ``extra`` (untyped dict) so older readers can still
    parse newer rows.

    The ``chain`` field is computed by the writer and MUST NOT be supplied
    by callers — :meth:`Writer.submit` overwrites it.
    """

    op: str
    """Canonical ops: put, move, delete, view. Free-form string — historical
    rows from earlier protocol generations carry their original op names
    verbatim and remain readable."""

    path: str
    """POSIX-style path relative to the store root."""

    actor: str
    """Principal identifier — e.g. 'stephen', 'coding-agent'."""

    ts_ns: int = 0
    """Wall-clock nanoseconds at flush time. Filled by the writer."""

    content_sha256: str = ""
    """SHA-256 of the file's bytes after the op. Empty for 'view'."""

    prev_chain: str = _GENESIS_CHAIN
    """The previous record's ``chain``. 64 zeros for the genesis record."""

    chain: str = ""
    """SHA-256(canonical_json(rec_minus_chain) || prev_chain). Filled by writer."""

    extra: dict[str, Any] = msgspec.field(default_factory=dict)
    """Free-form per-op fields (e.g. 'to' for rename, frontmatter excerpts)."""


# Module-level encoder/decoder. msgspec's ``order='sorted'`` makes the
# emitted JSON canonical for our closed schema — byte-identical to RFC 8785
# JCS output. The CI fuzz check in tests/core/test_canonical_equivalence.py
# enforces this against the legacy rfc8785 implementation.
_ENC: Final[msgspec.json.Encoder] = msgspec.json.Encoder(order="sorted")
_DEC: Final[msgspec.json.Decoder] = msgspec.json.Decoder(AuditRecord)


def _canonical(rec: AuditRecord) -> bytes:
    """Canonical-JSON encode a record. RFC 8785 equivalent for this schema."""
    return _ENC.encode(rec)


def _chain_hash(prev: str, rec_no_chain: AuditRecord) -> str:
    """LINK invariant: chain = SHA-256(canonical(rec_minus_chain) || prev_chain)."""
    h = hashlib.sha256()
    h.update(_canonical(rec_no_chain))
    h.update(bytes.fromhex(prev))
    return h.hexdigest()


# --- CRC32C (Castagnoli) ---------------------------------------------------
#
# Hardware-accelerated CRC-32C on Intel (SSE 4.2) and ARM (CRC32 instr).
# The `crc32c` PyPI wheel is ~3 GB/s on a 2024 M2; the zlib fallback is
# pure-Python and ~50 MB/s — fine for development, NOT production.
try:
    from crc32c import crc32c as _crc32c  # type: ignore[import-not-found]
    _CRC_IMPL: Final[str] = "hw"
except ImportError:  # pragma: no cover
    import zlib
    _CRC_IMPL = "zlib-fallback"

    def _crc32c(data: bytes) -> int:  # type: ignore[no-redef]
        """Fallback: NOT CRC-32C — plain CRC-32. Install `crc32c` for prod."""
        return zlib.crc32(data) & 0xFFFFFFFF


def crc_implementation() -> str:
    """Return ``'hw'`` if the SSE 4.2 / ARM CRC32 wheel is loaded, else fallback."""
    return _CRC_IMPL


@dataclass
class WriterConfig:
    """Tuning knobs for :class:`Writer`. Defaults are production-safe."""

    coalesce_window_ms: int = 5
    """Max ms to wait for more rows before flushing the current batch."""

    coalesce_max_batch: int = 16
    """Force flush when this many rows have accumulated, regardless of time."""

    fsync_strategy: str = STRATEGY_AUTO
    """Override the per-batch durability strategy. See :mod:`cyberos.core.fsync`."""

    checkpoint_strategy: str = STRATEGY_FFULL
    """Strategy for Merkle-checkpoint flushes — true power-loss barrier."""

    use_io_uring: bool = sys.platform.startswith("linux")
    """Try the io_uring fast path on Linux. Falls back to writev+fdatasync."""

    flush_timeout_s: float = _FLUSH_TIMEOUT_S
    """Per-submit timeout. Submitter aborts and raises if commit thread is stuck."""

    enable_mmr: bool = True
    """Append every batch's canonical-JSON payloads into a Merkle Mountain Range.

    Additive per PROPOSAL.md P2 Stage 1: the MMR runs alongside the
    per-row chain; the chain remains the source of truth. The
    ``ledger-mmr-cross-check`` doctor invariant cross-references the
    two. Disable for tests that don't care about MMR state or for
    write paths where the MMR persistence cost is unwanted.
    """


class WriterClosedError(RuntimeError):
    """Raised when :meth:`Writer.submit` is called after :meth:`Writer.close`."""


class CommitFailed(IOError):
    """Raised when a batch flush fails. The writer must be considered poisoned."""


class Writer:
    """The canonical Layer-1 writer.

    One instance per process per store. Producers (the six file ops) call
    :meth:`submit`; a single commit thread drains the queue every
    ``coalesce_window_ms`` (or sooner if ``coalesce_max_batch`` rows have
    accumulated), writes them all to the binlog with ONE durability
    barrier, then updates HEAD atomically.

    Crash safety:

    * If the writer is SIGKILL'd between writev and durable_sync, the tail
      frame's CRC will not match on next open; :meth:`_recover_tail`
      truncates it. The chain is intact from the start through the last
      durably-synced batch.
    * If the writer is SIGKILL'd between durable_sync and HEAD update,
      readers see the old HEAD value — they observe a consistent
      pre-commit view. The next writer's :meth:`_recover_tail` advances
      HEAD on next open.
    * If the HEAD update partially completes, the atomic ``tmp + sync +
      rename + parent-sync`` pattern ensures the old HEAD survives.
    """

    _mmr = None  # populated in open() iff cfg.enable_mmr

    def __init__(
        self,
        store: Path,
        *,
        config: WriterConfig | None = None,
    ):
        """Create a new Writer.

        Parameters
        ----------
        store:
            Path to ``.cyberos/memory/store/`` root.
        config:
            Tuning knobs; defaults are production-safe.
        """
        self.store = store
        self.cfg = config or WriterConfig()
        self._lock = StoreLock(store / ".lock")
        self._pending: deque[_PendingRecord] = deque()
        self._cv = threading.Condition()
        self._stop = threading.Event()
        self._closed = False
        self._binlog_fd: int | None = None
        self._binlog_path = store / "audit" / "current.binlog"
        self._head_path = store / "HEAD"
        self._last_chain: str = _GENESIS_CHAIN
        self._last_seq: int = 0
        self._fatal: BaseException | None = None
        self._thread = threading.Thread(
            target=self._commit_loop, daemon=True, name="cyberos-commit",
        )

    # -- public API ----------------------------------------------------------

    def open(self) -> None:
        """Acquire the store lock and start the commit thread."""
        self._lock.acquire_exclusive()
        self._binlog_path.parent.mkdir(parents=True, exist_ok=True)
        flags = (
            os.O_WRONLY
            | os.O_APPEND
            | os.O_CREAT
            | getattr(os, "O_CLOEXEC", 0)
        )
        self._binlog_fd = os.open(self._binlog_path, flags, 0o600)
        self._recover_tail()
        # PROPOSAL.md P2 Stage 1: build MMR alongside the chain.
        # auto_persist=False so we batch peaks.bin rewrites with the
        # writer's group commit (one persist per batch, not per leaf).
        if self.cfg.enable_mmr:
            from cyberos.core.mmr import OnDiskMMR  # noqa: WPS433 — lazy
            self._mmr = OnDiskMMR(self.store, auto_persist=False)
        self._thread.start()

    def close(self) -> None:
        """Stop the commit thread, flush pending, release the lock."""
        if self._closed:
            return
        self._closed = True
        self._stop.set()
        with self._cv:
            self._cv.notify_all()
        self._thread.join(timeout=_THREAD_JOIN_TIMEOUT_S)
        if self._binlog_fd is not None:
            try:
                os.close(self._binlog_fd)
            finally:
                self._binlog_fd = None
        self._lock.release()

    def __enter__(self) -> "Writer":
        self.open()
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()

    @property
    def head_seq(self) -> int:
        """Last durably-committed sequence number. Not lock-protected."""
        return self._last_seq

    def submit(self, rec: AuditRecord) -> int:
        """Submit one audit record. Blocks until durably committed.

        Returns the assigned sequence number. Thread-safe: any number of
        producer threads may call this concurrently; the commit thread
        serialises them into a single batched fsync.

        Raises
        ------
        WriterClosedError
            ``close()`` was called before this submit.
        CommitFailed
            The commit thread encountered an unrecoverable error. The
            writer is poisoned; the process must restart.
        TimeoutError
            The commit thread did not flush within ``flush_timeout_s``.
        """
        if self._closed:
            raise WriterClosedError("Writer is closed")
        if self._fatal is not None:
            raise CommitFailed("writer is poisoned") from self._fatal

        # Clear any caller-supplied chain/seq fields — only the commit
        # thread is allowed to assign these.
        pending = _PendingRecord(
            record=msgspec.structs.replace(rec, chain="", ts_ns=0),
            done=threading.Event(),
            seq=[-1],
            error=[None],
        )
        with self._cv:
            self._pending.append(pending)
            self._cv.notify()

        if not pending.done.wait(timeout=self.cfg.flush_timeout_s):
            raise TimeoutError(
                f"commit thread did not flush within {self.cfg.flush_timeout_s}s"
            )
        if pending.error[0] is not None:
            raise CommitFailed("flush failed") from pending.error[0]
        return pending.seq[0]

    def checkpoint(self) -> None:
        """Force a power-loss-safe sync of the ledger.

        Use sparingly — this is the expensive ``F_FULLFSYNC`` / true
        device-flush barrier. The standard use is when sealing a Merkle
        checkpoint (audit report §7.6). Per-batch flushes use the cheaper
        barrier mode set in ``cfg.fsync_strategy``.
        """
        if self._binlog_fd is None:
            return
        durable_sync(self._binlog_fd, strategy=self.cfg.checkpoint_strategy)
        durable_dir_sync(self._binlog_path.parent)

    # -- commit loop ---------------------------------------------------------

    def _commit_loop(self) -> None:
        window_s = self.cfg.coalesce_window_ms / 1000.0
        max_batch = self.cfg.coalesce_max_batch
        while not self._stop.is_set():
            batch: list[_PendingRecord] = []
            with self._cv:
                if not self._pending:
                    self._cv.wait(timeout=window_s)
                while self._pending and len(batch) < max_batch:
                    batch.append(self._pending.popleft())
            if not batch:
                continue
            try:
                self._flush_batch(batch)
            except BaseException as exc:  # noqa: BLE001 — we re-raise per-record
                self._fatal = exc
                for pending in batch:
                    pending.error[0] = exc
                    pending.done.set()
                # Drain remaining pending submitters; they would otherwise
                # hang on the timeout. Writer is poisoned at this point.
                with self._cv:
                    while self._pending:
                        stragglers = self._pending.popleft()
                        stragglers.error[0] = exc
                        stragglers.done.set()
                return

        # Stop event — flush any remaining queue under lock.
        with self._cv:
            remaining = list(self._pending)
            self._pending.clear()
        if remaining:
            try:
                self._flush_batch(remaining)
            except BaseException as exc:  # noqa: BLE001
                for pending in remaining:
                    pending.error[0] = exc
                    pending.done.set()

    def _flush_batch(self, batch: list["_PendingRecord"]) -> None:
        """Write N records as ONE writev + ONE durability barrier + ONE HEAD update."""
        assert self._binlog_fd is not None
        frames: list[bytes] = []
        first_seq = self._last_seq + 1
        now_ns = time.time_ns()

        for pending in batch:
            self._last_seq += 1
            # Stamp ts_ns at flush time so the on-disk ordering matches
            # the timestamp ordering — important for human reconciliation.
            rec_stamped = msgspec.structs.replace(
                pending.record,
                ts_ns=now_ns,
                prev_chain=self._last_chain,
                chain="",
            )
            chain = _chain_hash(self._last_chain, rec_stamped)
            final_rec = msgspec.structs.replace(rec_stamped, chain=chain)
            payload = _canonical(final_rec)
            hdr = _FRAME_HDR.pack(
                len(payload), _crc32c(payload), self._last_seq, now_ns,
            )
            frames.append(hdr + payload)
            self._last_chain = chain

        # ONE syscall for the write...
        if hasattr(os, "writev"):
            os.writev(self._binlog_fd, frames)
        else:  # pragma: no cover — Windows fallback
            os.write(self._binlog_fd, b"".join(frames))

        # ...ONE syscall for the durability barrier...
        durable_sync(self._binlog_fd, strategy=self.cfg.fsync_strategy)

        # ...then atomically publish the new HEAD value.
        self._publish_head(self._last_seq)

        # PROPOSAL.md P2 Stage 1: feed every record's canonical payload
        # into the MMR. Additive — failures here MUST NOT poison the
        # writer (the chain is still durable; MMR is a cross-check).
        # Append all leaves first, then persist peaks.bin ONCE per batch
        # (auto_persist=False on the OnDiskMMR opened in open()).
        if self._mmr is not None:
            try:
                for frame in frames:
                    payload_len = _FRAME_HDR.unpack_from(frame, 0)[0]
                    payload = frame[_FRAME_HDR.size:_FRAME_HDR.size + payload_len]
                    self._mmr.append_leaf(payload)
                self._mmr.persist()
            except Exception as exc:  # noqa: BLE001 — never crash the writer
                sys.stderr.write(
                    f"[cyberos.writer] MMR append failed (chain still durable): {exc!r}\n"
                )

        # Wake all submitters in batch order so seqs are returned correctly.
        for offset, pending in enumerate(batch):
            pending.seq[0] = first_seq + offset
            pending.done.set()

    def _publish_head(self, seq: int) -> None:
        """Atomic seq-counter publish for the seqlock reader pattern.

        Readers observe HEAD via mmap and retry if HEAD advanced between
        snapshot-before and snapshot-after their read. See
        :class:`cyberos.core.reader.Reader`.
        """
        tmp = self._head_path.with_suffix(".tmp")
        flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
        fd = os.open(tmp, flags, 0o600)
        try:
            os.write(fd, _HEAD_FMT.pack(seq))
            # HEAD is fixed-size, no metadata grows; fdatasync sufficient.
            if hasattr(os, "fdatasync"):
                os.fdatasync(fd)
            else:  # pragma: no cover
                os.fsync(fd)
        finally:
            os.close(fd)
        os.replace(tmp, self._head_path)
        durable_dir_sync(self._head_path.parent)

    # -- recovery ------------------------------------------------------------

    def _recover_tail(self) -> None:
        """Validate the binlog tail on open; truncate corrupt frame.

        Bitcask rule: "Recovery operations need to review only the last
        record or two written and verify CRC data"
        (riak.com/assets/bitcask-intro.pdf §Recovery).

        After truncation, HEAD is rewritten if it was ahead of the last
        intact frame — covers the SIGKILL-between-write-and-HEAD case.
        """
        from cyberos.core.walker import MmapWalker  # local import — heavy

        if not self._binlog_path.exists() or self._binlog_path.stat().st_size == 0:
            self._last_chain = _GENESIS_CHAIN
            self._last_seq = 0
            return

        good_end = 0
        last_chain = _GENESIS_CHAIN
        last_seq = 0
        with MmapWalker(self._binlog_path) as walker:
            for offset, rec in walker.iter_records():
                last_chain = rec.chain
                last_seq = int(rec.extra.get("_seq", last_seq + 1))
                good_end = offset + walker.frame_size_at(offset)

        assert self._binlog_fd is not None
        size = os.fstat(self._binlog_fd).st_size
        if size != good_end:
            os.ftruncate(self._binlog_fd, good_end)
            if hasattr(os, "fdatasync"):
                os.fdatasync(self._binlog_fd)
            else:  # pragma: no cover
                os.fsync(self._binlog_fd)

        self._last_chain = last_chain
        self._last_seq = last_seq

        # Re-publish HEAD if it advanced past our intact tail (SIGKILL
        # between durable_sync and HEAD update would create the reverse,
        # which we also fix — bring HEAD forward to last_seq).
        try:
            with open(self._head_path, "rb") as fh:
                cur_head_bytes = fh.read(8)
            cur_head = _HEAD_FMT.unpack(cur_head_bytes)[0] if len(cur_head_bytes) == 8 else 0
        except FileNotFoundError:
            cur_head = 0
        if cur_head != last_seq:
            self._publish_head(last_seq)


@dataclass
class _PendingRecord:
    """One slot in the producer→commit queue."""

    record: AuditRecord
    done: threading.Event
    seq: list[int]
    error: list[BaseException | None]


__all__ = [
    "AuditRecord",
    "Writer",
    "WriterConfig",
    "WriterClosedError",
    "CommitFailed",
    "crc_implementation",
    "_FRAME_HDR",  # exported for walker; underscore signals "internal protocol"
    "_HEAD_FMT",
    "_GENESIS_CHAIN",
    "_canonical",
    "_chain_hash",
    "_crc32c",
]
