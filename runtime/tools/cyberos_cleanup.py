#!/usr/bin/env python3
"""
cyberos_cleanup.py — detect + report leftover test artifacts.

Tier E (cleanup) of post-catalog improvements (Batch 15).

Scans the repo for files that look like test leftovers, scratch
artifacts, or experiments that should be removed. By default produces a
report and a `cleanup.sh` script the operator runs by hand. With
`--apply` it deletes (subject to sandbox permissions).

Categories scanned:
  - .cyberos-memory/cache/test-*           — anything I prefixed `test-` during dev
  - .cyberos-memory/cache/cold-test/       — test cold-storage archives
  - .cyberos-memory/cache/audit-bundle.zip — audit-script leftover
  - .cyberos-memory/staging/ — anything older than 24h still staged
  - .cyberos-memory/.branches/experiment-*  — experimental branches
  - .cyberos-memory/.branches/_pre-switch-* — stash branches
  - .cyberos-memory/.lock.shared / .lock.exclusive — lock files (kept)
  - .cyberos-memory/cache/test-fixtures/sync/*  + .cyberos-memory/cache/test-fixtures/sync-staging/* — sync test artefacts
  - .cyberos-memory/cache/site/   — static-site test render (regenerable)
  - .cyberos-memory/cache/council/REF-042-council.md — council test
  - /tmp/test-claude-settings.json (if present in cwd)
  - /tmp/cyberos-* tempdirs (chaos-test residue)

Doesn't touch:
  - any memory file under .cyberos-memory/memories/, persona/, company/
  - manifest.json, audit/*, meta/* (except .branches/.* listed above)
  - runtime/lib/brain_writer.py, runtime/starter/templates/, runtime/starter/cyberos-starter/
"""
from __future__ import annotations
import argparse
import shutil
import sys
import time
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


def collect_candidates(brain_root: Path) -> list[dict]:
    cands = []
    o = brain_root / "outputs"
    if o.exists():
        # test-* anything
        for p in o.glob("test-*"):
            cands.append({"path": p, "reason": ".cyberos-memory/cache/test-* dev scratch", "kind": "file" if p.is_file() else "dir"})
        for p in o.glob("audit-bundle*.zip"):
            cands.append({"path": p, "reason": "audit-script leftover bundle", "kind": "file"})
        # cold-test/
        if (o / "cold-test").exists():
            cands.append({"path": o / "cold-test", "reason": "test cold-storage archives", "kind": "dir"})
        # site/  — regenerable from cyberos static; mark for review
        if (o / "site").exists():
            cands.append({"path": o / "site", "reason": "static-site render (regenerable)", "kind": "dir"})
        # sync test artefacts
        if (o / "sync").exists():
            for p in (o / "sync").glob("*.md"):
                cands.append({"path": p, "reason": "sync import report (regenerable)", "kind": "file"})
        if (o / "sync-staging").exists():
            for p in (o / "sync-staging").iterdir():
                cands.append({"path": p, "reason": "sync staging (test artefact)", "kind": "dir" if p.is_dir() else "file"})
        # stale staged memories (>24h)
        if (o / "staged-memories").exists():
            cutoff = time.time() - 24 * 3600
            for p in (o / "staged-memories").glob("*.md"):
                try:
                    if p.stat().st_mtime < cutoff:
                        cands.append({"path": p, "reason": "staged memory > 24h old", "kind": "file"})
                except Exception:
                    continue
        # Test council session
        cm = o / "council"
        if cm.exists():
            for p in cm.glob("REF-*-council.md"):
                # Keep council files alive unless older than 7d
                cutoff = time.time() - 7 * 24 * 3600
                try:
                    if p.stat().st_mtime < cutoff:
                        cands.append({"path": p, "reason": "council session > 7d old", "kind": "file"})
                except Exception:
                    continue

    # .branches/ experiments
    branches = brain_root / ".cyberos-memory" / ".branches"
    if branches.exists():
        for b in branches.iterdir():
            if not b.is_dir():
                continue
            if b.name.startswith("experiment-") or b.name.startswith("_pre-switch-"):
                cands.append({"path": b, "reason": ".branches/ experimental snapshot", "kind": "dir"})

    # Stub manual file
    stub = brain_root / "docs" / "CyberOS-LAYER-1-MANUAL.md"
    if stub.exists() and stub.stat().st_size < 400:
        cands.append({"path": stub, "reason": "obsolete manual stub (content merged into README Parts 25-31)", "kind": "file"})

    # Top-level tmp scratch
    for n in ("audit.sh", "audit2.sh"):
        p = Path("/tmp") / n
        if p.exists():
            cands.append({"path": p, "reason": "shell audit scratch script in /tmp", "kind": "file"})

    return cands


def size_of(p: Path) -> int:
    try:
        if p.is_file():
            return p.stat().st_size
        total = 0
        for q in p.rglob("*"):
            if q.is_file():
                total += q.stat().st_size
        return total
    except Exception:
        return 0


def main():
    p = argparse.ArgumentParser(description="detect + clean leftover test artefacts (Batch 15 / Tier E cleanup)")
    p.add_argument("--apply", action="store_true", help="actually delete (subject to sandbox perms)")
    p.add_argument("--out-script", help="emit a host-side rm script for the operator")
    args = p.parse_args()

    brain_root = find_brain()
    cands = collect_candidates(brain_root)
    if not cands:
        print("  ✓ no leftovers detected")
        return 0

    total = sum(size_of(c["path"]) for c in cands)
    print(f"\n  Found {len(cands)} cleanup candidate(s) totalling {total:,} bytes:\n")
    for c in cands:
        size = size_of(c["path"])
        rel = c["path"].relative_to(brain_root) if c["path"].is_relative_to(brain_root) else c["path"]
        print(f"    {c['kind']:4s}  {size:>10,}B  {str(rel)}")
        print(f"          ↳ {c['reason']}")

    if args.out_script:
        lines = ["#!/bin/bash", f"# cyberos cleanup script — generated {datetime.now(ICT).isoformat(timespec='seconds')}",
                 f"# Run from: {brain_root}", "set -e", ""]
        for c in cands:
            rel = c["path"].relative_to(brain_root) if c["path"].is_relative_to(brain_root) else c["path"]
            cmd = "rm -f" if c["kind"] == "file" else "rm -rf"
            lines.append(f'{cmd} "{rel}"  # {c["reason"]}')
        Path(args.out_script).write_text("\n".join(lines) + "\n")
        Path(args.out_script).chmod(0o755)
        print(f"\n  ✓ host-side cleanup script: {args.out_script}")

    if args.apply:
        removed = 0
        failed = 0
        for c in cands:
            try:
                if c["kind"] == "file":
                    c["path"].unlink()
                else:
                    shutil.rmtree(c["path"])
                removed += 1
            except Exception as e:
                print(f"    ✗ could not remove {c['path']}: {e}")
                failed += 1
        print(f"\n  removed: {removed}    failed (sandbox/perms): {failed}")
        if failed:
            print(f"  Use --out-script to emit a host-side script the operator can run on the real filesystem.")
        return 0 if failed == 0 else 1

    print(f"\n  (dry-run; pass --apply to delete, or --out-script <path> to emit a rm script)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
