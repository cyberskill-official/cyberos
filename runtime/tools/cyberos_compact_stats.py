#!/usr/bin/env python3
"""
cyberos_compact_stats.py — audit-ledger compaction recommendations.

Aspect 9.4 of the Layer-1 improvement catalog.

Inspects per-month audit ledgers under `.cyberos-memory/audit/*.jsonl`,
reports row counts + bytes + the dominant op-type, recommends compaction
when a ledger crosses the threshold (default: >10,000 rows OR >5 MB OR
>90 days old).

Does NOT compact — that's `cyberos doctor --compact-ledger MM`. This tool
only surfaces which months are candidates.

Usage:
    cyberos compact-stats                       # text summary
    cyberos compact-stats --json
    cyberos compact-stats --row-cap 5000        # tune threshold
    cyberos compact-stats --byte-cap 1048576    # tune threshold (1 MB)
    cyberos compact-stats --age-days 30
"""
from __future__ import annotations
import argparse
import json
import re
import sys
from collections import Counter
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


def collect(brain_root: Path) -> list[dict]:
    audit_dir = brain_root / ".cyberos-memory" / "audit"
    if not audit_dir.exists():
        return []
    out = []
    now = datetime.now(ICT)
    for ledger in sorted(audit_dir.glob("*.jsonl")):
        size = ledger.stat().st_size
        rows = 0
        ops = Counter()
        latest_ts = None
        for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines():
            if not line.strip():
                continue
            rows += 1
            try:
                row = json.loads(line)
                ops[row.get("op", "?")] += 1
                ts_str = row.get("ts", "")
                if ts_str:
                    ts = datetime.fromisoformat(ts_str)
                    if latest_ts is None or ts > latest_ts:
                        latest_ts = ts
            except Exception:
                continue
        age_days = (now - latest_ts).days if latest_ts else -1
        # Year-month from filename (e.g. 2026-05.jsonl)
        m = re.match(r"^(\d{4})-(\d{2})", ledger.name)
        ym = f"{m.group(1)}-{m.group(2)}" if m else ledger.stem
        # Compaction flag — already-compacted ledgers carry .compacted suffix per §7.7
        compacted = ledger.suffix == ".compacted" or ledger.name.endswith(".jsonl.compacted")
        out.append({
            "ledger": ledger.name,
            "year_month": ym,
            "rows": rows,
            "size_bytes": size,
            "dominant_op": ops.most_common(1)[0] if ops else ("?", 0),
            "age_days": age_days,
            "compacted": compacted,
        })
    return out


def main():
    p = argparse.ArgumentParser(description="audit-ledger compaction recommendations")
    p.add_argument("--row-cap", type=int, default=10000)
    p.add_argument("--byte-cap", type=int, default=5_000_000)
    p.add_argument("--age-days", type=int, default=90)
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    brain_root = find_brain()
    rows = collect(brain_root)

    candidates = []
    for r in rows:
        if r["compacted"]:
            continue
        triggers = []
        if r["rows"] > args.row_cap:
            triggers.append(f"rows>{args.row_cap}")
        if r["size_bytes"] > args.byte_cap:
            triggers.append(f"bytes>{args.byte_cap:,}")
        if r["age_days"] > args.age_days:
            triggers.append(f"age>{args.age_days}d")
        if triggers:
            candidates.append({**r, "triggers": triggers})

    if args.json:
        print(json.dumps({
            "thresholds": {
                "row_cap": args.row_cap,
                "byte_cap": args.byte_cap,
                "age_days": args.age_days,
            },
            "ledgers": rows,
            "candidates": candidates,
        }, indent=2, default=str))
        return 1 if candidates else 0

    print()
    print(f"  Audit-ledger inventory (thresholds: rows>{args.row_cap:,} | bytes>{args.byte_cap:,} | age>{args.age_days}d)")
    print()
    print(f"  {'ledger':24s} {'rows':>8s}  {'size':>12s}  {'age':>6s}  {'dominant op':16s}  status")
    for r in rows:
        size_mb = r["size_bytes"] / (1024 * 1024)
        size_s = f"{size_mb:.2f} MB" if size_mb >= 0.01 else f"{r['size_bytes']:,} B"
        age_s = f"{r['age_days']}d" if r["age_days"] >= 0 else "?"
        op, n = r["dominant_op"]
        op_s = f"{op}:{n}"
        status = "COMPACTED" if r["compacted"] else ("⚠ CANDIDATE" if any(c["ledger"] == r["ledger"] for c in candidates) else "ok")
        print(f"  {r['ledger']:24s} {r['rows']:>8,}  {size_s:>12s}  {age_s:>6s}  {op_s:16s}  {status}")

    print()
    if candidates:
        print(f"  {len(candidates)} ledger(s) recommended for compaction:")
        for c in candidates:
            print(f"    cyberos doctor --compact-ledger {c['year_month']}    # triggers: {', '.join(c['triggers'])}")
        return 1
    print(f"  ✓ no compaction needed at current thresholds")
    return 0


if __name__ == "__main__":
    sys.exit(main())
