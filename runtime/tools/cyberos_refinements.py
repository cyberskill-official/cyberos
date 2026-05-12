#!/usr/bin/env python3
"""
cyberos_refinements.py — §0.4 refinement candidate dashboard.

Aspect 11.4 of the Layer-1 improvement catalog.

Surfaces three kinds of items related to the refinement loop:

  1. Drift candidates auto-detected by the Stop-hook (Aspect 3.1) —
     `.cyberos-memory/memories/drift/<date>-refinement-candidate-*.md`

  2. Council sessions awaiting synthesis (Aspect 3.3) —
     `outputs/council/REF-NNN-council.md` where no Synthesis verdict yet

  3. Rejected candidates with prior-art warning (Aspect 3.4) — files in
     `.cyberos-memory/rejected/` whose mtime is < 30 days

Used as: "what does the refinement loop want from me?" — a single
operator-facing view.

Usage:
    cyberos refinements                       # text table
    cyberos refinements --json                # machine-readable
    cyberos refinements --kind drift          # filter to one kind
"""
from __future__ import annotations
import argparse
import json
import re
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


def collect_drift(brain_root: Path) -> list[dict]:
    d = brain_root / ".cyberos-memory" / "memories" / "drift"
    if not d.exists():
        return []
    out = []
    for f in sorted(d.glob("*.md")):
        text = f.read_text(encoding="utf-8", errors="ignore")
        resolved = "## Resolution" in text or "resolution:" in text.lower()
        try:
            mtime = datetime.fromtimestamp(f.stat().st_mtime, tz=ICT)
            age_days = (datetime.now(ICT) - mtime).days
        except Exception:
            age_days = -1
        first_line = (text.splitlines() or [""])[0][:80]
        out.append({
            "kind": "drift",
            "path": f.relative_to(brain_root).as_posix(),
            "age_days": age_days,
            "resolved": resolved,
            "title": first_line,
        })
    return out


def collect_council(brain_root: Path) -> list[dict]:
    d = brain_root / "outputs" / "council"
    if not d.exists():
        return []
    out = []
    for f in sorted(d.glob("REF-*-council.md")):
        text = f.read_text(encoding="utf-8", errors="ignore")
        # Synthesis section is `## Synthesis (author fills...` if pending,
        # then filled in by operator. Look for `**Verdict:**` to know if done.
        verdict_match = re.search(r"\*\*Verdict:\*\*\s*\[?([A-Z][A-Z\- ]*?)[\]\n]", text)
        verdict = verdict_match.group(1).strip() if verdict_match else "pending"
        try:
            mtime = datetime.fromtimestamp(f.stat().st_mtime, tz=ICT)
            age_days = (datetime.now(ICT) - mtime).days
        except Exception:
            age_days = -1
        out.append({
            "kind": "council",
            "path": f.relative_to(brain_root).as_posix(),
            "age_days": age_days,
            "verdict": verdict,
        })
    return out


def collect_rejected(brain_root: Path) -> list[dict]:
    d = brain_root / ".cyberos-memory" / "rejected"
    if not d.exists():
        return []
    out = []
    cutoff = datetime.now(ICT) - timedelta(days=30)
    for f in sorted(d.glob("**/*.md")):
        try:
            mtime = datetime.fromtimestamp(f.stat().st_mtime, tz=ICT)
        except Exception:
            continue
        if mtime < cutoff:
            continue
        out.append({
            "kind": "rejected",
            "path": f.relative_to(brain_root).as_posix(),
            "age_days": (datetime.now(ICT) - mtime).days,
        })
    return out


def main():
    p = argparse.ArgumentParser(description="§0.4 refinement candidate dashboard")
    p.add_argument("--kind", choices=["drift", "council", "rejected", "all"], default="all")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    brain_root = find_brain()
    items: list[dict] = []
    if args.kind in ("drift", "all"):
        items.extend(collect_drift(brain_root))
    if args.kind in ("council", "all"):
        items.extend(collect_council(brain_root))
    if args.kind in ("rejected", "all"):
        items.extend(collect_rejected(brain_root))

    if args.json:
        print(json.dumps({"count": len(items), "items": items}, indent=2))
        return 0

    print()
    print(f"  §0.4 refinement dashboard")
    print(f"  Total: {len(items)} item(s) ({args.kind})")
    print()

    by_kind: dict[str, list] = {}
    for x in items:
        by_kind.setdefault(x["kind"], []).append(x)

    for kind in ("drift", "council", "rejected"):
        rows = by_kind.get(kind, [])
        if not rows:
            continue
        print(f"  ── {kind.upper()} ({len(rows)}) ──")
        for r in rows[:15]:
            extra = ""
            if kind == "drift":
                extra = "RESOLVED" if r["resolved"] else "open"
            elif kind == "council":
                extra = f"verdict={r['verdict']}"
            print(f"    {r['age_days']:4d}d  {r['path']}  {extra}")
        if len(rows) > 15:
            print(f"    … +{len(rows) - 15} more")
        print()

    if not items:
        print(f"  ✓ no candidates in queue")
        return 0
    print(f"  Action: review each item, then `cyberos doctor` or amend the source file.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
