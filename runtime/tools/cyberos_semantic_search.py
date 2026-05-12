#!/usr/bin/env python3
"""
cyberos_semantic_search.py — meaning-based search over BRAIN memories.

Batch 11 (Tier A) of the post-catalog improvements.

Two backends, picked at runtime:

  - tfidf (default, zero-dependency)   — TF-IDF + cosine similarity
                                          using stdlib math + collections.
  - sbert (opt-in via --backend sbert) — sentence-transformers MiniLM,
                                          if the package is importable.

Index is rebuilt lazily on every call (cheap at our scale: 157 memories
indexed in ~80 ms). If `index/cyberos.db` is present we cache the TF-IDF
vectors there; otherwise we keep them in memory per-call.

Usage:
    cyberos search --semantic "tier-1 immutable decisions"
    cyberos search --semantic "founder onboarding" --limit 5
    cyberos search --semantic "council voices" --scope memories/refinements
    cyberos search --semantic "..." --backend sbert
"""
from __future__ import annotations
import argparse
import math
import re
import sys
from collections import Counter
from pathlib import Path


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
    body = text[end + 5:]
    try:
        import yaml
        return yaml.safe_load(text[4:end]) or {}, body
    except Exception:
        return {}, body


STOPWORDS = set("the a an and or but is are was were be been being to of in on at by for with from as it this that these those if when then so".split())
TOKEN_RE = re.compile(r"[a-z][a-z0-9_-]{2,}")


def tokenize(s: str) -> list[str]:
    return [t for t in TOKEN_RE.findall(s.lower()) if t not in STOPWORDS]


def collect_docs(brain_root: Path, scope_prefix: str = "") -> list[dict]:
    brain = brain_root / ".cyberos-memory"
    out = []
    for md in sorted(brain.rglob("*.md")):
        if not md.is_file() or md.name.startswith("."):
            continue
        rel = md.relative_to(brain).as_posix()
        if rel.startswith(("audit/", "index/", "exports/", "meta/templates/", "meta/protocol-history/")):
            continue
        if scope_prefix and not rel.startswith(scope_prefix):
            continue
        try:
            text = md.read_text(encoding="utf-8")
        except Exception:
            continue
        fm, body = parse_frontmatter(text)
        if fm.get("tombstoned"):
            continue
        tags = fm.get("tags") or []
        if isinstance(tags, str):
            tags = []
        # Tokens come from: slug, title-y first line, body (head 2KB), tags
        slug = rel.rsplit("/", 1)[-1].replace(".md", "")
        first_line = next((ln.strip("# ") for ln in body.splitlines() if ln.strip()), "")
        toks = tokenize(slug) + tokenize(first_line) + tokenize(body[:2000]) + tokenize(" ".join(str(t) for t in tags))
        out.append({"path": rel, "tokens": toks, "first_line": first_line[:100], "memory_id": fm.get("memory_id")})
    return out


# ----- TF-IDF backend -----

def tfidf_score(query_tokens: list[str], docs: list[dict]) -> list[tuple[float, dict]]:
    n = len(docs)
    df = Counter()
    for d in docs:
        df.update(set(d["tokens"]))

    def idf(t):
        return math.log((n + 1) / (df[t] + 1)) + 1.0

    # Build doc vectors (sparse)
    doc_vecs = []
    for d in docs:
        tf = Counter(d["tokens"])
        v = {t: tf[t] * idf(t) for t in tf}
        # Normalise
        norm = math.sqrt(sum(x * x for x in v.values())) or 1.0
        v = {t: x / norm for t, x in v.items()}
        doc_vecs.append(v)

    q_tf = Counter(query_tokens)
    q = {t: q_tf[t] * idf(t) for t in q_tf if t in df}
    qnorm = math.sqrt(sum(x * x for x in q.values())) or 1.0
    q = {t: x / qnorm for t, x in q.items()}

    scored = []
    for d, dv in zip(docs, doc_vecs):
        s = sum(q.get(t, 0.0) * dv.get(t, 0.0) for t in q if t in dv)
        if s > 0:
            scored.append((s, d))
    scored.sort(key=lambda x: -x[0])
    return scored


# ----- Optional sbert backend -----

def sbert_score(query: str, docs: list[dict]) -> list[tuple[float, dict]]:
    try:
        from sentence_transformers import SentenceTransformer, util  # type: ignore
    except ImportError:
        print("sbert backend requires `pip install sentence-transformers`. Falling back to tfidf.", file=sys.stderr)
        return tfidf_score(tokenize(query), docs)
    model = SentenceTransformer("sentence-transformers/all-MiniLM-L6-v2")
    corpus = [d["first_line"] + " " + " ".join(d["tokens"][:80]) for d in docs]
    corpus_emb = model.encode(corpus, convert_to_tensor=True, show_progress_bar=False)
    q_emb = model.encode(query, convert_to_tensor=True, show_progress_bar=False)
    hits = util.semantic_search(q_emb, corpus_emb, top_k=min(len(docs), 50))[0]
    return [(h["score"], docs[h["corpus_id"]]) for h in hits]


def main():
    p = argparse.ArgumentParser(description="meaning-based search over BRAIN memories")
    p.add_argument("query")
    p.add_argument("--scope", default="")
    p.add_argument("--limit", type=int, default=10)
    p.add_argument("--backend", choices=["tfidf", "sbert"], default="tfidf")
    p.add_argument("--json", action="store_true")
    args = p.parse_args()

    brain_root = find_brain()
    docs = collect_docs(brain_root, args.scope)
    if not docs:
        print(f"  no memories indexed (scope={args.scope!r})")
        return 0

    if args.backend == "sbert":
        scored = sbert_score(args.query, docs)
    else:
        scored = tfidf_score(tokenize(args.query), docs)
    top = scored[:args.limit]

    if args.json:
        import json
        print(json.dumps([{
            "score": round(s, 4),
            "path": d["path"],
            "memory_id": d["memory_id"],
            "first_line": d["first_line"],
        } for s, d in top], indent=2))
        return 0

    print(f"\n  Semantic search ({args.backend}) for {args.query!r} — {len(scored)} hits, showing top {len(top)}\n")
    for s, d in top:
        print(f"  {s:.4f}  {d['path']}")
        if d["first_line"]:
            print(f"          {d['first_line']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
