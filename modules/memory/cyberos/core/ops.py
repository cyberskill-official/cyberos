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


class AclDenied(PermissionError):
    """Per AGENTS.md §14.4 / FR-MEMORY-117 — the active actor isn't permitted
    to write to this subtree under the governing STORE.yaml. Raised AFTER
    the `memory.acl_denied` aux row is emitted so operators have an audit
    trail of the attempt."""


class _PutIfResultMixin:
    """Lightweight namespace for the put_if result type — defined as a
    plain dataclass below to avoid pyclass import overhead in cold-CLI.
    """


from dataclasses import dataclass as _dataclass


@_dataclass(frozen=True)
class PutIfResult:
    """Result of a put_if op (FR-MEMORY-118 §1 #9, AGENTS.md §3.1.5).

    Attributes
    ----------
    outcome
        ``"written"`` on success; ``"rejected"`` on precondition mismatch
        or ACL refusal.
    reason
        Populated only on rejection. One of ``"precondition_failed"`` or
        ``"acl_denied"``.
    expected
        The precondition hash supplied by the caller (or ``None`` for the
        create-only variant).
    actual
        The on-disk body hash at check time, or the literal string
        ``"<absent>"`` when no file exists at the path.
    committed_seq
        The audit-row seq of the `put` row on success; ``None`` on rejection.
    """

    outcome: str  # "written" | "rejected"
    reason: "str | None" = None
    expected: "str | None" = None
    actual: "str | None" = None
    committed_seq: "int | None" = None


def _acl_check(writer: Writer, rel_path: str, actor: str, attempt_kind: str) -> bool:
    """Run the FR-MEMORY-117 ACL gate.

    Emits a `memory.acl_denied` aux row on every refusal (and on WARN-ONLY
    proceed-with-log). Returns True if the write may proceed; False on
    hard refusal.
    """
    from cyberos.core.store_acl import check_write  # noqa: WPS433

    res = check_write(writer.store, rel_path, actor)
    # Emit aux row whenever the resolved mode isn't read-write OR when
    # an explicit yaml file matched + denied (covers warn-only too).
    if res.reason is not None:
        writer.submit(
            AuditRecord(
                op="memory.acl_denied",
                path=rel_path,
                actor=actor,
                extra=res.to_aux_payload(actor, rel_path, attempt_kind),
            )
        )
    return res.allowed


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

    Audit row uses ``op="put"``. Per AGENTS.md §14.4 / FR-MEMORY-117, the
    write is also gated by the nearest `STORE.yaml` ACL — a denied write
    emits a `memory.acl_denied` aux row and refuses the put (or, in
    WARN-ONLY mode pre-§14.4-anchor, emits the row and proceeds).
    """
    seq, _record = put_with_record(
        writer,
        rel_path,
        body,
        actor=actor,
        kind=kind,
        extra=extra,
    )
    return seq


def put_with_record(
    writer: Writer,
    rel_path: str,
    body: bytes,
    *,
    actor: str,
    kind: str = "unknown",
    extra: dict | None = None,
):
    """Canonical v2 put, returning both seq and the committed audit record."""
    _check_rel_path(rel_path)
    if len(body) > _MAX_BYTES:
        raise ContentTooLarge(f"{len(body)} > {_MAX_BYTES}")

    # FR-MEMORY-117 ACL gate
    if not _acl_check(writer, rel_path, actor, "put"):
        # Denied + not warn-only → refuse the write; the aux row was
        # already emitted by _acl_check.
        raise AclDenied(f"ACL denies actor={actor!r} writing to {rel_path!r}")

    abs_path = writer.store / rel_path
    existed = abs_path.exists()
    before_sha = _sha256(abs_path.read_bytes()) if existed else None
    _atomic_write(abs_path, body)

    rec_extra: dict[str, object] = {"kind": kind}
    if existed and before_sha is not None:
        rec_extra["before_sha256"] = before_sha
    if extra:
        rec_extra.update(extra)
    return writer.submit_with_record(
        AuditRecord(
            op="put",
            path=rel_path,
            actor=actor,
            content_sha256=_sha256(body),
            extra=rec_extra,
        )
    )


def _has_section_3_1_put_if(store: Path) -> bool:
    """Anchor check for the FR-MEMORY-118 protocol amendment (§3.1 extension).

    Searches the same set of locations as the §7.7 and §14.4 checks. The
    extension must include `put_if` somewhere in the §3.1 canonical-op
    table — we look for the literal token in AGENTS.md.
    """
    import os
    candidates: list[Path] = [store / "AGENTS.md"]
    for parent in [store, *store.parents][:6]:
        candidates.append(parent / "modules" / "memory" / "AGENTS.md")
    for c in candidates:
        if c.exists():
            try:
                body = c.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue
            if "put_if" in body and "§3.1" in body:
                return True
    return False


_HEX_RE = __import__("re").compile(r"^[0-9a-f]{64}$")


def put_if(
    writer: Writer,
    rel_path: str,
    body: bytes,
    *,
    actor: str,
    precondition_body_hash: "str | None",
    kind: str = "unknown",
    extra: dict | None = None,
) -> PutIfResult:
    """Canonical op — content-conditional put.

    Per AGENTS.md §3.1 (extended by P21 / FR-MEMORY-118). The write
    proceeds only when the current on-disk body's SHA-256 matches
    ``precondition_body_hash`` (or, when the precondition is ``None``,
    the target MUST NOT currently exist — the create-only variant).

    Three rejection paths:

    * ``precondition_body_hash`` shape invalid (not 64-char lowercase
      hex, not ``None``) → ``ValueError``.
    * Protocol amendment §3.1 extension not anchored in AGENTS.md →
      ``ProtocolAmendmentMissing``.
    * ACL gate (FR-MEMORY-117) refuses → ``PutIfResult(outcome="rejected",
      reason="acl_denied")``.
    * Body-hash mismatch → ``PutIfResult(outcome="rejected",
      reason="precondition_failed", expected=..., actual=...)`` + a
      ``memory.precondition_failed`` aux audit row.

    On success the canonical row shape is identical to a regular ``put``
    (op=``"put"``, not ``"put_if"``) per §3.1.6, so downstream consumers
    (walker, doctor, dream, history) don't need to special-case the
    origin.
    """
    _check_rel_path(rel_path)
    if len(body) > _MAX_BYTES:
        raise ContentTooLarge(f"{len(body)} > {_MAX_BYTES}")

    # Shape check for precondition_body_hash
    if precondition_body_hash is not None:
        if not isinstance(precondition_body_hash, str):
            raise ValueError(
                f"precondition_body_hash must be 64-char lowercase hex or None; "
                f"got {type(precondition_body_hash).__name__}"
            )
        if not _HEX_RE.match(precondition_body_hash):
            raise ValueError(
                f"precondition_body_hash must be 64-char lowercase hex or None; "
                f"got {precondition_body_hash!r}"
            )

    # Protocol amendment anchor check (FR-MEMORY-118 §1 #12)
    if not _has_section_3_1_put_if(writer.store):
        from cyberos.core.dream.applier import ProtocolAmendmentMissing
        raise ProtocolAmendmentMissing(
            "AGENTS.md §3.1 extension (put_if) not anchored. Approve via:\n"
            "  APPROVE protocol change P21 §3.1\n"
            "and ensure AGENTS.md §3.1 includes `put_if` in the "
            "canonical-op table."
        )

    # FR-MEMORY-117 ACL gate (runs BEFORE the precondition check per
    # AGENTS.md §3.1.7 — policy is a stronger refusal than concurrency).
    if not _acl_check(writer, rel_path, actor, "put_if"):
        return PutIfResult(
            outcome="rejected",
            reason="acl_denied",
        )

    # Precondition check
    abs_path = writer.store / rel_path
    existed = abs_path.exists()
    actual_hash = _sha256(abs_path.read_bytes()) if existed else None

    if precondition_body_hash is None and existed:
        # "must not exist" path; current file exists → rejected
        _emit_precondition_failed(
            writer, rel_path, actor,
            expected=None, actual=actual_hash,
        )
        return PutIfResult(
            outcome="rejected",
            reason="precondition_failed",
            expected=None,
            actual=actual_hash,
        )
    if precondition_body_hash is not None and not existed:
        _emit_precondition_failed(
            writer, rel_path, actor,
            expected=precondition_body_hash, actual="<absent>",
        )
        return PutIfResult(
            outcome="rejected",
            reason="precondition_failed",
            expected=precondition_body_hash,
            actual="<absent>",
        )
    if precondition_body_hash is not None and actual_hash != precondition_body_hash:
        _emit_precondition_failed(
            writer, rel_path, actor,
            expected=precondition_body_hash, actual=actual_hash,
        )
        return PutIfResult(
            outcome="rejected",
            reason="precondition_failed",
            expected=precondition_body_hash,
            actual=actual_hash,
        )

    # All checks passed — proceed with the write. Emit a plain `put` row
    # (per §3.1.6 — indistinguishable from a regular put).
    before_sha = actual_hash  # same as before; reused for the `before_sha256` extra
    _atomic_write(abs_path, body)

    rec_extra: dict[str, object] = {"kind": kind}
    if existed and before_sha is not None:
        rec_extra["before_sha256"] = before_sha
    if extra:
        rec_extra.update(extra)
    committed_seq = writer.submit(
        AuditRecord(
            op="put",
            path=rel_path,
            actor=actor,
            content_sha256=_sha256(body),
            extra=rec_extra,
        )
    )
    return PutIfResult(
        outcome="written",
        committed_seq=committed_seq,
    )


def _emit_precondition_failed(
    writer: Writer,
    rel_path: str,
    actor: str,
    *,
    expected: "str | None",
    actual: "str | None",
) -> None:
    """Emit the AGENTS.md §3.1.5 / FR-MEMORY-118 §1 #7 aux row."""
    import time as _time
    from datetime import datetime, timezone

    writer.submit(
        AuditRecord(
            op="memory.precondition_failed",
            path=rel_path,
            actor=actor,
            extra={
                "actor": actor,
                "path": rel_path,
                "expected": expected,
                "actual": actual,
                "attempt_at": datetime.now(timezone.utc).isoformat(),
            },
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

    # FR-MEMORY-117 / AGENTS.md §14.4.5 — both src and dst must be writable
    if not _acl_check(writer, src_rel, actor, "move"):
        raise AclDenied(
            f"ACL denies actor={actor!r} moving from {src_rel!r} (src side)"
        )
    if not _acl_check(writer, dst_rel, actor, "move"):
        raise AclDenied(
            f"ACL denies actor={actor!r} moving to {dst_rel!r} (dst side)"
        )

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
    extra: dict | None = None,
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

    # FR-MEMORY-117 ACL gate
    if not _acl_check(writer, rel_path, actor, "delete"):
        raise AclDenied(f"ACL denies actor={actor!r} deleting {rel_path!r}")

    abs_path = writer.store / rel_path
    if not abs_path.is_file():
        raise NotFound(str(abs_path))
    data = abs_path.read_bytes()
    original_sha = _sha256(data)

    if mode == "tombstone":
        rec_extra: dict = {"mode": "tombstone", "reason": reason or ""}
        if extra:
            rec_extra.update(extra)
        return writer.submit(
            AuditRecord(
                op="delete",
                path=rel_path,
                actor=actor,
                content_sha256=original_sha,
                extra=rec_extra,
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
    "put_with_record",
    "move",
    "delete",
    # exceptions
    "PathTraversal",
    "ContentTooLarge",
    "NotFound",
    "PurgeRefused",
]
