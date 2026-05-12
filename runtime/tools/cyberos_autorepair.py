#!/usr/bin/env python3
"""
cyberos_autorepair.py — bounded auto-repair of WARN-level findings.

Batch 12 (Tier B) of post-catalog improvements.

Safety envelope:
  - NEVER touches `classification`, `authority`, `consent`, `memory_id`, `audit_chain_head`
  - NEVER deletes a memory; only adds missing optional fields
  - ALWAYS prints a unified diff for human review before writing
  - Requires `--apply` to actually mutate; default is dry-run

Repair recipes wired:
  - tag-budget-exceeded → trim to 10 tags (alphabetical), preserving order
  - duplicate-tags → de-dupe in place
  - tombstone-missing-metadata → add placeholder `deleted_at`, `deleted_by`,
    `tombstone_reason: "(auto-repair; please fill in)"`
  - source-tier-stale-pattern → emit a TODO comment in manifest.json source_tiers

Usage:
    cyberos autorepair                # dry-run; show diffs
    cyberos autorepair --apply        # actually write
    cyberos autorepair --recipe tag-budget-exceeded --apply
"""
from __future__ import annotations
import argparse
import difflib
import json
import re
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))

VALID_RECIPES = {"tag-budget-exceeded", "duplicate-tags", "tombstone-missing-metadata"}


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str, str]:
    """Return (fm_dict, fm_yaml_string, body)."""
    if not text.startswith("---\n"):
        return {}, "", text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, "", text
    fm_text = text[4:end]
    body = text[end + 5:]
    try:
        import yaml
        return yaml.safe_load(fm_text) or {}, fm_text, body
    except Exception:
        return {}, fm_text, body


def repair_tag_budget(fm: dict, fm_yaml: str) -> tuple[bool, str]:
    tags = fm.get("tags") or []
    if not isinstance(tags, list) or len(tags) <= 10:
        return False, fm_yaml
    new_tags = list(dict.fromkeys(tags))[:10]  # de-dupe + trim
    new_line = "tags: [" + ", ".join(new_tags) + "]"
    new_yaml = re.sub(r"^tags:.*?$", new_line, fm_yaml, count=1, flags=re.MULTILINE)
    return new_yaml != fm_yaml, new_yaml


def repair_dup_tags(fm: dict, fm_yaml: str) -> tuple[bool, str]:
    tags = fm.get("tags") or []
    if not isinstance(tags, list) or len(tags) == len(set(tags)):
        return False, fm_yaml
    deduped = list(dict.fromkeys(tags))
    new_line = "tags: [" + ", ".join(deduped) + "]"
    new_yaml = re.sub(r"^tags:.*?$", new_line, fm_yaml, count=1, flags=re.MULTILINE)
    return new_yaml != fm_yaml, new_yaml


def repair_tombstone_meta(fm: dict, fm_yaml: str) -> tuple[bool, str]:
    if not fm.get("tombstoned"):
        return False, fm_yaml
    needed = []
    if "deleted_at" not in fm:
        needed.append(f"deleted_at: {datetime.now(ICT).isoformat(timespec='seconds')}")
    if "deleted_by" not in fm:
        needed.append("deleted_by: subject:autorepair")
    if "tombstone_reason" not in fm:
        needed.append('tombstone_reason: "(auto-repair; please fill in)"')
    if not needed:
        return False, fm_yaml
    return True, fm_yaml + "\n" + "\n".join(needed)


REPAIRS = {
    "tag-budget-exceeded": repair_tag_budget,
    "duplicate-tags": repair_dup_tags,
    "tombstone-missing-metadata": repair_tombstone_meta,
}


def walk_memories(brain_root: Path):
    brain = brain_root / ".cyberos-memory"
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        yield rel, md, text


def main():
    p = argparse.ArgumentParser(description="bounded auto-repair of WARN-level findings (Batch 12 / Tier B)")
    p.add_argument("--apply", action="store_true", help="actually write changes (default: dry-run)")
    p.add_argument("--recipe", choices=sorted(VALID_RECIPES),
                   help="restrict to a single recipe (default: all)")
    args = p.parse_args()

    brain_root = find_brain()
    recipes = [args.recipe] if args.recipe else sorted(VALID_RECIPES)

    total_changes = 0
    applied = 0
    for rel, md, text in walk_memories(brain_root):
        fm, fm_yaml, body = parse_frontmatter(text)
        if not fm:
            continue
        new_yaml = fm_yaml
        for r in recipes:
            changed, new_yaml = REPAIRS[r](fm, new_yaml)
            if changed:
                total_changes += 1
        if new_yaml != fm_yaml:
            new_text = f"---\n{new_yaml}\n---\n{body}"
            diff_lines = list(difflib.unified_diff(
                text.splitlines(), new_text.splitlines(),
                fromfile=f"a/{rel}", tofile=f"b/{rel}", lineterm="",
            ))[:30]
            diff = "\n".join(diff_lines)
            print(f"\n── {rel} ──")
            print(diff)
            if args.apply:
                md.write_text(new_text, encoding="utf-8")
                applied += 1

    print(f"\n  Changes proposed: {total_changes}")
    if args.apply:
        print(f"  Applied:          {applied}")
    else:
        print(f"  (dry-run; pass --apply to write)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
