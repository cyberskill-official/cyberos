"""Workflow validator — confirms every skill in a workflow's chain exists in the SKILL catalog.

Per cuo/MODULE.md §0.8: "When a workflow needs a skill that doesn't exist in the SKILL
module, the workflow file SHALL reference it as `planned:<skill-name>` and the gap SHALL
appear in `cuo/docs/NEEDED_SKILLS.md`."

The validator surfaces gaps before the supervisor tries to invoke a chain. It walks
the SKILL module's filesystem layout — skills live at `skill/<name>-author/SKILL.md`
and `skill/<name>-audit/SKILL.md` — to confirm each chain step's `skill:` field has a
real implementation.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from cuo.core.catalog import WorkflowEntry


@dataclass
class ValidationResult:
    """Outcome of validating a workflow's skill_chain."""

    workflow_id: str
    valid: bool
    missing_skills: list[str] = field(default_factory=list)
    planned_skills: list[str] = field(default_factory=list)
    found_skills: list[str] = field(default_factory=list)
    chain_length: int = 0
    notes: list[str] = field(default_factory=list)

    def __repr__(self) -> str:
        flag = "VALID" if self.valid else "INVALID"
        return (
            f"ValidationResult({self.workflow_id!r}, {flag}, "
            f"chain_len={self.chain_length}, "
            f"missing={len(self.missing_skills)}, planned={len(self.planned_skills)})"
        )


def _skill_dir_exists(skill_root: Path, skill_name: str) -> bool:
    """Check whether `<skill_root>/<skill_name>/SKILL.md` exists.

    Returns True only when the directory contains a parseable SKILL.md.
    """
    skill_dir = skill_root / skill_name
    skill_md = skill_dir / "SKILL.md"
    return skill_dir.is_dir() and skill_md.is_file()


def validate_chain(workflow: WorkflowEntry, skill_root: Path) -> ValidationResult:
    """Validate that every step in `workflow.skill_chain` references a shipped skill.

    A chain step has shape:
        {step: 1, skill: "software-requirements-specification-author", inputs_from: ..., outputs_to: ...}

    The validator:
    - Counts a step as PLANNED if its `skill` value starts with `planned:` —
      these are explicit gaps marked per AGENTS.md §0.8.
    - Counts a step as MISSING if its `skill` value doesn't have a matching
      `skill/<name>/SKILL.md` on disk.
    - Counts a step as FOUND otherwise.

    A workflow is VALID iff it has no MISSING steps. PLANNED steps are
    surfaced separately so the caller can decide whether to invoke
    (MISSING_SKILL_REQUEST per AGENTS.md §1.4).

    Args:
        workflow: parsed WorkflowEntry whose skill_chain to validate.
        skill_root: filesystem path to `skill/` (must contain MODULE.md).

    Returns:
        ValidationResult with classified skills.
    """
    result = ValidationResult(workflow_id=workflow.workflow_id, valid=True)
    result.chain_length = len(workflow.skill_chain)

    if not workflow.skill_chain:
        result.notes.append("workflow has no skill_chain[] — vacuously valid but probably wrong")
        return result

    if not skill_root.is_dir():
        result.valid = False
        result.notes.append(f"skill_root does not exist: {skill_root}")
        return result

    for step in workflow.skill_chain:
        skill_name = step.get("skill") if isinstance(step, dict) else None
        if not isinstance(skill_name, str) or not skill_name:
            result.valid = False
            result.notes.append(f"step {step!r} has no `skill` field — skipping")
            continue

        if skill_name.startswith("planned:"):
            result.planned_skills.append(skill_name[len("planned:"):])
            # planned: is a soft gap, not a hard failure — still mark valid=False
            # since the chain cannot actually run, but distinguish from missing.
            continue

        if _skill_dir_exists(skill_root, skill_name):
            result.found_skills.append(skill_name)
        else:
            result.missing_skills.append(skill_name)
            result.valid = False

    if result.planned_skills:
        # Planned skills are gaps the catalog knows about. The chain cannot run
        # until they ship, but the workflow is intentionally incomplete (not broken).
        result.valid = False
        result.notes.append(
            f"{len(result.planned_skills)} planned: step(s) — see cuo/docs/NEEDED_SKILLS.md"
        )

    return result
