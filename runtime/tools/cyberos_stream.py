#!/usr/bin/env python3
"""
cyberos_stream.py — audit-row streaming + alert webhooks.

Tier E.5 of post-catalog improvements (Batch 15).

Two subcommands:

  audit-stream         long-poll the current-month audit ledger and emit
                       each new row to stdout (one JSON per line). Pipe
                       to anywhere.

  alert add <name>     register a periodic rule that fires a webhook
                       when satisfied. Rules live in
                       `.cyberos-memory/meta/alerts.json`.
  alert list
  alert run            evaluate all rules once
  alert remove <name>

Rule syntax (simple expressions):
    "CRITICAL > 0"               # validator findings
    "drift > 5"                  # drift candidate count
    "audit_ops_24h > 100"        # last 24h activity
    "council_pending > 0"

Action types:
    --action slack-webhook https://hooks.slack.com/...
    --action stdout                # print to stdout (testing)
    --action exec "cmd args"       # run a shell command

Webhooks POST a JSON body: {rule, value, threshold, ts}.
"""
from __future__ import annotations
import argparse
import json
import os
import re
import shlex
import subprocess
import sys
import time
import urllib.request
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


def current_ledger(brain_root: Path) -> Path:
    now = datetime.now(ICT)
    return brain_root / ".cyberos-memory" / "audit" / f"{now.year}-{now.month:02d}.jsonl"


def cmd_audit_stream(args):
    brain_root = find_brain()
    ledger = current_ledger(brain_root)
    if not ledger.exists():
        ledger.parent.mkdir(parents=True, exist_ok=True)
        ledger.touch()
    last_size = ledger.stat().st_size if not args.from_start else 0
    print(f"# streaming {ledger.name} (from {'start' if args.from_start else 'tail'})", file=sys.stderr)
    while True:
        try:
            cur_size = ledger.stat().st_size
            if cur_size > last_size:
                with open(ledger, "r") as f:
                    f.seek(last_size)
                    chunk = f.read()
                last_size = cur_size
                for line in chunk.splitlines():
                    if line.strip():
                        # Just echo; downstream pipes JSON through.
                        print(line, flush=True)
            time.sleep(args.poll)
        except KeyboardInterrupt:
            return 0


def alerts_path(brain_root: Path) -> Path:
    return brain_root / ".cyberos-memory" / "meta" / "alerts.json"


def load_alerts(brain_root: Path) -> dict:
    p = alerts_path(brain_root)
    if not p.exists():
        return {"alerts": []}
    try:
        return json.loads(p.read_text())
    except Exception:
        return {"alerts": []}


def save_alerts(brain_root: Path, data: dict):
    p = alerts_path(brain_root)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")


def collect_metrics(brain_root: Path) -> dict:
    """Cheap metrics for rule evaluation."""
    brain = brain_root / ".cyberos-memory"
    metrics = {"CRITICAL": 0, "WARN": 0, "INFO": 0,
               "drift": 0, "council_pending": 0, "audit_ops_24h": 0}
    # Drift
    drift_dir = brain / "memories" / "drift"
    if drift_dir.exists():
        metrics["drift"] = sum(1 for _ in drift_dir.glob("*.md"))
    # Council
    council_dir = brain_root / "outputs" / "council"
    if council_dir.exists():
        metrics["council_pending"] = sum(1 for _ in council_dir.glob("REF-*-council.md"))
    # Validate (cheap WARN/CRITICAL/INFO count via subprocess)
    try:
        out = subprocess.run(["python3", str(brain_root / "runtime" / "tools" / "cyberos_validate.py"),
                              "--format", "json", str(brain)],
                             capture_output=True, text=True, timeout=15)
        if out.stdout.strip():
            d = json.loads(out.stdout)
            for f in d.get("findings", []):
                sev = f.get("severity", "INFO")
                metrics[sev] = metrics.get(sev, 0) + 1
    except Exception:
        pass
    # Audit 24h
    now = datetime.now(ICT); cutoff = now - timedelta(hours=24)
    audit_dir = brain / "audit"
    if audit_dir.exists():
        for ledger in audit_dir.glob("*.jsonl"):
            for line in ledger.read_text(encoding="utf-8", errors="ignore").splitlines():
                if not line.strip():
                    continue
                try:
                    r = json.loads(line)
                    if datetime.fromisoformat(r.get("ts", "")) >= cutoff:
                        metrics["audit_ops_24h"] += 1
                except Exception:
                    continue
    return metrics


def evaluate_rule(rule: str, metrics: dict) -> tuple[bool, float, float]:
    """Return (fired, value, threshold). Supports `<key> <op> <number>` only."""
    m = re.match(r"^\s*(\w+)\s*([<>=!]+)\s*(\d+(?:\.\d+)?)\s*$", rule)
    if not m:
        return False, 0.0, 0.0
    key, op, threshold = m.group(1), m.group(2), float(m.group(3))
    value = float(metrics.get(key, 0))
    fired = False
    if op == ">": fired = value > threshold
    elif op == ">=": fired = value >= threshold
    elif op == "<": fired = value < threshold
    elif op == "<=": fired = value <= threshold
    elif op == "==": fired = value == threshold
    elif op == "!=": fired = value != threshold
    return fired, value, threshold


def fire_action(action_type: str, action_arg: str, payload: dict):
    if action_type == "stdout":
        print(json.dumps(payload))
    elif action_type == "slack-webhook":
        try:
            req = urllib.request.Request(action_arg, method="POST",
                                         data=json.dumps({"text": json.dumps(payload)}).encode(),
                                         headers={"Content-Type": "application/json"})
            urllib.request.urlopen(req, timeout=5)
        except Exception as e:
            print(f"  ✗ webhook failed: {e}", file=sys.stderr)
    elif action_type == "exec":
        try:
            subprocess.run(shlex.split(action_arg), env={**os.environ, "ALERT_PAYLOAD": json.dumps(payload)}, timeout=10)
        except Exception as e:
            print(f"  ✗ exec failed: {e}", file=sys.stderr)


def cmd_alert_add(args):
    brain_root = find_brain()
    data = load_alerts(brain_root)
    rule_str = args.rule
    action_parts = args.action.split(" ", 1)
    action_type = action_parts[0]
    action_arg = action_parts[1] if len(action_parts) > 1 else ""
    data["alerts"].append({
        "name": args.name, "rule": rule_str,
        "action_type": action_type, "action_arg": action_arg,
        "created_at": datetime.now(ICT).isoformat(timespec="seconds"),
    })
    save_alerts(brain_root, data)
    print(f"  ✓ alert {args.name!r} added")
    return 0


def cmd_alert_list(_args):
    brain_root = find_brain()
    data = load_alerts(brain_root)
    if not data.get("alerts"):
        print("  no alerts configured")
        return 0
    for a in data["alerts"]:
        print(f"  {a['name']:24s}  rule={a['rule']!r}  action={a['action_type']} {a.get('action_arg', '')[:40]}")
    return 0


def cmd_alert_remove(args):
    brain_root = find_brain()
    data = load_alerts(brain_root)
    before = len(data["alerts"])
    data["alerts"] = [a for a in data["alerts"] if a["name"] != args.name]
    if len(data["alerts"]) == before:
        print(f"  no alert named {args.name!r}")
        return 1
    save_alerts(brain_root, data)
    print(f"  ✓ alert {args.name!r} removed")
    return 0


def cmd_alert_run(_args):
    brain_root = find_brain()
    data = load_alerts(brain_root)
    if not data.get("alerts"):
        print("  no alerts configured")
        return 0
    metrics = collect_metrics(brain_root)
    fired_count = 0
    for a in data["alerts"]:
        fired, val, thr = evaluate_rule(a["rule"], metrics)
        marker = "🔥" if fired else "·"
        print(f"  {marker} {a['name']:24s}  rule={a['rule']!r}  value={val}  threshold={thr}  fired={fired}")
        if fired:
            payload = {"rule": a["rule"], "value": val, "threshold": thr,
                       "ts": datetime.now(ICT).isoformat(timespec="seconds"),
                       "alert": a["name"]}
            fire_action(a["action_type"], a.get("action_arg", ""), payload)
            fired_count += 1
    return 1 if fired_count else 0


def main():
    p = argparse.ArgumentParser(description="audit streaming + alerting (Tier E.5)")
    sub = p.add_subparsers(dest="cmd", required=True)

    pas = sub.add_parser("audit-stream")
    pas.add_argument("--poll", type=float, default=1.0)
    pas.add_argument("--from-start", action="store_true")
    pas.set_defaults(func=cmd_audit_stream)

    pa = sub.add_parser("alert")
    asub = pa.add_subparsers(dest="alert_cmd", required=True)
    aa = asub.add_parser("add")
    aa.add_argument("name")
    aa.add_argument("--rule", required=True)
    aa.add_argument("--action", required=True, help="e.g. 'stdout' or 'slack-webhook https://...' or 'exec cmd'")
    aa.set_defaults(func=cmd_alert_add)
    asub.add_parser("list").set_defaults(func=cmd_alert_list)
    ar = asub.add_parser("remove"); ar.add_argument("name"); ar.set_defaults(func=cmd_alert_remove)
    asub.add_parser("run").set_defaults(func=cmd_alert_run)

    args = p.parse_args()
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())
