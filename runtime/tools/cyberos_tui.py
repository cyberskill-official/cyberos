#!/usr/bin/env python3
"""
cyberos_tui.py — curses-based live dashboard.

Batch 11 (Tier A) of post-catalog improvements.

Single full-screen view auto-refreshing every N seconds:

  ┌─────────────────────────────────────────────────────────┐
  │  CyberOS BRAIN — <project>   (q to quit, r refresh)     │
  ├─────────────────────────────────────────────────────────┤
  │  HEALTHY?     0 CRITICAL  11 WARN  1 INFO               │
  │  MEMORIES     157  (audit head: sha256:...)             │
  │  AUDIT 24h    27 ops   (10 sessions)                    │
  │  DRIFT QUEUE  1                                         │
  │  PENDING REF  1 council                                 │
  └─────────────────────────────────────────────────────────┘
  ┌─ Recent audit rows ──────────────────────────────────────┐
  │  04:01:23  create   memories/facts/FACT-015-...md       │
  │  03:58:14  manifest-update                              │
  └─────────────────────────────────────────────────────────┘

Quit: q. Force refresh: r.
"""
from __future__ import annotations
import argparse
import curses
import json
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


def gather_state(brain_root: Path) -> dict:
    brain = brain_root / ".cyberos-memory"
    manifest = {}
    try:
        manifest = json.loads((brain / "manifest.json").read_text(encoding="utf-8"))
    except Exception:
        pass
    memories = sum(1 for p in brain.rglob("*.md") if p.is_file() and not p.name.startswith("."))

    # Audit 24h
    now = datetime.now(ICT)
    cutoff = now - timedelta(hours=24)
    audit_dir = brain / "audit"
    recent_rows = []
    op_counts = {}
    session_count = 0
    if audit_dir.exists():
        for ledger in sorted(audit_dir.glob("*.jsonl")):
            for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines():
                if not line.strip():
                    continue
                try:
                    r = json.loads(line)
                    ts = datetime.fromisoformat(r.get("ts", ""))
                    if ts >= cutoff:
                        op_counts[r.get("op", "?")] = op_counts.get(r.get("op", "?"), 0) + 1
                        if r.get("op") == "session.start":
                            session_count += 1
                        recent_rows.append(r)
                except Exception:
                    continue

    drift_n = sum(1 for _ in (brain / "memories" / "drift").glob("*.md")) if (brain / "memories" / "drift").exists() else 0
    council_dir = brain_root / "outputs" / "council"
    council_pending = sum(1 for _ in council_dir.glob("REF-*-council.md")) if council_dir.exists() else 0

    return {
        "project": manifest.get("project", {}).get("name", "?"),
        "memory_count": memories,
        "audit_head": manifest.get("audit_chain_head", "")[:32],
        "audit_24h_total": sum(op_counts.values()),
        "audit_24h_sessions": session_count,
        "audit_24h_ops": op_counts,
        "drift": drift_n,
        "council_pending": council_pending,
        "recent_rows": recent_rows[-8:],
        "ts": now.isoformat(timespec="seconds"),
    }


def draw(stdscr, brain_root: Path, interval: int):
    curses.curs_set(0)
    stdscr.nodelay(True)
    while True:
        state = gather_state(brain_root)
        stdscr.erase()
        rows, cols = stdscr.getmaxyx()
        line = 0
        stdscr.addstr(line, 0, f"CyberOS BRAIN — {state['project']}   (q quit · r refresh · auto {interval}s)", curses.A_BOLD)
        line += 1
        stdscr.addstr(line, 0, "─" * min(cols - 1, 70))
        line += 1
        stdscr.addstr(line, 0, f"MEMORIES     {state['memory_count']:>6}   audit head: {state['audit_head']}…")
        line += 1
        stdscr.addstr(line, 0, f"AUDIT 24h    {state['audit_24h_total']:>6} ops  ({state['audit_24h_sessions']} sessions)")
        line += 1
        # Top 3 ops
        ops_sorted = sorted(state["audit_24h_ops"].items(), key=lambda x: -x[1])[:3]
        for op, n in ops_sorted:
            stdscr.addstr(line, 0, f"                {n:>5}  {op}")
            line += 1
        stdscr.addstr(line, 0, f"DRIFT QUEUE  {state['drift']:>6}")
        line += 1
        stdscr.addstr(line, 0, f"COUNCIL      {state['council_pending']:>6}  pending synthesis")
        line += 1
        stdscr.addstr(line, 0, "─" * min(cols - 1, 70))
        line += 1
        stdscr.addstr(line, 0, "Recent audit rows", curses.A_BOLD)
        line += 1
        for r in state["recent_rows"][::-1]:
            if line >= rows - 1:
                break
            ts = r.get("ts", "")[:19]
            op = r.get("op", "?")
            payload = r.get("path") or r.get("from_path") or r.get("memory_id", "")
            text = f"{ts}  {op:14s}  {payload}"
            stdscr.addstr(line, 0, text[:cols - 1])
            line += 1

        stdscr.refresh()
        # Wait + handle keys
        deadline = time.time() + interval
        while time.time() < deadline:
            try:
                ch = stdscr.getch()
                if ch == ord("q"):
                    return
                if ch == ord("r"):
                    break
            except KeyboardInterrupt:
                return
            time.sleep(0.1)


def main():
    p = argparse.ArgumentParser(description="curses TUI dashboard (Batch 11 / Tier A)")
    p.add_argument("--interval", type=int, default=10)
    args = p.parse_args()
    brain_root = find_brain()
    curses.wrapper(draw, brain_root, args.interval)
    return 0


if __name__ == "__main__":
    import sys
    sys.exit(main())
