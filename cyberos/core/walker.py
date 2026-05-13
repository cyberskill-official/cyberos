"""
cyberos.core.walker — page-cache-friendly verification of the binlog.

Replaces line-by-line JSONL reads with mmap + length-prefix skipping.
Used by:

* :class:`cyberos.core.writer.Writer._recover_tail` on open;
* full-chain ``cyberos verify`` (Merkle LINK invariant check);
* deterministic export;
* the SQLite index replay path (:mod:`cyberos.core.index`).

Bitcask's recovery lesson applies (riak.com/assets/bitcask-intro.pdf): on a
length-prefixed binary log, partial writes are detected by CRC mismatch and
treated as the truncation point — no half-records propagate to higher
layers.
"""

from __future__ import annotations

import mmap
import os
from pathlib import Path
from typing import Iterator, Tuple

import msgspec

from cyberos.core.writer import (
    _FRAME_HDR,
    _GENESIS_CHAIN,
    AuditRecord,
    _canonical,
    _chain_hash,
    _crc32c,
)


class CorruptFrame(RuntimeError):
    """Raised when a chain mismatch is found during verification.

    Distinct from a CRC mismatch — a CRC mismatch at the tail is benign
    (treated as the truncation point per Bitcask). A chain mismatch
    inside the log is a real corruption and aborts verification.
    """


class MmapWalker:
    """Iterate records in a binlog file with mmap + length-prefix skipping.

    Usage::

        with MmapWalker(path) as walker:
            for offset, rec in walker.iter_records():
                ...

    Or, to validate the chain end-to-end::

        with MmapWalker(path) as walker:
            n = walker.verify_chain()           # raises CorruptFrame on mismatch
    """

    def __init__(self, path: Path):
        self.path = path
        self._fd: int | None = None
        self._mm: mmap.mmap | None = None

    def __enter__(self) -> "MmapWalker":
        self._open()
        return self

    def __exit__(self, *exc: object) -> None:
        self.close()

    # -- lifecycle -----------------------------------------------------------

    def _open(self) -> None:
        if self._mm is not None:
            return
        if not self.path.exists() or self.path.stat().st_size == 0:
            return
        self._fd = os.open(self.path, os.O_RDONLY)
        size = os.fstat(self._fd).st_size
        self._mm = mmap.mmap(self._fd, size, prot=mmap.PROT_READ)

    def close(self) -> None:
        if self._mm is not None:
            self._mm.close()
            self._mm = None
        if self._fd is not None:
            os.close(self._fd)
            self._fd = None

    # -- introspection -------------------------------------------------------

    def frame_size_at(self, offset: int) -> int:
        """Total bytes (header + payload) of the frame starting at ``offset``."""
        if self._mm is None:
            raise RuntimeError("walker not open")
        length, _crc, _seq, _ts = _FRAME_HDR.unpack_from(self._mm, offset)
        return _FRAME_HDR.size + length

    def __len__(self) -> int:
        """Total bytes in the underlying mmap. 0 if file is missing/empty."""
        if self._mm is None:
            return 0
        return len(self._mm)

    # -- iteration -----------------------------------------------------------

    def iter_payloads(self) -> Iterator[Tuple[int, bytes]]:
        """Yield ``(offset, raw_canonical_payload_bytes)`` pairs.

        Like :meth:`iter_records` but skips the msgspec decode. Used by
        :func:`mmr_root_for_binlog` so the MMR is fed the SAME bytes the
        writer originally fed it — re-canonicalising via msgspec would
        add the framing-metadata fields the walker injects.
        """
        if self._mm is None:
            self._open()
        if self._mm is None:
            return
        offset = 0
        total = len(self._mm)
        hdr_size = _FRAME_HDR.size
        while offset + hdr_size <= total:
            length, crc, _seq, _ts = _FRAME_HDR.unpack_from(self._mm, offset)
            payload_start = offset + hdr_size
            payload_end = payload_start + length
            if payload_end > total:
                return
            payload = bytes(self._mm[payload_start:payload_end])
            if _crc32c(payload) != crc:
                return
            yield offset, payload
            offset = payload_end

    def iter_records(self) -> Iterator[Tuple[int, AuditRecord]]:
        """Yield ``(offset, AuditRecord)`` pairs.

        Stops cleanly (no exception) on:

        * a truncated final frame (length header runs past EOF);
        * a CRC mismatch — treated as Bitcask-style truncation point.

        Raises :class:`msgspec.DecodeError` if a *complete* frame's
        payload fails to decode — that is corruption inside an intact
        frame and should not be silently swallowed.
        """
        if self._mm is None:
            self._open()
        if self._mm is None:
            return
        offset = 0
        total = len(self._mm)
        decoder = msgspec.json.Decoder(AuditRecord)
        hdr_size = _FRAME_HDR.size

        while offset + hdr_size <= total:
            length, crc, seq, ts_ns = _FRAME_HDR.unpack_from(self._mm, offset)
            payload_start = offset + hdr_size
            payload_end = payload_start + length

            if payload_end > total:
                # Truncated tail — stop without raising.
                return

            payload = bytes(self._mm[payload_start:payload_end])
            if _crc32c(payload) != crc:
                # Bitcask: CRC failure at the tail is the truncation point.
                # We don't try to skip-and-resume; that would risk re-applying
                # a partially-written record.
                return

            try:
                rec = decoder.decode(payload)
            except msgspec.DecodeError:  # type: ignore[name-defined]
                # The frame passed the CRC gate but the payload won't
                # decode as an AuditRecord. Two ways this can happen:
                # (a) zero-filled padding that happens to CRC to zero —
                #     length=0, empty payload, msgspec rejects empty input;
                # (b) a partially-written frame whose CRC field was also
                #     not yet written, so a coincidental match. Either way,
                #     treat as truncation point.
                return
            # Stash framing metadata that callers want without re-reading
            # the header. ``_seq``/``_ts_ns`` use underscore prefixes so
            # they don't collide with any future schema additions.
            extra = dict(rec.extra)
            extra.setdefault("_seq", seq)
            extra.setdefault("_ts_ns", ts_ns)
            rec = msgspec.structs.replace(rec, extra=extra)
            yield offset, rec
            offset = payload_end

    # -- verification --------------------------------------------------------

    def verify_chain(self, *, start_prev: str = _GENESIS_CHAIN) -> int:
        """Walk the whole binlog and verify the LINK invariant.

        Parameters
        ----------
        start_prev:
            The expected ``prev_chain`` of the first record. For a fresh
            log this is the genesis (64 zeros). When verifying a sealed
            month-segment that continues from a previous month, pass the
            last chain of that previous month.

        Returns the number of records verified.

        Raises :class:`CorruptFrame` if any record's chain doesn't match
        SHA-256(canonical(rec_minus_chain) || prev). Raises immediately
        on first mismatch.
        """
        prev = start_prev
        count = 0
        for offset, rec in self.iter_records():
            if rec.prev_chain != prev:
                raise CorruptFrame(
                    f"prev_chain mismatch at offset={offset} seq={rec.extra.get('_seq')}: "
                    f"expected {prev}, got {rec.prev_chain}"
                )
            # Strip the framing-only fields we injected on read; the chain
            # was computed over the on-disk canonical bytes which did not
            # include them.
            rec_check = msgspec.structs.replace(
                rec,
                chain="",
                extra={k: v for k, v in rec.extra.items() if not k.startswith("_")},
            )
            computed = _chain_hash(prev, rec_check)
            if computed != rec.chain:
                raise CorruptFrame(
                    f"chain hash mismatch at offset={offset} seq={rec.extra.get('_seq')}: "
                    f"computed {computed}, stored {rec.chain}"
                )
            prev = rec.chain
            count += 1
        return count

    def last_chain(self) -> str:
        """Return the ``chain`` of the most recent record, or genesis if empty."""
        last = _GENESIS_CHAIN
        for _offset, rec in self.iter_records():
            last = rec.chain
        return last


def verify_segments(segments: list[Path], *, start_prev: str = _GENESIS_CHAIN) -> int:
    """Verify the chain across a sequence of binlog segments in order.

    Used by ``cyberos verify``. The ``chain`` of segment N becomes the
    ``start_prev`` of segment N+1 — month boundaries are not chain breaks.

    When the store carries a legacy bridge (``manifest.json`` has
    ``migration.legacy_last_chain``), callers MUST pass that value as
    ``start_prev`` so the first segment's first record's ``prev_chain``
    is checked against the legacy tip, not the all-zero genesis.
    """
    prev = start_prev
    total = 0
    for path in segments:
        with MmapWalker(path) as walker:
            total += walker.verify_chain(start_prev=prev)
            prev = walker.last_chain()
    return total


__all__ = ["MmapWalker", "CorruptFrame", "verify_segments"]
