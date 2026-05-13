"""
bench/append.py — group-commit throughput benchmark.

Targets (audit report §5):

    | scenario                | current | target  | hard floor |
    | single producer         | 250/s   | 6,000/s | 3,000/s    |
    | 8 concurrent producers  | n/a     | 9,000/s | 5,000/s    |

The benchmark hammers a fresh store with N AuditRecords, then runs
``cyberos verify`` to assert the chain is intact. A run that achieves
its target but fails verification is a benchmark FAIL — performance
must never come at the cost of correctness.

Run:

    python -m bench.append --producers 1 --records 50000
    python -m bench.append --producers 8 --records 50000
    python -m bench.append --producers 1 --records 50000 --window-ms 0 --batch 1   # baseline
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
import threading
import time
from pathlib import Path

# Sandbox path: this benchmark creates a one-off store under /tmp, never
# inside the real .cyberos-memory/. Cleaned up on exit unless --keep.


def _producer(writer, n: int, actor: str, start_idx: int) -> None:
    from cyberos.core.writer import AuditRecord
    for i in range(n):
        writer.submit(
            AuditRecord(
                op="view",
                path=f"memories/bench/{start_idx + i:08d}.md",
                actor=actor,
                content_sha256="0" * 64,
            )
        )


def run(producers: int, records: int, window_ms: int, batch: int, keep: bool) -> dict:
    from cyberos.core.writer import Writer, WriterConfig
    from cyberos.core.walker import verify_segments

    workdir = Path(tempfile.mkdtemp(prefix="cyberos-bench-"))
    store = workdir / ".cyberos-memory"
    (store / "audit").mkdir(parents=True, exist_ok=True)
    try:
        cfg = WriterConfig(coalesce_window_ms=window_ms, coalesce_max_batch=batch)
        per_thread = records // producers
        total = per_thread * producers

        writer = Writer(store, config=cfg)
        writer.open()

        t0 = time.perf_counter_ns()
        threads: list[threading.Thread] = []
        for p in range(producers):
            t = threading.Thread(
                target=_producer,
                args=(writer, per_thread, f"prod-{p}", p * per_thread),
                daemon=True,
            )
            threads.append(t)
            t.start()
        for t in threads:
            t.join()
        wall_ns = time.perf_counter_ns() - t0

        writer.close()

        # Verify the chain — bench failure if corrupt.
        segments = sorted(
            p for p in (store / "audit").glob("*.binlog") if p.name != "current.binlog"
        )
        current = store / "audit" / "current.binlog"
        if current.exists():
            segments.append(current)
        n_verified = verify_segments(segments)
        assert n_verified == total, f"verified {n_verified} != written {total}"

        wall_s = wall_ns / 1_000_000_000
        return {
            "producers": producers,
            "records": total,
            "window_ms": window_ms,
            "batch": batch,
            "wall_s": wall_s,
            "throughput_per_s": total / wall_s,
            "verified": n_verified,
        }
    finally:
        if not keep:
            shutil.rmtree(workdir, ignore_errors=True)


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--producers", type=int, default=1)
    ap.add_argument("--records", type=int, default=50_000)
    ap.add_argument("--window-ms", type=int, default=5)
    ap.add_argument("--batch", type=int, default=16)
    ap.add_argument("--keep", action="store_true", help="don't clean up workdir")
    args = ap.parse_args(argv)

    result = run(args.producers, args.records, args.window_ms, args.batch, args.keep)
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
