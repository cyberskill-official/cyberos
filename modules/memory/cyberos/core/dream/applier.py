"""
cyberos.core.dream.applier — replay DreamDiff proposals into the chain
(TASK-MEMORY-115 §1 #4, §1 #10..#14).

The applier is the operator-gated half of dreaming. It refuses to run
unless:

1. AGENTS.md §7.7 anchor is present (P19 amendment merged).
2. Every proposal's ``precondition_body_hashes`` map matches current
   on-disk SHA-256s.

If both gates pass, the applier walks the proposal list and emits the
appropriate canonical-op rows (``put`` / ``delete`` / ``move``) via the
existing :mod:`cyberos.core.ops` helpers. Every emitted row carries
``extra.dream_id`` + ``extra.proposal_id`` provenance per §7.7.2.

Slice-3 semantics:

* Atomic-per-proposal — each proposal is applied within a single
  ``Writer`` context. A precondition failure on proposal N aborts the
  apply BEFORE any writes from proposal N (or any later proposal) land.
  Earlier proposals that already succeeded remain committed; the
  applier returns ``{applied_count: M, rejected: K, rejection_reason: …}``
  so the operator can inspect.
* Idempotent — re-applying the same dream against unchanged state is a
  no-op (the precondition check ensures no double-write).
"""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Optional

from cyberos.core.dream.proposals import DreamDiff, DreamProposal


class PreconditionFailed(RuntimeError):
    """Raised when an on-disk body hash doesn't match the proposal's
    recorded precondition (TASK-MEMORY-115 §1 #10)."""


class ProtocolAmendmentMissing(RuntimeError):
    """Raised when AGENTS.md §7.7 is not yet anchored (TASK-MEMORY-115 §1 #11)."""


def _has_section_7_7(store: Path) -> bool:
    """Check AGENTS.md for the §7.7 anchor.

    Looks in two places:
    1. ``<store>/AGENTS.md`` if the store has its own copy.
    2. ``<store>/../AGENTS.md`` (repo root, the canonical location).
    """
    candidates: list[Path] = [
        store / "AGENTS.md",
        store.parent / "AGENTS.md",
    ]
    # Walk up to find AGENTS.md in case the store is
    # nested inside a project that imported the protocol files.
    for parent in [store, *store.parents][:6]:
        cand = parent / "AGENTS.md"
        if cand.exists() and cand not in candidates:
            candidates.append(cand)
    for c in candidates:
        if c.exists():
            try:
                body = c.read_text(encoding="utf-8", errors="ignore")
            except Exception:
                continue
            if "## §7.7" in body or "§7.7  Dreaming" in body or "§7.7 Dreaming" in body:
                return True
    return False


def _sha256_body(path: Path) -> str:
    if not path.exists():
        return ""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def apply(
    writer,  # cyberos.core.writer.Writer
    diff: DreamDiff,
    *,
    proposal_ids: Optional[set[str]] = None,
    actor: str = "dream-applier",
    enforce_section_7_7: bool = True,
) -> dict:
    """Apply selected proposals from ``diff``.

    Parameters
    ----------
    proposal_ids
        Filter — apply only proposals whose ``proposal_id`` is in this
        set. None ⇒ apply all proposals in the diff.
    enforce_section_7_7
        Slice-3 ships True (production). Tests may disable to exercise
        apply logic without requiring the protocol amendment files.

    Returns
    -------
    Summary dict with ``{applied_count, rejected, errors}``.
    """
    from cyberos.core.ops import put as _put, delete as _delete, NotFound
    from cyberos.core.writer import AuditRecord

    if enforce_section_7_7 and not _has_section_7_7(writer.store):
        raise ProtocolAmendmentMissing(
            "AGENTS.md §7.7 not anchored. Approve via:\n"
            "  APPROVE protocol change P19 §7.7\n"
            "and ensure AGENTS.md contains the §7.7 Dreaming section."
        )

    targets = [
        p for p in diff.proposals
        if proposal_ids is None or p.proposal_id in proposal_ids
    ]

    # ── Strict-idempotency pass — skip proposals already applied (TASK-MEMORY-115 §1 #10) ──
    # Walk the audit chain once; collect (dream_id, proposal_id) pairs already
    # present as dream.proposal_applied rows. Any proposal in that set is a
    # no-op on this apply call.
    already_applied: set[tuple[str, str]] = set()
    try:
        from cyberos.core.dream._audit_iter import iter_audit_rows
        for row in iter_audit_rows(writer.store):
            if row.get("op") == "dream.proposal_applied":
                ex = row.get("extra") or {}
                d, pid = ex.get("dream_id"), ex.get("proposal_id")
                if d and pid:
                    already_applied.add((d, pid))
    except Exception:
        already_applied = set()

    fresh_targets: list[DreamProposal] = []
    skipped_idempotent = 0
    for p in targets:
        if (diff.dream_id, p.proposal_id) in already_applied:
            skipped_idempotent += 1
            continue
        fresh_targets.append(p)

    # ── Precondition pass — all must match before any writes ──
    errors: list[dict] = []
    for p in fresh_targets:
        for path, expected in p.precondition_body_hashes.items():
            actual = _sha256_body(writer.store / path)
            if actual != expected:
                raise PreconditionFailed(
                    f"Proposal {p.proposal_id}: body-hash drift at {path!r} "
                    f"(expected {expected[:12]}…, got {actual[:12]}…)"
                )

    # ── Apply pass — each proposal advances HEAD via the canonical ops ──
    applied = 0
    for p in fresh_targets:
        try:
            _apply_one(writer, p, diff.dream_id, actor=actor)
            applied += 1
            # Per-proposal aux row
            writer.submit(AuditRecord(
                op="dream.proposal_applied",
                path=(p.paths[0] if p.paths else ""),
                actor=actor,
                extra={
                    "dream_id": diff.dream_id,
                    "proposal_id": p.proposal_id,
                    "proposal_op": p.op,
                    "affected_paths": list(p.paths),
                },
            ))
        except (NotFound, FileNotFoundError, ValueError) as e:
            errors.append({
                "proposal_id": p.proposal_id,
                "op": p.op,
                "error": f"{type(e).__name__}: {e}",
            })

    return {
        "applied_count": applied,
        "rejected": len(fresh_targets) - applied,
        "skipped_idempotent": skipped_idempotent,
        "errors": errors,
        "dream_id": diff.dream_id,
    }


def _apply_one(writer, p: DreamProposal, dream_id: str, *, actor: str) -> None:
    """Dispatch a single proposal to the appropriate canonical op."""
    from cyberos.core.ops import put as _put, delete as _delete
    from cyberos.core.frontmatter import parse, parse_legacy_yaml, looks_like_yaml, serialize, Frontmatter
    import time

    common_extra = {
        "dream_id": dream_id,
        "proposal_id": p.proposal_id,
    }

    if p.op == "merge":
        # Canonical = first path (or `into` if specified); others tombstoned
        canonical = p.into or (p.paths[0] if p.paths else "")
        for src in p.paths:
            if src == canonical:
                continue
            try:
                _delete(
                    writer, src,
                    actor=actor, mode="tombstone",
                    extra={
                        **common_extra,
                        "merged_into": canonical,
                        "rationale": p.rationale,
                    },
                )
            except Exception:
                # Re-raise as a structured error so the outer apply loop
                # captures it without aborting siblings.
                raise

    elif p.op == "stale":
        for path in p.paths:
            _delete(
                writer, path,
                actor=actor, mode="tombstone",
                extra={
                    **common_extra,
                    "reason": "stale",
                    "rationale": p.rationale,
                },
            )

    elif p.op == "new":
        # Create a new memory under memories/refinements/ (or wherever
        # the proposal pointed). The body is the content_preview wrapped
        # in standard frontmatter.
        if not p.paths:
            raise ValueError("DreamProposal op=new requires at least one path")
        target = p.paths[0]
        fm = Frontmatter(
            id=f"REF-{p.proposal_id}",
            kind="refinement",
            ts_ns=time.time_ns(),
            actor=actor,
            tags=["dream", "refinement"],
            extra={
                "dream_id": dream_id,
                "proposal_id": p.proposal_id,
                "rationale": p.rationale,
            },
        )
        body = (p.content_preview or "").encode("utf-8")
        if not body.strip():
            body = b"(empty refinement; see dream proposal for context)\n"
        file_bytes = serialize(fm, body)
        _put(
            writer, target, file_bytes,
            actor=actor, kind="refinement",
            extra=common_extra,
        )

    elif p.op == "verify":
        # Annotate meta.last_verified_at on existing files. Cheapest
        # implementation: read body, parse frontmatter, set
        # extra.last_verified_at = now, re-write. The new put row's
        # body-hash differs only in frontmatter — that's fine.
        from datetime import datetime, timezone
        now_iso = datetime.now(timezone.utc).isoformat()
        for path in p.paths:
            abs_p = writer.store / path
            if not abs_p.exists():
                continue
            raw = abs_p.read_bytes()
            try:
                fm, body = parse(raw)
            except Exception:
                if looks_like_yaml(raw):
                    fm, body = parse_legacy_yaml(raw)
                else:
                    continue
            new_extra = dict(fm.extra or {})
            new_extra["last_verified_at"] = now_iso
            new_fm = Frontmatter(
                id=fm.id,
                kind=fm.kind,
                ts_ns=fm.ts_ns,
                actor=fm.actor,
                tags=list(fm.tags),
                extra=new_extra,
            )
            new_bytes = serialize(new_fm, body)
            _put(
                writer, path, new_bytes,
                actor=actor, kind=fm.kind,
                extra={
                    **common_extra,
                    "verify_only": True,
                },
            )

    else:
        raise ValueError(f"unknown proposal op {p.op!r}")
