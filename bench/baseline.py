"""
bench/baseline.py — record + compare performance baselines.

Workflow::

    # Record a fresh baseline (one-shot, manual)
    python -m bench.baseline record --out bench/baseline.json

    # Compare current run against baseline (called by nightly scheduled task)
    python -m bench.baseline check --baseline bench/baseline.json

The baseline is HARDWARE-SPECIFIC. Two callers comparing baselines from
different machines is a category error — we record the platform/CPU/SSD
where the baseline was taken so the comparison emits a warning if it
looks like the host changed.

Regression threshold defaults to 30% slower vs baseline at p50. Easy to
override per-metric in the JSON itself.
"""

from __future__ import annotations

import argparse
import json
import platform
import sys
import time
from pathlib import Path


_DEFAULT_REGRESSION_PCT = 30.0


def _host_fingerprint() -> dict:
    """Capture enough metadata to recognise when the host changed."""
    import os
    return {
        "platform": sys.platform,
        "machine": platform.machine(),
        "processor": platform.processor() or "unknown",
        "python": sys.version.split()[0],
        "cpu_count": os.cpu_count(),
        "node": platform.node(),
    }


def _run_frontmatter_compare() -> dict:
    """Run msgspec-vs-pyyaml bench and return both summaries."""
    from bench import frontmatter as fm_bench
    corpus = Path("/tmp/cyberos-fm-corpus-baseline")
    if not corpus.exists() or len(list(corpus.glob("*.md"))) < 2000:
        fm_bench.gen_corpus(corpus, 2000)
    files = [p.read_bytes() for p in sorted(corpus.glob("*.md"))[:2000]]
    return {
        "msgspec": fm_bench.summary("msgspec", fm_bench.bench_msgspec(files)),
        "pyyaml": fm_bench.summary("pyyaml", fm_bench.bench_pyyaml(files)),
    }


def _run_append(producers: int, records: int) -> dict:
    """Single append bench at the given concurrency."""
    from bench import append as ap_bench
    return ap_bench.run(producers, records, window_ms=5, batch=16, keep=False)


def _run_cold_cli() -> dict:
    """Time `cyberos --help` 20× and summarise."""
    import statistics
    import subprocess
    times_ms = []
    for _ in range(20):
        t0 = time.perf_counter_ns()
        rc = subprocess.run(
            [sys.executable, "-m", "cyberos", "--help"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
        ).returncode
        if rc != 0:
            continue
        times_ms.append((time.perf_counter_ns() - t0) / 1_000_000)
    times_ms.sort()
    return {
        "n": len(times_ms),
        "p50_ms": statistics.median(times_ms),
        "p95_ms": times_ms[int(len(times_ms) * 0.95)] if times_ms else 0,
        "max_ms": max(times_ms) if times_ms else 0,
    }


def record(out_path: Path) -> dict:
    """Run every benchmark and dump the result as the new baseline."""
    print("recording baseline (this takes ~30s)...", file=sys.stderr)
    payload = {
        "version": 1,
        "recorded_at_ns": time.time_ns(),
        "host": _host_fingerprint(),
        "regression_threshold_pct": _DEFAULT_REGRESSION_PCT,
        "metrics": {
            "frontmatter_msgspec_p50_us": _run_frontmatter_compare()["msgspec"]["p50_us"],
            "frontmatter_msgspec_p99_us": _run_frontmatter_compare()["msgspec"]["p99_us"],
            "append_1producer_per_s": _run_append(1, 3000)["throughput_per_s"],
            "append_4producer_per_s": _run_append(4, 8000)["throughput_per_s"],
            "cold_cli_help_p50_ms": _run_cold_cli()["p50_ms"],
        },
    }
    out_path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"baseline → {out_path}", file=sys.stderr)
    return payload


def check(baseline_path: Path) -> int:
    """Run benchmarks; compare against baseline; report regressions."""
    if not baseline_path.is_file():
        print(f"FATAL: baseline file not found: {baseline_path}", file=sys.stderr)
        return 2
    baseline = json.loads(baseline_path.read_text(encoding="utf-8"))
    threshold = baseline.get("regression_threshold_pct", _DEFAULT_REGRESSION_PCT)

    # Host fingerprint check — comparing baselines across machines is a
    # category error. Exit early with a clear message.
    current_host = _host_fingerprint()
    baseline_host = baseline.get("host", {})
    drift_keys = [
        k for k in ("platform", "machine", "processor")
        if baseline_host.get(k) != current_host.get(k)
    ]
    if drift_keys:
        print(
            f"WARN: host changed since baseline was recorded ({drift_keys}). "
            f"  baseline host: {baseline_host}\n"
            f"  current host:  {current_host}\n"
            f"Re-record with: python -m bench.baseline record",
            file=sys.stderr,
        )
        # Exit 2 means "setup error / comparator can't reason"; nightly
        # task treats this as a one-shot notification, not a P-incident.
        return 2

    print("running benchmarks for comparison...", file=sys.stderr)
    current_fm = _run_frontmatter_compare()
    current_app_1 = _run_append(1, 3000)
    current_app_4 = _run_append(4, 8000)
    current_cli = _run_cold_cli()
    current = {
        "frontmatter_msgspec_p50_us": current_fm["msgspec"]["p50_us"],
        "frontmatter_msgspec_p99_us": current_fm["msgspec"]["p99_us"],
        "append_1producer_per_s": current_app_1["throughput_per_s"],
        "append_4producer_per_s": current_app_4["throughput_per_s"],
        "cold_cli_help_p50_ms": current_cli["p50_ms"],
    }

    # Per-metric direction: for *_per_s, higher = better; for *_us/_ms, lower.
    higher_better = {"append_1producer_per_s", "append_4producer_per_s"}
    regressions: list[str] = []
    print(file=sys.stderr)
    print(f"{'metric':<42} {'baseline':>14} {'current':>14}  {'delta':>10}", file=sys.stderr)
    for name, base_val in baseline["metrics"].items():
        cur_val = current.get(name)
        if cur_val is None:
            continue
        if name in higher_better:
            delta_pct = ((cur_val - base_val) / base_val) * 100 if base_val else 0
            regressed = delta_pct < -threshold
        else:
            delta_pct = ((cur_val - base_val) / base_val) * 100 if base_val else 0
            regressed = delta_pct > threshold
        marker = " ⚠️ REG" if regressed else ""
        print(
            f"{name:<42} {base_val:>14.2f} {cur_val:>14.2f}  {delta_pct:+9.1f}%{marker}",
            file=sys.stderr,
        )
        if regressed:
            regressions.append(
                f"{name}: {base_val:.2f} → {cur_val:.2f} ({delta_pct:+.1f}%)"
            )

    print(file=sys.stderr)
    if regressions:
        print(f"REGRESSIONS ({len(regressions)}):", file=sys.stderr)
        for r in regressions:
            print(f"  - {r}", file=sys.stderr)
        return 1
    print("OK — no regressions vs baseline", file=sys.stderr)
    return 0


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(prog="bench.baseline")
    sub = p.add_subparsers(dest="cmd", required=True)
    rec = sub.add_parser("record")
    rec.add_argument("--out", default="bench/baseline.json")
    chk = sub.add_parser("check")
    chk.add_argument("--baseline", default="bench/baseline.json")
    args = p.parse_args(argv)

    if args.cmd == "record":
        record(Path(args.out))
        return 0
    if args.cmd == "check":
        return check(Path(args.baseline))
    return 2


if __name__ == "__main__":
    sys.exit(main())
