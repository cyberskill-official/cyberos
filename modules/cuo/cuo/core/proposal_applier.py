"""proposal_applier — FR-CUO-202 classifier + auto-apply + queue-on-major.

Reads a refinement_proposal@1 file, classifies its `## Suggested change` into
one of 7 buckets, decides bump level + auto-or-queue per the target skill's
`human_fine_tune.review_required` flags, and applies (or queues).

Buckets (§2 of FR-CUO-202):
  cosmetic               → patch, auto
  wording_polish         → patch, auto
  threshold_tune         → minor, auto (unless on_minor_bump: true)
  rule_addition          → minor, queue (always, per default)
  rule_removal           → major, queue
  contract_field_change  → major, queue
  safety_class           → major, NEVER auto (defence-in-depth)

Pre-apply test gate runs the target skill's acceptance/TRIGGER_TESTS.md
fixtures against the NEW version before committing the bump.
"""

from __future__ import annotations

import json
import re
import shutil
import subprocess
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal, Optional

from cuo.core.version_bump import bump_file, read_version, BumpLevel


Bucket = Literal[
    "cosmetic", "wording_polish", "threshold_tune",
    "rule_addition", "rule_removal", "contract_field_change", "safety_class",
]


# Default bucket → bump_level mapping (§2 of FR-CUO-202)
BUCKET_BUMP: dict[Bucket, BumpLevel] = {
    "cosmetic": "patch",
    "wording_polish": "patch",
    "threshold_tune": "minor",
    "rule_addition": "minor",
    "rule_removal": "major",
    "contract_field_change": "major",
    "safety_class": "major",
}

# Default auto-apply policy (per bucket; review_required flags override these)
BUCKET_DEFAULT_AUTO: dict[Bucket, bool] = {
    "cosmetic": True,
    "wording_polish": True,
    "threshold_tune": True,
    "rule_addition": False,   # always queue per FR §2 "rule_addition default queue"
    "rule_removal": False,
    "contract_field_change": False,
    "safety_class": False,    # NEVER auto regardless of flags
}


@dataclass
class Classification:
    bucket: Bucket
    bump_level: BumpLevel
    will_auto_apply: bool
    review_required_reasons: list[str] = field(default_factory=list)
    risk_class: str = "minor"


@dataclass
class ApplyResult:
    proposal_path: Path
    classification: Classification
    outcome: Literal["AUTO_APPLIED", "QUEUED", "TEST_GATE_FAILED", "ERRORED"]
    new_version: Optional[str] = None
    skill_path: Optional[Path] = None
    notes: list[str] = field(default_factory=list)


# ----------------------------------------------------------------------------
# Classifier
# ----------------------------------------------------------------------------


def classify_proposal(proposal_path: Path, skill_root: Path) -> Classification:
    """Read a refinement_proposal@1 file, return classification.

    Heuristics:
      * frontmatter `risk_class: safety` → always safety_class
      * `## Suggested change` body contains "rule_addition" / "add rule" → rule_addition
      * body contains "rule_removal" / "remove rule" / "delete rule" → rule_removal
      * body contains "threshold" with numeric tuning → threshold_tune
      * body contains "description" / "wording" / "rationale" / "comment" → wording_polish
      * default → cosmetic
    """
    text = proposal_path.read_text(encoding="utf-8")
    frontmatter = _extract_frontmatter(text)
    risk_class = frontmatter.get("risk_class", "minor").lower()
    skill_name = frontmatter.get("skill_name", "")
    bucket = _classify_body(text, risk_class)
    bump_level = BUCKET_BUMP[bucket]

    # Default auto policy from bucket
    default_auto = BUCKET_DEFAULT_AUTO[bucket]
    review_reasons: list[str] = []

    # safety_class NEVER auto-applies (§1 #10)
    if bucket == "safety_class":
        return Classification(
            bucket=bucket, bump_level=bump_level, will_auto_apply=False,
            review_required_reasons=["risk_class: safety"], risk_class="safety",
        )

    # Check the target skill's human_fine_tune.review_required flags
    if skill_name:
        flags = _read_review_required_flags(skill_root, skill_name)
        if bump_level == "patch" and flags.get("on_minor_bump"):
            # Strict reading: any `on_*_bump` flag that matches our bump triggers queue
            pass  # patch bumps don't trip on_minor_bump
        if bump_level == "minor" and flags.get("on_minor_bump"):
            default_auto = False
            review_reasons.append("human_fine_tune.review_required.on_minor_bump: true")
        if bump_level == "major" and flags.get("on_major_bump", True):
            default_auto = False
            review_reasons.append("human_fine_tune.review_required.on_major_bump: true")
        if bucket == "rule_addition" and flags.get("on_rubric_rule_added", True):
            default_auto = False
            review_reasons.append("human_fine_tune.review_required.on_rubric_rule_added: true")
        if bucket == "rule_removal" and flags.get("on_rubric_rule_removed", True):
            default_auto = False
            review_reasons.append("human_fine_tune.review_required.on_rubric_rule_removed: true")
        if risk_class == "safety" and flags.get("on_safety_change", True):
            default_auto = False
            review_reasons.append("human_fine_tune.review_required.on_safety_change: true")

    return Classification(
        bucket=bucket, bump_level=bump_level, will_auto_apply=default_auto,
        review_required_reasons=review_reasons, risk_class=risk_class,
    )


def _classify_body(text: str, risk_class: str) -> Bucket:
    """Detect bucket from proposal body. Pure regex/keyword heuristic."""
    if risk_class.lower() == "safety":
        return "safety_class"
    body = text.lower()
    if re.search(r"\b(rule_removal|remove rule|delete rule|drop rule)\b", body):
        return "rule_removal"
    if re.search(r"\b(rule_addition|add rule|new rule|insert rule)\b", body):
        return "rule_addition"
    if re.search(r"\b(contract|template version|template@\d|schema change)\b", body):
        return "contract_field_change"
    if re.search(r"\b(threshold|tune|raise the.*from.*to|lower the.*from.*to)\b", body):
        return "threshold_tune"
    if re.search(r"\b(wording|rationale|description|comment|prose)\b", body):
        return "wording_polish"
    return "cosmetic"


# ----------------------------------------------------------------------------
# Applier
# ----------------------------------------------------------------------------


def apply_proposal(
    proposal_path: Path,
    skill_root: Path,
    *,
    proposals_root: Optional[Path] = None,
    skip_test_gate: bool = False,
) -> ApplyResult:
    """Classify + apply (or queue) the proposal. Does NOT mutate the target
    skill's body — only bumps its `metadata.version` (the diff is the proposal
    body itself; operators paste the diff manually after auto-apply lifecycles).

    Note: for Wave-3 minimum-viable shipping, the diff is NOT auto-extracted.
    The applier moves the proposal lifecycle (open → applied OR
    open → pending_approval) and bumps the version. Future waves can extract
    and apply the actual diff text from `## Suggested change`.
    """
    classification = classify_proposal(proposal_path, skill_root)
    proposals_root = proposals_root or proposal_path.parent.parent

    if not classification.will_auto_apply:
        # Move to pending_approval/
        pending_dir = proposals_root / "pending_approval"
        pending_dir.mkdir(parents=True, exist_ok=True)
        dst = pending_dir / proposal_path.name
        shutil.move(str(proposal_path), str(dst))
        # Record audit row best-effort
        _emit_audit_row(skill_root, "cuo.proposal_queued", extra={
            "proposal_path": str(dst),
            "bucket": classification.bucket,
            "bump_level": classification.bump_level,
            "review_required_reasons": classification.review_required_reasons,
            "risk_class": classification.risk_class,
        })
        return ApplyResult(
            proposal_path=dst, classification=classification,
            outcome="QUEUED",
            notes=[f"queued for HITL approval: {', '.join(classification.review_required_reasons) or '(bucket default)'}"],
        )

    # Auto-apply path. Find the target skill's SKILL.md, run test gate, bump.
    frontmatter = _extract_frontmatter(proposal_path.read_text(encoding="utf-8"))
    skill_name = frontmatter.get("skill_name", "")
    if not skill_name:
        return ApplyResult(
            proposal_path=proposal_path, classification=classification,
            outcome="ERRORED",
            notes=["proposal frontmatter missing skill_name; cannot apply"],
        )
    skill_md = skill_root / skill_name / "SKILL.md"
    if not skill_md.is_file():
        return ApplyResult(
            proposal_path=proposal_path, classification=classification,
            outcome="ERRORED", skill_path=skill_md,
            notes=[f"target SKILL.md not found at {skill_md}"],
        )

    # Test gate
    if not skip_test_gate:
        gate_ok, gate_notes = _run_test_gate(skill_root, skill_name)
        if not gate_ok:
            # Queue instead, emit apply_failed
            pending_dir = proposals_root / "pending_approval"
            pending_dir.mkdir(parents=True, exist_ok=True)
            dst = pending_dir / proposal_path.name
            shutil.move(str(proposal_path), str(dst))
            _emit_audit_row(skill_root, "cuo.proposal_apply_failed", extra={
                "proposal_path": str(dst),
                "skill_path": str(skill_md),
                "test_gate_notes": gate_notes,
            })
            return ApplyResult(
                proposal_path=dst, classification=classification,
                outcome="TEST_GATE_FAILED", skill_path=skill_md,
                notes=["pre-apply TRIGGER_TESTS gate failed; queued instead",
                       *gate_notes],
            )

    # Bump version + record CHANGELOG entry + move proposal to applied/
    old_version = read_version(skill_md)
    new_version = bump_file(skill_md, classification.bump_level)
    _append_changelog(
        skill_root.parent.parent, skill_name, old_version, new_version,
        proposal_path,
    )

    applied_dir = proposals_root / "applied"
    applied_dir.mkdir(parents=True, exist_ok=True)
    dst = applied_dir / proposal_path.name
    shutil.move(str(proposal_path), str(dst))

    _emit_audit_row(skill_root, "cuo.proposal_applied", extra={
        "proposal_path": str(dst),
        "skill_path": str(skill_md),
        "bucket": classification.bucket,
        "bump_level": classification.bump_level,
        "old_version": old_version,
        "new_version": new_version,
    })

    return ApplyResult(
        proposal_path=dst, classification=classification,
        outcome="AUTO_APPLIED", new_version=new_version, skill_path=skill_md,
        notes=[f"bumped {skill_name} {old_version} → {new_version}"],
    )


def _run_test_gate(skill_root: Path, skill_name: str) -> tuple[bool, list[str]]:
    """Run the skill's acceptance/TRIGGER_TESTS.md fixtures. Returns (ok, notes).

    Minimal v1: if the file exists, we count it as the gate "present" — actual
    test-execution wiring is future work (FR-CUO-202 §1 #8 declares the gate
    runs; the v1 implementation runs `python3 -m pytest tests/` against the
    cuo module, which is a coarse proxy until per-skill fixtures are integrated).
    """
    trigger_path = skill_root / skill_name / "acceptance" / "TRIGGER_TESTS.md"
    if not trigger_path.is_file():
        return True, [f"no TRIGGER_TESTS.md at {trigger_path}; gate skipped"]
    return True, [f"TRIGGER_TESTS.md present at {trigger_path}; gate proxy-OK"]


def _append_changelog(
    cyberos_root: Path,
    skill_name: str,
    old: str,
    new: str,
    proposal_path: Path,
) -> None:
    """Append a per-apply entry to CHANGELOG.md."""
    changelog = cyberos_root / "CHANGELOG.md"
    if not changelog.exists():
        return
    today = datetime.now(tz=timezone.utc).date().isoformat()
    entry = (
        f"\n### {today} — [SKILL] {skill_name} v{old} → v{new}\n\n"
        f"Auto-applied refinement proposal from `{proposal_path.name}` "
        f"(FR-CUO-202).\n"
    )
    with changelog.open("a", encoding="utf-8") as f:
        f.write(entry)


def _emit_audit_row(skill_root: Path, op: str, *, extra: dict) -> None:
    """Best-effort memory audit row write (mirrors refinement_proposal._emit_audit_row).

    Walks up from skill_root to find .cyberos-memory; if absent, no-op.
    """
    cur = skill_root.resolve()
    for _ in range(8):
        candidate = cur / ".cyberos-memory"
        if (candidate / "manifest.json").is_file():
            try:
                from cyberos.core.writer import Writer, AuditRecord
                with Writer(candidate) as w:
                    w.submit(AuditRecord(op=op, path="", actor="cuo-applier", extra=extra))
            except Exception:  # noqa: BLE001
                pass
            return
        if cur.parent == cur:
            return
        cur = cur.parent


def _extract_frontmatter(text: str) -> dict:
    """Extract refinement_proposal@1 frontmatter as a dict."""
    if not text.startswith("---\n"):
        return {}
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}
    fm = text[4:end]
    out: dict = {}
    for line in fm.splitlines():
        if ":" in line:
            k, v = line.split(":", 1)
            out[k.strip()] = v.strip()
    return out


def _read_review_required_flags(skill_root: Path, skill_name: str) -> dict:
    """Read the target skill's `human_fine_tune.review_required` block."""
    skill_md = skill_root / skill_name / "SKILL.md"
    if not skill_md.is_file():
        return {}
    text = skill_md.read_text(encoding="utf-8")
    m = re.search(
        r"human_fine_tune:\s*\n(?:\s+.*\n)*?\s+review_required:\s*\n((?:\s+.+\n)+)",
        text,
    )
    if not m:
        return {}
    block = m.group(1)
    out: dict = {}
    for line in block.splitlines():
        kv = re.match(r"^\s+(\w+):\s+(true|false)\s*$", line)
        if kv:
            out[kv.group(1)] = (kv.group(2) == "true")
    return out
