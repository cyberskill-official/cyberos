#!/usr/bin/env python3
"""
cyberos_prune.py — staleness + contradiction detection.

Aspect 1.1 + 9.7 of the Layer-1 improvement catalog.

Surfaces candidates the operator should review. NEVER auto-deletes; only
reports. The operator decides via `cyberos doctor` or by manually
tombstoning the memory.

Three checks:

  1. Staleness (Aspect 9.7) — memories past their `retention.earliest_delete`
     or whose `last_updated_at` is older than the retention rule allows.
     Drives the §8.6 drift candidate surface for very-stale source-tiered
     facts.

  2. Contradiction (Aspect 3.1) — pairs of non-tombstoned memories in the
     same scope, with overlapping tags, where one carries `supersedes` or
     `contradicts` relationship to the other but the older one was never
     tombstoned. Surfaces neglected supersedes-chains.

  3. Unresolved drift (Aspect 8.6) — `memories/drift/*.md` candidates
     older than N days (default 30) without resolution. Drift candidates
     are intended to be short-lived; long-lived ones indicate an
     untriaged source change.

Usage:
    cyberos prune                       # text summary
    cyberos prune --staleness-days 365  # mark anything >1 year stale
    cyberos prune --drift-days 30       # drift older than 30 days
    cyberos prune --json                # machine-readable
    cyberos prune --interactive         # step through each candidate
"""
from __future__ import annotations
import argparse
import json
import re
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


def parse_frontmatter(text: str) -> tuple[dict | None, str]:
    if not text.startswith("---\n"):
        return None, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return None, text
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, text[end + 5:]
    except Exception:
        return None, text[end + 5:]


def iter_memories(brain_root: Path):
    brain = brain_root / ".cyberos-memory"
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/")):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, body = parse_frontmatter(text)
        if not fm:
            continue
        yield rel, fm, body


def check_staleness(brain_root: Path, staleness_days: int) -> list[dict]:
    """Return memories older than staleness_days based on last_updated_at."""
    out = []
    now = datetime.now(ICT)
    cutoff = now - timedelta(days=staleness_days)
    for rel, fm, _ in iter_memories(brain_root):
        if fm.get("tombstoned"):
            continue
        # Retention rule overrides
        ret = fm.get("retention") or {}
        if ret.get("rule") == "indefinite":
            continue
        last = fm.get("last_updated_at") or fm.get("created_at")
        if not isinstance(last, str):
            continue
        try:
            ts = datetime.fromisoformat(last)
        except Exception:
            continue
        if ts < cutoff:
            age = (now - ts).days
            out.append({
                "path": rel,
                "memory_id": fm.get("memory_id"),
                "age_days": age,
                "last_updated_at": last,
                "retention_rule": ret.get("rule"),
                "earliest_delete": ret.get("earliest_delete"),
            })
    out.sort(key=lambda x: -x["age_days"])
    return out


def check_contradictions(brain_root: Path) -> list[dict]:
    """Pairs of memories where one supersedes/contradicts the other but the
    older one was never tombstoned."""
    by_id: dict[str, tuple[str, dict]] = {}
    for rel, fm, _ in iter_memories(brain_root):
        mid = fm.get("memory_id")
        if mid:
            by_id[mid] = (rel, fm)

    out = []
    for rel, fm in by_id.values():
        if fm.get("tombstoned"):
            continue
        sup = fm.get("supersedes")
        if sup:
            target = by_id.get(sup)
            if target and not target[1].get("tombstoned"):
                out.append({
                    "kind": "supersedes-not-tombstoned",
                    "winner": rel,
                    "loser": target[0],
                    "loser_id": sup,
                })
        for edge in (fm.get("relationships") or []):
            if not isinstance(edge, dict):
                continue
            if edge.get("kind") == "contradicts":
                tgt = by_id.get(edge.get("target", ""))
                if tgt and not tgt[1].get("tombstoned") and not fm.get("tombstoned"):
                    out.append({
                        "kind": "contradicts-both-alive",
                        "a": rel,
                        "b": tgt[0],
                    })
    return out


def check_unresolved_drift(brain_root: Path, drift_days: int) -> list[dict]:
    """Drift candidates older than drift_days without resolution."""
    drift_dir = brain_root / ".cyberos-memory" / "memories" / "drift"
    if not drift_dir.exists():
        return []
    now = datetime.now(ICT)
    cutoff = now - timedelta(days=drift_days)
    out = []
    for d in sorted(drift_dir.glob("*.md")):
        try:
            mtime = datetime.fromtimestamp(d.stat().st_mtime, tz=ICT)
        except Exception:
            continue
        if mtime >= cutoff:
            continue
        # Check if a resolution section is present (heuristic)
        text = d.read_text(encoding="utf-8", errors="ignore")
        resolved = "## Resolution" in text or "resolved: true" in text or "resolution:" in text.lower()
        if resolved:
            continue
        out.append({
            "path": d.relative_to(brain_root / ".cyberos-memory").as_posix(),
            "age_days": (now - mtime).days,
            "size_bytes": d.stat().st_size,
        })
    return out


def main():
    p = argparse.ArgumentParser(description="Staleness + contradiction surface")
    p.add_argument("--staleness-days", type=int, default=365,
                   help="flag memories whose last_updated_at is older than N days (default 365)")
    p.add_argument("--drift-days", type=int, default=30,
                   help="flag drift candidates older than N days (default 30)")
    p.add_argument("--json", action="store_true")
    p.add_argument("--interactive", action="store_true",
                   help="step through each candidate and prompt for action")
    args = p.parse_args()

    brain_root = find_brain()

    stale = check_staleness(brain_root, args.staleness_days)
    contras = check_contradictions(brain_root)
    drift = check_unresolved_drift(brain_root, args.drift_days)

    if args.json:
        print(json.dumps({
            "staleness_days_threshold": args.staleness_days,
            "drift_days_threshold": args.drift_days,
            "stale": stale,
            "contradictions": contras,
            "unresolved_drift": drift,
            "totals": {
                "stale": len(stale),
                "contradictions": len(contras),
                "unresolved_drift": len(drift),
            },
        }, indent=2))
        return 1 if (stale or contras or drift) else 0

    print()
    print(f"  cyberos prune — surface only (NEVER auto-deletes)")
    print(f"  stale threshold:  {args.staleness_days} days")
    print(f"  drift threshold:  {args.drift_days} days")
    print()

    if not stale and not contras and not drift:
        print("  ✓ no candidates surfaced")
        return 0

    if stale:
        print(f"  Stale memories ({len(stale)} past {args.staleness_days}-day threshold):")
        for s in stale[:10]:
            print(f"    {s['age_days']:4d}d  {s['path']}  ({s['retention_rule'] or 'no-rule'})")
        if len(stale) > 10:
            print(f"    … +{len(stale) - 10} more")
        print()

    if contras:
        print(f"  Contradictions ({len(contras)} alive-but-superseded / both-alive-contradicts):")
        for c in contras[:10]:
            if c["kind"] == "supersedes-not-tombstoned":
                print(f"    SUPERSEDED-ALIVE: {c['loser']}  (newer winner: {c['winner']})")
            else:
                print(f"    BOTH-ALIVE-CONTRADICTS: {c['a']}  ⟷  {c['b']}")
        if len(contras) > 10:
            print(f"    … +{len(contras) - 10} more")
        print()

    if drift:
        print(f"  Unresolved drift candidates ({len(drift)} past {args.drift_days}-day threshold):")
        for d in drift[:10]:
            print(f"    {d['age_days']:4d}d  {d['path']}  ({d['size_bytes']:,}B)")
        if len(drift) > 10:
            print(f"    … +{len(drift) - 10} more")
        print()

    if args.interactive:
        all_items = (
            [("STALE", s) for s in stale]
            + [("CONTRADICTION", c) for c in contras]
            + [("DRIFT", d) for d in drift]
        )
        if not all_items:
            return 0
        print(f"  Interactive review — {len(all_items)} item(s)")
        for kind, item in all_items:
            print(f"\n  ── {kind} ──")
            print(f"    {json.dumps(item, default=str, indent=4)}")
            r = input("    [s]kip | [t]ombstone-cmd | [o]pen | [q]uit ? ").strip().lower() or "s"
            if r in ("q", "quit"):
                break
            if r in ("t", "tombstone-cmd"):
                path = item.get("path") or item.get("loser") or item.get("a")
                print(f"    cmd: cyberos doctor tombstone-orphan --memory <id> --reason 'pruned via cyberos prune'")
                print(f"    target: {path}")
            elif r in ("o", "open"):
                path = item.get("path") or item.get("loser") or item.get("a")
                print(f"    open {brain_root}/.cyberos-memory/{path}")

    print()
    print(f"  Resolve via: cyberos doctor [tombstone-orphan | resolve-conflict | manual-rollback]")
    return 1 if (stale or contras or drift) else 0


if __name__ == "__main__":
    sys.exit(main())
