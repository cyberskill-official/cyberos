#!/usr/bin/env python3
"""
cyberos_index_hook.py — incremental SQLite index updates.

Aspect 9.2 of the Layer-1 improvement catalog.

Today `cyberos_index.py build` rebuilds the SQLite index in full. For
write-heavy sessions this means the index lags behind reality between
rebuilds. This hook is invoked by `brain_writer` after each successful
op:create / str_replace / delete / rename, triggering a targeted
`cyberos_index.py update` on the affected memory only.

Two modes:

  - On-write (default): called by `brain_writer.py` as a subprocess
    after a successful append. Best-effort — failure is non-fatal.

  - Stop-hook (alternative): wired as a Claude Code Stop hook to
    refresh on session end if the on-write path skipped (e.g. sandbox
    permission denied).

Usage:
    cyberos_index_hook.py on-write <relative-memory-path>
    cyberos_index_hook.py stop-hook

The hook does NOT need to know the operation — it always calls
`cyberos_index.py update`, which is idempotent. If the index does not
exist (`cyberos_index.py build` never ran), this hook is a no-op so the
write succeeds.
"""
from __future__ import annotations
import argparse
import os
import subprocess
import sys
import time
from pathlib import Path


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def index_db_exists(brain_root: Path) -> bool:
    return (brain_root / ".cyberos-memory" / "index" / "cyberos.db").exists()


def run_index_update(brain_root: Path, *, quiet: bool = True) -> int:
    """Best-effort update. Returns rc; never raises."""
    tool = brain_root / "runtime" / "tools" / "cyberos_index.py"
    if not tool.exists():
        return 0
    if not index_db_exists(brain_root):
        return 0  # nothing to update
    try:
        out = subprocess.run(
            ["python3", str(tool), str(brain_root / ".cyberos-memory"), "update"],
            capture_output=quiet, text=True, timeout=30,
        )
        return out.returncode
    except Exception:
        return -1


def cmd_on_write(args):
    """Called by brain_writer after a successful write."""
    brain_root = find_brain()
    t0 = time.perf_counter()
    rc = run_index_update(brain_root, quiet=not args.verbose)
    dt = (time.perf_counter() - t0) * 1000
    if args.verbose:
        print(f"  index-hook on-write: rc={rc} dt={dt:.1f}ms path={args.path}")
    # Always exit 0 — index update is best-effort; never block the write
    return 0


def cmd_stop_hook(_args):
    """Run on Claude Code session.end to refresh the index."""
    brain_root = find_brain()
    rc = run_index_update(brain_root, quiet=True)
    # Stop hooks are silent unless they have something to flag
    if rc not in (0, -1):
        print(f"  ⚠ cyberos_index.py update returned rc={rc}", file=sys.stderr)
    return 0


def main():
    p = argparse.ArgumentParser(description="incremental SQLite index update hook (Aspect 9.2)")
    sub = p.add_subparsers(dest="cmd", required=True)
    pow = sub.add_parser("on-write")
    pow.add_argument("path", help="relative memory path just written")
    pow.add_argument("--verbose", "-v", action="store_true")
    pow.set_defaults(func=cmd_on_write)
    sub.add_parser("stop-hook").set_defaults(func=cmd_stop_hook)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
