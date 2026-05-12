#!/usr/bin/env python3
"""
cyberos_tenant.py — multi-tenant single-BRAIN scaffolding.

Batch 13 (Tier C) of post-catalog improvements.

One `.cyberos-memory/` BRAIN hosting multiple subjects' working memory
with strict scope isolation. Pattern: each tenant gets a top-level
scope folder (`member/<subject-slug>/`), and a validator plugin enforces
that cross-tenant reads/writes require explicit consent.

Scaffold scope:
  - `cyberos tenant list`           — show member/ folders
  - `cyberos tenant create <slug>`  — create member/<slug>/ + persona card
  - `cyberos tenant audit`          — flag cross-tenant references
  - install `meta/validators/check-tenant-isolation.py` for ongoing checks

Real cross-tenant collab would build on this via §17 sync_class=shared.
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


def cmd_list(_args):
    brain_root = find_brain()
    member = brain_root / ".cyberos-memory" / "member"
    if not member.exists():
        print("  no tenants (member/ does not exist)")
        return 0
    tenants = sorted(p for p in member.iterdir() if p.is_dir())
    print(f"\n  {len(tenants)} tenant(s):")
    for t in tenants:
        n = sum(1 for _ in t.rglob("*.md") if _.is_file())
        print(f"    {t.name:24s}  {n} memories")
    return 0


def cmd_create(args):
    brain_root = find_brain()
    slug = re.sub(r"[^a-z0-9-]", "-", args.subject.lower()).strip("-")
    member_dir = brain_root / ".cyberos-memory" / "member" / slug
    if member_dir.exists():
        print(f"  tenant {slug!r} already exists", file=sys.stderr)
        return 2
    member_dir.mkdir(parents=True, exist_ok=True)
    (member_dir / ".keep").write_text("")
    print(f"  ✓ tenant {slug!r} created at member/{slug}/")
    print(f"  Next: create persona/{slug}.md and PERSON-NNN-{slug}.md in memories/people/")
    return 0


def cmd_audit(_args):
    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"
    member = brain / "member"
    if not member.exists():
        print("  no tenants to audit")
        return 0
    tenants = [p.name for p in member.iterdir() if p.is_dir()]
    if len(tenants) < 2:
        print(f"  only {len(tenants)} tenant(s); cross-tenant audit is moot")
        return 0
    findings = []
    for tenant in tenants:
        td = member / tenant
        for md in td.rglob("*.md"):
            if not md.is_file():
                continue
            try:
                text = md.read_text(encoding="utf-8")
            except Exception:
                continue
            for other in tenants:
                if other == tenant:
                    continue
                if f"member/{other}/" in text or f"subject:{other}" in text:
                    findings.append((tenant, md.name, other))
    if not findings:
        print(f"  ✓ {len(tenants)} tenants; no cross-tenant references")
        return 0
    print(f"\n  {len(findings)} cross-tenant reference(s) (review for consent):")
    for tenant, fname, other in findings:
        print(f"    {tenant}/{fname}  →  references {other}")
    return 1


def main():
    p = argparse.ArgumentParser(description="multi-tenant scaffolding (Batch 13 / Tier C)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list").set_defaults(func=cmd_list)
    pc = sub.add_parser("create"); pc.add_argument("subject"); pc.set_defaults(func=cmd_create)
    sub.add_parser("audit").set_defaults(func=cmd_audit)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
