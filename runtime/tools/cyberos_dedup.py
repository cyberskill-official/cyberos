#!/usr/bin/env python3
"""
cyberos_dedup.py — duplicate-memory detection by content fingerprint.

Aspect 9.6 of the Layer-1 improvement catalog.

Surfaces likely-duplicate memories across the BRAIN using two signals:

  1. Body-shingle fingerprint — 5-gram shingling on body text after
     normalisation (lowercase, collapse whitespace, strip frontmatter),
     Jaccard ≥ 0.8 → candidate pair.

  2. Slug stem similarity — strip NNN prefix and trailing version
     suffix, compute Levenshtein-style similarity on the slug;
     ≥ 0.85 → flagged for review.

Does NOT auto-merge. Surfaces candidates as a report. Operator runs
`cyberos doctor resolve-conflict` or manually tombstones the duplicate
and writes a §3 reconciliation row in the audit ledger.

Usage:
    cyberos dedup                      # full BRAIN report (stdout)
    cyberos dedup --scope memories/facts   # restrict to one bucket
    cyberos dedup --threshold 0.7      # lower Jaccard threshold
    cyberos dedup --json               # machine-readable output
    cyberos dedup --since 30d          # only memories edited in last 30d
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


def parse_frontmatter(text: str) -> tuple[dict, str]:
    if not text.startswith("---\n"):
        return {}, text
    end = text.find("\n---\n", 4)
    if end < 0:
        return {}, text
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, text[end + 5:]
    except Exception:
        return {}, text[end + 5:]


def normalise(body: str) -> str:
    """Lowercase, strip markdown chrome, collapse whitespace."""
    text = body.lower()
    text = re.sub(r"```.*?```", " ", text, flags=re.DOTALL)  # drop code fences
    text = re.sub(r"[#*_`>\-]", " ", text)  # markdown punctuation
    text = re.sub(r"https?://\S+", " ", text)  # urls
    text = re.sub(r"\s+", " ", text).strip()
    return text


def shingles(text: str, n: int = 5) -> set:
    """Word-level n-gram shingle set."""
    words = text.split()
    if len(words) < n:
        return {tuple(words)}
    return {tuple(words[i:i+n]) for i in range(len(words) - n + 1)}


def jaccard(a: set, b: set) -> float:
    if not a or not b:
        return 0.0
    inter = len(a & b)
    union = len(a | b)
    return inter / union if union else 0.0


def stem_slug(filename: str) -> str:
    """REF-042-foo-bar.md → foo-bar"""
    base = filename.rsplit(".", 1)[0]
    base = re.sub(r"^[A-Z]+-\d+-?", "", base)
    return base


def slug_similarity(a: str, b: str) -> float:
    """Cheap similarity: longest common substring ratio."""
    if a == b:
        return 1.0
    if not a or not b:
        return 0.0
    # Jaccard over 3-grams
    A = {a[i:i+3] for i in range(max(1, len(a) - 2))}
    B = {b[i:i+3] for i in range(max(1, len(b) - 2))}
    return jaccard(A, B)


def parse_since(s: str) -> timedelta:
    """Parse '30d', '4w', '6h' into timedelta."""
    m = re.match(r"^(\d+)\s*([dwhDWH])$", s.strip())
    if not m:
        raise ValueError(f"invalid --since: {s!r}; use 7d, 4w, 24h")
    n = int(m.group(1))
    unit = m.group(2).lower()
    return {"d": timedelta(days=n), "w": timedelta(weeks=n), "h": timedelta(hours=n)}[unit]


def main():
    p = argparse.ArgumentParser(description="Detect likely-duplicate memories by content fingerprint")
    p.add_argument("--scope", help="restrict to scope prefix (e.g. memories/facts)")
    p.add_argument("--threshold", type=float, default=0.8, help="Jaccard threshold (default 0.8)")
    p.add_argument("--slug-threshold", type=float, default=0.85,
                   help="slug-similarity threshold for slug-only matches (default 0.85)")
    p.add_argument("--since", help="only memories edited within last N (e.g. 30d, 4w, 24h)")
    p.add_argument("--json", action="store_true", help="JSON output")
    args = p.parse_args()

    brain_root = find_brain()
    brain = brain_root / ".cyberos-memory"

    cutoff = None
    if args.since:
        cutoff = datetime.now(ICT) - parse_since(args.since)

    # Collect memories
    entries: list[dict] = []
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        # Protocol-history files are deliberate near-duplicates (versioned snapshots);
        # excluded unless user explicitly scopes there.
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/")):
            continue
        if rel.startswith("meta/protocol-history/") and not (args.scope and args.scope.startswith("meta/protocol-history")):
            continue
        if args.scope and not rel.startswith(args.scope):
            continue
        if cutoff:
            try:
                mtime = datetime.fromtimestamp(md.stat().st_mtime, tz=ICT)
                if mtime < cutoff:
                    continue
            except Exception:
                pass
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, body = parse_frontmatter(text)
        if fm.get("tombstoned"):
            continue
        norm = normalise(body)
        if len(norm.split()) < 8:
            # Skip near-empty bodies (templates, stubs)
            continue
        entries.append({
            "rel": rel,
            "memory_id": fm.get("memory_id"),
            "scope": fm.get("scope"),
            "tags": fm.get("tags", []),
            "stem": stem_slug(md.name),
            "body_norm": norm,
            "shingles": shingles(norm),
        })

    # Pairwise comparison
    pairs = []
    for i in range(len(entries)):
        for j in range(i + 1, len(entries)):
            a, b = entries[i], entries[j]
            body_sim = jaccard(a["shingles"], b["shingles"])
            slug_sim = slug_similarity(a["stem"], b["stem"])
            if body_sim >= args.threshold or slug_sim >= args.slug_threshold:
                # Skip the legitimate DEC↔REF implements-pair pattern:
                # high slug similarity but low body similarity across decisions/ + refinements/.
                buckets = {a["rel"].split("/")[1] if "/" in a["rel"] else "", b["rel"].split("/")[1] if "/" in b["rel"] else ""}
                if buckets == {"decisions", "refinements"} and body_sim < 0.3:
                    continue
                pairs.append({
                    "a": a["rel"],
                    "a_id": a["memory_id"],
                    "b": b["rel"],
                    "b_id": b["memory_id"],
                    "body_jaccard": round(body_sim, 3),
                    "slug_similarity": round(slug_sim, 3),
                    "same_scope": a["scope"] == b["scope"],
                    "tags_overlap": list(set(a["tags"]) & set(b["tags"])),
                })

    pairs.sort(key=lambda p: (-p["body_jaccard"], -p["slug_similarity"]))

    if args.json:
        print(json.dumps({"pairs": pairs, "total": len(pairs)}, indent=2))
        return 0

    if not pairs:
        print(f"  ✓ no duplicate pairs found ({len(entries)} memories scanned)")
        return 0

    print(f"  Scanned {len(entries)} memories. Found {len(pairs)} suspicious pair(s).")
    print()
    for i, pair in enumerate(pairs, 1):
        marker = "★" if pair["body_jaccard"] >= 0.9 else " "
        scope_marker = "[same-scope]" if pair["same_scope"] else "[cross-scope]"
        print(f"  {i:3d}. {marker} body={pair['body_jaccard']:.2f}  slug={pair['slug_similarity']:.2f}  {scope_marker}")
        print(f"        A: {pair['a']}")
        print(f"        B: {pair['b']}")
        if pair["tags_overlap"]:
            print(f"        shared tags: {', '.join(pair['tags_overlap'])}")
        print()
    print(f"  Resolve via: cyberos doctor resolve-conflict --memory <id>, OR")
    print(f"               tombstone the duplicate + write a §3 reconciliation row.")
    return 1 if pairs else 0


if __name__ == "__main__":
    sys.exit(main())
