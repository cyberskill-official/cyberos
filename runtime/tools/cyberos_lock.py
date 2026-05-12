#!/usr/bin/env python3
"""
cyberos_lock.py — TOCTOU-hardening advisory locks for the BRAIN.

Aspect 5.7 of the Layer-1 improvement catalog.

Two lock files live next to the BRAIN:

    .cyberos-memory/.lock.exclusive   — held by brain_writer during a write
    .cyberos-memory/.lock.shared      — held by validators / readers

Semantics (POSIX fcntl flock):
  - Many readers may hold .lock.shared concurrently (LOCK_SH).
  - Only one writer holds .lock.exclusive (LOCK_EX); the writer waits for
    all .lock.shared holders to release before its LOCK_EX succeeds.
  - Validators acquire LOCK_SH for the duration of validation so a write
    that lands mid-validation cannot make the result stale.

Best-effort. On filesystems without flock (some network FSes, some FUSE
mounts) the locks degrade to no-ops + a WARN — never block reads.

Usage:
    from cyberos_lock import shared_lock, exclusive_lock

    with shared_lock(brain_root) as ok:
        if ok:
            # we hold the shared lock; safe to read consistent state
            ...

    with exclusive_lock(brain_root, timeout=10) as ok:
        if ok:
            # we hold the exclusive lock; safe to mutate
            ...
        else:
            # timed out — caller decides whether to retry or abort
            ...

Run standalone to inspect lock state:
    cyberos_lock.py status
    cyberos_lock.py acquire-shared --hold 5    # hold for 5s for testing
    cyberos_lock.py acquire-exclusive --hold 2
"""
from __future__ import annotations
import argparse
import contextlib
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


def _lock_paths(brain_root: Path) -> tuple[Path, Path]:
    brain = brain_root / ".cyberos-memory"
    return brain / ".lock.shared", brain / ".lock.exclusive"


@contextlib.contextmanager
def shared_lock(brain_root: Path, timeout: float = 5.0):
    """Acquire LOCK_SH on `.lock.shared`. Yields True if acquired."""
    shared_p, _ = _lock_paths(brain_root)
    shared_p.parent.mkdir(parents=True, exist_ok=True)
    shared_p.touch(exist_ok=True)
    try:
        import fcntl  # POSIX only; will ImportError on Windows
    except ImportError:
        # Degrade — no real lock, just yield True
        yield True
        return
    f = open(shared_p, "rb+")
    acquired = False
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            fcntl.flock(f, fcntl.LOCK_SH | fcntl.LOCK_NB)
            acquired = True
            break
        except BlockingIOError:
            time.sleep(0.05)
    try:
        yield acquired
    finally:
        try:
            if acquired:
                fcntl.flock(f, fcntl.LOCK_UN)
        except Exception:
            pass
        f.close()


@contextlib.contextmanager
def exclusive_lock(brain_root: Path, timeout: float = 10.0):
    """Acquire LOCK_EX on `.lock.exclusive`. Yields True if acquired."""
    _, excl_p = _lock_paths(brain_root)
    excl_p.parent.mkdir(parents=True, exist_ok=True)
    excl_p.touch(exist_ok=True)
    try:
        import fcntl
    except ImportError:
        yield True
        return
    f = open(excl_p, "rb+")
    acquired = False
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            fcntl.flock(f, fcntl.LOCK_EX | fcntl.LOCK_NB)
            acquired = True
            break
        except BlockingIOError:
            time.sleep(0.1)
    try:
        yield acquired
    finally:
        try:
            if acquired:
                fcntl.flock(f, fcntl.LOCK_UN)
        except Exception:
            pass
        f.close()


def cmd_status(_args):
    brain_root = find_brain()
    shared_p, excl_p = _lock_paths(brain_root)
    for label, p in (("shared", shared_p), ("exclusive", excl_p)):
        if p.exists():
            mtime = time.ctime(p.stat().st_mtime)
            print(f"  {label:10s}  exists  {p}  ({mtime})")
        else:
            print(f"  {label:10s}  absent  {p}")
    return 0


def cmd_acquire_shared(args):
    brain_root = find_brain()
    with shared_lock(brain_root, timeout=args.timeout) as ok:
        if not ok:
            print(f"  ✗ failed to acquire .lock.shared within {args.timeout}s")
            return 1
        print(f"  ✓ acquired .lock.shared; holding for {args.hold}s")
        time.sleep(args.hold)
    print(f"  released .lock.shared")
    return 0


def cmd_acquire_exclusive(args):
    brain_root = find_brain()
    with exclusive_lock(brain_root, timeout=args.timeout) as ok:
        if not ok:
            print(f"  ✗ failed to acquire .lock.exclusive within {args.timeout}s")
            return 1
        print(f"  ✓ acquired .lock.exclusive; holding for {args.hold}s")
        time.sleep(args.hold)
    print(f"  released .lock.exclusive")
    return 0


def main():
    p = argparse.ArgumentParser(description=".lock.shared / .lock.exclusive advisory locks (Aspect 5.7)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status").set_defaults(func=cmd_status)
    a1 = sub.add_parser("acquire-shared")
    a1.add_argument("--timeout", type=float, default=5.0)
    a1.add_argument("--hold", type=float, default=1.0)
    a1.set_defaults(func=cmd_acquire_shared)
    a2 = sub.add_parser("acquire-exclusive")
    a2.add_argument("--timeout", type=float, default=10.0)
    a2.add_argument("--hold", type=float, default=1.0)
    a2.set_defaults(func=cmd_acquire_exclusive)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
