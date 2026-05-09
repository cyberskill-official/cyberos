#!/usr/bin/env python3
"""
benchmark — measure cyberos-validate + cyberos-export performance against any
`.cyberos-memory/` store. Tracks the Stage 1 / Stage 2 / Stage 4 success metrics.

Usage:
    python3 benchmark.py <store-path> [--runs 5]
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
import sys
import time
from pathlib import Path

HERE = Path(__file__).parent


def time_run(cmd: list[str]) -> float:
    t0 = time.perf_counter()
    subprocess.run(cmd, capture_output=True, check=True)
    return time.perf_counter() - t0


def percentile(values: list[float], p: float) -> float:
    if not values:
        return float("nan")
    s = sorted(values)
    k = max(0, min(len(s) - 1, int(round((p / 100) * (len(s) - 1)))))
    return s[k]


def benchmark_validate(store: Path, runs: int) -> dict:
    times = []
    for _ in range(runs):
        times.append(time_run(["python3",
                               str(HERE / "cyberos_validate.py"),
                               str(store)]))
    return {
        "runs": runs,
        "p50_ms": round(percentile(times, 50) * 1000, 1),
        "p95_ms": round(percentile(times, 95) * 1000, 1),
        "p99_ms": round(percentile(times, 99) * 1000, 1),
        "min_ms": round(min(times) * 1000, 1),
        "max_ms": round(max(times) * 1000, 1),
    }


def benchmark_export(store: Path, runs: int) -> dict:
    import tempfile
    times = []
    bundle_size = None
    file_count = None
    for _ in range(runs):
        with tempfile.TemporaryDirectory() as tmp:
            t0 = time.perf_counter()
            result = subprocess.run(
                ["python3", str(HERE / "cyberos_export.py"),
                 str(store), "-o", tmp],
                capture_output=True, check=True, text=True)
            times.append(time.perf_counter() - t0)
            summary = json.loads(result.stdout)
            bundle_size = summary["total_compressed"]
            file_count = summary["file_count"]
    return {
        "runs": runs,
        "p50_ms": round(percentile(times, 50) * 1000, 1),
        "p95_ms": round(percentile(times, 95) * 1000, 1),
        "min_ms": round(min(times) * 1000, 1),
        "bundle_size_bytes": bundle_size,
        "file_count": file_count,
    }


def store_stats(store: Path) -> dict:
    audit = store / "audit"
    audit_rows = 0
    audit_bytes = 0
    if audit.exists():
        for f in audit.glob("*.jsonl"):
            audit_bytes += f.stat().st_size
            with f.open("rb") as fp:
                audit_rows += sum(1 for line in fp if line.strip())
    memory_files = sum(1 for p in store.rglob("*.md") if p.is_file())
    total_bytes = sum(p.stat().st_size for p in store.rglob("*")
                      if p.is_file()
                      and not p.relative_to(store).as_posix().startswith(
                          ("index/", "exports/")))
    return {
        "audit_rows": audit_rows,
        "audit_bytes": audit_bytes,
        "memory_files": memory_files,
        "total_bytes": total_bytes,
    }


def main() -> int:
    parser = argparse.ArgumentParser(prog="benchmark")
    parser.add_argument("path")
    parser.add_argument("--runs", type=int, default=5)
    args = parser.parse_args()

    store = Path(args.path).resolve()
    if (store / ".cyberos-memory").is_dir():
        store = store / ".cyberos-memory"
    if not store.is_dir():
        print(f"error: {store} not a directory", file=sys.stderr)
        return 3

    print(f"# Benchmark: {store}")
    print()

    stats = store_stats(store)
    print("## Store stats")
    for k, v in stats.items():
        print(f"  {k}: {v}")
    print()

    print(f"## cyberos-validate ({args.runs} runs)")
    val = benchmark_validate(store, args.runs)
    for k, v in val.items():
        print(f"  {k}: {v}")
    print()

    print(f"## cyberos-export ({args.runs} runs)")
    exp = benchmark_export(store, args.runs)
    for k, v in exp.items():
        print(f"  {k}: {v}")
    print()

    # Stage 1/2 success-metric checks
    print("## Stage success-metric check")
    print(f"  Stage 2 — validate <500ms p95 on fresh store: "
          f"{'✅' if val['p95_ms'] < 500 else '❌'} "
          f"({val['p95_ms']}ms)")
    print(f"  Stage 2 — validate <2000ms p95 on max-cap store: "
          f"{'✅' if val['p95_ms'] < 2000 else '❌'} "
          f"({val['p95_ms']}ms)")

    return 0


if __name__ == "__main__":
    sys.exit(main())
