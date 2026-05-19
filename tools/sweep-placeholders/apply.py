#!/usr/bin/env python3
"""tools/sweep-placeholders/apply.py — FR-SKILL-115 sweep applier.

Applies the suggestions from `tools/sweep-placeholders/report-YYYY-MM-DD.md`
to the actual SKILL.md frontmatter files. Per FR-SKILL-115 §1 #4 + §1 #10:

- Operator-attestation required for non-trivial substitutions.
- The "stage → cross" default is high-confidence and can be auto-applied.
- Description / allowed_memory_scopes placeholders need per-skill review.

Modes:
- `--stage-only`: apply ONLY the high-confidence metadata.stage substitutions
  (whichever stage the suggester recommended; usually "cross"). Surface the
  rest as a smaller list for operator review.
- `--all`: apply every suggestion in the report. Use only after operator review.
- `--dry-run`: preview changes without writing.
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_ROOT = REPO_ROOT / "modules" / "skill"


@dataclass(frozen=True)
class Substitution:
    skill_rel_path: str  # e.g. "skill/account-plan-author/SKILL.md"
    field_path: str      # e.g. "root.metadata.stage"
    current: str
    suggested: str
    rationale: str

    @property
    def is_stage_default(self) -> bool:
        """True if this is the canonical 'metadata.stage → cross' default."""
        return self.field_path == "root.metadata.stage" and self.suggested.strip() == "cross"


# ─── Report parser ──────────────────────────────────────────────────────────────

_SECTION_RE = re.compile(r"^### (.+)$", re.MULTILINE)


def parse_report(report_path: Path) -> list[Substitution]:
    """Parse the report.md and extract Substitution rows."""
    text = report_path.read_text(encoding="utf-8")
    subs: list[Substitution] = []

    # Each `### <skill>` section has 0+ field entries shaped like:
    #   - **<field_path>**
    #     - **current:** `<value>`
    #     - **suggest:** `<value>`
    #     - **why:** <rationale>
    sections = re.split(r"^### ", text, flags=re.MULTILINE)
    for sec in sections[1:]:  # first chunk is preamble
        lines = sec.split("\n", 1)
        skill_path = lines[0].strip()
        body = lines[1] if len(lines) > 1 else ""

        # Find each `- **<field>**` block + its 3 sub-lines
        for m in re.finditer(
            r"-\s+\*\*([^*]+)\*\*\n"
            r"\s+-\s+\*\*current:\*\*\s+`([^`]+)`\n"
            r"\s+-\s+\*\*suggest:\*\*\s+`([^`]+)`\n"
            r"\s+-\s+\*\*why:\*\*\s+(.+)",
            body,
        ):
            field_path = m.group(1).strip()
            current = m.group(2)
            suggested = m.group(3)
            rationale = m.group(4).strip().split("\n")[0]
            subs.append(Substitution(
                skill_rel_path=skill_path,
                field_path=field_path,
                current=current,
                suggested=suggested,
                rationale=rationale,
            ))
    return subs


# ─── Substitution applier ───────────────────────────────────────────────────────

def apply_to_file(skill_md: Path, sub: Substitution) -> bool:
    """Apply one substitution to a SKILL.md file. Returns True if file changed."""
    if not skill_md.exists():
        print(f"  ✗ {sub.skill_rel_path}: file not found", file=sys.stderr)
        return False
    text = skill_md.read_text(encoding="utf-8")

    # Only operate on the frontmatter block (between leading --- and next ---)
    if not text.startswith("---\n"):
        print(f"  ✗ {sub.skill_rel_path}: no frontmatter", file=sys.stderr)
        return False
    try:
        end = text.index("\n---\n", 4)
    except ValueError:
        print(f"  ✗ {sub.skill_rel_path}: no closing frontmatter delim", file=sys.stderr)
        return False
    fm = text[4:end]
    body = text[end:]

    # Locate the field. For root.metadata.stage we look for a "stage:" line
    # under a "metadata:" parent — naive but matches CyberOS's convention.
    new_fm = _replace_field_value(fm, sub)
    if new_fm is None:
        print(f"  ⊘ {sub.skill_rel_path}: pattern not found for {sub.field_path}", file=sys.stderr)
        return False
    if new_fm == fm:
        return False  # no-op (already-applied)
    new_text = "---\n" + new_fm + body
    skill_md.write_text(new_text, encoding="utf-8")
    return True


def _replace_field_value(fm_text: str, sub: Substitution) -> str | None:
    """Replace the literal current value with the suggested value within a YAML field block.

    Strategy: search for the field's leaf key as the *anchor* (e.g. for
    `root.metadata.stage` we search for a `\\bstage:\\s+<current>` line in the
    frontmatter). Replace the value portion only.
    """
    field_parts = sub.field_path.split(".")
    if field_parts[0] == "root":
        field_parts = field_parts[1:]
    if not field_parts:
        return None
    leaf_key = field_parts[-1]
    # Trim trailing `[N]` indexing artefact (e.g. `write[0]`)
    leaf_key = re.sub(r"\[\d+\]$", "", leaf_key)

    # Escape regex specials in the current value
    cur_esc = re.escape(sub.current)
    # Allow YAML quote styles
    pattern = re.compile(
        rf"(\b{re.escape(leaf_key)}\s*:\s*)(?:\"|'|)" +
        cur_esc +
        r"(?:\"|'|)",
        re.MULTILINE,
    )
    new_fm, n = pattern.subn(rf"\g<1>{sub.suggested}", fm_text, count=1)
    if n == 0:
        return None
    return new_fm


# ─── Main ───────────────────────────────────────────────────────────────────────

def main() -> int:
    p = argparse.ArgumentParser(description="FR-SKILL-115 sweep applier")
    p.add_argument("--report", type=Path, required=True, help="Path to the report.md")
    p.add_argument("--stage-only", action="store_true",
                   help="Only apply the high-confidence metadata.stage='cross' substitutions")
    p.add_argument("--all", action="store_true",
                   help="Apply every suggestion in the report (operator-reviewed)")
    p.add_argument("--dry-run", action="store_true", help="Preview only")
    args = p.parse_args()

    if not args.stage_only and not args.all:
        print("Pick a mode: --stage-only or --all", file=sys.stderr)
        return 2
    if not args.report.exists():
        print(f"Report not found: {args.report}", file=sys.stderr)
        return 2

    subs = parse_report(args.report)
    print(f"Parsed {len(subs)} substitution(s) from {args.report.name}")

    # Filter to mode
    if args.stage_only:
        eligible = [s for s in subs if s.is_stage_default]
        deferred = [s for s in subs if not s.is_stage_default]
    else:
        eligible = subs
        deferred = []

    print(f"  Eligible for this run: {len(eligible)}")
    print(f"  Deferred for review: {len(deferred)}")
    print()

    # Group by file
    by_file: dict[Path, list[Substitution]] = {}
    for s in eligible:
        path = REPO_ROOT / s.skill_rel_path.replace("skill/", "modules/skill/", 1) \
            if not s.skill_rel_path.startswith("modules/") else REPO_ROOT / s.skill_rel_path
        by_file.setdefault(path, []).append(s)

    applied = 0
    skipped = 0
    not_found = 0
    for path, file_subs in sorted(by_file.items()):
        for sub in file_subs:
            if args.dry_run:
                print(f"  would: {sub.skill_rel_path} :: {sub.field_path} → {sub.suggested!r}")
                applied += 1
                continue
            changed = apply_to_file(path, sub)
            if changed:
                applied += 1
                rel = path.relative_to(REPO_ROOT) if path.is_absolute() else path
                print(f"  ✓ {rel} :: {sub.field_path} → {sub.suggested!r}")
            else:
                skipped += 1

    print()
    action = "Would apply" if args.dry_run else "Applied"
    print(f"{action}: {applied} substitution(s)")
    print(f"Skipped (no-op or pattern-miss): {skipped}")
    if deferred:
        print()
        print(f"Deferred ({len(deferred)} substitutions need operator review):")
        for s in deferred[:25]:
            print(f"  {s.skill_rel_path}")
            print(f"    {s.field_path}: {s.current!r} → {s.suggested!r}")
        if len(deferred) > 25:
            print(f"  ... + {len(deferred) - 25} more")
    return 0


if __name__ == "__main__":
    sys.exit(main())
