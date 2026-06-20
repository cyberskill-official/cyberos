#!/usr/bin/env python3
"""retro-summary.py — aggregate retrospective scores per protocol version.

The improvement loop's own reporting accuracy (structural review F7): turns
"did v<x.y.z> actually help?" from anecdote into trend by summarizing every
`improve/retros/*.md` score against the protocol version it ran under. This
feeds the escalation path in improve/CRITIC.md — graduate to automated
optimization only with a standardized project type, a single metric, and
~50+ labeled examples; this script is how you watch those examples accrue.

Usage:
  python3 evals/scripts/retro-summary.py            # human table
  python3 evals/scripts/retro-summary.py --json     # machine-readable

Stdlib only, like everything else in evals/.
"""

import argparse
import json
import re
import statistics
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
RETROS = HERE.parent.parent / "improve" / "retros"

VERSION_RE = re.compile(r"Protocol:\s*(v\d+\.\d+\.\d+)")
TOTAL_RE = re.compile(r"TOTAL:?\s*\*{0,2}(\d+)\s*/\s*20")
DATE_RE = re.compile(r"Date:\s*([0-9]{4}-[0-9]{2}-[0-9]{2})")
PROJECT_RE = re.compile(r"Project:\s*([^|]+)")
MODE_RE = re.compile(r"Mode:\s*([^|\n]+)")


def parse_retro(path: Path):
    text = path.read_text(encoding="utf-8")
    v, t, d, p, m = (r.search(text) for r in (VERSION_RE, TOTAL_RE, DATE_RE, PROJECT_RE, MODE_RE))
    return {
        "file": path.name,
        "protocol_version": v.group(1) if v else None,
        "score": int(t.group(1)) if t else None,
        "date": d.group(1) if d else None,
        "project": p.group(1).strip() if p else None,
        "mode": m.group(1).strip() if m else None,
    }


def version_key(v):
    return tuple(int(x) for x in v.lstrip("v").split("."))


def summarize(retros):
    by_version = {}
    for r in retros:
        if r["protocol_version"] and r["score"] is not None:
            by_version.setdefault(r["protocol_version"], []).append(r["score"])
    versions = []
    for v in sorted(by_version, key=version_key):
        scores = by_version[v]
        versions.append({
            "protocol_version": v,
            "retros": len(scores),
            "median": statistics.median(scores),
            "mean": round(statistics.mean(scores), 2),
            "min": min(scores),
            "max": max(scores),
        })
    unparsed = [r["file"] for r in retros if r["score"] is None or not r["protocol_version"]]
    return {
        "retros_total": len(retros),
        "retros_scored": sum(1 for r in retros if r["score"] is not None),
        "labeled_examples_toward_escalation": sum(1 for r in retros if r["score"] is not None),
        "per_version": versions,
        "unparsed": unparsed,
        "retros": retros,
    }


FB_VERSION_RE = re.compile(r"(?m)^protocol_version:\s*(v\d+\.\d+\.\d+)")
FB_SCORE_RE = re.compile(r"(?m)^retro_score:\s*(\d+)")
FB_ID_RE = re.compile(r"(?m)^run_id:\s*(\S+)")


def parse_feedback(path: Path):
    """feedback@1 field records (YAML or JSON) — only the trend fields are
    read; full semantics live in schemas/feedback.v1.json."""
    text = path.read_text(encoding="utf-8", errors="replace")
    if path.suffix == ".json":
        try:
            d = json.loads(text)
            return {"file": path.name, "protocol_version": d.get("protocol_version"),
                    "score": d.get("retro_score"), "date": None,
                    "project": d.get("run_id"), "mode": "field"}
        except json.JSONDecodeError:
            return {"file": path.name, "protocol_version": None, "score": None,
                    "date": None, "project": None, "mode": "field"}
    v, s, i = FB_VERSION_RE.search(text), FB_SCORE_RE.search(text), FB_ID_RE.search(text)
    return {"file": path.name, "protocol_version": v.group(1) if v else None,
            "score": int(s.group(1)) if s else None, "date": None,
            "project": i.group(1) if i else None, "mode": "field"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--retros-dir", default=str(RETROS))
    ap.add_argument("--feedback-dir", default=None,
                    help="also ingest feedback@1 records (the private field-data repo checkout) — per-version FIELD trend")
    args = ap.parse_args()
    rdir = Path(args.retros_dir)
    if not rdir.is_dir():
        print(f"retros directory not found: {rdir}", file=sys.stderr)
        sys.exit(2)
    retros = [parse_retro(p) for p in sorted(rdir.glob("*.md"))]
    if args.feedback_dir:
        fdir = Path(args.feedback_dir)
        if not fdir.is_dir():
            print(f"feedback directory not found: {fdir}", file=sys.stderr)
            sys.exit(2)
        retros += [parse_feedback(p) for p in sorted(list(fdir.glob("*.yaml")) + list(fdir.glob("*.yml")) + list(fdir.glob("*.json")))]
    summary = summarize(retros)
    if args.json:
        print(json.dumps(summary, indent=2))
        return
    print(f"{'Version':10s} {'n':>3s} {'median':>7s} {'mean':>6s} {'min':>4s} {'max':>4s}")
    for v in summary["per_version"]:
        print(f"{v['protocol_version']:10s} {v['retros']:3d} {v['median']:7.1f} {v['mean']:6.2f} {v['min']:4d} {v['max']:4d}")
    print(f"\n{summary['retros_scored']}/{summary['retros_total']} retros scored "
          f"— {summary['labeled_examples_toward_escalation']} labeled examples toward the ~50 escalation bar (CRITIC.md)")
    for f in summary["unparsed"]:
        print(f"  unparsed (no Protocol/TOTAL header): {f}")
    for r in summary["retros"]:
        if r["score"] is not None:
            print(f"  {r['file']:34s} {r['protocol_version'] or '?':8s} {r['score']:2d}/20  {r['project'] or ''}")


if __name__ == "__main__":
    main()
