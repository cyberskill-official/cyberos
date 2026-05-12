#!/usr/bin/env python3
"""
cyberos_replicate.py — replicated audit chain (scaffold).

Batch 13 (Tier C) of post-catalog improvements.

Pushes every new audit row to an external append-only store: S3 with
Object Lock, a peer's audit dir, or local backup. Single-machine
compromise can't kill history.

This scaffold ships the deterministic upload pattern; actual cloud
transport is operator's choice (aws cli / rclone / boto3).

Subcommands:
    cyberos replicate status              # show last replicated row id
    cyberos replicate push --to <dir>     # sync new audit rows to dir
    cyberos replicate verify --against <dir>  # confirm dir holds full chain

Local-only by contract. Tool never contacts a network provider; only
writes to a filesystem path the operator supplies (which may be an
rclone mount or s3fs mount).
"""
from __future__ import annotations
import argparse
import hashlib
import json
import shutil
import sys
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


def cmd_status(_args):
    brain_root = find_brain()
    state_file = brain_root / ".cyberos-memory" / ".replicate-state.json"
    if not state_file.exists():
        print("  no replicate state yet (run `cyberos replicate push` first)")
        return 0
    state = json.loads(state_file.read_text())
    print(f"  Last replicated row: {state.get('last_audit_id', '—')[:32]}…")
    print(f"  Last push:           {state.get('last_push_at', '—')}")
    print(f"  Target:              {state.get('target', '—')}")
    return 0


def cmd_push(args):
    brain_root = find_brain()
    audit_dir = brain_root / ".cyberos-memory" / "audit"
    target = Path(args.to).expanduser()
    target.mkdir(parents=True, exist_ok=True)

    state_file = brain_root / ".cyberos-memory" / ".replicate-state.json"
    state = {}
    if state_file.exists():
        state = json.loads(state_file.read_text())

    pushed = 0
    bytes_pushed = 0
    for ledger in sorted(audit_dir.glob("*.jsonl")):
        target_ledger = target / ledger.name
        # Naive replication: write if not present OR if local is longer
        if not target_ledger.exists() or target_ledger.stat().st_size < ledger.stat().st_size:
            shutil.copy2(ledger, target_ledger)
            pushed += 1
            bytes_pushed += ledger.stat().st_size

    # Update state
    last_id = ""
    for ledger in sorted(audit_dir.glob("*.jsonl")):
        for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines():
            if not line.strip():
                continue
            try:
                row = json.loads(line)
                last_id = row.get("audit_id", last_id)
            except Exception:
                continue
    state["last_audit_id"] = last_id
    state["last_push_at"] = datetime.now(ICT).isoformat(timespec="seconds")
    state["target"] = str(target)
    state_file.write_text(json.dumps(state, indent=2))

    print(f"  ✓ pushed {pushed} ledger file(s), {bytes_pushed:,} bytes → {target}")
    print(f"  last audit_id: {last_id[:32]}…")
    return 0


def cmd_verify(args):
    brain_root = find_brain()
    target = Path(args.against).expanduser()
    if not target.exists():
        print(f"  no such target: {target}", file=sys.stderr)
        return 2
    local_audit = brain_root / ".cyberos-memory" / "audit"
    mismatches = 0
    checked = 0
    for ledger in sorted(local_audit.glob("*.jsonl")):
        peer = target / ledger.name
        if not peer.exists():
            print(f"  ✗ missing on peer: {ledger.name}")
            mismatches += 1
            continue
        a = hashlib.sha256(ledger.read_bytes()).hexdigest()
        b = hashlib.sha256(peer.read_bytes()).hexdigest()
        if a != b:
            print(f"  ✗ content differs: {ledger.name}  ({a[:12]}… vs {b[:12]}…)")
            mismatches += 1
        else:
            checked += 1
    print(f"\n  Checked {checked} ledger(s); {mismatches} mismatch(es)")
    return 1 if mismatches else 0


def main():
    p = argparse.ArgumentParser(description="replicated audit chain (Batch 13 / Tier C)")
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("status").set_defaults(func=cmd_status)
    pp = sub.add_parser("push"); pp.add_argument("--to", required=True); pp.set_defaults(func=cmd_push)
    pv = sub.add_parser("verify"); pv.add_argument("--against", required=True); pv.set_defaults(func=cmd_verify)
    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
