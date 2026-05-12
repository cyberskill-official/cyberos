#!/usr/bin/env python3
"""
cyberos_lazy.py — streaming / lazy session-start loader.

Aspect 9.1 of the Layer-1 improvement catalog.

For BRAIN sizes >1000 files, eagerly loading every memory at session
start is wasteful. This loader walks `.cyberos-memory/` in two phases:

  Phase A — eager (always loaded):
    - manifest.json
    - meta/protocol-history/ index (paths only, no contents)
    - reconciliation_checkpoint (single audit row resolution)
    - meta/legacy-ids.md, meta/legacy-files.md (small allowlists)

  Phase B — lazy (loaded on first access):
    - memory bodies   (.cyberos-memory/**/*.md)
    - audit ledgers   (.cyberos-memory/audit/*.jsonl)
    - index database  (.cyberos-memory/index/cyberos.db)

The streaming pattern: yield each entry as the walker encounters it, so
agents that only need "give me the first 5 facts in scope X" don't pay
the full-walk cost.

This module is a utility. It does not replace `brain_writer.session-start`;
it's intended for read-mostly consumers (cyberos_show.py, cyberos_search,
MCP server) that don't need the full chain validated upfront.

Quick benchmark inside this file:
    python3 cyberos_lazy.py benchmark

Compares eager-load vs lazy-load wall time.
"""
from __future__ import annotations
import argparse
import json
import sys
import time
from pathlib import Path
from typing import Iterator


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def load_eager(brain_root: Path) -> dict:
    """Phase A — load only the per-project pointers + checkpoints.

    Returns a dict with manifest, checkpoint_audit_id, legacy_ids,
    legacy_files. Total disk reads: ≤ 5 files (typically << 100 KB).
    """
    brain = brain_root / ".cyberos-memory"
    out = {}
    mf = brain / "manifest.json"
    if mf.exists():
        try:
            out["manifest"] = json.loads(mf.read_text(encoding="utf-8"))
        except Exception:
            out["manifest"] = {}
    else:
        out["manifest"] = {}

    out["checkpoint_audit_id"] = (
        out["manifest"].get("reconciliation_checkpoint", {}).get("audit_id")
    )

    legacy_ids = brain / "meta" / "legacy-ids.md"
    out["legacy_ids"] = legacy_ids.read_text(encoding="utf-8") if legacy_ids.exists() else ""
    legacy_files = brain / "meta" / "legacy-files.md"
    out["legacy_files"] = legacy_files.read_text(encoding="utf-8") if legacy_files.exists() else ""
    return out


def stream_memories(brain_root: Path, scope_prefix: str = "") -> Iterator[tuple[str, Path]]:
    """Phase B — lazy walk. Yields (rel_path, path) one at a time.

    Caller can break out early; we never read content here.
    """
    brain = brain_root / ".cyberos-memory"
    for md in brain.rglob("*.md"):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/")):
            continue
        if scope_prefix and not rel.startswith(scope_prefix):
            continue
        yield rel, md


def cmd_eager(args):
    brain_root = find_brain()
    t0 = time.perf_counter()
    state = load_eager(brain_root)
    dt = (time.perf_counter() - t0) * 1000
    print(f"  eager-load:    {dt:.2f} ms")
    print(f"  manifest fields: {len(state['manifest'])}")
    print(f"  checkpoint:    {state['checkpoint_audit_id']}")
    print(f"  legacy_ids:    {len(state['legacy_ids']):,} chars")
    return 0


def cmd_lazy_count(args):
    brain_root = find_brain()
    t0 = time.perf_counter()
    n = sum(1 for _ in stream_memories(brain_root, args.scope))
    dt = (time.perf_counter() - t0) * 1000
    print(f"  lazy-walk:     {dt:.2f} ms  ({n} memories matched)")
    return 0


def cmd_benchmark(args):
    brain_root = find_brain()
    # Eager
    t0 = time.perf_counter()
    state = load_eager(brain_root)
    eager_ms = (time.perf_counter() - t0) * 1000
    # Full eager (every memory's frontmatter parsed) — simulates today's behaviour
    import yaml
    t0 = time.perf_counter()
    full_count = 0
    for rel, p in stream_memories(brain_root):
        try:
            text = p.read_text(encoding="utf-8")
            if text.startswith("---\n"):
                end = text.find("\n---\n", 4)
                if end > 0:
                    yaml.safe_load(text[4:end])
            full_count += 1
        except Exception:
            continue
    full_ms = (time.perf_counter() - t0) * 1000
    # Lazy "first 5"
    t0 = time.perf_counter()
    first5 = []
    for rel, p in stream_memories(brain_root):
        first5.append(rel)
        if len(first5) >= 5:
            break
    lazy_first5_ms = (time.perf_counter() - t0) * 1000

    print(f"\n  Benchmark — brain: {brain_root}\n")
    print(f"  Eager load (manifest only):           {eager_ms:>7.2f} ms")
    print(f"  Full eager load ({full_count} memories' YAML): {full_ms:>7.2f} ms")
    print(f"  Lazy first-5 walk:                    {lazy_first5_ms:>7.2f} ms")
    if full_ms > 0:
        speedup = full_ms / max(lazy_first5_ms, 0.01)
        print(f"  Speedup (full → lazy-first-5):        {speedup:>7.1f}×")
    return 0


def main():
    p = argparse.ArgumentParser(description="streaming session-start loader (Aspect 9.1)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("eager").set_defaults(func=cmd_eager)
    plc = sub.add_parser("lazy-count")
    plc.add_argument("--scope", default="")
    plc.set_defaults(func=cmd_lazy_count)
    sub.add_parser("benchmark").set_defaults(func=cmd_benchmark)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
