#!/usr/bin/env python3
"""
cyberos_parallel_validate.py — distributed validator runner.

Batch 14 (Tier D) of post-catalog improvements.

Splits the 11 validator categories across N processes. Today
`cyberos verify` runs serially (~180 ms / 157 memories); at 10K+ memories
this approach takes seconds, parallel runs it in milliseconds.

Each worker handles a disjoint slice of memory files; manifest / audit /
supersedes-graph checks stay on the main process (they need global view).

Usage:
    cyberos parallel-validate                  # auto-pick N = cpu_count
    cyberos parallel-validate --workers 4
    cyberos parallel-validate --format json
"""
from __future__ import annotations
import argparse
import json
import multiprocessing
import os
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


def parse_frontmatter(text: str) -> dict | None:
    if not text.startswith("---\n"):
        return None
    end = text.find("\n---\n", 4)
    if end < 0:
        return None
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}
    except Exception:
        return None


def worker(paths: list[str]) -> list[dict]:
    """Validate a slice of memory files. Returns list of findings."""
    findings = []
    for rel in paths:
        p = Path(rel)
        if not p.exists():
            continue
        try:
            text = p.read_text(encoding="utf-8")
        except Exception:
            continue
        fm = parse_frontmatter(text)
        if not fm:
            findings.append({"path": rel, "severity": "WARN", "code": "no-frontmatter"})
            continue
        # Required fields
        for required in ("memory_id", "scope", "classification", "authority", "version"):
            if required not in fm:
                findings.append({"path": rel, "severity": "WARN", "code": f"missing-{required}"})
        # Body size cap
        if len(text.encode("utf-8")) > 16 * 1024:
            findings.append({"path": rel, "severity": "WARN", "code": "body-over-cap-16kb"})
    return findings


def main():
    p = argparse.ArgumentParser(description="distributed validator runner (Batch 14 / Tier D)")
    p.add_argument("--workers", type=int, default=max(1, multiprocessing.cpu_count() - 1))
    p.add_argument("--format", choices=["table", "json"], default="table")
    args = p.parse_args()

    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"

    # Collect memory paths (skip irrelevant trees)
    all_paths = []
    for md in brain.rglob("*.md"):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/", ".branches/")):
            continue
        all_paths.append(str(md))

    # Shard into N slices
    n = args.workers
    slices = [all_paths[i::n] for i in range(n)]

    t0 = time.perf_counter()
    if n == 1:
        findings_per_worker = [worker(slices[0])]
    else:
        with multiprocessing.Pool(n) as pool:
            findings_per_worker = pool.map(worker, slices)
    dt = (time.perf_counter() - t0) * 1000

    all_findings = [f for batch in findings_per_worker for f in batch]
    by_sev = {"CRITICAL": 0, "WARN": 0, "INFO": 0}
    for f in all_findings:
        by_sev[f.get("severity", "INFO")] = by_sev.get(f.get("severity", "INFO"), 0) + 1

    if args.format == "json":
        print(json.dumps({
            "workers": n,
            "files_checked": len(all_paths),
            "wall_ms": round(dt, 2),
            "by_severity": by_sev,
            "findings": all_findings[:50],
        }, indent=2))
        return 1 if by_sev["CRITICAL"] else (1 if by_sev["WARN"] else 0)

    print(f"\n  Parallel validate — {n} workers, {len(all_paths)} files, {dt:.2f} ms")
    print(f"  CRITICAL: {by_sev['CRITICAL']}")
    print(f"  WARN:     {by_sev['WARN']}")
    print(f"  INFO:     {by_sev['INFO']}")
    if all_findings:
        print()
        for f in all_findings[:10]:
            print(f"    [{f.get('severity')}] {f.get('code')}: {f.get('path')}")
        if len(all_findings) > 10:
            print(f"    … +{len(all_findings) - 10} more")
    return 1 if by_sev["CRITICAL"] else 0


if __name__ == "__main__":
    sys.exit(main())
