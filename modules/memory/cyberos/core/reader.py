"""
cyberos.core.reader — lock-free read path for memory files.

The reader NEVER takes a flock for ``view`` operations. Instead it uses a
seqlock pattern over the ``HEAD`` file:

  1. Snapshot HEAD (single 8-byte read of a write-once atomic-rename file).
  2. ``stat`` the target memory file (record mtime + size).
  3. mmap and copy the file bytes.
  4. Re-stat the file and re-read HEAD.
  5. If both observations match → return the parsed result.
     If HEAD advanced AND mtime changed → retry (writer overlapped).

This is the classic seqlock pattern and is conceptually equivalent to
LMDB's "wait-free readers" model:

    "readers run with no locks; writers cannot block readers, and readers
     don't block writers"
    — en.wikipedia.org/wiki/Lightning_Memory-Mapped_Database

Soundness sketch — the writer always:

* writes the new memory file atomically via ``tmp + sync + rename``;
* ``durable_sync``s the binlog;
* bumps HEAD atomically (``tmp + sync + rename + parent-sync``).

So if a reader observes the same HEAD before AND after its read AND the
file's mtime didn't change, it observed a consistent view: either the
pre-write file or the post-write file, never a torn one. If either
observation changed, the read may be on a stale mmap region — retry.
"""

from __future__ import annotations

import mmap
import os
import struct
from pathlib import Path
from typing import Final, Tuple

from cyberos.core.frontmatter import (
    Frontmatter,
    looks_like_yaml,
    parse,
    parse_legacy_yaml,
)

_HEAD_FMT: Final[struct.Struct] = struct.Struct("<Q")
_DEFAULT_MAX_RETRIES: Final[int] = 4


class ReaderUnstable(RuntimeError):
    """Raised when the reader cannot stabilise within ``max_retries``.

    In practice this can only happen if a writer is committing faster
    than the reader can complete a single mmap+stat — extremely rare.
    The caller should back off briefly and try again.
    """


class Reader:
    """Lock-free reader for memory files.

    Cheap to instantiate (no fds opened). Safe to use from many threads;
    each read takes its own ephemeral fds and mmaps.

    Parameters
    ----------
    store:
        Path to the ``.cyberos/memory/store/`` directory root.
    """

    def __init__(self, store: Path):
        self.store = store
        self._head_path = store / "HEAD"

    # -- public API ----------------------------------------------------------

    def view(
        self,
        rel_path: str,
        *,
        max_retries: int = _DEFAULT_MAX_RETRIES,
    ) -> Tuple[Frontmatter, bytes]:
        """Lock-free read of a memory file.

        Returns ``(frontmatter, body_bytes)``. Frontmatter is parsed as
        JSON (the new format); falls back to YAML for legacy files
        during the migration window.

        Retries on detected writer overlap up to ``max_retries`` times;
        raises :class:`ReaderUnstable` after that.
        """
        abs_path = self.store / rel_path
        for _attempt in range(max_retries):
            h0 = self._read_head()
            try:
                st = os.stat(abs_path)
            except FileNotFoundError:
                # Tombstoned or never-existed; caller must check audit
                # log to disambiguate. We surface the OS exception so
                # the difference between "missing" and "stale read" is
                # never silently coerced.
                raise

            raw = self._read_bytes(abs_path, st.st_size)

            # Re-observe; if either stat or HEAD changed, retry.
            try:
                st2 = os.stat(abs_path)
            except FileNotFoundError:
                continue
            h1 = self._read_head()
            if h0 == h1 and st.st_mtime_ns == st2.st_mtime_ns and st.st_size == st2.st_size:
                if looks_like_yaml(raw):
                    return parse_legacy_yaml(raw)
                return parse(raw)
            # else: writer overlapped — retry.

        raise ReaderUnstable(
            f"could not stabilise read of {rel_path!r} after {max_retries} retries"
        )

    def view_bytes(
        self,
        rel_path: str,
        *,
        max_retries: int = _DEFAULT_MAX_RETRIES,
    ) -> bytes:
        """Like :meth:`view`, but returns the raw file bytes unparsed.

        Useful for non-memory files (e.g. ``manifest.json``) where the
        seqlock guarantee matters but frontmatter parsing doesn't.
        """
        abs_path = self.store / rel_path
        for _attempt in range(max_retries):
            h0 = self._read_head()
            st = os.stat(abs_path)
            raw = self._read_bytes(abs_path, st.st_size)
            st2 = os.stat(abs_path)
            h1 = self._read_head()
            if h0 == h1 and st.st_mtime_ns == st2.st_mtime_ns and st.st_size == st2.st_size:
                return raw
        raise ReaderUnstable(
            f"could not stabilise read of {rel_path!r} after {max_retries} retries"
        )

    # -- internals -----------------------------------------------------------

    def _read_head(self) -> int:
        """Read HEAD's 8-byte sequence counter, or 0 if missing.

        The writer atomically renames a tmp file over HEAD, so this read
        is guaranteed to see either the old or the new value, never a
        torn write. (Single 8-byte aligned write would also be torn-free
        on x86/ARM but we don't rely on that — atomic-rename is portable.)
        """
        try:
            with open(self._head_path, "rb") as fh:
                buf = fh.read(8)
        except FileNotFoundError:
            return 0
        if len(buf) != 8:
            return 0
        return _HEAD_FMT.unpack(buf)[0]

    @staticmethod
    def _read_bytes(path: Path, size: int) -> bytes:
        """mmap-and-copy a file's bytes. Falls back to plain read for size=0."""
        if size == 0:
            return b""
        fd = os.open(path, os.O_RDONLY)
        try:
            with mmap.mmap(fd, size, prot=mmap.PROT_READ) as mm:
                return bytes(mm)
        finally:
            os.close(fd)


__all__ = ["Reader", "ReaderUnstable"]
