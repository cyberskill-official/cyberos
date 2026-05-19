#!/usr/bin/env python3
"""tools/sweep-placeholders/detect.py — FR-SKILL-115 catalog scanner.

Thin CLI wrapper around `cuo.placeholder_check`. Walks `modules/skill/`,
parses every SKILL.md frontmatter, identifies stale `<placeholder>` tokens.

Usage:
    python3 detect.py                       # Plain summary to stdout
    python3 detect.py --json                # Machine-readable JSON
    python3 detect.py --report              # Generate the report markdown
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
CATALOG_ROOT = REPO_ROOT / "modules" / "skill"

# Make cuo.placeholder_check importable without installing the package
sys.path.insert(0, str(REPO_ROOT / "modules" / "cuo"))
from cuo.placeholder_check import run_all, summarize  # noqa: E402


def main() -> int:
    p = argparse.ArgumentParser(description="FR-SKILL-115 placeholder detector")
    p.add_argument("--json", action="store_true", help="JSON output for machine consumption")
    p.add_argument("--report", action="store_true", help="Generate operator-reviewable report.md")
    p.add_argument("--catalog", type=Path, default=CATALOG_ROOT, help=f"Catalog root (default: {CATALOG_ROOT})")
    args = p.parse_args()

    results = run_all(args.catalog)

    if args.report:
        # Delegate to suggest.py's report generator
        from suggest import generate_report
        out = generate_report(results, args.catalog)
        # Save to tools/sweep-placeholders/report-YYYY-MM-DD.md
        from datetime import date
        report_path = Path(__file__).parent / f"report-{date.today().isoformat()}.md"
        report_path.write_text(out, encoding="utf-8")
        print(f"Report written: {report_path}")
        # Exit 0 even if hits exist — the report is the deliverable
        return 0

    if args.json:
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
                if r.hits or (r.error and not r.exempt)
            },
        }
        print(json.dumps(payload, indent=2))
    else:
        print(summarize(results))

    any_hits = any(r.hits for r in results.values())
    return 0 if not any_hits else 1


if __name__ == "__main__":
    sys.exit(main())
