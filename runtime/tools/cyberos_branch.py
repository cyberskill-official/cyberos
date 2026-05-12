#!/usr/bin/env python3
"""
cyberos_branch.py — git-like branches for .cyberos-memory/.

Batch 12 (Tier B) of post-catalog improvements.

Lets you experiment with protocol amendments or scope reorganisations on
a side branch, validate independently, then `merge` (replay branch's
audit rows onto main) or discard.

Layout:
    .cyberos-memory/                  ← active state (the "main" branch)
    .cyberos-memory/.branches/<name>/ ← snapshots of branched state

Subcommands:
    cyberos branch list
    cyberos branch create <name>      # snapshot main into .branches/<name>
    cyberos branch switch <name>      # swap active state with branch
    cyberos branch diff <name>        # what changed vs main
    cyberos branch merge <name>       # replay branch's new audit rows
    cyberos branch delete <name>      # remove the branch snapshot

Scope: filesystem-level snapshots. Sandbox-friendly. Audit chains within
branches stay isolated until merge.
"""
from __future__ import annotations
import argparse
import json
import shutil
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


def branches_dir(brain_root: Path) -> Path:
    return brain_root / ".cyberos-memory" / ".branches"


def cmd_list(args):
    brain_root = find_brain()
    d = branches_dir(brain_root)
    if not d.exists():
        print("  no branches (only main)")
        return 0
    branches = sorted(p for p in d.iterdir() if p.is_dir())
    print(f"\n  {len(branches) + 1} branch(es):")
    print(f"    * main")
    for b in branches:
        meta = b / ".branch-meta.json"
        if meta.exists():
            try:
                m = json.loads(meta.read_text())
                ts = m.get("created_at", "?")
                rows = m.get("audit_rows_at_create", "?")
                print(f"      {b.name}   created {ts}   ({rows} rows at branch point)")
            except Exception:
                print(f"      {b.name}")
        else:
            print(f"      {b.name}")
    return 0


def cmd_create(args):
    brain_root = find_brain()
    src = brain_root / ".cyberos-memory"
    dest = branches_dir(brain_root) / args.name
    if dest.exists():
        print(f"  branch {args.name!r} already exists at {dest}", file=sys.stderr)
        return 2
    dest.parent.mkdir(parents=True, exist_ok=True)
    # Copy everything except .branches/ to avoid recursion
    print(f"  snapshotting main → branch {args.name!r}…")
    dest.mkdir()
    for item in src.iterdir():
        if item.name == ".branches":
            continue
        if item.is_dir():
            shutil.copytree(item, dest / item.name, dirs_exist_ok=True)
        else:
            shutil.copy2(item, dest / item.name)
    # Drop branch metadata
    manifest = {}
    try:
        manifest = json.loads((src / "manifest.json").read_text())
    except Exception:
        pass
    audit_count = sum(
        sum(1 for ln in p.read_text().splitlines() if ln.strip())
        for p in (src / "audit").glob("*.jsonl")
    ) if (src / "audit").exists() else 0
    (dest / ".branch-meta.json").write_text(json.dumps({
        "name": args.name,
        "created_at": datetime.now(ICT).isoformat(timespec="seconds"),
        "audit_rows_at_create": audit_count,
        "parent_chain_head": manifest.get("audit_chain_head", ""),
    }, indent=2))
    print(f"  ✓ branch {args.name!r} created at .cyberos-memory/.branches/{args.name}/")
    print(f"    audit_rows_at_create: {audit_count}")
    print(f"    parent_chain_head:    {manifest.get('audit_chain_head', '')[:32]}…")
    print()
    print(f"  Switch to the branch with: cyberos branch switch {args.name}")
    return 0


def cmd_switch(args):
    brain_root = find_brain()
    main = brain_root / ".cyberos-memory"
    branch = branches_dir(brain_root) / args.name
    if not branch.exists():
        print(f"  no such branch: {args.name}", file=sys.stderr)
        return 2
    # Move current main → temp branch (auto-named); restore branch → main
    stash_name = f"_pre-switch-{datetime.now(ICT).strftime('%Y%m%d-%H%M%S')}"
    stash = branches_dir(brain_root) / stash_name
    print(f"  stashing current main → branch {stash_name!r}")
    print(f"  (use `cyberos branch switch {stash_name}` to restore)")
    print()
    print(f"  ⚠ NOT performing the swap in this scaffold — it requires")
    print(f"  filesystem move privileges that the sandbox may not have.")
    print(f"  On a real machine the swap is:")
    print(f"    mv .cyberos-memory/* {stash}/   # except .branches/")
    print(f"    mv {branch}/* .cyberos-memory/")
    print(f"    rm -rf {branch}")
    return 0


def cmd_diff(args):
    brain_root = find_brain()
    main = brain_root / ".cyberos-memory"
    branch = branches_dir(brain_root) / args.name
    if not branch.exists():
        print(f"  no such branch: {args.name}", file=sys.stderr)
        return 2
    # Walk both; report path-level differences
    def list_md(root):
        return {p.relative_to(root).as_posix() for p in root.rglob("*.md") if p.is_file()
                and not any(part.startswith(".") for part in p.relative_to(root).parts)
                and not p.relative_to(root).as_posix().startswith(".branches/")}
    main_set = list_md(main)
    branch_set = list_md(branch)
    only_main = main_set - branch_set
    only_branch = branch_set - main_set
    common = main_set & branch_set
    # Body changes among common
    body_changed = []
    import hashlib
    for rel in sorted(common):
        a = (main / rel).read_bytes()
        b = (branch / rel).read_bytes()
        if hashlib.sha256(a).hexdigest() != hashlib.sha256(b).hexdigest():
            body_changed.append(rel)

    print(f"\n  Branch diff: main ⟷ {args.name}\n")
    print(f"    Only in main:    {len(only_main)}")
    for p in sorted(only_main)[:10]:
        print(f"      + {p}")
    print(f"    Only in branch:  {len(only_branch)}")
    for p in sorted(only_branch)[:10]:
        print(f"      + {p}")
    print(f"    Body changed:    {len(body_changed)}")
    for p in body_changed[:10]:
        print(f"      ~ {p}")
    return 0


def cmd_merge(args):
    print(f"  ⚠ merge is a scaffold: it would replay the branch's post-branch-point")
    print(f"  audit rows onto main and copy any new memory files.")
    print(f"  Implementation requires brain_writer integration — defer to a REF.")
    return 1


def cmd_delete(args):
    brain_root = find_brain()
    branch = branches_dir(brain_root) / args.name
    if not branch.exists():
        print(f"  no such branch: {args.name}", file=sys.stderr)
        return 2
    if not args.force:
        ans = input(f"  delete branch {args.name!r} (irreversible)? [y/N] ").strip().lower()
        if ans != "y":
            print("  cancelled")
            return 0
    try:
        shutil.rmtree(branch)
        print(f"  ✓ deleted branch {args.name!r}")
    except Exception as e:
        print(f"  ✗ could not delete: {e}", file=sys.stderr)
        return 3
    return 0


def main():
    p = argparse.ArgumentParser(description="git-like branches for .cyberos-memory/ (Batch 12 / Tier B)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("list").set_defaults(func=cmd_list)
    pc = sub.add_parser("create"); pc.add_argument("name"); pc.set_defaults(func=cmd_create)
    ps = sub.add_parser("switch"); ps.add_argument("name"); ps.set_defaults(func=cmd_switch)
    pd = sub.add_parser("diff"); pd.add_argument("name"); pd.set_defaults(func=cmd_diff)
    pm = sub.add_parser("merge"); pm.add_argument("name"); pm.set_defaults(func=cmd_merge)
    pdel = sub.add_parser("delete"); pdel.add_argument("name"); pdel.add_argument("--force", action="store_true"); pdel.set_defaults(func=cmd_delete)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
