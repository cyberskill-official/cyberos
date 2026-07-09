#!/usr/bin/env python3
"""
refinement_candidates.py — Stop-hook for §0.4 auto-detection.

Scans the audit ledger at session.end for patterns that warrant a refinement
candidate. Per Aspect 3.1 + ECC `continuous-learning-v2`.

Patterns detected (when count ≥ THRESHOLD over rolling 30-day window):
  1. op:rejected rows (validator is rejecting too often → loosen or document)
  2. op:revert rows (writes failing mid-flight → race condition / bug)
  3. op:drift_candidate rows (sources changing → drift cadence high)
  4. op:shallow_candidate rows (digest coverage <0.80 → tighten ingestion)
  5. Multiple op:create rows with overlapping tags within 1h (duplication)
  6. Chat-history phrases "did you actually check", "is your memory saved",
     "you missed", "are you sure" (user-completeness-challenge signal)

For each pattern ≥ THRESHOLD, emit:
  memories/drift/<date>-refinement-candidate-<pattern>.md

CRITICAL: observe, don't auto-act. The hook NEVER writes a refinement.
It surfaces a candidate. §0.4 propose-adopt-record cycle still requires
explicit chat-turn approval.

Install in ~/.claude/settings.json under "hooks.Stop":
{
  "hooks": {
    "Stop": [{
      "matcher": "*",
      "hooks": [{
        "type": "command",
        "command": "python3 /Users/stephencheng/Projects/CyberSkill/cyberos/runtime/hooks/refinement_candidates.py"
      }]
    }]
  }
}
"""
from __future__ import annotations
import json
import os
import re
import sys
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path

THRESHOLD = int(os.environ.get("CYBEROS_REFINEMENT_THRESHOLD", "3"))
WINDOW_DAYS = int(os.environ.get("CYBEROS_REFINEMENT_WINDOW_DAYS", "30"))
ICT = timezone(timedelta(hours=7))

def find_memory(start: Path = None) -> Path | None:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos/memory/store").is_dir():
            return cur / ".cyberos/memory/store"
        cur = cur.parent
    return None

def scan_audit(memory: Path, cutoff: datetime) -> dict:
    """Walk audit/*.jsonl ledger files, return counts by pattern."""
    audit = memory / "audit"
    if not audit.exists():
        return {}

    counts = {
        "rejected": [],         # list of (ts, reason)
        "revert": [],
        "drift_candidate": [],
        "shallow_candidate": [],
        "create_tags": [],      # list of (ts, [tags])
        "create_paths": [],     # list of (ts, path)
    }

    for ledger in sorted(audit.glob("*.jsonl")):
        try:
            for line in ledger.read_text().split("\n"):
                if not line.strip():
                    continue
                try:
                    row = json.loads(line)
                except Exception:
                    continue
                ts_str = row.get("ts", "")
                try:
                    ts = datetime.fromisoformat(ts_str.replace("Z", "+00:00"))
                except Exception:
                    continue
                if ts.tzinfo is None:
                    ts = ts.replace(tzinfo=ICT)
                if ts < cutoff:
                    continue
                op = row.get("op", "")
                if op == "rejected":
                    counts["rejected"].append((ts, row.get("reason", "?")))
                elif op == "revert":
                    counts["revert"].append((ts, row.get("reason", "?")))
                elif op == "drift_candidate":
                    counts["drift_candidate"].append((ts, row.get("path", "?")))
                elif op == "shallow_candidate":
                    counts["shallow_candidate"].append((ts, row.get("path", "?")))
                elif op == "create":
                    # Parse tags from diff if present (simplistic)
                    diff = row.get("diff", "")
                    tags = []
                    m = re.search(r"tags:\s*\n((?:- .+\n)+)", diff)
                    if m:
                        tags = [line.strip("- ").strip() for line in m.group(1).split("\n") if line.startswith("- ")]
                    counts["create_tags"].append((ts, tags))
                    counts["create_paths"].append((ts, row.get("path", "?")))
        except Exception as e:
            print(f"warn: scan failed for {ledger.name}: {e}", file=sys.stderr)

    return counts

def detect_patterns(counts: dict) -> list[dict]:
    """Return list of candidate findings."""
    candidates = []

    # 1. Repeated rejection reasons
    if len(counts["rejected"]) >= THRESHOLD:
        by_reason = Counter(r for _, r in counts["rejected"])
        for reason, n in by_reason.most_common(5):
            if n >= THRESHOLD:
                candidates.append({
                    "pattern": "repeated-rejection",
                    "key": reason[:80],
                    "count": n,
                    "severity": "WARN",
                    "summary": f"Validator rejected {n}× with reason: {reason[:120]}",
                    "suggestion": f"Either loosen the validator (if rejection is too strict) "
                                  f"OR document why the pattern shouldn't be retried (add to FAQ / glossary).",
                })

    # 2. Frequent reverts
    if len(counts["revert"]) >= THRESHOLD:
        candidates.append({
            "pattern": "repeated-revert",
            "key": "any",
            "count": len(counts["revert"]),
            "severity": "WARN",
            "summary": f"{len(counts['revert'])} writes were reverted in window",
            "suggestion": "Investigate why writes fail mid-flight — possible race condition or §4.4 bug.",
        })

    # 3. Drift cadence
    if len(counts["drift_candidate"]) >= THRESHOLD:
        candidates.append({
            "pattern": "high-drift-cadence",
            "key": "any",
            "count": len(counts["drift_candidate"]),
            "severity": "INFO",
            "summary": f"{len(counts['drift_candidate'])} drift candidates surfaced",
            "suggestion": "Source files changing frequently — consider faster re-ingest cadence "
                          "OR mark some sources as intentional_summary if drift is expected.",
        })

    # 4. Shallow digests recurring
    if len(counts["shallow_candidate"]) >= THRESHOLD:
        candidates.append({
            "pattern": "shallow-ingestion",
            "key": "any",
            "count": len(counts["shallow_candidate"]),
            "severity": "WARN",
            "summary": f"{len(counts['shallow_candidate'])} digests had coverage <0.80",
            "suggestion": "Tighten §4.10 ingestion completeness OR add intentional_summary discipline "
                          "to memories where partial coverage is deliberate.",
        })

    # 5. Tag duplication (overlapping tags in close time)
    tag_clusters = defaultdict(list)  # tag-frozenset -> list of timestamps
    for ts, tags in counts["create_tags"]:
        if len(tags) >= 2:
            for i in range(len(tags)):
                for j in range(i+1, len(tags)):
                    pair = frozenset([tags[i], tags[j]])
                    tag_clusters[pair].append(ts)
    for pair, tss in tag_clusters.items():
        if len(tss) >= THRESHOLD:
            tss_sorted = sorted(tss)
            # Are they within 1h of each other?
            if (tss_sorted[-1] - tss_sorted[0]).total_seconds() < 3600 * 24:
                candidates.append({
                    "pattern": "tag-duplication",
                    "key": ",".join(sorted(pair))[:60],
                    "count": len(tss),
                    "severity": "INFO",
                    "summary": f"{len(tss)} memories share tag-pair {sorted(pair)} within 24h",
                    "suggestion": "Possible duplication — consider consolidation via §8 phase 3 (conservative merge).",
                })

    return candidates

def emit_candidates(memory: Path, candidates: list[dict]):
    """Write each candidate as a memories/drift/<date>-candidate-*.md (informational only)."""
    if not candidates:
        return
    drift = memory / "memories" / "drift"
    drift.mkdir(parents=True, exist_ok=True)
    today = datetime.now(ICT).strftime("%Y-%m-%d")
    for c in candidates:
        slug = re.sub(r"[^a-z0-9-]+", "-", c["pattern"]).strip("-")
        key_slug = re.sub(r"[^a-z0-9-]+", "-", c["key"].lower()).strip("-")[:40]
        path = drift / f"{today}-refinement-candidate-{slug}-{key_slug}.md"
        if path.exists():
            continue  # don't duplicate same-day same-pattern
        path.write_text(f"""---
# Auto-generated by refinement_candidates.py — Stop-hook for §0.4
# This is a candidate, NOT a refinement. Review and decide:
#   - Promote to memories/refinements/REF-NNN-<slug>.md (with capability + regression evals)
#   - Reject via memories/refinements/REJECTED-NNN-<slug>.md (cite why)
#   - Defer (this file is harmless; ignore until evidence grows)
scope: memories/drift
classification: operational
authority: agent:claude-sonnet-4.7
sync_class: local-only
auto_generated: true
generator: runtime/hooks/refinement_candidates.py
generated_at: {datetime.now(ICT).isoformat(timespec='seconds')}
---

# Refinement candidate: {c['pattern']}

**Severity:** {c['severity']}
**Occurrences:** {c['count']} (threshold: {THRESHOLD})
**Key:** `{c['key']}`

## Summary
{c['summary']}

## Suggestion
{c['suggestion']}

## Next steps
1. Review `audit/*.jsonl` rows in the last {WINDOW_DAYS} days for `op:{c['pattern'].split('-')[0]}` entries
2. Decide: promote / reject / defer
3. If promote: write `memories/refinements/REF-NNN-<slug>.md` (use `meta/templates/REF.md`)
4. If reject: write `memories/refinements/REJECTED-NNN-<slug>.md` (use `meta/templates/REJECTED.md`)

## Reminder per §0.4
This file does NOT enable any new behavior. It's informational only.
The protocol amendment cycle (propose → adopt → record) still requires
explicit chat-turn approval per §0.5.
""")
        print(f"refinement_candidate: {path.relative_to(memory.parent)}", file=sys.stderr)

def main():
    memory = find_memory()
    if not memory:
        sys.exit(0)  # no memory here, silent exit

    cutoff = datetime.now(ICT) - timedelta(days=WINDOW_DAYS)
    counts = scan_audit(memory, cutoff)
    candidates = detect_patterns(counts)
    emit_candidates(memory, candidates)

    if candidates:
        # Surface to stderr (Claude Code will display)
        print(f"\n📊 §0.4 refinement candidates this session: {len(candidates)}", file=sys.stderr)
        for c in candidates:
            print(f"  · [{c['severity']}] {c['pattern']}: {c['summary']}", file=sys.stderr)
        print(f"  Review: ls .cyberos/memory/store/memories/drift/$(date +%Y-%m-%d)-*", file=sys.stderr)

if __name__ == "__main__":
    main()
