#!/usr/bin/env python3
"""Platform view of awh gate coverage: per module, is it gated, does it have a baseline,
and what is the FR status mix. One glance at how far the out-of-band gate reaches.

  awh_gate_coverage.py            # human table
  awh_gate_coverage.py --json
"""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

FR_ROOT = Path("docs/feature-requests")
ROADMAP = ["memory", "skill", "cuo", "auth", "chat", "proj"]
# modules whose service crate is known red on main (do not gate green).
# Empty now that ai is gated green (FR-AI-001..022 shipped 2026-06-19).
KNOWN_RED: set[str] = set()


def fr_status_counts(module: str) -> dict[str, int]:
    d = FR_ROOT / module
    counts: dict[str, int] = {}
    if not d.is_dir():
        return counts
    for p in d.glob("FR-*.md"):
        if p.name.endswith(".audit.md"):
            continue
        m = re.search(r"^status:\s*['\"`]?([A-Za-z_]+)", p.read_text(encoding="utf-8")[:1500], re.M)
        s = m.group(1) if m else "?"
        counts[s] = counts.get(s, 0) + 1
    return counts


def held_out(module: str) -> str | None:
    gs = Path("modules") / module / ".awh" / "goldenset.yaml"
    if not gs.is_file():
        return None
    for line in gs.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if line.startswith("- id: acceptance"):
            return line.split("id:", 1)[1].strip()
    return None


def modules() -> list[str]:
    fr_mods = {p.name for p in FR_ROOT.iterdir()
               if p.is_dir() and p.name != "docs" and not p.name.startswith(".")}
    awh_mods = {p.parent.parent.name for p in Path("modules").glob("*/.awh/goldenset.yaml")}
    ordered = ROADMAP + sorted((fr_mods | awh_mods) - set(ROADMAP))
    return [m for m in ordered if m in (fr_mods | awh_mods)]


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    args = ap.parse_args()
    rows = []
    for m in modules():
        gs = (Path("modules") / m / ".awh" / "goldenset.yaml").is_file()
        bl = (Path("modules") / m / ".awh" / "eval-baseline.json").is_file()
        counts = fr_status_counts(m)
        rows.append({
            "module": m, "gated": gs, "baseline": bl,
            "held_out": held_out(m), "red": m in KNOWN_RED,
            "fr_total": sum(counts.values()), "status": counts,
        })

    if args.json:
        print(json.dumps(rows, indent=2))
        return 0

    print(f"awh gate coverage: {sum(r['gated'] for r in rows)}/{len(rows)} modules gated, "
          f"{sum(r['baseline'] for r in rows)} with a committed baseline\n")
    print(f"  {'module':9s} {'gated':5s} {'baseline':8s} {'red':3s} {'FRs':>4} status-mix")
    for r in rows:
        rt = r["status"].get("ready_to_test", 0)
        dr = r["status"].get("draft", 0)
        ri = r["status"].get("ready_to_implement", 0)
        mix = f"ready_to_test:{rt} draft:{dr} ready_to_impl:{ri}"
        mark = lambda b: " yes " if b else "  -  "
        print(f"  {r['module']:9s} {mark(r['gated'])} {mark(r['baseline']):8s} "
              f"{'RED' if r['red'] else '  -':3s} {r['fr_total']:>4} {mix}")
    print("\nroadmap waves: " + " -> ".join(m.upper() for m in ROADMAP))
    red_mods = [r["module"] for r in rows if r["red"]]
    ungated = sum(1 for r in rows if not r["gated"])
    if red_mods:
        print("note: " + ", ".join(red_mods) + " implemented but RED on main; gating as a "
              "regression floor is deferred until the crate's tests are green.")
    else:
        print(f"note: every gated module is green; {ungated} ungated modules are still draft "
              "(not yet implemented, so nothing to gate).")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
