#!/usr/bin/env python3
"""
apply_brain_rename.py — replay the fr->task rename into the BRAIN as NEW audit rows.

WHY THIS EXISTS
---------------
The BRAIN store cannot be sed'd. It is a hash chain:

    AGENTS.md §6.3   chain = SHA-256(canonical(record_minus_chain) || prev_chain)
    AGENTS.md §5.3   a memory file's body SHA-256 is recorded in its audit row
    AGENTS.md §6.5   in-place edit / reorder / delete of a written row is FORBIDDEN
    AGENTS.md §12    an invariant failure moves the store to FROZEN_RECOVERABLE

The live store here is 226 MB across 252,133 rows, with 446 filenames and 500
bodies carrying legacy FR ids. A `sed -i` over it does not rename the BRAIN, it
bricks it: every recorded content_sha256 goes wrong at once and `cyberos doctor`
freezes.

The protocol already has the answer. A rename is not an edit — it is a new
operation. Replay it as move() + put() through the canonical writer and the chain
RECORDS the rename instead of being invalidated by it. Old rows keep citing old
paths, which is correct: that is what happened.

Usage:
    python3 scripts/migrate_fr_to_task.py --emit-brain-ops > /tmp/brain.ndjson
    python3 scripts/apply_brain_rename.py /tmp/brain.ndjson          # dry run
    python3 scripts/apply_brain_rename.py /tmp/brain.ndjson --apply
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("ndjson", type=Path)
    ap.add_argument("--apply", action="store_true")
    ap.add_argument("--store", default=None, help="override <memory-root>")
    args = ap.parse_args()

    ops = []
    for line in args.ndjson.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            ops.append(json.loads(line))
        except json.JSONDecodeError:
            print(f"skip unparseable line: {line[:60]}", file=sys.stderr)

    kinds = Counter(o["op"] for o in ops)
    print(f"{len(ops)} ops: {dict(kinds)}")
    print("chain effect: +1 audit row per op. Old rows are untouched — they keep")
    print("citing the old paths, which is the correct historical record.\n")

    if not args.apply:
        for o in ops[:6]:
            if o["op"] == "move":
                print(f"  move  {o['src']}\n     -> {o['dst']}")
            else:
                print(f"  put   {o['path']}  ({len(o['body'])} bytes)")
        print(f"\n  ... and {max(0, len(ops) - 6)} more")
        print("\nDRY RUN. Re-run with --apply to write them.")
        return 0

    # `cyberos move <src> <dst>` and `cyberos put <path> <body_file>` — positional.
    done = Counter()
    for i, o in enumerate(ops, 1):
        if o["op"] == "move":
            cmd = [sys.executable, "-m", "cyberos", "move", o["src"], o["dst"]]
            r = subprocess.run(cmd, capture_output=True, text=True)
        else:
            tmp = Path("/tmp") / f"brain_body_{i}.md"
            tmp.write_text(o["body"], encoding="utf-8")
            cmd = [sys.executable, "-m", "cyberos", "put", o["path"], str(tmp),
                   "--kind", "refinements"]
            r = subprocess.run(cmd, capture_output=True, text=True)
        if r.returncode != 0:
            print(f"FAILED op {i} ({o['op']}): {r.stderr.strip()[:160]}", file=sys.stderr)
            print("HALTING. The chain is append-only — nothing is half-written, but "
                  "fix the cause before re-running.", file=sys.stderr)
            return 1
        done[o["op"]] += 1
        if i % 100 == 0:
            print(f"  {i}/{len(ops)} ...")

    print(f"\napplied {dict(done)}")
    print("Now run `cyberos doctor` — it must report READY, not FROZEN_*.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
