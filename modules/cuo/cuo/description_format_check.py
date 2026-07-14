"""Description-format validator — Python mirror of the Rust description_validator.

Per TASK-SKILL-111 SKB-020..023:
- Length 80-1024 chars (flattened single-line equivalent).
- No unescaped < or > characters.
- ≥1 verb stem from the canonical list.
- ≥2 quoted trigger phrases (negative triggers don't count).

Used to verify the 104-pair catalog before the Rust broker is built.
"""

from __future__ import annotations

import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

import yaml


# Mirrors VERB_STEMS in services/skill-broker/src/frontmatter/description_validator.rs
VERB_STEMS = re.compile(
    r"(?i)\b(generate|author|audit|review|draft|emit|build|propose|render|extract|"
    r"classify|tag|score|track|enforce|validate|orchestrate|chain|select|pin|halt|"
    r"resume|escalate|wrap|publish|deliver|test|simulate|"
    r"translate|convert|provide|reference|describe|surface|run|compute|export|"
    r"summarise|summarize|notify|capture|persist|reconcile|reject)\b"
)
QUOTED_TRIGGER = re.compile(r'"([^"]{1,80})"')
NEGATIVE_PREFIX = re.compile(r"(?i)\bdo\s+not\s+use\s+(for|when|with)\b")
# SKB-021: forbid XML-tag-shaped patterns (e.g. `<foo>`, `<foo/>`, `</foo>`).
# A bare `>` or `<` math operator is acceptable; only paired XML-tag shapes
# trigger the system-prompt-injection concern Anthropic Reference B cites.
XML_TAG_PATTERN = re.compile(r"</?[a-zA-Z][a-zA-Z0-9_-]*(?:\s+[^<>]*?)?\s*/?>")
DESCRIPTION_MIN_LEN = 80
DESCRIPTION_MAX_LEN = 1024


@dataclass(frozen=True)
class DescriptionViolation:
    code: str  # too_short / too_long / forbidden_brackets / missing_what / insufficient_triggers
    detail: str


@dataclass(frozen=True)
class DescriptionResult:
    skill_path: str
    description: str
    flat_length: int
    valid: bool
    violation: DescriptionViolation | None = None
    positive_trigger_count: int = 0


def validate(description: str) -> DescriptionViolation | None:
    flat = description.replace("\n", " ").strip()
    length = len(flat)
    if length < DESCRIPTION_MIN_LEN:
        return DescriptionViolation("too_short", f"len={length} < {DESCRIPTION_MIN_LEN}")
    if length > DESCRIPTION_MAX_LEN:
        return DescriptionViolation("too_long", f"len={length} > {DESCRIPTION_MAX_LEN}")
    if XML_TAG_PATTERN.search(flat):
        return DescriptionViolation("forbidden_brackets", "contains an XML-tag-shaped pattern (e.g. <foo>)")
    if not VERB_STEMS.search(flat):
        return DescriptionViolation("missing_what", "no verb stem from canonical list")

    # Count quoted phrases excluding negative-trigger preamble (40-char lookback)
    positives = 0
    for m in QUOTED_TRIGGER.finditer(flat):
        preceding = flat[: m.start()]
        window = preceding[-40:]
        if NEGATIVE_PREFIX.search(window):
            continue
        positives += 1
    if positives < 2:
        return DescriptionViolation(
            "insufficient_triggers",
            f"found {positives} quoted positive trigger(s), need 2",
        )
    return None


def scan(skill_md: Path) -> DescriptionResult:
    text = skill_md.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return DescriptionResult(
            str(skill_md), "", 0, False,
            DescriptionViolation("no_frontmatter", "missing leading ---"),
        )
    end = text.index("\n---", 4)
    fm = yaml.safe_load(text[4:end])
    desc = (fm or {}).get("description", "")
    flat = desc.replace("\n", " ").strip()
    violation = validate(desc)
    positives = 0
    if not violation or violation.code == "insufficient_triggers":
        for m in QUOTED_TRIGGER.finditer(flat):
            preceding = flat[: m.start()]
            window = preceding[-40:]
            if not NEGATIVE_PREFIX.search(window):
                positives += 1
    return DescriptionResult(
        str(skill_md), desc, len(flat), violation is None, violation, positives,
    )


def run_all(catalog_root: Path) -> dict[str, DescriptionResult]:
    results: dict[str, DescriptionResult] = {}
    for f in sorted(catalog_root.glob("**/SKILL.md")):
        if "_template/" in str(f) or "/_template" in str(f):
            continue
        results[str(f.relative_to(catalog_root.parent) if f.is_absolute() else f)] = scan(f)
    return results


def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    if not argv:
        print("Usage: python -m cuo.description_format_check <skill_path>")
        print("       python -m cuo.description_format_check --catalog <root>")
        return 2

    if argv[0] == "--catalog":
        if len(argv) < 2:
            print("Error: --catalog requires a path", file=sys.stderr)
            return 2
        results = run_all(Path(argv[1]))
        total = len(results)
        valid = sum(1 for r in results.values() if r.valid)
        invalid = total - valid
        print(f"Scanned: {total} SKILL.md file(s)")
        print(f"  Valid (SKB-020..023): {valid}")
        print(f"  Invalid: {invalid}")
        if invalid:
            print()
            print("Violations by code:")
            from collections import Counter
            codes = Counter(r.violation.code for r in results.values() if r.violation)
            for code, count in codes.most_common():
                print(f"  {count}× {code}")
        return 0 if invalid == 0 else 1

    result = scan(Path(argv[0]))
    if result.valid:
        print(f"✓ {result.skill_path}: valid ({result.flat_length} chars, {result.positive_trigger_count} positive triggers)")
        return 0
    print(f"✗ {result.skill_path}: {result.violation.code} — {result.violation.detail}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
