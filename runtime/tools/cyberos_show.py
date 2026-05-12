#!/usr/bin/env python3
"""
cyberos_show.py — list memories with metadata (Aspect 1.1).

Walks `.cyberos-memory/`, parses frontmatter, prints structured table.

Usage:
    python3 runtime/tools/cyberos_show.py [path]
    python3 runtime/tools/cyberos_show.py --scope memories/decisions
    python3 runtime/tools/cyberos_show.py --tag pricing
    python3 runtime/tools/cyberos_show.py --class personnel
    python3 runtime/tools/cyberos_show.py --tombstoned
    python3 runtime/tools/cyberos_show.py --recent 7d
"""
from __future__ import annotations
import argparse
import re
import sys
import yaml
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))

def parse_frontmatter(path: Path) -> dict | None:
    """Extract YAML frontmatter from a memory file. Returns None if no valid frontmatter."""
    try:
        text = path.read_text(encoding="utf-8")
    except Exception:
        return None
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    try:
        return yaml.safe_load(text[4:end]) or {}
    except Exception:
        return None

def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur / ".cyberos-memory"
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")

def collect(brain: Path, args) -> list[tuple[Path, dict]]:
    results = []
    cutoff = None
    if args.recent:
        m = re.match(r"^(\d+)([dwmy])$", args.recent)
        if m:
            n = int(m.group(1))
            unit = m.group(2)
            days = {"d": n, "w": n*7, "m": n*30, "y": n*365}[unit]
            cutoff = datetime.now(ICT) - timedelta(days=days)

    for f in brain.rglob("*.md"):
        if not f.is_file() or f.name.startswith("."):
            continue
        rel = f.relative_to(brain)
        if args.scope and not str(rel).startswith(args.scope.rstrip("/")):
            continue
        fm = parse_frontmatter(f)
        if fm is None:
            # Files without frontmatter (e.g., README, protocol-history) — skip unless --include-all
            if not args.include_all:
                continue
            fm = {"memory_id": None, "scope": "(no-fm)", "tags": []}

        if args.tag and args.tag not in (fm.get("tags") or []):
            continue
        if args.classification and fm.get("classification") != args.classification:
            continue
        if args.authority and fm.get("authority") != args.authority:
            continue
        if args.tombstoned and not fm.get("tombstoned"):
            continue
        if not args.tombstoned and fm.get("tombstoned") and not args.include_tombstoned:
            continue
        if cutoff:
            ts_str = fm.get("last_updated_at") or fm.get("created_at")
            if ts_str:
                try:
                    ts = datetime.fromisoformat(str(ts_str))
                    if ts < cutoff:
                        continue
                except Exception:
                    pass

        results.append((rel, fm))

    return results

def print_table(rows, args):
    if not rows:
        print("(no matching memories)")
        return
    if args.format == "json":
        import json as J
        out = []
        for rel, fm in rows:
            out.append({"path": str(rel), "memory_id": fm.get("memory_id"),
                        "scope": fm.get("scope"), "classification": fm.get("classification"),
                        "authority": fm.get("authority"), "tags": fm.get("tags", []),
                        "version": fm.get("version"), "last_updated_at": str(fm.get("last_updated_at"))})
        print(J.dumps(out, indent=2, default=str))
        return

    # Plain table
    print(f"{'PATH':<60} {'CLASS':<12} {'AUTH':<16} {'TAGS':<24} {'UPDATED'}")
    print("-" * 130)
    for rel, fm in rows:
        path_s = str(rel)[:58]
        cls = (fm.get("classification") or "-")[:11]
        auth = (fm.get("authority") or "-")[:15]
        tags = ",".join(fm.get("tags") or [])[:22]
        upd = str(fm.get("last_updated_at") or "-")[:19]
        marker = " 💀" if fm.get("tombstoned") else ""
        print(f"{path_s:<60} {cls:<12} {auth:<16} {tags:<24} {upd}{marker}")
    print(f"\nTotal: {len(rows)} memories")

def main():
    p = argparse.ArgumentParser(description="cyberos memory browser")
    p.add_argument("--scope", help="filter by scope path prefix (e.g. memories/decisions)")
    p.add_argument("--tag", help="filter by single tag")
    p.add_argument("--classification", "--class", help="filter by classification")
    p.add_argument("--authority", help="filter by authority level")
    p.add_argument("--tombstoned", action="store_true", help="show only tombstoned")
    p.add_argument("--include-tombstoned", action="store_true", help="include tombstoned in results")
    p.add_argument("--include-all", action="store_true", help="include files without frontmatter")
    p.add_argument("--recent", help="only show updated within (e.g. 7d, 4w, 1m)")
    p.add_argument("--format", choices=["table", "json"], default="table")
    args = p.parse_args()

    brain = find_brain()
    rows = collect(brain, args)
    print_table(rows, args)

if __name__ == "__main__":
    sys.exit(main() or 0)
