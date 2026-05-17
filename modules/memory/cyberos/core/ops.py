"""
cyberos.core.ops — the canonical file ops.

Implements the protocol's file operations:

* :func:`view`   — read a memory file
* :func:`put`    — create-or-replace a memory file (idempotent)
* :func:`move`   — rename a memory file
* :func:`delete` — soft-delete (tombstone) or hard-purge a memory

Every op decomposes into exactly one :class:`AuditRecord` appended via
:class:`cyberos.core.writer.Writer`. No other call site is permitted to
append to the ledger — single-writer is a protocol invariant
(Tigerbeetle-style "single writer, deterministic order" applied to a
personal store).

Path-traversal guard, content gate, and frontmatter validation hooks
live here so these ops are the only ingress into the store.

These functions are thin and synchronous from the caller's POV — they
call :meth:`Writer.submit` which blocks until the record is durably
committed. For batch loads (migration, import) the caller is encouraged
to submit from multiple threads so the group-commit window can coalesce.
"""

from __future__ import annotations

import hashlib
import os
import re
from pathlib import Path
from typing import Final

from cyberos.core.fsync import durable_dir_sync, durable_sync
from cyberos.core.writer import AuditRecord, Writer

# Path-traversal guard: relative-only, no '..' components, no absolute paths,
# no leading whitespace or NUL. Mirrors §4.1 of the legacy AGENTS.md.
_REL_PATH_RE: Final[re.Pattern[str]] = re.compile(
    r"^[A-Za-z0-9_][A-Za-z0-9_./\-]*$",
)
_FORBIDDEN_SEGMENTS: Final[frozenset[str]] = frozenset({"..", ".", ""})

# Max file size we accept on create / str_replace. Larger memories should
# decompose into multiple files; the cap prevents accidental "ingest the
# whole codebase into one file" mishaps.
_MAX_BYTES: Final[int] = 1 * 1024 * 1024  # 1 MiB


class PathTraversal(ValueError):
    """The relative path failed §4.1 path-traversal guard."""


class ContentTooLarge(ValueError):
    """The bytes argument exceeds the per-file size cap."""


class NotFound(FileNotFoundError):
    """The target memory file does not exist (or was tombstoned)."""


def _check_rel_path(rel_path: str) -> None:
    if not rel_path:
        raise PathTraversal("empty rel_path")
    if rel_path.startswith(("/", "\\")):
        raise PathTraversal(f"absolute path rejected: {rel_path!r}")
    parts = rel_path.replace("\\", "/").split("/")
    for part in parts:
        if part in _FORBIDDEN_SEGMENTS:
            raise PathTraversal(f"forbidden segment {part!r} in {rel_path!r}")
        if "\x00" in part:
            raise PathTraversal(f"NUL byte in path component {part!r}")
    if not _REL_PATH_RE.match(rel_path.replace("\\", "/")):
        raise PathTraversal(f"path failed regex check: {rel_path!r}")


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _atomic_write(target: Path, data: bytes) -> None:
    """Atomic-write ``data`` to ``target`` using tmp+sync+rename+parent-sync.

    Mirrors the lwn.net/Articles/457667 pattern with platform-correct
    durability via :mod:`cyberos.core.fsync`.
    """
    target.parent.mkdir(parents=True, exist_ok=True)
    tmp = target.with_name(target.name + ".tmp")
    flags = os.O_WRONLY | os.O_CREAT | os.O_TRUNC | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(tmp, flags, 0o600)
    try:
        os.write(fd, data)
        durable_sync(fd)  # fbarrier on Darwin, fdatasync on Linux
    finally:
        os.close(fd)
    os.replace(tmp, target)
    durable_dir_sync(target.parent)


def _read_bytes(target: Path) -> bytes:
    if not target.is_file():
        raise NotFound(str(target))
    return target.read_bytes()


# -- the six ops ------------------------------------------------------------


def view(
    writer: Writer,
    rel_path: str,
    *,
    actor: str,
    audit: bool = False,
) -> bytes:
    """Read a memory file. Per AGENTS.md v2 §3.2, ``view`` is implicit
    on read and does NOT emit an audit row by default.

    Set ``audit=True`` to opt in to the legacy v1 behaviour (one
    ``op="view"`` row appended per read). Used for high-sensitivity
    paths where every read MUST be traced.
    """
    _check_rel_path(rel_path)
    abs_path = writer.store / rel_path
    data = _read_bytes(abs_path)
    if audit:
        writer.submit(
            AuditRecord(
                op="view",
                path=rel_path,
                actor=actor,
                content_sha256=_sha256(data),
            )
        )
    return data


def put(
    writer: Writer,
    rel_path: str,
    body: bytes,
    *,
    actor: str,
    kind: str = "unknown",
    extra: dict | None = None,
) -> int:
    """Canonical v2 op — create-or-replace a memory file.

    Per AGENTS.md §3.1: ``put`` is one of three canonical operations
    (alongside ``move`` and ``delete``). Idempotent given identical args.
    Content-addressed: the on-disk effect is identical regardless of
    whether ``rel_path`` previously existed.

    Audit row uses ``op="put"``.
    """
    _check_rel_path(rel_path)
    if len(body) > _MAX_BYTES:
        raise ContentTooLarge(f"{len(body)} > {_MAX_BYTES}")
    abs_path = writer.store / rel_path
    existed = abs_path.exists()
    before_sha = _sha256(abs_path.read_bytes()) if existed else None
    _atomic_write(abs_path, body)

    rec_extra: dict[str, object] = {"kind": kind}
    if existed and before_sha is not None:
        rec_extra["before_sha256"] = before_sha
    if extra:
        rec_extra.update(extra)
    return writer.submit(
        AuditRecord(
            op="put",
            path=rel_path,
            actor=actor,
            content_sha256=_sha256(body),
            extra=rec_extra,
        )
    )


def move(
    writer: Writer,
    src_rel: str,
    dst_rel: str,
    *,
    actor: str,
) -> int:
    """Canonical op — rename within ``<memory-root>/``.

    Per AGENTS.md §3.1, ``move`` preserves the content hash. Implements
    POSIX ``rename(2)``; fails if ``dst_rel`` already exists. Audit row
    uses ``op="move"``.
    """
    _check_rel_path(src_rel)
    _check_rel_path(dst_rel)
    if src_rel == dst_rel:
        raise ValueError("src_rel and dst_rel are identical")
    src = writer.store / src_rel
    dst = writer.store / dst_rel
    if not src.is_file():
        raise NotFound(str(src))
    if dst.exists():
        raise FileExistsError(str(dst))
    dst.parent.mkdir(parents=True, exist_ok=True)
    data = src.read_bytes()
    os.rename(src, dst)
    durable_dir_sync(src.parent)
    durable_dir_sync(dst.parent)
    return writer.submit(
        AuditRecord(
            op="move",
            path=src_rel,
            actor=actor,
            content_sha256=_sha256(data),
            extra={"to": dst_rel},
        )
    )


class PurgeRefused(PermissionError):
    """Raised when delete(mode='purge') is called without the gate phrase."""


# Magic phrase per AGENTS.md v2 §16.2. The user provides this through the
# CLI flag, an env var, or by passing it directly to ops.delete(...).
# Any other value MUST refuse the purge.
_PURGE_MAGIC_PHRASE: Final[str] = "APPROVE protocol change P4 §3.6"


def delete(
    writer: Writer,
    rel_path: str,
    *,
    actor: str,
    mode: str = "tombstone",
    reason: str | None = None,
    approval_phrase: str | None = None,
) -> int:
    """Delete a memory file. Two modes (AGENTS.md v2 §3.5–§3.6).

    Parameters
    ----------
    mode:
        ``"tombstone"`` (default) — append an audit row marking the
        file deleted; the body is preserved on disk so the soft-delete
        can be audited and (with consent) reversed.

        ``"purge"`` — GDPR Article 17 compliance. The body is
        overwritten with a fixed-length redaction marker (NOT zeros, so
        the redaction is visible). The audit row carries the original
        body's content_sha256 and the redaction reason; the *fact* of
        purge is itself a ledger leaf and MUST NOT be erasable. Gated
        by ``approval_phrase`` exactly equal to the magic phrase from
        the AGENTS.md v2 §16.2 grammar.

    reason:
        Free-form text. REQUIRED for purge; OPTIONAL for tombstone.

    approval_phrase:
        Required for purge. Must equal :data:`_PURGE_MAGIC_PHRASE`.
        Read from the ``CYBEROS_PURGE_APPROVAL`` env var if omitted.

    Raises
    ------
    PurgeRefused
        ``mode="purge"`` without a valid approval phrase, or without a
        reason.
    """
    _check_rel_path(rel_path)
    if mode not in ("tombstone", "purge"):
        raise ValueError(f"unknown delete mode: {mode!r}")

    abs_path = writer.store / rel_path
    if not abs_path.is_file():
        raise NotFound(str(abs_path))
    data = abs_path.read_bytes()
    original_sha = _sha256(data)

    if mode == "tombstone":
        return writer.submit(
            AuditRecord(
                op="delete",
                path=rel_path,
                actor=actor,
                content_sha256=original_sha,
                extra={"mode": "tombstone", "reason": reason or ""},
            )
        )

    # mode == "purge"
    if not reason or not reason.strip():
        raise PurgeRefused(
            "delete(mode='purge') requires a non-empty reason"
        )
    phrase = approval_phrase or os.environ.get("CYBEROS_PURGE_APPROVAL", "")
    if phrase != _PURGE_MAGIC_PHRASE:
        raise PurgeRefused(
            "delete(mode='purge') requires approval_phrase exactly equal to "
            f"{_PURGE_MAGIC_PHRASE!r}; got {phrase!r}. Set via the "
            "CYBEROS_PURGE_APPROVAL env var or --approval-phrase CLI flag."
        )

    # Overwrite the body with a fixed-length redaction marker. We KEEP
    # the file (don't unlink) so the path's existence + size remain as
    # forensic evidence; only the bytes are gone. Marker is ASCII so
    # `cat`-ing the file is obvious to a human.
    marker = (
        b"<<<CYBEROS:PURGED " + original_sha.encode() + b" "
        + str(writer.head_seq + 1).encode() + b">>>\n"
    )
    _atomic_write(abs_path, marker)

    return writer.submit(
        AuditRecord(
            op="delete",
            path=rel_path,
            actor=actor,
            content_sha256=original_sha,
            extra={
                "mode": "purge",
                "reason": reason,
                "redacted_sha256": original_sha,
                "purge_marker_size": len(marker),
            },
        )
    )


__all__ = [
    # canonical ops (AGENTS.md §3.1)
    "view",
    "put",
    "move",
    "delete",
    # exceptions
    "PathTraversal",
    "ContentTooLarge",
    "NotFound",
    "PurgeRefused",
]
