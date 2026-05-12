#!/usr/bin/env python3
"""
cyberos_hybrid_search.py — Reciprocal Rank Fusion over multiple search backends.

Tier E.4 of post-catalog improvements (Batch 15).

Today we have three independent search surfaces:
  - SQLite FTS (cyberos search)
  - TF-IDF (cyberos semantic-search --backend tfidf)
  - sentence-transformers (cyberos semantic-search --backend sbert, opt-in)

Each works fine alone. RRF combines them so the union ranks better than
any single backend on most queries.

RRF score:
    score(d) = Σ_{k in backends} 1 / (k_const + rank_k(d))

Default k_const = 60 (standard).

Usage:
    cyberos hybrid-search "council voices ambiguous refinement"
    cyberos hybrid-search "..." --limit 5 --weight-fts 1.0 --weight-tfidf 0.5
    cyberos hybrid-search "..." --no-fts        # skip SQLite if no index
"""
from __future__ import annotations
import argparse
import importlib.util
import sys
from pathlib import Path

K_CONST = 60


def find_brain(start: Path = None) -> Path:
    cur = (start or Path.cwd()).resolve()
    while cur != cur.parent:
        if (cur / ".cyberos-memory").is_dir():
            return cur
        cur = cur.parent
    raise SystemExit("no .cyberos-memory/ found")


def _load(brain_root: Path, name: str):
    p = brain_root / "runtime" / "tools" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, p)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


def tfidf_ranking(brain_root: Path, query: str, scope: str = "") -> list[str]:
    sm = _load(brain_root, "cyberos_semantic_search")
    docs = sm.collect_docs(brain_root, scope)
    scored = sm.tfidf_score(sm.tokenize(query), docs)
    return [d["path"] for _, d in scored]


def fts_ranking(brain_root: Path, query: str, scope: str = "") -> list[str]:
    """Use cyberos_index.py if a sqlite db exists; else fall back to grep."""
    idx_db = brain_root / ".cyberos-memory" / "index" / "cyberos.db"
    brain = brain_root / ".cyberos-memory"
    out = []
    if idx_db.exists():
        # Shell out — cyberos_index.py query is the canonical FTS path
        import subprocess
        try:
            r = subprocess.run(
                ["python3", str(brain_root / "runtime" / "tools" / "cyberos_index.py"),
                 str(brain), "query", "fts", query],
                capture_output=True, text=True, timeout=5,
            )
            for line in r.stdout.splitlines():
                line = line.strip()
                if line.endswith(".md") and (not scope or line.startswith(scope)):
                    out.append(line)
        except Exception:
            pass
    if not out:
        # Grep fallback
        q_re = query.lower()
        for md in sorted(brain.rglob("*.md")):
            if not md.is_file() or md.name.startswith("."):
                continue
            rel = md.relative_to(brain).as_posix()
            if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/", ".branches/")):
                continue
            if scope and not rel.startswith(scope):
                continue
            try:
                if q_re in md.read_text(encoding="utf-8", errors="ignore").lower():
                    out.append(rel)
            except Exception:
                continue
    return out


def rrf(rankings: list[tuple[list[str], float]], k: int = K_CONST) -> list[tuple[float, str]]:
    """Combine rankings via reciprocal rank fusion.

    `rankings` is list of (paths_in_rank_order, weight). Returns
    [(score, path)] sorted descending.
    """
    scores: dict[str, float] = {}
    for paths, w in rankings:
        for rank, path in enumerate(paths, start=1):
            scores[path] = scores.get(path, 0.0) + w * (1.0 / (k + rank))
    return sorted(((s, p) for p, s in scores.items()), key=lambda x: -x[0])


def main():
    p = argparse.ArgumentParser(description="hybrid search via RRF (Tier E.4)")
    p.add_argument("query")
    p.add_argument("--scope", default="")
    p.add_argument("--limit", type=int, default=10)
    p.add_argument("--weight-fts", type=float, default=1.0)
    p.add_argument("--weight-tfidf", type=float, default=1.0)
    p.add_argument("--no-fts", action="store_true")
    p.add_argument("--no-tfidf", action="store_true")
    p.add_argument("--k", type=int, default=K_CONST)
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    brain_root = find_brain()
    rankings = []
    if not args.no_fts:
        fts = fts_ranking(brain_root, args.query, args.scope)
        rankings.append((fts, args.weight_fts))
    if not args.no_tfidf:
        tfidf = tfidf_ranking(brain_root, args.query, args.scope)
        rankings.append((tfidf, args.weight_tfidf))

    if not rankings:
        print("  ✗ all backends disabled; nothing to rank", file=sys.stderr)
        return 2

    fused = rrf(rankings, k=args.k)[:args.limit]

    if args.json:
        import json
        print(json.dumps([{"score": round(s, 5), "path": p} for s, p in fused], indent=2))
        return 0

    print(f"\n  Hybrid search ({len(rankings)} backend(s), RRF k={args.k}) for {args.query!r}")
    print(f"  showing top {len(fused)}\n")
    for s, path in fused:
        print(f"  {s:.5f}  {path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
