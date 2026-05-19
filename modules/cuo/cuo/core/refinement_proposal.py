"""refinement_proposal — FR-CUO-201 stripe-deduped proposal emitter.

Load-bearing logic:
  * First occurrence of a stripe → write proposal to `<root>/open/<stripe>-<ts>.md`,
    return Emitted, chain continues.
  * Second occurrence of unresolved stripe → write NO new file, emit
    `cuo.stripe_repeat_halt` aux row, set workflow outcome to HITL_HALT.
  * Applied/rejected proposals re-open the stripe (the dedup window is only
    `open/`).

Proposal body shape:
    ---
    template: refinement_proposal@1
    stripe_id: ...
    kind: skill_refinement | workflow_refinement
    ---

    ## Stripe
    ## Triggering signal
    ## Evidence rows
    ## Suggested change
    ## Risk class
"""

from __future__ import annotations

import json
import os
import re
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

from cuo.core.stripe import StripeId, compute_stripe


# Outcome variants
@dataclass
class Emitted:
    stripe_id: str
    proposal_path: Path
    is_new: bool = True


@dataclass
class StripeRepeatHalt:
    stripe_id: str
    existing_proposal_path: Path
    new_evidence_row_ids: list[str] = field(default_factory=list)


@dataclass
class Suppressed:
    stripe_id: str
    reason: str


EmissionResult = Emitted | StripeRepeatHalt | Suppressed


def emit_or_halt(
    skill_name: str,
    signal_id: str,
    evidence_rows: list[dict],
    proposals_root: Path,
    *,
    risk_class: str = "minor",
    suggested_change: str = "",
    kind: str = "skill_refinement",
    memory_root: Optional[Path] = None,
    actor: str = "cuo-harness",
) -> EmissionResult:
    """Decide whether to emit a new proposal OR halt on stripe repeat.

    Returns one of `Emitted` / `StripeRepeatHalt` / `Suppressed`. Side effects:
      * On Emitted: writes `<root>/open/<stripe>-<ts>.md` + emits
        `cuo.refinement_proposal_emitted` memory aux row.
      * On StripeRepeatHalt: emits `cuo.stripe_repeat_halt` memory aux row;
        no file written.
    """
    proposals_root.mkdir(parents=True, exist_ok=True)
    open_dir = proposals_root / "open"
    open_dir.mkdir(exist_ok=True)
    (proposals_root / "applied").mkdir(exist_ok=True)
    (proposals_root / "rejected").mkdir(exist_ok=True)
    (proposals_root / "pending_approval").mkdir(exist_ok=True)

    stripe = compute_stripe(skill_name, signal_id, evidence_rows)
    stripe_id = str(stripe)

    # Workflow stripes contain `/` which would create subdirs in the filename;
    # escape them to `--` for filesystem safety while keeping stripe_id intact.
    stripe_id_safe = stripe_id.replace("/", "--")
    # Check `open/` for any existing file matching the stripe prefix.
    existing = list(open_dir.glob(f"{stripe_id_safe}-*.md"))
    if existing:
        # Repeat → halt, no new file, emit stripe_repeat_halt
        _emit_audit_row(
            memory_root, actor, "cuo.stripe_repeat_halt",
            path=str(existing[0]),
            extra={
                "stripe_id": stripe_id,
                "existing_proposal_path": str(existing[0]),
                "new_evidence_row_ids": [r.get("row_id", "") for r in evidence_rows[:10]],
            },
        )
        return StripeRepeatHalt(
            stripe_id=stripe_id,
            existing_proposal_path=existing[0],
            new_evidence_row_ids=[r.get("row_id", "") for r in evidence_rows[:10]],
        )

    # First occurrence — write proposal
    now = datetime.now(tz=timezone.utc)
    ts = now.strftime("%Y%m%dT%H%M%S") + f"{now.microsecond // 1000:03d}Z"
    proposal_path = open_dir / f"{stripe_id_safe}-{ts}.md"
    # Defensive: if collision still (rare — same ms), bump until unique.
    suffix = 0
    while proposal_path.exists():
        suffix += 1
        proposal_path = open_dir / f"{stripe_id_safe}-{ts}-{suffix:02d}.md"
    body = _format_proposal(
        stripe=stripe, skill_name=skill_name, signal_id=signal_id,
        evidence_rows=evidence_rows, risk_class=risk_class,
        suggested_change=suggested_change, kind=kind, generated_at=now,
    )
    proposal_path.write_text(body, encoding="utf-8")

    _emit_audit_row(
        memory_root, actor, "cuo.refinement_proposal_emitted",
        path=str(proposal_path),
        extra={
            "stripe_id": stripe_id,
            "skill_name": skill_name,
            "signal_id": signal_id,
            "evidence_row_ids": [r.get("row_id", "") for r in evidence_rows[:10]],
            "proposal_path": str(proposal_path),
        },
    )

    return Emitted(stripe_id=stripe_id, proposal_path=proposal_path, is_new=True)


def _format_proposal(
    stripe: StripeId,
    skill_name: str,
    signal_id: str,
    evidence_rows: list[dict],
    risk_class: str,
    suggested_change: str,
    kind: str,
    generated_at: datetime,
) -> str:
    """Render the refinement_proposal@1 markdown body."""
    parts: list[str] = []
    parts.append("---")
    parts.append("template: refinement_proposal@1")
    parts.append(f"stripe_id: {stripe}")
    parts.append(f"kind: {kind}")
    parts.append(f"skill_name: {skill_name}")
    parts.append(f"signal_id: {signal_id}")
    parts.append(f"risk_class: {risk_class}")
    parts.append(f"generated_at: {generated_at.isoformat()}")
    parts.append("---")
    parts.append("")
    parts.append(f"# Refinement proposal — `{stripe}`")
    parts.append("")
    parts.append("## Stripe")
    parts.append("")
    parts.append(f"`{stripe}`")
    parts.append("")
    parts.append(f"- **scope:** `{stripe.scope}`")
    parts.append(f"- **signal:** `{stripe.signal_id}`")
    parts.append(f"- **pattern hash:** `{stripe.pattern_hash}` (8 hex chars)")
    parts.append("")
    parts.append("## Triggering signal")
    parts.append("")
    parts.append(f"Skill `{skill_name}` tripped `{signal_id}` over the analysis window.")
    parts.append(f"Evidence row count: **{len(evidence_rows)}**.")
    parts.append("")
    parts.append("## Evidence rows")
    parts.append("")
    if evidence_rows:
        parts.append("| row_id | op | summary |")
        parts.append("|---|---|---|")
        for r in evidence_rows[:20]:
            row_id = r.get("row_id", "?")
            op = r.get("op", "?")
            summary = _row_summary(r)
            parts.append(f"| `{row_id}` | `{op}` | {summary} |")
    else:
        parts.append("*(no evidence rows captured)*")
    parts.append("")
    parts.append("## Suggested change")
    parts.append("")
    parts.append(suggested_change or "*(LLM did not provide a suggested change — operator review required)*")
    parts.append("")
    parts.append("## Risk class")
    parts.append("")
    parts.append(f"**{risk_class}** — see FR-CUO-202 §2 bump-level table for classifier semantics.")
    parts.append("")
    return "\n".join(parts)


def _row_summary(row: dict) -> str:
    """One-line evidence-row summary for the table."""
    extra = row.get("extra") or {}
    bits = []
    if "skill" in extra:
        bits.append(f"skill={extra['skill']}")
    if "outcome" in extra:
        bits.append(f"outcome={extra['outcome']}")
    if "fr_id" in extra:
        bits.append(f"fr={extra['fr_id']}")
    if "rework_reason" in extra:
        bits.append(f"reason={extra['rework_reason'][:40]}")
    return ", ".join(bits) or "*(no metadata)*"


# ----------------------------------------------------------------------------
# Operator workflow — list / show / apply / reject
# ----------------------------------------------------------------------------


def list_proposals(proposals_root: Path) -> dict[str, list[Path]]:
    """Return open/applied/rejected/pending_approval proposal paths."""
    out: dict[str, list[Path]] = {}
    for status in ("open", "applied", "rejected", "pending_approval"):
        d = proposals_root / status
        out[status] = sorted(d.glob("*.md")) if d.is_dir() else []
    return out


def reject_proposal(
    proposals_root: Path,
    stripe_id: str,
    reason: str,
) -> Optional[Path]:
    """Move an open proposal to rejected/ with a Rejection rationale section appended.

    Returns the new path, or None if no matching open proposal exists.
    """
    open_dir = proposals_root / "open"
    rejected_dir = proposals_root / "rejected"
    rejected_dir.mkdir(parents=True, exist_ok=True)

    matches = sorted(open_dir.glob(f"{stripe_id.replace('/', '--')}-*.md"))
    if not matches:
        return None
    src = matches[0]
    body = src.read_text(encoding="utf-8")
    body += "\n## Rejection rationale\n\n"
    body += f"Rejected at {datetime.now(tz=timezone.utc).isoformat()}.\n\n"
    body += f"{reason}\n"
    dst = rejected_dir / src.name
    dst.write_text(body, encoding="utf-8")
    src.unlink()
    return dst


def approve_proposal(
    proposals_root: Path,
    stripe_id: str,
) -> Optional[Path]:
    """Move a pending_approval proposal to applied/. The actual diff-apply
    is FR-CUO-202's responsibility; this is just the lifecycle move.

    Returns the new path, or None if no matching pending proposal exists.
    """
    pending_dir = proposals_root / "pending_approval"
    applied_dir = proposals_root / "applied"
    applied_dir.mkdir(parents=True, exist_ok=True)

    matches = sorted(pending_dir.glob(f"{stripe_id.replace('/', '--')}-*.md"))
    if not matches:
        return None
    src = matches[0]
    dst = applied_dir / src.name
    os.replace(src, dst)
    return dst


def apply_proposal_lifecycle(
    proposals_root: Path,
    stripe_id: str,
) -> Optional[Path]:
    """Move an open proposal to applied/ — lifecycle step only.

    FR-CUO-202 will extend this with the actual diff-apply machinery.
    """
    open_dir = proposals_root / "open"
    applied_dir = proposals_root / "applied"
    applied_dir.mkdir(parents=True, exist_ok=True)

    matches = sorted(open_dir.glob(f"{stripe_id.replace('/', '--')}-*.md"))
    if not matches:
        return None
    src = matches[0]
    dst = applied_dir / src.name
    os.replace(src, dst)
    return dst


# ----------------------------------------------------------------------------
# Audit-row emitter — opportunistic
# ----------------------------------------------------------------------------


def _emit_audit_row(
    memory_root: Optional[Path],
    actor: str,
    op: str,
    *,
    path: str,
    extra: dict,
) -> None:
    """Best-effort `memory_root/audit` write via Writer. No-op if memory module
    not importable or memory_root not initialised."""
    if memory_root is None:
        return
    if not (memory_root / "manifest.json").is_file():
        return
    try:
        from cyberos.core.writer import Writer, AuditRecord
    except ImportError:
        return
    try:
        with Writer(memory_root) as w:
            w.submit(AuditRecord(
                op=op, path=path, actor=actor, extra=extra,
            ))
    except Exception:  # noqa: BLE001
        pass
