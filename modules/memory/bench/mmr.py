"""
bench/mmr.py — MMR scale characterization.

Measures:

* Append rate (leaves/sec) at 1k / 10k / 100k leaves.
* Root-computation latency at each size.
* Inclusion-proof construction latency at each size (with O(n) replay
  cost included — this is the real cost in the binlog-as-leaf-store
  model).
* On-disk ``peaks.bin`` size at each size.

The Stage-3 promotion gate from ``PROPOSAL.md`` Appendix mentions a 2-week
soak; this benchmark gives the absolute numbers that gate is implicitly
comparing against. If append rate at 100k leaves is materially worse
than the existing chain throughput, Stage-3 is premature.

Run::

    python -m bench.mmr --sizes 1000,10000
    python -m bench.mmr --sizes 1000,10000,100000   # ~5 minutes
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
import tempfile
import time
from pathlib import Path
from typing import Sequence

from cyberos.core.mmr import MMR, OnDiskMMR


def _bench_size(n: int, *, persist: bool, store_root: Path) -> dict:
    """Append n synthetic leaves; measure append/root/proof costs."""
    leaves = [f"leaf-{i:08x}-{i}".encode() for i in range(n)]

    if persist:
        store = store_root / f"size-{n}" / ".cyberos/memory/store"
        store.mkdir(parents=True, exist_ok=True)
        (store / "audit" / "mmr").mkdir(parents=True, exist_ok=True)
        mmr = OnDiskMMR(store)
    else:
        mmr = MMR()

    # Append rate
    t0 = time.perf_counter_ns()
    for data in leaves:
        mmr.append_leaf(data)
    append_total_ns = time.perf_counter_ns() - t0
    append_rate = n / (append_total_ns / 1e9)

    # Root compute (warm cache; the peak stack is in memory)
    root_samples = []
    for _ in range(10):
        t0 = time.perf_counter_ns()
        _ = mmr.root()
        root_samples.append(time.perf_counter_ns() - t0)
    root_ns_p50 = statistics.median(root_samples)

    # Inclusion proof at random leaf indices (3 samples for p50 stability)
    import random
    rng = random.Random(0xC0DE)
    proof_samples = []
    for _ in range(min(5, n)):
        target = rng.randrange(n)
        t0 = time.perf_counter_ns()
        _ = mmr.inclusion_proof(target, iter(leaves))
        proof_samples.append(time.perf_counter_ns() - t0)
    proof_ns_p50 = statistics.median(proof_samples)

    on_disk = -1
    if persist:
        peaks_path = mmr.store / "audit" / "mmr" / "peaks.bin"
        if peaks_path.is_file():
            on_disk = peaks_path.stat().st_size

    return {
        "leaf_count": n,
        "append_total_s": append_total_ns / 1e9,
        "append_rate_per_s": append_rate,
        "root_us_p50": root_ns_p50 / 1000,
        "inclusion_proof_us_p50": proof_ns_p50 / 1000,
        "peaks_bytes_on_disk": on_disk,
        "peak_count": len(mmr.peaks),
    }


def main(argv: Sequence[str] | None = None) -> int:
    ap = argparse.ArgumentParser(prog="bench.mmr")
    ap.add_argument("--sizes", default="1000,10000",
                    help="comma-separated leaf counts (default '1000,10000')")
    ap.add_argument("--no-persist", action="store_true",
                    help="skip on-disk persistence; in-memory only")
    args = ap.parse_args(argv)

    sizes = [int(s) for s in args.sizes.split(",") if s.strip()]
    results: list[dict] = []
    with tempfile.TemporaryDirectory(prefix="cyberos-bench-mmr-") as td:
        for n in sizes:
            r = _bench_size(n, persist=not args.no_persist, store_root=Path(td))
            results.append(r)
            print(
                f"{n:>7} leaves: "
                f"append {r['append_rate_per_s']:>8.0f}/s, "
                f"root {r['root_us_p50']:>6.1f}us, "
                f"proof {r['inclusion_proof_us_p50']:>8.0f}us, "
                f"peaks.bin {r['peaks_bytes_on_disk']:>6}B "
                f"({r['peak_count']} peaks)",
                file=sys.stderr,
            )
    print(json.dumps({"sizes": results}, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
