#!/usr/bin/env python3
"""tools/sweep-placeholders/suggest.py — FR-SKILL-115 substitution proposer.

Per FR-SKILL-115 §1 #3-#4: read each skill's body + sibling files + propose
a substitution per placeholder. Suggestions are ADVISORY — operator review +
edit is mandatory before sweep is applied.

Heuristics per field:
- `metadata.stage`: grep body for SDP §2 stage letter references (`stage b`,
  `stage c`, `stage d`, etc.). Most-common stage = suggestion. Ties or
  cross-cutting → `"cross"`.
- `description`: read CONTRACT_ECHO template_version; substitute `<input>` →
  `{artefact} source`, `<artifact>` → `{artefact}`, `<ARTIFACT>` → upper.
- `allowed_brain_scopes.write[*]`: replace bare placeholders like `<fr_id>`
  or `<run_id>` with `project:*` (broad write scope — operator narrows).

Usage:
    python3 suggest.py <skill_path>     # Per-skill suggestion
    python3 suggest.py --catalog        # Generate full report.md
"""

from __future__ import annotations

import argparse
import re
import sys
from collections import Counter
from dataclasses import dataclass
from datetime import date
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CATALOG_ROOT = REPO_ROOT / "modules" / "skill"

sys.path.insert(0, str(REPO_ROOT / "modules" / "cuo"))
from cuo.placeholder_check import PlaceholderHit, ScanResult, run_all, scan  # noqa: E402


# ─── Suggesters ─────────────────────────────────────────────────────────────────

SDP_STAGE_RE = re.compile(r"\bstage[s]?\s+([a-h])\b", re.IGNORECASE)
SDP_SECTION_RE = re.compile(r"§\s*2\s*[\(\.]?([a-h])\)?", re.IGNORECASE)
TEMPLATE_VERSION_RE = re.compile(r"template_version:\s+([a-z][a-z_0-9-]*)@\d+", re.IGNORECASE)
PROMPT_REVISION_RE = re.compile(r"prompt_revision:\s+([a-z][a-z_0-9-]*?)_(?:author|audit)@", re.IGNORECASE)


def suggest_stage(skill_dir: Path) -> tuple[str, str]:
    """Propose metadata.stage. Returns (suggestion, rationale)."""
    skill_path = skill_dir / "SKILL.md"
    if not skill_path.exists():
        return ("?", "skill file missing — cannot infer")
    text = skill_path.read_text(encoding="utf-8")
    # Strip frontmatter — look only at body
    body = text
    if body.startswith("---\n"):
        try:
            end = body.index("\n---\n", 4)
            body = body[end + 5:]
        except ValueError:
            pass

    # Find SDP stage references
    stages = SDP_STAGE_RE.findall(body) + SDP_SECTION_RE.findall(body)
    if not stages:
        return ("cross", "no SDP stage references in body — defaulting to cross-cutting")
    counts = Counter(s.lower() for s in stages)
    if len(counts) > 1:
        # Multiple stages → cross-cutting
        top = counts.most_common()
        return ("cross", f"body references multiple stages {[s for s, _ in top]} — cross-cutting")
    stage = counts.most_common(1)[0][0]
    return (stage, f"body references SDP stage {stage} ({counts[stage]}× mentions)")


def suggest_description_substitution(skill_dir: Path, original: str) -> tuple[str, str]:
    """Propose substitutions for <input> / <artifact> / <ARTIFACT> in description."""
    skill_path = skill_dir / "SKILL.md"
    if not skill_path.exists():
        return (original, "skill file missing")
    text = skill_path.read_text(encoding="utf-8")

    # Try template_version first, then prompt_revision
    m = TEMPLATE_VERSION_RE.search(text)
    artefact = m.group(1) if m else None
    if not artefact:
        m = PROMPT_REVISION_RE.search(text)
        artefact = m.group(1) if m else None

    if not artefact:
        # Fall back to the skill's folder name minus -author/-audit suffix
        folder = skill_dir.name
        artefact = folder.removesuffix("-author").removesuffix("-audit")

    new_value = original
    new_value = new_value.replace("<input artefact(s)>", f"{artefact} source artefact(s)")
    new_value = new_value.replace("<input>", f"{artefact} source")
    new_value = new_value.replace("<artifact>", artefact)
    new_value = new_value.replace("<ARTIFACT>", artefact.upper())
    new_value = new_value.replace("<reason>", "user-provided reason")
    return (new_value, f"derived artefact name '{artefact}' from template_version / prompt_revision / folder name")


def suggest_brain_scope(token: str) -> tuple[str, str]:
    """Propose a substitution for a BRAIN-scope glob placeholder."""
    # Common patterns from real CyberOS skills
    mapping = {
        "fr_id": "project:fr/*",
        "run_id": "project:runs/*",
        "skill_id": "project:skills/*",
        "trace_id": "project:traces/*",
        "session_id": "project:sessions/*",
    }
    if token in mapping:
        return (mapping[token], f"canonical scope glob for {token}")
    # Default conservative suggestion
    return (f"project:{token}/*", f"conservative glob for unknown placeholder {token!r}; operator should narrow")


# ─── Single-skill suggestion entry point ────────────────────────────────────────

@dataclass(frozen=True)
class Suggestion:
    field_path: str
    original: str
    proposed: str
    rationale: str


def suggest_for_skill(skill_dir: Path) -> list[Suggestion]:
    """Run all suggesters on a skill's placeholder hits."""
    skill_path = skill_dir / "SKILL.md"
    result = scan(skill_path)
    if result.exempt or result.error or not result.hits:
        return []

    suggestions: list[Suggestion] = []
    for hit in result.hits:
        if hit.field_path == "root.metadata.stage":
            sug, why = suggest_stage(skill_dir)
            suggestions.append(Suggestion(hit.field_path, hit.value, sug, why))
        elif hit.field_path == "root.description":
            sug, why = suggest_description_substitution(skill_dir, hit.value)
            suggestions.append(Suggestion(hit.field_path, hit.value, sug, why))
        elif hit.field_path.startswith("root.allowed_brain_scopes."):
            sug, why = suggest_brain_scope(hit.token)
            suggestions.append(Suggestion(hit.field_path, f"<{hit.token}>", sug, why))
        else:
            # Generic fallback
            suggestions.append(Suggestion(
                hit.field_path,
                hit.value,
                "(no automated suggestion; operator must hand-edit)",
                "field path not covered by suggester — manual review required",
            ))
    return suggestions


# ─── Report generator ───────────────────────────────────────────────────────────

def generate_report(results: dict, catalog_root: Path) -> str:
    """Produce the operator-reviewable report.md per FR-SKILL-115 §3."""
    lines = [
        f"# FR-SKILL-115 sweep report — {date.today().isoformat()}",
        "",
        "> **Generated:** by `tools/sweep-placeholders/detect.py --report`",
        "> **Per:** FR-SKILL-115 §1 #3-#4 (advisory suggestions; operator review mandatory before sweep).",
        "> **Workflow:**",
        "> 1. Read this report top-to-bottom.",
        "> 2. For each skill, accept the suggested substitution OR override.",
        "> 3. Commit the edits in persona-grouped batches (see §1 #7).",
        "> 4. Run `python3 tools/sweep-placeholders/detect.py` to verify zero residual hits.",
        "",
        "## Summary",
        "",
    ]

    total = len(results)
    with_hits = [r for r in results.values() if r.hits]
    exempt = sum(1 for r in results.values() if r.exempt)
    lines.extend([
        f"- Total SKILL.md scanned: {total}",
        f"- Exempt (`_template/`): {exempt}",
        f"- With stale placeholders: {len(with_hits)}",
        f"- Total placeholder tokens to substitute: {sum(len(r.hits) for r in with_hits)}",
        "",
    ])

    # Group by persona-like prefix for batch planning
    # CyberOS skills are flat at modules/skill/<skill-name>/SKILL.md — no persona
    # prefix in the path. Group by the first heuristic-letter-of-author/auditor.
    # Simpler: sort alphabetically + chunk by 15-20 for batch commits.
    lines.extend([
        "## Per-skill substitutions",
        "",
        "Each entry below: skill path → field → current value → proposed substitution → rationale.",
        "**Operator action:** for each row, either (a) accept the suggestion as-is in your sweep, or",
        "(b) override with a specific value in your sweep commit. Multi-line values are flattened for display.",
        "",
    ])

    for skill_rel_path in sorted(results.keys()):
        result = results[skill_rel_path]
        if not result.hits:
            continue
        skill_dir = catalog_root.parent / skill_rel_path
        skill_dir = skill_dir.parent  # strip the SKILL.md filename
        suggestions = suggest_for_skill(skill_dir)

        lines.append(f"### {skill_rel_path}")
        lines.append("")
        for s in suggestions:
            flat_orig = s.original.replace("\n", " ").strip()
            flat_orig = (flat_orig[:80] + "…") if len(flat_orig) > 80 else flat_orig
            flat_prop = s.proposed.replace("\n", " ").strip()
            flat_prop = (flat_prop[:80] + "…") if len(flat_prop) > 80 else flat_prop
            lines.append(f"- **{s.field_path}**")
            lines.append(f"  - **current:** `{flat_orig}`")
            lines.append(f"  - **suggest:** `{flat_prop}`")
            lines.append(f"  - **why:** {s.rationale}")
        lines.append("")

    lines.extend([
        "---",
        "",
        "## Next steps (per FR-SKILL-115 §1 #7)",
        "",
        "1. Review each suggestion in this report. Adjust where the suggester guessed wrong.",
        "2. Group skills by persona for batch commits (P0 cpo+cto first → P1 → P2+).",
        "3. Run sweep commits with operator-attested rationale per FR-SKILL-115 §1 #10.",
        "4. After all batches land: `python3 tools/sweep-placeholders/detect.py` exits 0.",
        "5. Bump registry version v0.2.5 → v0.2.6 in repo-root `CHANGELOG.md`.",
        "",
    ])
    return "\n".join(lines)


# ─── CLI ────────────────────────────────────────────────────────────────────────

def main() -> int:
    p = argparse.ArgumentParser(description="FR-SKILL-115 substitution suggester")
    p.add_argument("skill_path", nargs="?", help="Path to one skill folder")
    p.add_argument("--catalog", action="store_true", help="Generate the full report.md")
    args = p.parse_args()

    if args.catalog:
        results = run_all(CATALOG_ROOT)
        report = generate_report(results, CATALOG_ROOT)
        report_path = Path(__file__).parent / f"report-{date.today().isoformat()}.md"
        report_path.write_text(report, encoding="utf-8")
        print(f"Report written: {report_path}")
        return 0

    if not args.skill_path:
        p.print_help()
        return 2

    skill_dir = Path(args.skill_path)
    suggestions = suggest_for_skill(skill_dir)
    if not suggestions:
        print(f"{skill_dir.name}: no placeholders found (or exempt).")
        return 0
    for s in suggestions:
        print(f"Field: {s.field_path}")
        print(f"  Current: {s.original}")
        print(f"  Suggest: {s.proposed}")
        print(f"  Why:     {s.rationale}")
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
