"""
cyberos.core.dream.detectors — the four built-in dream detectors
(FR-MEMORY-115 §1 #3).

Each detector is a pure async function ``run(store, since, scope, …)``
returning a list of :class:`DreamProposal`. No writes. The runner
orchestrates them; the applier replays the proposals into the chain.

Slice-3 ships heuristic implementations that don't require LLM calls —
they work entirely from on-disk frontmatter + body content. Slice-4 can
swap in LLM-driven detectors via the ``Invoker`` pattern that
FR-MEMORY-114 introduced.

The four detectors mirror the Anthropic talk's design exactly:

* :func:`run_duplicates` — pairwise cosine-style overlap ≥ threshold
  within scope → ``merge`` proposals.
* :func:`run_stale` — memory body contradicted by a later audit row
  carrying a ``correction_to:`` pointer or a re-write that changed
  meaning → ``stale`` proposal.
* :func:`run_patterns` — recurring (task fingerprint, outcome) combos
  across ``episode.logged`` aux rows → ``new`` proposal under
  ``memories/refinements/``.
* :func:`run_verify` — memory whose claims were used in a session
  without correction → ``verify`` proposal annotating
  ``meta.last_verified_at``.
"""

from __future__ import annotations

import hashlib
import re
from collections import Counter, defaultdict
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Optional

from cyberos.core.dream.proposals import (
    DreamProposal,
    generate_proposal_id,
)


_WORD_RE = re.compile(r"[A-Za-z0-9]+")


# ────────────────────────────────────────────────────────────────────
# duplicates detector
# ────────────────────────────────────────────────────────────────────


async def run_duplicates(
    store: Path,
    since: Optional[timedelta] = None,
    scope: str = "",
    *,
    threshold: float = 0.92,
    invoker_name: Optional[str] = None,
) -> list[DreamProposal]:
    """Find near-duplicate memories within ``scope`` (cosine-style)."""
    candidates = _enumerate_memories(store, scope)
    proposals: list[DreamProposal] = []
    seen: set[str] = set()

    # Cheap pairwise comparison with bag-of-words Jaccard as a stand-in
    # for cosine over int8 embeddings. Slice-4 promotes this to real
    # sentence-transformers cosine when ``cyberos.core.semantic.available()``.
    docs: list[tuple[str, set[str]]] = []
    for rel_path, body_bytes in candidates:
        tokens = _bag(body_bytes)
        if not tokens:
            continue
        docs.append((rel_path, tokens))

    for i, (a_path, a_tokens) in enumerate(docs):
        if a_path in seen:
            continue
        cluster = [a_path]
        for j in range(i + 1, len(docs)):
            b_path, b_tokens = docs[j]
            if b_path in seen:
                continue
            sim = _jaccard(a_tokens, b_tokens)
            if sim >= threshold:
                cluster.append(b_path)
                seen.add(b_path)
        if len(cluster) >= 2:
            canonical = cluster[0]
            seen.add(canonical)
            preconds = {
                p: _body_hash(store / p) for p in cluster
            }
            proposals.append(DreamProposal(
                proposal_id=generate_proposal_id(),
                op="merge",
                paths=cluster,
                into=canonical,
                rationale=(
                    f"Duplicates detector: {len(cluster)} files with bag-of-words "
                    f"similarity ≥ {threshold}. Canonical = {canonical!r}; "
                    f"others tombstoned with extra.merged_into."
                ),
                content_preview=_first_body_excerpt(store / canonical),
                precondition_body_hashes=preconds,
            ))
    return proposals


# ────────────────────────────────────────────────────────────────────
# stale detector
# ────────────────────────────────────────────────────────────────────


async def run_stale(
    store: Path,
    since: Optional[timedelta] = None,
    scope: str = "",
    *,
    invoker_name: Optional[str] = None,
) -> list[DreamProposal]:
    """Find memories explicitly corrected by later audit rows.

    Heuristic: scan the audit chain for ``put`` rows carrying
    ``extra.correction_to: <path>``. Each pointer marks ``<path>`` as
    superseded → emit a ``stale`` proposal.
    """
    from cyberos.core.dream._audit_iter import iter_audit_rows  # lazy

    proposals: list[DreamProposal] = []
    cutoff_ns: Optional[int] = None
    if since is not None:
        cutoff_ns = int((datetime.now(timezone.utc) - since).timestamp() * 1e9)

    try:
        rows = list(iter_audit_rows(store))
    except Exception:
        return proposals

    for row in rows:
        if cutoff_ns is not None and row.get("ts_ns", 0) < cutoff_ns:
            continue
        if row.get("op") != "put":
            continue
        extra = row.get("extra") or {}
        target = extra.get("correction_to")
        if not target or not isinstance(target, str):
            continue
        if scope and not target.startswith(scope.rstrip("/")):
            continue
        target_abs = store / target
        if not target_abs.exists():
            continue
        proposals.append(DreamProposal(
            proposal_id=generate_proposal_id(),
            op="stale",
            paths=[target],
            rationale=(
                f"Stale detector: audit row seq={row.get('extra', {}).get('_seq')} "
                f"corrects this memory ({row.get('path')})."
            ),
            input_audit_seqs=[int(row.get("extra", {}).get("_seq") or 0)],
            precondition_body_hashes={target: _body_hash(target_abs)},
        ))
    return proposals


# ────────────────────────────────────────────────────────────────────
# patterns detector
# ────────────────────────────────────────────────────────────────────


async def run_patterns(
    store: Path,
    since: Optional[timedelta] = None,
    scope: str = "",
    *,
    invoker_name: Optional[str] = None,
    min_recurrence: int = 3,
) -> list[DreamProposal]:
    """Find recurring task/outcome combinations across episode.logged rows.

    Aggregates ``episode.logged`` aux audit rows (FR-MEMORY-112) by task
    fingerprint. When the same task fingerprint appears ``min_recurrence``+
    times within the window, emit a ``new`` proposal pointing at a
    ``memories/refinements/<slug>.md`` that summarises the pattern.
    """
    from cyberos.core.dream._audit_iter import iter_audit_rows

    cutoff_ns: Optional[int] = None
    if since is not None:
        cutoff_ns = int((datetime.now(timezone.utc) - since).timestamp() * 1e9)

    by_task: dict[str, list[dict]] = defaultdict(list)
    try:
        rows = list(iter_audit_rows(store))
    except Exception:
        return []

    for row in rows:
        if cutoff_ns is not None and row.get("ts_ns", 0) < cutoff_ns:
            continue
        if row.get("op") != "episode.logged":
            continue
        payload = (row.get("extra") or {})
        path = payload.get("path", "")
        if scope and not path.startswith(scope.rstrip("/")):
            continue
        # Fingerprint = first 60 chars of the task field on the original
        # episode file's frontmatter. Fall back to the row's path.
        fingerprint = _episode_task_fingerprint(store, path)
        if not fingerprint:
            continue
        by_task[fingerprint].append(payload)

    proposals: list[DreamProposal] = []
    for fingerprint, occurrences in by_task.items():
        if len(occurrences) < min_recurrence:
            continue
        outcomes = Counter(o.get("outcome", "unknown") for o in occurrences)
        slug = hashlib.sha256(fingerprint.encode("utf-8")).hexdigest()[:8]
        target_path = f"memories/refinements/{slug}-recurring.md"
        avg_duration = sum(int(o.get("duration_ms") or 0) for o in occurrences) // len(occurrences)
        proposals.append(DreamProposal(
            proposal_id=generate_proposal_id(),
            op="new",
            paths=[target_path],
            content_preview=(
                f"# Recurring task pattern\n\n"
                f"Observed {len(occurrences)} runs of the same task fingerprint.\n"
                f"Outcomes: {dict(outcomes)}\n"
                f"Average duration: {avg_duration} ms.\n"
                f"Fingerprint: {fingerprint[:80]}\n"
            ),
            rationale=(
                f"Patterns detector: {len(occurrences)} episodes share task "
                f"fingerprint {fingerprint[:40]!r}. Outcomes: {dict(outcomes)}."
            ),
        ))
    return proposals


# ────────────────────────────────────────────────────────────────────
# verify detector
# ────────────────────────────────────────────────────────────────────


async def run_verify(
    store: Path,
    since: Optional[timedelta] = None,
    scope: str = "",
    *,
    invoker_name: Optional[str] = None,
) -> list[DreamProposal]:
    """Emit verify proposals for memories that have NOT been touched +
    were referenced by some episode in the window.

    Heuristic: a memory is "verifiable" if (a) it exists, (b) it's
    been read (`op=view`) at least once in the window, and (c) it has
    NOT been the target of a `correction_to` row. Slice-4 sharpens
    this with semantic equivalence checks; slice-3 keeps it simple.
    """
    from cyberos.core.dream._audit_iter import iter_audit_rows

    cutoff_ns: Optional[int] = None
    if since is not None:
        cutoff_ns = int((datetime.now(timezone.utc) - since).timestamp() * 1e9)

    viewed: set[str] = set()
    corrected: set[str] = set()
    try:
        rows = list(iter_audit_rows(store))
    except Exception:
        return []
    for row in rows:
        if cutoff_ns is not None and row.get("ts_ns", 0) < cutoff_ns:
            continue
        op = row.get("op")
        path = row.get("path") or ""
        if scope and not path.startswith(scope.rstrip("/")):
            continue
        if op == "view":
            viewed.add(path)
        elif op == "put":
            t = (row.get("extra") or {}).get("correction_to")
            if isinstance(t, str):
                corrected.add(t)

    proposals: list[DreamProposal] = []
    for path in sorted(viewed - corrected):
        abs_p = store / path
        if not abs_p.exists():
            continue
        proposals.append(DreamProposal(
            proposal_id=generate_proposal_id(),
            op="verify",
            paths=[path],
            rationale=(
                f"Verify detector: memory was read in the window without "
                f"any correction_to row pointing at it; safe to annotate "
                f"meta.last_verified_at."
            ),
            precondition_body_hashes={path: _body_hash(abs_p)},
        ))
    return proposals


# ────────────────────────────────────────────────────────────────────
# helpers
# ────────────────────────────────────────────────────────────────────


def _enumerate_memories(store: Path, scope: str) -> list[tuple[str, bytes]]:
    """Yield (relative_path, body_bytes) for every memory in scope."""
    from cyberos.core.frontmatter import parse, parse_legacy_yaml, looks_like_yaml

    out: list[tuple[str, bytes]] = []
    root = store / scope if scope else store
    if not root.exists():
        return out
    for md_path in root.rglob("*.md"):
        try:
            raw = md_path.read_bytes()
            try:
                _, body = parse(raw)
            except Exception:
                if looks_like_yaml(raw):
                    _, body = parse_legacy_yaml(raw)
                else:
                    continue
            rel = str(md_path.relative_to(store))
            out.append((rel, body))
        except Exception:
            continue
    return out


def _bag(body: bytes) -> set[str]:
    text = body.decode("utf-8", errors="ignore").lower()
    return set(_WORD_RE.findall(text))


def _jaccard(a: set[str], b: set[str]) -> float:
    if not a or not b:
        return 0.0
    return len(a & b) / len(a | b)


def _body_hash(path: Path) -> str:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except FileNotFoundError:
        return ""


def _first_body_excerpt(path: Path, n: int = 400) -> str:
    try:
        from cyberos.core.frontmatter import parse, parse_legacy_yaml, looks_like_yaml
        raw = path.read_bytes()
        try:
            _, body = parse(raw)
        except Exception:
            if looks_like_yaml(raw):
                _, body = parse_legacy_yaml(raw)
            else:
                body = raw
        return body[:n].decode("utf-8", errors="ignore")
    except Exception:
        return ""


def _episode_task_fingerprint(store: Path, rel_path: str) -> str:
    """Read the episode file's `task` extra, normalise + truncate."""
    if not rel_path:
        return ""
    abs_p = store / rel_path
    if not abs_p.exists():
        return ""
    try:
        from cyberos.core.frontmatter import parse, parse_legacy_yaml, looks_like_yaml
        raw = abs_p.read_bytes()
        try:
            fm, _ = parse(raw)
        except Exception:
            if looks_like_yaml(raw):
                fm, _ = parse_legacy_yaml(raw)
            else:
                return ""
        if fm.kind != "episode":
            return ""
        task = (fm.extra or {}).get("task", "")
        # Fingerprint: lower-cased, word-bag, first 80 chars joined
        words = sorted(set(_WORD_RE.findall(str(task).lower())))
        return " ".join(words)[:80]
    except Exception:
        return ""
