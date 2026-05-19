#!/usr/bin/env python3
"""Tighten description fields that exceed the 1024-char limit post-backfill.

Strategy:
- Find skills where description > 1024 chars (flat).
- Locate the FIRST `Use when user asks to "..." or "..."` phrase — that's
  the trigger boilerplate we added in backfill.py.
- Keep everything before that (the WHAT statement), truncate to ~750 chars
  if needed, then re-attach the trigger boilerplate.
- Final length should fit in 1024.

Run after `backfill.py --apply`.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SKILL_ROOT = REPO_ROOT / "modules" / "skill"

sys.path.insert(0, str(REPO_ROOT / "modules" / "cuo"))
from cuo.description_format_check import scan as scan_description  # noqa: E402

TRIGGER_BOILERPLATE_RE = re.compile(r"\s*Use when user asks to\b", re.IGNORECASE)
MAX_WHAT_LEN = 700  # leaves ~300 chars for trigger boilerplate; total ~1000 < 1024


def tighten_description(desc: str) -> str:
    """Trim the WHAT statement so total stays <= 1024."""
    flat = desc.replace("\n", " ").strip()
    flat = re.sub(r"\s+", " ", flat)

    # Locate trigger-boilerplate start
    m = TRIGGER_BOILERPLATE_RE.search(flat)
    if not m:
        # No trigger boilerplate yet — just truncate
        return flat[:1000].rstrip() + "..."

    what_part = flat[: m.start()].rstrip()
    trigger_part = flat[m.start():].strip()

    if len(what_part) > MAX_WHAT_LEN:
        # Truncate WHAT at the last sentence end before MAX_WHAT_LEN
        truncated = what_part[:MAX_WHAT_LEN]
        # Find the last full-stop or em-dash before the cut, fall back to bare truncation
        last_stop = max(truncated.rfind(". "), truncated.rfind(" — "), truncated.rfind("; "))
        if last_stop > 200:
            truncated = truncated[: last_stop + 1]
        else:
            truncated = truncated.rstrip() + "."
        what_part = truncated

    return f"{what_part} {trigger_part}".strip()


def replace_description_block(text: str, new_desc: str) -> str | None:
    pattern = re.compile(
        r"(^description:\s*[|>][^\n]*\n)((?:[ \t]+[^\n]*\n)+)",
        re.MULTILINE,
    )
    new_block = f"description: >-\n  {new_desc}\n"
    new_text, n = pattern.subn(new_block, text, count=1)
    if n == 1:
        return new_text
    return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--apply", action="store_true")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    apply = bool(args.apply)
    if not args.apply and not args.dry_run:
        parser.print_help()
        return 2

    fixed = 0
    skipped = 0
    for f in sorted(SKILL_ROOT.glob("**/SKILL.md")):
        if "_template/" in str(f) or "/_template" in str(f):
            continue
        result = scan_description(f)
        if result.valid:
            continue
        if result.violation.code != "too_long":
            # Different problem — leave for separate sweep
            skipped += 1
            continue

        new_desc = tighten_description(result.description)
        if len(new_desc) > 1024:
            # Still too long — strip more aggressively
            new_desc = new_desc[:1020].rstrip() + "..."
        if apply:
            text = f.read_text(encoding="utf-8")
            new_text = replace_description_block(text, new_desc)
            if new_text:
                f.write_text(new_text, encoding="utf-8")
                fixed += 1
                print(f"  ✓ {f.parent.name}: {len(result.description.replace(chr(10), ' ').strip())} → {len(new_desc)} chars")
            else:
                print(f"  ⊘ {f.parent.name}: replacement failed")
        else:
            print(f"  would: {f.parent.name}: {len(result.description.replace(chr(10), ' ').strip())} → {len(new_desc)} chars")
            fixed += 1

    print()
    action = "Would tighten" if not apply else "Tightened"
    print(f"{action}: {fixed} skill(s); {skipped} skipped (non-too_long violations)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
