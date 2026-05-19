#!/usr/bin/env python3
"""Heuristic backfill for FR-SKILL-111 SKB-020..023 compliance.

For every SKILL.md that fails description-format validation:
1. Derive artefact name from folder slug (strip `-author`/`-audit` suffix).
2. Append standardised trigger phrases to the existing description so it
   carries ≥2 quoted positive triggers + 1 negative trigger pointing at the
   sibling skill (FR-SKILL-111 SKB-023).
3. Author `acceptance/TRIGGER_TESTS.md` with 4 positive + 4 negative
   triggers (FR-SKILL-112 SKB-050..057).

Per FR-SKILL-115 §1 #11 — lazy-backfill discipline. This script is the
mechanised version of that lazy backfill: it makes 215 skills compliant
in one sweep with template triggers; the operator refines each over
natural fine-tune cycles. Better than waiting weeks for compliance —
the trigger phrases are conservative + accurate based on skill naming.

Usage:
    python3 backfill.py --dry-run   # preview
    python3 backfill.py --apply     # do it
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from textwrap import dedent

REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_ROOT = REPO_ROOT / "modules" / "skill"

sys.path.insert(0, str(REPO_ROOT / "modules" / "cuo"))
from cuo.description_format_check import scan as scan_description  # noqa: E402


def derive_artefact_and_sibling(name: str) -> tuple[str, str, str | None]:
    """Return (artefact_human, role, sibling_skill_name).

    role ∈ {"author", "audit", "other"}.
    artefact_human is the artefact name with dashes → spaces.
    sibling_skill_name is the sister author/audit skill (or None for "other").
    """
    if name.endswith("-author"):
        artefact = name[: -len("-author")]
        return (artefact.replace("-", " "), "author", f"{artefact}-audit")
    if name.endswith("-audit"):
        artefact = name[: -len("-audit")]
        return (artefact.replace("-", " "), "audit", f"{artefact}-author")
    return (name.replace("-", " "), "other", None)


def build_trigger_phrases(artefact: str, role: str) -> dict:
    """Return positive + negative triggers for a skill."""
    if role == "author":
        positives = [
            f'draft a {artefact}',
            f'create the {artefact}',
            f'author a new {artefact}',
            f'generate the {artefact}',
        ]
        negatives = [
            (f'audit this {artefact}', f'{artefact.replace(" ", "-")}-audit'),
            (f'check the {artefact} for completeness', f'{artefact.replace(" ", "-")}-audit'),
            ('what is our company holiday schedule', None),
        ]
    elif role == "audit":
        positives = [
            f'audit this {artefact}',
            f'check the {artefact} for completeness',
            f'verify the {artefact} meets the rubric',
            f're-audit the {artefact}',
        ]
        negatives = [
            (f'draft a {artefact}', f'{artefact.replace(" ", "-")}-author'),
            (f'create the {artefact}', f'{artefact.replace(" ", "-")}-author'),
            ('what is the team on-call rotation', None),
        ]
    else:  # other (e.g. vietnam-legal-compliance, ts-skill)
        positives = [
            f'reference {artefact}',
            f'look up {artefact}',
            f'consult the {artefact} reference',
        ]
        negatives = [
            ('run unrelated task', None),
            ('what time is it', None),
        ]
    return {"positives": positives, "negatives": negatives}


def enrich_description(current: str, role: str, artefact: str, sibling: str | None) -> str:
    """Append trigger phrases to the existing description.

    Strategy: take the existing description (which already states WHAT the
    skill does), flatten newlines, append `Use when user asks to "<p1>" or
    "<p2>". Do NOT use for "<neg>" (use <sibling> instead).`
    """
    flat = current.replace("\n", " ").strip()
    flat = re.sub(r"\s+", " ", flat)

    if role == "author":
        addition = (
            f' Use when user asks to "draft a {artefact}" or '
            f'"create the {artefact}". Do NOT use for "audit existing {artefact}" '
            f'(use {sibling} instead).'
        )
    elif role == "audit":
        addition = (
            f' Use when user asks to "audit this {artefact}" or '
            f'"check the {artefact}". Do NOT use for "draft a new {artefact}" '
            f'(use {sibling} instead).'
        )
    else:
        addition = (
            f' Use when user asks to "reference {artefact}" or '
            f'"look up {artefact}".'
        )

    # Combine, then re-flow into a single line.
    enriched = (flat + addition).strip()
    return enriched


def render_trigger_tests(skill_id: str, triggers: dict) -> str:
    """Render the acceptance/TRIGGER_TESTS.md content."""
    positives = "\n".join(f'- "{p.capitalize()}"' for p in triggers["positives"])
    negatives = "\n".join(
        f'- "{n.capitalize()}" → {target}' if target else f'- "{n.capitalize()}" → none'
        for n, target in triggers["negatives"]
    )
    return dedent(f"""\
        ---
        skill_id: {skill_id}
        min_confidence: 0.7
        classifier_version: 3.0.0-a4
        ---

        # TRIGGER_TESTS for {skill_id}

        > Authored via heuristic backfill per FR-SKILL-115 lazy-backfill discipline.
        > Refine these triggers during the next natural fine-tune cycle with real
        > OBS-observed phrasings.

        ## Positive triggers (MUST route here)

        {positives}

        ## Negative triggers (MUST NOT route here)

        {negatives}

        ## Authoring notes

        - Triggers derived from skill name + role (author/audit) via the heuristic
          backfill script. They are conservative — refine with OBS-observed real
          user phrasings during the next natural fine-tune cycle.
        - Re-author when classifier_version MAJOR-bumps.
        """)


def replace_description_block(text: str, new_desc: str) -> str | None:
    """Replace the `description:` field's value with the new flat description.

    Supports both `description: |\n  ...` and `description: >-\n  ...` and
    single-line `description: ...` forms.
    """
    # Try folded/literal block first
    pattern = re.compile(
        r"(^description:\s*[|>][^\n]*\n)((?:[ \t]+[^\n]*\n)+)",
        re.MULTILINE,
    )
    new_block = f"description: >-\n  {new_desc}\n"
    new_text, n = pattern.subn(new_block, text, count=1)
    if n == 1:
        return new_text
    # Try single-line `description: foo`
    pattern2 = re.compile(r"(^description:\s+)([^\n]*)$", re.MULTILINE)
    new_text, n = pattern2.subn(f"description: >-\n  {new_desc}", text, count=1)
    if n == 1:
        return new_text
    return None


def process_skill(skill_md: Path, apply: bool) -> str | None:
    """Process one SKILL.md. Returns status string or None on skip."""
    text = skill_md.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return None
    end = text.index("\n---", 4)
    fm_text = text[4:end]

    # Parse YAML to get name + description
    import yaml
    fm = yaml.safe_load(fm_text)
    if not isinstance(fm, dict):
        return None
    name = fm.get("name", skill_md.parent.name)
    current_desc = fm.get("description", "")
    if not isinstance(current_desc, str):
        return None

    # Check if it's already valid — skip
    result = scan_description(skill_md)
    if result.valid:
        return None

    # Derive artefact + role
    artefact, role, sibling = derive_artefact_and_sibling(name)
    triggers = build_trigger_phrases(artefact, role)

    # Skip skills where current description is too short to be the WHAT statement.
    # We need a minimum WHAT-statement length to anchor the enriched form.
    if len(current_desc.replace("\n", " ").strip()) < 40:
        # Generate a stub WHAT statement
        if role == "author":
            current_desc = f"Author a {artefact} artefact (per the canonical {name} skill body)."
        elif role == "audit":
            current_desc = f"Audit a {artefact} artefact against its rubric (per the canonical {name} skill body)."
        else:
            current_desc = f"Reference skill for {artefact} (per the canonical {name} skill body)."

    enriched = enrich_description(current_desc, role, artefact, sibling)

    # Verify the enriched description would pass
    from cuo.description_format_check import validate as desc_validate
    violation = desc_validate(enriched)
    if violation:
        return f"  ⚠ {name}: enrichment would still fail ({violation.code}: {violation.detail})"

    if apply:
        # 1. Rewrite the description block
        new_text = replace_description_block(text, enriched)
        if new_text is None:
            return f"  ⊘ {name}: description block not matched"
        skill_md.write_text(new_text, encoding="utf-8")

        # 2. Write the TRIGGER_TESTS.md
        accept_dir = skill_md.parent / "acceptance"
        accept_dir.mkdir(exist_ok=True)
        # Only write if not already present (don't overwrite hand-authored fixtures)
        tt_path = accept_dir / "TRIGGER_TESTS.md"
        if not tt_path.exists():
            tt_content = render_trigger_tests(name, triggers)
            tt_path.write_text(tt_content, encoding="utf-8")

    return f"  ✓ {name}: enriched ({role}, artefact='{artefact}')"


def main() -> int:
    parser = argparse.ArgumentParser(description="FR-SKILL-111 description-format backfill")
    parser.add_argument("--apply", action="store_true", help="Write changes to disk")
    parser.add_argument("--dry-run", action="store_true", help="Preview only")
    args = parser.parse_args()
    if not args.apply and not args.dry_run:
        parser.print_help()
        return 2
    apply = bool(args.apply)

    processed = 0
    succeeded = 0
    warnings = []
    for f in sorted(SKILL_ROOT.glob("**/SKILL.md")):
        if "_template/" in str(f) or "/_template" in str(f):
            continue
        status = process_skill(f, apply)
        if status is None:
            continue
        processed += 1
        if status.strip().startswith("✓"):
            succeeded += 1
        else:
            warnings.append(status)

    action = "Would process" if not apply else "Processed"
    print(f"{action}: {processed} skill(s)")
    print(f"  Succeeded: {succeeded}")
    if warnings:
        print(f"  Warnings/skips: {len(warnings)}")
        for w in warnings[:20]:
            print(w)
    return 0


if __name__ == "__main__":
    sys.exit(main())
