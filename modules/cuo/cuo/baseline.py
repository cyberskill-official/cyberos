"""BASELINE.md validator for FR-SKILL-114.

Validates the design-time performance baseline artefact at v0.x → v1.0 promotion.
Used by CI gates + skill-bundle auditors.

Per FR-SKILL-114 §1.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path

import yaml


REQUIRED_SECTIONS = [
    "## Workflow under test",
    "## Without-skill baseline",
    "## With-skill measurements",
    "## Token-budget transparency",
    "## Trust calibration",
    "## Authoring notes",
]
REQUIRED_FM_KEYS = frozenset({
    "skill_id",
    "baseline_version",
    "baseline_measured_at",
    "attested_by",
    "next_review_due",
})
# Matches `cuo-cpo`, `cuo-clo-legal`, `human:stephen-cheng`, etc.
ATTESTOR_RE = re.compile(r"^(cuo-[a-z][a-z0-9-]*|human:[a-z][a-z0-9_-]*)$")


@dataclass(frozen=True)
class BaselineValidationResult:
    skill_id: str
    valid: bool
    issues: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)

    def summary(self) -> str:
        if self.valid and not self.warnings:
            return f"✓ {self.skill_id} — baseline valid"
        lines = []
        if self.valid:
            lines.append(f"✓ {self.skill_id} — baseline valid (with warnings)")
        else:
            lines.append(f"✗ {self.skill_id} — baseline invalid:")
        for i in self.issues:
            lines.append(f"  - error: {i}")
        for w in self.warnings:
            lines.append(f"  - warning: {w}")
        return "\n".join(lines)


def validate(path: Path) -> BaselineValidationResult:
    """Validate a BASELINE.md file. Returns BaselineValidationResult."""
    if not path.exists():
        return BaselineValidationResult(
            skill_id="?", valid=False, issues=["file_missing"],
        )
    text = path.read_text(encoding="utf-8")
    issues: list[str] = []
    warnings: list[str] = []
    skill_id = "?"

    # 1. Parse frontmatter
    if not text.startswith("---\n"):
        return BaselineValidationResult(
            skill_id="?", valid=False,
            issues=["frontmatter_invalid: missing leading '---' delimiter"],
        )
    try:
        end = text.index("\n---\n", 4)
    except ValueError:
        return BaselineValidationResult(
            skill_id="?", valid=False,
            issues=["frontmatter_invalid: missing closing '---' delimiter"],
        )

    try:
        fm = yaml.safe_load(text[4:end])
    except yaml.YAMLError as e:
        return BaselineValidationResult(
            skill_id="?", valid=False,
            issues=[f"frontmatter_invalid: YAML parse error: {e}"],
        )
    if not isinstance(fm, dict):
        return BaselineValidationResult(
            skill_id="?", valid=False,
            issues=["frontmatter_invalid: not a YAML mapping"],
        )
    body = text[end + 5:]

    skill_id = fm.get("skill_id", "?")

    # 2. Required keys
    missing = REQUIRED_FM_KEYS - set(fm.keys())
    if missing:
        issues.append(f"frontmatter_invalid: missing keys {sorted(missing)}")

    # 3. attested_by format
    attested_by = fm.get("attested_by", "")
    if attested_by and not ATTESTOR_RE.match(str(attested_by)):
        issues.append(f"attested_by_invalid: '{attested_by}' must match ^(cuo-<role>|human:<id>)$")

    # 4. next_review_due
    next_review = fm.get("next_review_due")
    if next_review is not None:
        try:
            # Accept ISO 8601 dates and datetimes (date-only is fine; YAML parser may have already parsed as date)
            if hasattr(next_review, "year") and not hasattr(next_review, "hour"):
                # YAML 1.1 parsed as date — pad to datetime
                due = datetime(next_review.year, next_review.month, next_review.day, tzinfo=timezone.utc)
            else:
                due = datetime.fromisoformat(str(next_review).replace("Z", "+00:00"))
                if due.tzinfo is None:
                    due = due.replace(tzinfo=timezone.utc)
            now = datetime.now(timezone.utc)
            days_overdue = (now - due).days
            if days_overdue > 365:
                issues.append(f"review_overdue: stale baseline (>{days_overdue} days overdue; threshold 365)")
            elif days_overdue > 0:
                warnings.append(f"review_overdue: {days_overdue} days past next_review_due")
        except (ValueError, TypeError) as e:
            issues.append(f"frontmatter_invalid: next_review_due is not parseable ISO 8601: {e}")

    # 5. Required body sections
    for section in REQUIRED_SECTIONS:
        if section not in body:
            issues.append(f"section_missing: '{section}'")

    valid = (len(issues) == 0)
    return BaselineValidationResult(
        skill_id=str(skill_id), valid=valid, issues=issues, warnings=warnings,
    )


def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help"):
        print("Usage: python -m cuo.baseline <skill_path>")
        print("       python -m cuo.baseline <skill_path>/BASELINE.md  (direct file path)")
        return 2
    arg = Path(argv[0])
    # Accept either skill folder (look for BASELINE.md inside) or direct file path
    if arg.is_dir():
        path = arg / "BASELINE.md"
    else:
        path = arg
    result = validate(path)
    print(result.summary())
    return 0 if result.valid else 1


if __name__ == "__main__":
    sys.exit(main())
