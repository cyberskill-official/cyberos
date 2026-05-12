#!/usr/bin/env python3
"""
cyberos_analytics.py — local-only usage telemetry.

Aspect 11.1 + 11.2 of the Layer-1 improvement catalog.

LOCAL-ONLY. Never sends anywhere. Per `autonomous-agent-harness`
Consent-and-Safety-Boundaries: any remote send is opt-in, never silent.

Logs every `cyberos` subcommand invocation to ~/.cyberos/analytics/skill-usage.jsonl

Usage:
    # Called automatically by `cyberos` binary at end of every command
    cyberos_analytics.py log <cmd> <outcome> <duration_ms>

    # Reports
    cyberos_analytics.py report                  # last 7 days, default
    cyberos_analytics.py report --period 30d
    cyberos_analytics.py report --format json
    cyberos_analytics.py purge                   # delete log (full reset)
"""
from __future__ import annotations
import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

ICT = timezone(timedelta(hours=7))
LOG = Path.home() / ".cyberos" / "analytics" / "skill-usage.jsonl"
COST_LOG = Path.home() / ".cyberos" / "analytics" / "llm-cost.jsonl"

def log_event(cmd: str, outcome: str, duration_ms: int):
    LOG.parent.mkdir(parents=True, exist_ok=True)
    row = {
        "ts": datetime.now(ICT).isoformat(timespec='seconds'),
        "cmd": cmd,
        "outcome": outcome,
        "duration_ms": int(duration_ms),
        "session": os.environ.get("CYBEROS_SESSION_ID", os.environ.get("CLAUDE_SESSION_ID", "default")),
    }
    # Append atomically
    with open(LOG, "a") as f:
        f.write(json.dumps(row) + "\n")

def parse_period(s: str) -> timedelta:
    m = re.match(r"^(\d+)([dwmy])$", s)
    if not m:
        raise ValueError(f"bad period: {s}")
    n = int(m.group(1))
    unit = m.group(2)
    return {"d": timedelta(days=n), "w": timedelta(weeks=n),
            "m": timedelta(days=n*30), "y": timedelta(days=n*365)}[unit]

def read_log(since: datetime) -> list[dict]:
    if not LOG.exists():
        return []
    rows = []
    for line in LOG.read_text().split("\n"):
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            ts = datetime.fromisoformat(r["ts"])
            if ts >= since:
                rows.append(r)
        except Exception:
            pass
    return rows

def report(period: str = "7d", fmt: str = "table"):
    delta = parse_period(period)
    since = datetime.now(ICT) - delta
    rows = read_log(since)
    if not rows:
        print(f"(no usage data in last {period})")
        return

    # Aggregate
    by_cmd = Counter(r["cmd"] for r in rows)
    by_outcome = Counter(r["outcome"] for r in rows)
    durations = defaultdict(list)
    for r in rows:
        durations[r["cmd"]].append(r["duration_ms"])

    fail_n = sum(by_outcome[o] for o in by_outcome if o not in ("ok", "0", 0))
    fail_rate = fail_n / len(rows) * 100 if rows else 0

    if fmt == "json":
        out = {
            "period": period,
            "total": len(rows),
            "by_cmd": dict(by_cmd),
            "by_outcome": dict(by_outcome),
            "fail_rate_pct": round(fail_rate, 2),
            "median_ms": {c: sorted(d)[len(d)//2] for c, d in durations.items()},
        }
        print(json.dumps(out, indent=2))
        return

    print(f"\n📊 CyberOS usage — last {period} ({len(rows)} invocations)\n")
    print(f"Top commands:")
    days = max(delta.days, 1)
    for c, n in by_cmd.most_common(10):
        per_day = n / days
        marker = "  ← never used" if n == 0 else ""
        print(f"  {c:24s} : {n:4d}  ({per_day:.1f}/day){marker}")

    print(f"\nMedian latency:")
    for c, ds in sorted(durations.items(), key=lambda x: -len(x[1]))[:8]:
        md = sorted(ds)[len(ds)//2] / 1000
        marker = " (interactive)" if md > 5 else ""
        print(f"  {c:24s} : {md:.1f}s{marker}")

    print(f"\nFailure rate: {fail_rate:.1f}% ({fail_n} of {len(rows)})")
    if fail_n:
        print("  Top failure outcomes:")
        for o, n in by_outcome.most_common():
            if o not in ("ok", "0", 0):
                print(f"    {o:20s} : {n}")

    # Underused / never-used commands
    all_subcmds = [
        "status", "verify", "doctor", "export", "search", "stats",
        "show", "voice", "panic", "doc-consistency", "add", "prune",
        "onboard", "sync", "conflicts", "eval", "analytics"
    ]
    never_used = [c for c in all_subcmds if c not in by_cmd]
    if never_used:
        print(f"\nNever used in {period}: {', '.join(never_used)}")
        print("  Consider: do you need them? Deprecate or document.")

def purge():
    if LOG.exists():
        n = sum(1 for _ in LOG.read_text().split("\n") if _.strip())
        LOG.unlink()
        print(f"purged {n} events from {LOG}")
    else:
        print("no analytics log to purge")

def log_cost(model: str, op: str, input_tokens: int, output_tokens: int,
             input_per_mtok: float, output_per_mtok: float, note: str = ""):
    """Aspect 11.5 — append an LLM cost record. Local-only, never sent.

    Cost computed at call time using the per-million-token rates supplied
    by the operator (rates change; we don't hardcode model pricing).
    """
    COST_LOG.parent.mkdir(parents=True, exist_ok=True)
    cost = (input_tokens / 1_000_000.0) * input_per_mtok + (output_tokens / 1_000_000.0) * output_per_mtok
    row = {
        "ts": datetime.now(ICT).isoformat(timespec="seconds"),
        "model": model,
        "op": op,
        "input_tokens": int(input_tokens),
        "output_tokens": int(output_tokens),
        "input_per_mtok": float(input_per_mtok),
        "output_per_mtok": float(output_per_mtok),
        "cost_usd": round(cost, 6),
        "note": note,
        "session": os.environ.get("CYBEROS_SESSION_ID", os.environ.get("CLAUDE_SESSION_ID", "default")),
    }
    with open(COST_LOG, "a") as f:
        f.write(json.dumps(row) + "\n")
    return row


def cost_report(period: str, fmt: str):
    if not COST_LOG.exists():
        if fmt == "json":
            print(json.dumps({"total_usd": 0.0, "rows": []}))
        else:
            print("  no cost log yet — call `cyberos analytics cost-log` to record an LLM call")
        return
    delta = parse_period(period)
    cutoff = datetime.now(ICT) - delta

    rows = []
    for line in COST_LOG.read_text().splitlines():
        if not line.strip():
            continue
        try:
            r = json.loads(line)
            ts = datetime.fromisoformat(r.get("ts", ""))
            if ts >= cutoff:
                rows.append(r)
        except Exception:
            continue

    by_op = defaultdict(lambda: {"calls": 0, "input_tokens": 0, "output_tokens": 0, "cost_usd": 0.0})
    by_model = defaultdict(lambda: {"calls": 0, "cost_usd": 0.0})
    total = 0.0
    for r in rows:
        op = r.get("op", "?")
        m = r.get("model", "?")
        by_op[op]["calls"] += 1
        by_op[op]["input_tokens"] += r.get("input_tokens", 0)
        by_op[op]["output_tokens"] += r.get("output_tokens", 0)
        by_op[op]["cost_usd"] += r.get("cost_usd", 0.0)
        by_model[m]["calls"] += 1
        by_model[m]["cost_usd"] += r.get("cost_usd", 0.0)
        total += r.get("cost_usd", 0.0)

    if fmt == "json":
        print(json.dumps({
            "period": period,
            "rows": len(rows),
            "total_usd": round(total, 4),
            "by_op": {k: {kk: (round(vv, 4) if isinstance(vv, float) else vv) for kk, vv in v.items()} for k, v in by_op.items()},
            "by_model": {k: {kk: (round(vv, 4) if isinstance(vv, float) else vv) for kk, vv in v.items()} for k, v in by_model.items()},
        }, indent=2))
        return

    print(f"\n  LLM cost report — last {period}")
    print(f"  Records:  {len(rows)}")
    print(f"  Total:    ${total:.4f} USD")
    print()
    if by_op:
        print(f"  By op:")
        for op, m in sorted(by_op.items(), key=lambda x: -x[1]["cost_usd"]):
            print(f"    {op:24s}  {m['calls']:4d} calls  in={m['input_tokens']:>10,}  out={m['output_tokens']:>10,}  ${m['cost_usd']:.4f}")
    if by_model:
        print()
        print(f"  By model:")
        for model, m in sorted(by_model.items(), key=lambda x: -x[1]["cost_usd"]):
            print(f"    {model:32s}  {m['calls']:4d} calls  ${m['cost_usd']:.4f}")


def main():
    p = argparse.ArgumentParser(description="cyberos local analytics")
    sub = p.add_subparsers(dest="cmd", required=True)

    pl = sub.add_parser("log", help="(internal) log an event")
    pl.add_argument("subcommand")
    pl.add_argument("outcome")
    pl.add_argument("duration_ms", type=int)

    pr = sub.add_parser("report", help="usage report")
    pr.add_argument("--period", default="7d", help="e.g. 7d, 4w, 1m")
    pr.add_argument("--format", choices=["table", "json"], default="table")

    sub.add_parser("purge", help="delete analytics log")

    # Aspect 11.5 — cost tracking
    pcl = sub.add_parser("cost-log", help="record an LLM call cost (local-only)")
    pcl.add_argument("--model", required=True)
    pcl.add_argument("--op", required=True, help="e.g. council, add-body, sync-merge")
    pcl.add_argument("--input-tokens", type=int, required=True)
    pcl.add_argument("--output-tokens", type=int, required=True)
    pcl.add_argument("--input-per-mtok", type=float, required=True, help="USD per 1M input tokens at call time")
    pcl.add_argument("--output-per-mtok", type=float, required=True, help="USD per 1M output tokens at call time")
    pcl.add_argument("--note", default="")

    pcr = sub.add_parser("cost-report", help="LLM cost report")
    pcr.add_argument("--period", default="30d")
    pcr.add_argument("--format", choices=["table", "json"], default="table")

    args = p.parse_args()
    if args.cmd == "log":
        log_event(args.subcommand, args.outcome, args.duration_ms)
    elif args.cmd == "report":
        report(args.period, args.format)
    elif args.cmd == "purge":
        purge()
    elif args.cmd == "cost-log":
        row = log_cost(args.model, args.op, args.input_tokens, args.output_tokens,
                       args.input_per_mtok, args.output_per_mtok, args.note)
        print(f"  ✓ logged ${row['cost_usd']:.6f} for {args.model} {args.op}")
    elif args.cmd == "cost-report":
        cost_report(args.period, args.format)

if __name__ == "__main__":
    main()
