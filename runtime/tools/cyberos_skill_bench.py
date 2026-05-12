#!/usr/bin/env python3
"""
cyberos_skill_bench.py — skill cost + accuracy benchmarks (Tier α.6).

Runs a skill's test corpus N times, records token usage, latency, accuracy.
Baseline file at runtime/tests/skills/<skill>/baseline.json lets future
runs detect regressions.

Usage:
    cyberos skill-bench fr-with-tasks                  # against current baseline
    cyberos skill-bench fr-with-tasks --record         # rewrite baseline.json
    cyberos skill-bench fr-with-tasks --runs 3         # average 3 runs
    cyberos skill-bench fr-with-tasks --no-llm         # harness only

Baseline schema:
    {
      "skill": "fr-with-tasks",
      "skill_version": "0.1.0",
      "recorded_at": "...",
      "model": "claude-sonnet-4-6",
      "fixtures": {
        "slack-hr-bot-mvp": {
          "tokens_p50": 18000, "tokens_p95": 22000,
          "iterations_p50": 1, "iterations_p95": 2,
          "cost_p50_usd": 0.12, "cost_p95_usd": 0.18,
          "pass_rate": 1.0
        },
        ...
      }
    }
"""
from __future__ import annotations
import argparse
import json
import statistics
import sys
import tempfile
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


def cmd_bench(args):
    brain_root = find_brain()
    fixtures_dir = brain_root / "runtime" / "tests" / "skills" / args.skill_id / "fixtures"
    if not fixtures_dir.exists():
        print(f"  ✗ no fixtures dir for {args.skill_id}", file=sys.stderr); return 2
    try:
        import yaml
    except ImportError:
        print(f"  ✗ pyyaml required", file=sys.stderr); return 3

    sys.path.insert(0, str(brain_root / "runtime" / "skill_runners"))
    try:
        from base import load_runner  # type: ignore
    except ImportError:
        print(f"  ✗ runner base missing", file=sys.stderr); return 3

    skill_id = args.skill_id if "/" in args.skill_id else f"cuo/cpo/{args.skill_id}"
    runner = load_runner(skill_id, brain_root)
    if runner is None:
        skill_id = args.skill_id if "/" in args.skill_id else f"cuo/cto/{args.skill_id}"
        runner = load_runner(skill_id, brain_root)
    if runner is None:
        print(f"  ✗ no runner for {args.skill_id}", file=sys.stderr); return 2

    runner.model = args.model
    fixtures = sorted(fixtures_dir.glob("*.yaml")) + sorted(fixtures_dir.glob("*.yml"))
    if not fixtures:
        print(f"  ✗ no fixtures in {fixtures_dir}", file=sys.stderr); return 2

    per_fixture: dict[str, list[dict]] = {}
    for fx in fixtures:
        fx_data = yaml.safe_load(fx.read_text(encoding="utf-8"))
        name = fx_data.get("name", fx.stem)
        per_fixture[name] = []
        for r in range(args.runs):
            if args.no_llm:
                per_fixture[name].append({"tokens": 0, "cost": 0.0,
                                          "iterations": 0, "passed": True, "latency_s": 0.0})
                continue
            t0 = time.perf_counter()
            with tempfile.TemporaryDirectory() as td:
                rr = runner.run(inputs={"pitch": fx_data["pitch"]},
                                output_dir=Path(td), max_iterations=args.max_iterations,
                                cache=None)
            dt = time.perf_counter() - t0
            per_fixture[name].append({
                "tokens": rr.tokens_used, "cost": rr.cost_usd,
                "iterations": rr.iterations,
                "passed": rr.status == "PASS",
                "latency_s": round(dt, 2),
            })

    # Aggregate
    def pct(values, p):
        if not values: return 0
        sv = sorted(values)
        idx = int(round((p / 100.0) * (len(sv) - 1)))
        return sv[idx]

    summary = {
        "skill": args.skill_id,
        "skill_version": runner.skill_version,
        "recorded_at": datetime.now(ICT).isoformat(timespec="seconds"),
        "model": runner.model,
        "runs_per_fixture": args.runs,
        "fixtures": {},
    }
    for name, runs in per_fixture.items():
        tok = [r["tokens"] for r in runs]
        cost = [r["cost"] for r in runs]
        it = [r["iterations"] for r in runs]
        passed = sum(1 for r in runs if r["passed"])
        summary["fixtures"][name] = {
            "tokens_p50": pct(tok, 50), "tokens_p95": pct(tok, 95),
            "iterations_p50": pct(it, 50), "iterations_p95": pct(it, 95),
            "cost_p50_usd": round(pct(cost, 50), 4),
            "cost_p95_usd": round(pct(cost, 95), 4),
            "pass_rate": passed / max(1, len(runs)),
            "latency_p50_s": pct([r["latency_s"] for r in runs], 50),
        }

    baseline_path = fixtures_dir.parent / "baseline.json"
    if args.record:
        baseline_path.write_text(json.dumps(summary, indent=2) + "\n")
        print(f"  ✓ baseline recorded: {baseline_path.relative_to(brain_root)}")
        return 0

    # Compare against baseline
    if baseline_path.exists():
        baseline = json.loads(baseline_path.read_text())
        regressions = []
        for name, m in summary["fixtures"].items():
            base = baseline.get("fixtures", {}).get(name)
            if not base: continue
            # Regression: tokens_p95 grew > 30%, cost_p95 grew > 30%, pass_rate dropped
            if m["tokens_p95"] > base["tokens_p95"] * 1.3 and base["tokens_p95"] > 0:
                regressions.append(f"{name}: tokens_p95 {base['tokens_p95']} → {m['tokens_p95']}")
            if m["cost_p95_usd"] > base["cost_p95_usd"] * 1.3 and base["cost_p95_usd"] > 0:
                regressions.append(f"{name}: cost_p95 ${base['cost_p95_usd']:.4f} → ${m['cost_p95_usd']:.4f}")
            if m["pass_rate"] < base["pass_rate"]:
                regressions.append(f"{name}: pass_rate {base['pass_rate']:.0%} → {m['pass_rate']:.0%}")
        if args.json:
            print(json.dumps({"summary": summary, "regressions": regressions}, indent=2))
        else:
            print(f"\n  Benchmark: {args.skill_id}  ({args.runs} run(s) per fixture)\n")
            for name, m in summary["fixtures"].items():
                print(f"  {name}")
                print(f"    tokens p50/p95: {m['tokens_p50']}/{m['tokens_p95']}")
                print(f"    cost   p50/p95: ${m['cost_p50_usd']:.4f}/${m['cost_p95_usd']:.4f}")
                print(f"    iter   p50/p95: {m['iterations_p50']}/{m['iterations_p95']}")
                print(f"    pass rate: {m['pass_rate']:.0%}   latency p50: {m['latency_p50_s']}s")
            if regressions:
                print(f"\n  ⚠ {len(regressions)} regression(s):")
                for r in regressions:
                    print(f"    - {r}")
            else:
                print(f"\n  ✓ no regressions vs baseline at {baseline_path.relative_to(brain_root)}")
        return 1 if regressions else 0
    else:
        if args.json:
            print(json.dumps(summary, indent=2))
        else:
            print(f"\n  Bench result for {args.skill_id} (no baseline yet — run --record to save):")
            for name, m in summary["fixtures"].items():
                print(f"    {name}: p50_tokens={m['tokens_p50']} cost=${m['cost_p50_usd']:.4f} pass={m['pass_rate']:.0%}")
        return 0


def main():
    p = argparse.ArgumentParser(description="skill cost + accuracy benchmarks (Tier α.6)")
    p.add_argument("skill_id")
    p.add_argument("--runs", type=int, default=1)
    p.add_argument("--max-iterations", type=int, default=2)
    p.add_argument("--model", default="claude-sonnet-4-6")
    p.add_argument("--record", action="store_true")
    p.add_argument("--no-llm", action="store_true")
    p.add_argument("--json", action="store_true")
    p.set_defaults(func=cmd_bench)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
