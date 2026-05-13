"""
bench/cold_cli.py — cold-start ``cyberos --help`` benchmark wrapper.

Target (audit report §5): ``cyberos --help`` < 30ms p50; hard floor 50ms.

Hyperfine is the right tool for this; we shell out to it. If hyperfine
isn't installed, falls back to a coarse Python timer (less precise —
hyperfine handles warmup, GC variance, and statistical reporting).

Run:

    python -m bench.cold_cli                  # uses hyperfine if available
    python -m bench.cold_cli --runs 50        # override default 20 runs

The cold path is gated by lazy imports inside :mod:`cyberos.__main__`.
A regression here typically means someone added a top-level import to
__main__.py or __init__.py that pulls in msgspec / sqlite3 / mmap.
"""

from __future__ import annotations

import argparse
import json
import shutil
import statistics
import subprocess
import sys
import time
from pathlib import Path


def _have_hyperfine() -> bool:
    return shutil.which("hyperfine") is not None


def _python_timer(runs: int) -> dict:
    times_ms: list[float] = []
    for _ in range(runs):
        t0 = time.perf_counter_ns()
        rc = subprocess.run(
            [sys.executable, "-m", "cyberos", "--help"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        ).returncode
        elapsed_ms = (time.perf_counter_ns() - t0) / 1_000_000
        if rc != 0:
            continue
        times_ms.append(elapsed_ms)
    if not times_ms:
        return {"error": "no successful runs"}
    quantiles = statistics.quantiles(times_ms, n=100, method="inclusive")
    return {
        "runs": len(times_ms),
        "mean_ms": statistics.mean(times_ms),
        "p50_ms": quantiles[49],
        "p95_ms": quantiles[94],
        "p99_ms": quantiles[98],
        "min_ms": min(times_ms),
        "max_ms": max(times_ms),
    }


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--runs", type=int, default=20)
    args = ap.parse_args(argv)

    if _have_hyperfine():
        # Hyperfine handles warmup + statistics; let it own the run.
        subprocess.run(
            [
                "hyperfine",
                "--warmup", "5",
                "--runs", str(args.runs),
                f"{sys.executable} -m cyberos --help",
            ],
            check=False,
        )
        return 0

    print(json.dumps(_python_timer(args.runs), indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
