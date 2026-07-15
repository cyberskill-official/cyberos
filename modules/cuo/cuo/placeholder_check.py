"""Placeholder-syntax detector for TASK-SKILL-115 / SKB-030.

Identifies stale `<placeholder>` tokens in SKILL.md frontmatter values.

Distinct from SKB-040 (no-xml-in-frontmatter, the security boundary):
SKB-030 targets the operator-UX + portability boundary — template-scaffold
leftovers like `<SDP §2 stage letter or "cross">` that never got substituted
with real values.

Used by:
- Standalone CLI: `python -m cuo.placeholder_check <skill_path>` or
  `python -m cuo.placeholder_check --catalog modules/skill/`
- CI gate: integrates with the existing CUO pytest suite

Per TASK-SKILL-115 §3.
"""

from __future__ import annotations

import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable

import yaml


# Match `<token>` where token contains the literal characters that appear in
# real CyberOS placeholder syntax. Includes §, /, ", spaces, parens etc.
PLACEHOLDER_RE = re.compile(r"<([a-zA-Z][a-zA-Z0-9_§ /\"|.()\-]*)>")

# Tags that ARE valid HTML / mermaid + should never be flagged as placeholders.
SAFE_TAGS = frozenset({"br", "b", "i", "em", "strong", "sub", "sup", "span", "div"})

# Paths exempt from the rule (contract scaffolds that intentionally use placeholders).
EXEMPT_PATH_PARTS = ("_template/", "/_template")


@dataclass(frozen=True)
class PlaceholderHit:
    field_path: str
    value: str
    token: str


@dataclass(frozen=True)
class ScanResult:
    skill_path: str
    exempt: bool
    hits: list[PlaceholderHit] = field(default_factory=list)
    error: str | None = None

    @property
    def passed(self) -> bool:
        return self.exempt or (not self.hits and self.error is None)


def _walk_value(field_path: str, value, hits: list[PlaceholderHit]) -> None:
    """Recursively walk a YAML value tree and append placeholder hits."""
    if isinstance(value, str):
        for m in PLACEHOLDER_RE.finditer(value):
            tok = m.group(1)
            # Whitelist safe HTML tags
            if tok.lower() in SAFE_TAGS:
                continue
            hits.append(PlaceholderHit(
                field_path=field_path,
                value=value[:200],
                token=tok,
            ))
    elif isinstance(value, dict):
        for k, v in value.items():
            _walk_value(f"{field_path}.{k}", v, hits)
    elif isinstance(value, list):
        for i, v in enumerate(value):
            _walk_value(f"{field_path}[{i}]", v, hits)


def scan(skill_path: Path) -> ScanResult:
    """Scan one SKILL.md for stale placeholders.

    Returns ScanResult with exempt=True for paths under _template/.
    """
    path_str = str(skill_path)
    exempt = any(p in path_str for p in EXEMPT_PATH_PARTS)

    if not skill_path.exists():
        return ScanResult(path_str, exempt=False, error="file_missing")

    text = skill_path.read_text(encoding="utf-8")
    if not text.startswith("---\n"):
        return ScanResult(path_str, exempt=exempt, error="no_frontmatter")

    try:
        end = text.index("\n---\n", 4)
    except ValueError:
        return ScanResult(path_str, exempt=exempt, error="no_closing_frontmatter_delimiter")

    try:
        fm = yaml.safe_load(text[4:end])
    except yaml.YAMLError as e:
        return ScanResult(path_str, exempt=exempt, error=f"yaml_parse: {e}")
    if not isinstance(fm, dict):
        return ScanResult(path_str, exempt=exempt, error="frontmatter_not_dict")

    hits: list[PlaceholderHit] = []
    _walk_value("root", fm, hits)

    # Exempt paths report their hits as 0 (regardless of what they actually have);
    # the rule does not fire on scaffolds.
    if exempt:
        return ScanResult(path_str, exempt=True)
    return ScanResult(path_str, exempt=False, hits=hits)


def run_all(catalog_root: Path) -> dict[str, ScanResult]:
    """Walk catalog_root and scan every SKILL.md. Returns dict keyed by skill path."""
    results: dict[str, ScanResult] = {}
    for f in sorted(catalog_root.glob("**/SKILL.md")):
        result = scan(f)
        results[str(f.relative_to(catalog_root.parent) if f.is_absolute() else f)] = result
    return results


def summarize(results: dict[str, ScanResult], status_filter: str | None = None) -> str:
    """Produce a human-readable summary of run_all() results.

    `status_filter` (optional): if provided, only count skills at that frontmatter
    status. (Not yet wired — placeholder for future status-based gating per TASK-115 §1 #11.)
    """
    total = len(results)
    exempt = sum(1 for r in results.values() if r.exempt)
    errors = [r for r in results.values() if r.error and not r.exempt]
    with_hits = [r for r in results.values() if r.hits]
    clean = total - exempt - len(errors) - len(with_hits)

    lines = [
        f"Scanned: {total} SKILL.md file(s)",
        f"  Clean (zero hits): {clean}",
        f"  Exempt (_template/): {exempt}",
        f"  Parse errors: {len(errors)}",
        f"  With placeholder hits: {len(with_hits)}",
    ]
    if with_hits:
        lines.append("")
        lines.append("Files with stale placeholders (top 20):")
        for r in with_hits[:20]:
            lines.append(f"  {r.skill_path}: {len(r.hits)} hit(s)")
            for h in r.hits[:3]:
                lines.append(f"    {h.field_path}: <{h.token}>")
            if len(r.hits) > 3:
                lines.append(f"    ... + {len(r.hits) - 3} more")
    if errors:
        lines.append("")
        lines.append("Parse errors:")
        for r in errors[:10]:
            lines.append(f"  {r.skill_path}: {r.error}")
    return "\n".join(lines)


def main(argv: list[str] | None = None) -> int:
    argv = argv if argv is not None else sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help"):
        print("Usage: python -m cuo.placeholder_check <skill_path>")
        print("       python -m cuo.placeholder_check --catalog <catalog_root>")
        print("       python -m cuo.placeholder_check --catalog <catalog_root> --json")
        return 2

    if argv[0] == "--catalog":
        if len(argv) < 2:
            print("Error: --catalog requires a path argument", file=sys.stderr)
            return 2
        catalog_root = Path(argv[1])
        if not catalog_root.exists():
            print(f"Error: catalog root not found: {catalog_root}", file=sys.stderr)
            return 2
        results = run_all(catalog_root)
        if "--json" in argv:
            payload = {
                "total": len(results),
                "exempt": sum(1 for r in results.values() if r.exempt),
                "with_hits": sum(1 for r in results.values() if r.hits),
                "errors": sum(1 for r in results.values() if r.error and not r.exempt),
                "skills": {
                    p: {
                        "exempt": r.exempt,
                        "error": r.error,
                        "hits": [
                            {"field": h.field_path, "value": h.value, "token": h.token}
                            for h in r.hits
                        ],
                    }
                    for p, r in results.items()
                    if r.hits or r.error
                },
            }
            print(json.dumps(payload, indent=2))
        else:
            print(summarize(results))
        any_hits = any(r.hits for r in results.values())
        return 0 if not any_hits else 1

    # Single-skill mode
    skill_path = Path(argv[0])
    result = scan(skill_path)
    if result.exempt:
        print(f"✓ {skill_path}: exempt (under _template/)")
        return 0
    if result.error:
        print(f"✗ {skill_path}: {result.error}")
        return 1
    if not result.hits:
        print(f"✓ {skill_path}: clean (no stale placeholders)")
        return 0
    print(f"✗ {skill_path}: {len(result.hits)} placeholder hit(s):")
    for h in result.hits:
        print(f"    {h.field_path}: <{h.token}>")
    return 1


if __name__ == "__main__":
    sys.exit(main())
