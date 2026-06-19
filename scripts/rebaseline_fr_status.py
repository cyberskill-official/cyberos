#!/usr/bin/env python3
"""Re-baseline CyberOS FR statuses from evidence (Step 2).

Deterministic and idempotent. Reads the canonical `status:` field in each FR spec
file's YAML frontmatter (docs/feature-requests/<module>/FR-*.md, excluding *.audit.md),
applies two transforms, and writes a reviewable unified diff. Writes nothing unless
--apply is passed.

Transforms, in order:

1. Normalize legacy status tokens to the canonical 10-state enum
   (modules/skill/contracts/feature-request/STATUS-REFERENCE.md). The 2026-05 wave
   already canonicalized the field, so this is expected to be a no-op; it is kept so
   the script is safe to run on any older tree.

2. Reset every FR currently past `implementing`
   (ready_to_review | reviewing | ready_to_test | testing | done) to `ready_to_test`.
   Rationale: the code already exists on `main`; the work is independent awh
   re-verification, which is what ready_to_test / the test gate is for. This matches
   STATUS-REFERENCE section 1.4's re-audit convention (done -> ready_to_review /
   ready_to_test). `implementing` and `ready_to_implement` are reserved only for FRs
   where `main` genuinely lacks a real implementation; pass those ids in --lacks-impl.

Idempotent: ready_to_test is in the reset set and maps to itself, so a second run is a
no-op. draft, ready_to_implement, implementing, on_hold, and closed are never touched.

The script edits only the `status:` line, byte-for-byte preserving the rest of each
file. It does NOT touch *.audit.md files and does NOT regenerate BACKLOG.md (the index
regeneration is a separate Step 3 action, kept out so this diff stays clean).
"""
from __future__ import annotations

import argparse
import difflib
import json
import re
import sys
from pathlib import Path

CANONICAL = {
    "draft", "ready_to_implement", "implementing", "ready_to_review",
    "reviewing", "ready_to_test", "testing", "done", "on_hold", "closed",
}

# Legacy -> canonical (from the BACKLOG STATUS-WAVE-2026-05 migration note).
LEGACY = {
    "planned": "ready_to_implement", "accepted": "ready_to_implement",
    "audited": "ready_to_implement", "in_review": "ready_to_implement",
    "building": "implementing", "in_progress": "implementing",
    "shipped": "done", "verified": "done", "tested": "done",
    "deferred": "on_hold", "rejected": "closed", "superseded": "closed",
    "blocked": "ready_to_implement", "failed": "ready_to_implement",
    "backlog": "ready_to_implement", "todo": "ready_to_implement", "wip": "implementing",
}

# FRs past implementing -> reset target.
PAST_IMPLEMENTING = {"ready_to_review", "reviewing", "ready_to_test", "testing", "done"}
RESET_TARGET = "ready_to_test"

STATUS_RE = re.compile(r"^(status:\s*)([^\s#\"'`]+)(.*)$")


def frontmatter_bounds(lines: list[str]) -> tuple[int, int] | None:
    """Return (start, end) line indices of the leading --- frontmatter block."""
    if not lines or lines[0].strip() != "---":
        return None
    for i in range(1, len(lines)):
        if lines[i].strip() == "---":
            return 0, i
    return None


def normalize(value: str) -> str:
    v = value.strip().strip("\"'`").lower()
    if v in CANONICAL:
        return v
    return LEGACY.get(v, v)  # unknown values pass through and are reported


def plan_file(path: Path, lacks_impl: set[str]) -> dict | None:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines(keepends=True)
    fb = frontmatter_bounds([ln.rstrip("\n") for ln in lines])
    if fb is None:
        return None
    start, end = fb
    fr_id = path.stem.split("-")[0:3]
    fr_id = "-".join(fr_id) if fr_id else path.stem
    for idx in range(start, end + 1):
        m = STATUS_RE.match(lines[idx].rstrip("\n"))
        if not m:
            continue
        raw = m.group(2)
        canon = normalize(raw)
        unknown = canon not in CANONICAL
        if canon in PAST_IMPLEMENTING:
            new = "ready_to_implement" if fr_id in lacks_impl else RESET_TARGET
        else:
            new = canon
        if new == raw and not unknown:
            return {"path": path, "fr_id": fr_id, "old": raw, "new": new,
                    "changed": False, "unknown": unknown}
        newline = f"{m.group(1)}{new}{m.group(3)}\n"
        new_lines = lines[:idx] + [newline] + lines[idx + 1:]
        return {"path": path, "fr_id": fr_id, "old": raw, "new": new,
                "changed": new != raw, "unknown": unknown,
                "old_text": text, "new_text": "".join(new_lines)}
    return {"path": path, "fr_id": fr_id, "old": None, "new": None,
            "changed": False, "unknown": False}  # no status field


def main() -> int:
    ap = argparse.ArgumentParser(description="Re-baseline FR statuses (Step 2).")
    ap.add_argument("--root", default="docs/feature-requests",
                    help="feature-requests root (default: docs/feature-requests)")
    ap.add_argument("--apply", action="store_true",
                    help="write changes (default: dry-run, no writes)")
    ap.add_argument("--lacks-impl", default="",
                    help="comma-separated FR ids to send to ready_to_implement instead")
    ap.add_argument("--json", action="store_true", help="emit a JSON summary")
    ap.add_argument("--max-diffs", type=int, default=12,
                    help="how many unified diffs to print in dry-run")
    args = ap.parse_args()

    root = Path(args.root)
    if not root.is_dir():
        print(f"error: root not found: {root}", file=sys.stderr)
        return 2
    lacks_impl = {x.strip() for x in args.lacks_impl.split(",") if x.strip()}

    files = sorted(p for p in root.rglob("FR-*.md") if not p.name.endswith(".audit.md"))
    plans = [p for p in (plan_file(f, lacks_impl) for f in files) if p is not None]

    changed = [p for p in plans if p["changed"]]
    unknown = [p for p in plans if p["unknown"]]
    no_status = [p for p in plans if p["old"] is None]

    transitions: dict[str, int] = {}
    for p in changed:
        key = f'{p["old"]} -> {p["new"]}'
        transitions[key] = transitions.get(key, 0) + 1

    if args.apply:
        for p in changed:
            p["path"].write_text(p["new_text"], encoding="utf-8")

    if args.json:
        print(json.dumps({
            "scanned": len(plans), "changed": len(changed),
            "transitions": transitions,
            "unknown_status": [p["fr_id"] for p in unknown],
            "no_status_field": [str(p["path"]) for p in no_status],
            "changed_frs": [p["fr_id"] for p in changed],
            "applied": args.apply,
        }, indent=2))
        return 0

    print(f"scanned {len(plans)} FR spec files under {root}")
    print(f"would change {len(changed)} file(s)" if not args.apply
          else f"applied {len(changed)} change(s)")
    print("transitions:")
    for k, v in sorted(transitions.items()):
        print(f"  {v:4d}  {k}")
    if unknown:
        print(f"UNKNOWN status values (review): {[p['fr_id'] for p in unknown]}")
    if no_status:
        print(f"files with no status field (review): {len(no_status)}")
    if not args.apply:
        shown = 0
        for p in changed:
            if shown >= args.max_diffs:
                print(f"... {len(changed) - shown} more changed file(s) not shown")
                break
            diff = difflib.unified_diff(
                p["old_text"].splitlines(keepends=True),
                p["new_text"].splitlines(keepends=True),
                fromfile=str(p["path"]), tofile=str(p["path"]), n=1)
            sys.stdout.writelines(diff)
            shown += 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
