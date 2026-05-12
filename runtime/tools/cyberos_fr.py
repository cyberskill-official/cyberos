#!/usr/bin/env python3
"""
cyberos_fr.py — feature-request browser + task-graph view.

S2.3 of skills-Stage-1 improvements (Batch 16).

Walks `planning/*/FR-NNN-*.md` and `.cyberos-memory/memories/projects/*/FR-NNN-*.md`,
parses the embedded `tasks:` list (per `task@1` contract), and surfaces:

  list           — every FR, with task count + sizing summary
  show <FR>      — one FR in detail (frontmatter + tasks)
  graph          — Mermaid graph of all FRs + their dependency chains
  task-graph FR  — DAG of one FR's tasks
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, text[end + 5:]
    except Exception:
        return {}, text[end + 5:]


def collect_frs(brain_root: Path) -> list[dict]:
    """Walk planning/ + memories/projects/ for FR-* markdown files."""
    out = []
    search_dirs = [
        brain_root / "planning",
        brain_root / ".cyberos-memory" / "memories" / "projects",
        brain_root / "outputs" / "staged-memories",
    ]
    for d in search_dirs:
        if not d.exists():
            continue
        for md in d.rglob("FR-*.md"):
            if not md.is_file():
                continue
            try:
                text = md.read_text(encoding="utf-8")
            except Exception:
                continue
            fm, body = parse_frontmatter(text)
            if not fm:
                continue
            tasks = fm.get("tasks") or []
            sizes = {"S": 0, "M": 0, "L": 0, "XL": 0}
            assign_h = assign_a = assign_either = 0
            for t in tasks:
                sz = t.get("sizing", "?")
                if sz in sizes:
                    sizes[sz] += 1
                asn = t.get("assignable_to") or []
                if asn == ["human"]:
                    assign_h += 1
                elif asn == ["ai-agent"]:
                    assign_a += 1
                else:
                    assign_either += 1
            out.append({
                "path": md.relative_to(brain_root).as_posix(),
                "fr_id": fm.get("fr_id") or md.stem.split("-")[0] + "-" + md.stem.split("-")[1] if "-" in md.stem else md.stem,
                "title": fm.get("title", body.split("\n", 1)[0][:80] if body else "?"),
                "profile": fm.get("profile", "?"),
                "task_count": len(tasks),
                "sizes": sizes,
                "assign_h": assign_h, "assign_a": assign_a, "assign_either": assign_either,
                "tasks": tasks,
                "status": fm.get("status", "draft"),
            })
    return sorted(out, key=lambda x: x["path"])


def cmd_list(_args):
    brain_root = find_brain()
    frs = collect_frs(brain_root)
    if not frs:
        print("  no FRs found (looked in planning/, memories/projects/, outputs/staged-memories/)")
        return 0
    print(f"\n  {len(frs)} feature request(s):\n")
    print(f"  {'fr_id':12s}  {'profile':8s}  tasks  S/M/L/XL  hum/ai/either  path")
    for f in frs:
        s = f["sizes"]
        print(f"  {f['fr_id']:12s}  {f['profile']:8s}  {f['task_count']:>5}  "
              f"{s['S']}/{s['M']}/{s['L']}/{s['XL']:<2}  "
              f"{f['assign_h']}/{f['assign_a']}/{f['assign_either']}  "
              f"{f['path']}")
    return 0


def cmd_show(args):
    brain_root = find_brain()
    frs = collect_frs(brain_root)
    matches = [f for f in frs if args.fr_id in f["fr_id"] or args.fr_id in f["path"]]
    if not matches:
        print(f"  no FR matches {args.fr_id!r}")
        return 1
    f = matches[0]
    print(f"\n  {f['fr_id']} — {f['title']}")
    print(f"  path:    {f['path']}")
    print(f"  profile: {f['profile']}")
    print(f"  status:  {f['status']}")
    print(f"  tasks ({f['task_count']}):")
    for t in f["tasks"]:
        deps = t.get("dependencies") or []
        deps_s = f"deps={deps}" if deps else "deps=none"
        print(f"    {t.get('id', '?'):14s}  [{t.get('sizing', '?')}]  {t.get('title', '')[:60]}")
        print(f"      assignable_to={t.get('assignable_to')}  par={t.get('parallelisable')}  {deps_s}")
    return 0


def cmd_graph(_args):
    brain_root = find_brain()
    frs = collect_frs(brain_root)
    if not frs:
        print("  no FRs to graph")
        return 0
    print("```mermaid")
    print("flowchart LR")
    for f in frs:
        sym = "[" + f["fr_id"] + "]"
        print(f'  {f["fr_id"]}{sym}')
    print("```")
    print(f"\n  Generated graph for {len(frs)} FR(s)")
    return 0


def cmd_task_graph(args):
    brain_root = find_brain()
    frs = collect_frs(brain_root)
    matches = [f for f in frs if args.fr_id in f["fr_id"] or args.fr_id in f["path"]]
    if not matches:
        print(f"  no FR matches {args.fr_id!r}")
        return 1
    f = matches[0]
    tasks = f["tasks"]
    if not tasks:
        print(f"  {f['fr_id']} has no embedded tasks")
        return 0
    print("```mermaid")
    print("flowchart TD")
    for t in tasks:
        tid = t.get("id", "T?")
        sz = t.get("sizing", "?")
        ttl = (t.get("title", "")[:40]).replace('"', "'")
        sym = f'{tid}["{tid} [{sz}]<br/>{ttl}"]'
        print(f"  {sym}")
    print()
    for t in tasks:
        tid = t.get("id", "T?")
        for dep in (t.get("dependencies") or []):
            print(f"  {dep} --> {tid}")
    print("```")
    print(f"\n  {len(tasks)} tasks in {f['fr_id']}")
    return 0


def main():
    p = argparse.ArgumentParser(description="feature-request browser + task-graph view (S2.3)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list").set_defaults(func=cmd_list)
    ps = sub.add_parser("show"); ps.add_argument("fr_id"); ps.set_defaults(func=cmd_show)
    sub.add_parser("graph").set_defaults(func=cmd_graph)
    pt = sub.add_parser("task-graph"); pt.add_argument("fr_id"); pt.set_defaults(func=cmd_task_graph)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
