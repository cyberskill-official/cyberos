#!/usr/bin/env python3
"""
cyberos_ref_from_drift.py — pre-fill a REF from a drift candidate.

Batch 12 (Tier B) of post-catalog improvements.

Reads a `memories/drift/*.md` file (typically auto-written by the
Aspect 3.1 Stop-hook), extracts the trigger pattern + suggested AGENTS
section, and stages a REF draft at `.cyberos-memory/staging/REF-NNN-...md`
with: Trigger / Tier / AGENTS section / capability+regression eval
skeletons / Implementation steps placeholder.

If `--with-llm` is set and ANTHROPIC_API_KEY is available, the body of
the REF is drafted by Claude from the drift candidate; otherwise the
body is a structural skeleton the operator fills in.

Usage:
    cyberos ref-from-drift memories/drift/2026-05-12-refinement-candidate-repeated-revert-any.md
    cyberos ref-from-drift drift-*.md --with-llm --tier 2
"""
from __future__ import annotations
import argparse
import os
import re
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def next_ref_nnn(brain_root: Path) -> str:
    d = brain_root / ".cyberos-memory" / "memories" / "refinements"
    max_n = 0
    if d.exists():
        for f in d.glob("REF-*.md"):
            m = re.match(r"^REF-(\d+)", f.name)
            if m:
                max_n = max(max_n, int(m.group(1)))
    return f"{max_n + 1:03d}"


def llm_draft(drift_text: str, tier: int) -> str:
    try:
        import anthropic  # type: ignore
    except ImportError:
        return ""
    if not os.environ.get("ANTHROPIC_API_KEY"):
        return ""
    client = anthropic.Anthropic()
    prompt = f"""You are drafting a CyberOS BRAIN protocol refinement (REF) from a drift
candidate the Stop-hook auto-detected. Read the drift candidate below and
produce a REF body following this structure:

# REF-NNN <short title>
## Trigger
<2-3 sentences describing the signal>
## Tier
TIER {tier} (justify)
## AGENTS.md section
<§-anchor>
## Exact prose to insert
<the literal text to add to AGENTS.md>
## Capability eval
- What new behavior:
- Test fixture:
- Pass criteria:
## Regression eval
- What to verify:
- Test fixture:
- Pass criteria:
## Implementation steps
1. ...
2. ...

Drift candidate:
---
{drift_text}
---

Respond with ONLY the REF body (no extra commentary). ≤ 400 words. No em
dashes. No marketing language.
"""
    try:
        msg = client.messages.create(
            model="claude-sonnet-4-6",
            max_tokens=1200,
            messages=[{"role": "user", "content": prompt}],
        )
        return "\n".join(b.text for b in msg.content if hasattr(b, "text"))
    except Exception as e:
        return f"_(LLM draft failed: {e})_"


def skeleton(drift_text: str, tier: int, nnn: str, slug: str) -> str:
    return f"""# REF-{nnn} {slug.replace('-', ' ').title()}

## Trigger
_(Extracted from drift candidate; rewrite in operator voice.)_

{drift_text[:600]}

## Tier
TIER {tier} (justify here; tier 1 = AGENTS.md text edit; tier 2 = manifest field; tier 3 = §0.5 protocol pin bump)

## AGENTS.md section
§TBD

## Exact prose to insert
<insert exact text>

## Capability eval
- **What new behavior:**
- **Test fixture:** runtime/tests/refinements/REF-{nnn}/capability.test.py
- **Pass criteria:**

## Regression eval
- **What to verify:** all existing memories still validate; chain LINK invariant preserved
- **Test fixture:** runtime/tests/refinements/REF-{nnn}/regression.test.py
- **Pass criteria:** cyberos verify returns 0 CRITICAL after fix

## Implementation steps
1.
2.

## Related
- Drift candidate source: <path>
"""


def main():
    p = argparse.ArgumentParser(description="pre-fill a REF from a drift candidate (Batch 12 / Tier B)")
    p.add_argument("drift_path", help="path to memories/drift/*.md candidate")
    p.add_argument("--tier", type=int, default=2)
    p.add_argument("--slug", help="kebab-case slug; auto-derived from drift if absent")
    p.add_argument("--with-llm", action="store_true", help="use anthropic SDK to draft the body")
    p.add_argument("--out", help="custom output path; default .cyberos-memory/staging/")
    args = p.parse_args()

    brain_root = find_brain()
    drift_path = Path(args.drift_path)
    if not drift_path.exists():
        drift_path = brain_root / ".cyberos-memory" / args.drift_path
    if not drift_path.exists():
        print(f"  no such drift file: {args.drift_path}", file=sys.stderr)
        return 2
    drift_text = drift_path.read_text(encoding="utf-8")

    nnn = next_ref_nnn(brain_root)
    # Derive slug from drift filename
    slug = args.slug or re.sub(r"^[\d-]+(refinement-candidate-)?", "", drift_path.stem).strip("-") or "drift-derived"

    body = ""
    if args.with_llm:
        body = llm_draft(drift_text, args.tier)
    if not body:
        body = skeleton(drift_text, args.tier, nnn, slug)

    out = Path(args.out) if args.out else (brain_root / "outputs" / "staged-memories" / f"REF-{nnn}-{slug}.md")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(body, encoding="utf-8")
    print(f"  ✓ staged REF draft: {out.relative_to(brain_root)}")
    print(f"  Edit, then commit via: cyberos add REF --slug {slug} (or brain_writer write directly)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
