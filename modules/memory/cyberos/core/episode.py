"""
cyberos.core.episode — episodic memory (FR-MEMORY-112).

Episodes are the agent's records of completed tasks. They are structurally
distinct from the other memory kinds (`decision`, `fact`, `person`, …) which
describe knowledge; an Episode describes an *event* — what task the agent
did, how it approached it, and what the outcome was.

Episodes live at ``memories/episodes/<hex>/<hex>/<slug>.md`` (date- and
content-hash-sharded) and share the standard frontmatter shape: the
`Frontmatter.extra` dict carries the Episode-specific fields. This keeps
the closed top-level schema intact while letting downstream callers
(``recall-similar``, FR-MEMORY-115 dream, FR-MEMORY-120 history) project
the rich shape.

Per FR-MEMORY-112 §1:

* Per-kind validator enforces the closed `outcome` enum + `quality_score`
  range + `duration_ms` non-negativity.
* The Episode dataclass constructs a body (the "searchable document") + a
  frontmatter (with `kind="episode"` + extras dict) + a path under
  `memories/episodes/<hex>/<hex>/`. Writes route through
  ``cyberos.core.ops.put`` — the canonical writer path.
* Recall-similar reads ``kind=="episode"`` rows from the FTS5 index (or
  semantic vector index when ``--semantic`` is available) and ranks by
  combined score ``relevance · w_r + quality · w_i + recency · w_t``.
  Ranking is delegated to ``cyberos.core.ranking.score_hits()`` (FR-MEMORY-113)
  when that module is available; FR-MEMORY-112 ships with the placeholder
  ``recency=1.0`` per §1 #9.
"""

from __future__ import annotations

import hashlib
import os
import time
import uuid
from dataclasses import dataclass, field
from pathlib import Path
from typing import Iterable, Literal, Optional

EpisodeOutcome = Literal["success", "partial", "failure"]
_OUTCOMES: frozenset[str] = frozenset({"success", "partial", "failure"})


@dataclass
class Episode:
    """An Episode — one completed task."""

    task: str
    approach: str
    outcome: EpisodeOutcome
    duration_ms: int
    token_cost: Optional[int] = None
    quality_score: Optional[float] = None
    notes: str = ""
    error: Optional[str] = None

    def __post_init__(self) -> None:
        if not isinstance(self.task, str) or not self.task.strip():
            raise ValueError("Episode.task must be a non-empty string after trim")
        if not isinstance(self.approach, str) or not self.approach.strip():
            raise ValueError("Episode.approach must be a non-empty string after trim")
        if self.outcome not in _OUTCOMES:
            raise ValueError(
                f"Episode.outcome {self.outcome!r} not in closed enum {sorted(_OUTCOMES)}"
            )
        if not isinstance(self.duration_ms, int) or self.duration_ms < 0:
            raise ValueError(
                f"Episode.duration_ms must be an int ≥ 0; got {self.duration_ms!r}"
            )
        if self.token_cost is not None and (
            not isinstance(self.token_cost, int) or self.token_cost < 0
        ):
            raise ValueError(
                f"Episode.token_cost must be int ≥ 0 if set; got {self.token_cost!r}"
            )
        if self.quality_score is not None and not (
            0.0 <= float(self.quality_score) <= 1.0
        ):
            raise ValueError(
                f"Episode.quality_score must be in [0.0, 1.0] if set; "
                f"got {self.quality_score!r}"
            )
        if self.outcome != "success" and not (self.error or "").strip():
            raise ValueError(
                f"Episode.outcome={self.outcome!r} requires non-empty error"
            )

    def searchable_document(self) -> str:
        """Body bytes for FTS5 + semantic embeddings.

        Per FR-MEMORY-112 §1 #7, the deterministic format is::

            Task: <task>
            Approach: <approach>
            Outcome: <outcome>
            Notes: <notes>

        The `error` field is deliberately EXCLUDED from the searchable
        document — stack-trace noise would pollute embeddings. It remains
        on the frontmatter for `cyberos history` and dream-pipeline
        introspection.
        """
        return (
            f"Task: {self.task}\n"
            f"Approach: {self.approach}\n"
            f"Outcome: {self.outcome}\n"
            f"Notes: {self.notes}"
        )

    def extra_fields(self) -> dict:
        """The Episode-specific subset of `Frontmatter.extra`."""
        out: dict = {
            "task": self.task,
            "approach": self.approach,
            "outcome": self.outcome,
            "duration_ms": self.duration_ms,
        }
        if self.token_cost is not None:
            out["token_cost"] = self.token_cost
        if self.quality_score is not None:
            out["quality_score"] = float(self.quality_score)
        if self.notes:
            out["notes"] = self.notes
        if self.error:
            out["error"] = self.error
        return out


def validate_episode_extras(extras: dict) -> None:
    """Walker invariant — validates that an episode's `extras` are well-formed.

    Called by the per-kind frontmatter validator + by the walker's
    ``episode-outcome-closed-enum`` invariant. Raises ``ValueError`` with a
    structured message on violation.
    """
    if "task" not in extras or not (extras.get("task") or "").strip():
        raise ValueError("episode-frontmatter: missing or empty `task`")
    if "approach" not in extras or not (extras.get("approach") or "").strip():
        raise ValueError("episode-frontmatter: missing or empty `approach`")
    if "outcome" not in extras:
        raise ValueError("episode-frontmatter: missing `outcome`")
    if extras["outcome"] not in _OUTCOMES:
        raise ValueError(
            f"episode-outcome-closed-enum: outcome={extras['outcome']!r} "
            f"not in {sorted(_OUTCOMES)}"
        )
    duration_ms = extras.get("duration_ms")
    if not isinstance(duration_ms, int) or duration_ms < 0:
        raise ValueError(
            f"episode-duration-non-negative: duration_ms={duration_ms!r}"
        )
    if "quality_score" in extras:
        qs = extras["quality_score"]
        try:
            if not (0.0 <= float(qs) <= 1.0):
                raise ValueError
        except (TypeError, ValueError):
            raise ValueError(
                f"episode-quality-score-range: quality_score={qs!r} "
                f"not in [0.0, 1.0]"
            )
    if extras["outcome"] != "success":
        if not (extras.get("error") or "").strip():
            raise ValueError(
                f"episode-error-on-non-success: outcome={extras['outcome']!r} "
                f"requires non-empty `error` extra"
            )


def _episode_rel_path(task: str) -> str:
    """Stable shard path derived from sha256(task) + a uuid suffix.

    The hex sharding mirrors AGENTS.md §3.2's
    ``memories/<kind>/<hex>/<hex>/<file>.md`` layout. We use 8 hex chars
    for the first directory and 8 for the slug prefix to keep listings
    human-skimmable.
    """
    digest = hashlib.sha256(task.encode("utf-8")).hexdigest()
    return f"memories/episodes/{digest[:2]}/{digest[2:4]}/{digest[:8]}-{uuid.uuid4().hex[:6]}.md"


def log(
    writer,  # cyberos.core.writer.Writer
    episode: Episode,
    *,
    actor: str = "agent",
    rel_path: Optional[str] = None,
) -> tuple[int, str]:
    """Append an Episode to the memory.

    Routes through ``cyberos.core.ops.put`` — the canonical writer entry
    point — emitting one `op="put"` audit row. The Episode's extras live
    inside the frontmatter's `extra` dict so downstream readers can
    project them without a separate Episode-specific reader path.

    Returns (head_seq, rel_path).
    """
    # Lazy import to avoid pulling ops into the cold-CLI startup cost.
    from cyberos.core.ops import put
    from cyberos.core.frontmatter import Frontmatter, serialize

    if rel_path is None:
        rel_path = _episode_rel_path(episode.task)

    fm = Frontmatter(
        id=f"EP-{hashlib.sha256(episode.task.encode('utf-8')).hexdigest()[:10]}",
        kind="episode",
        ts_ns=time.time_ns(),
        actor=actor,
        tags=["episode", f"outcome:{episode.outcome}"],
        extra=episode.extra_fields(),
    )
    body_text = episode.searchable_document() + "\n"
    file_bytes = serialize(fm, body_text.encode("utf-8"))

    seq = put(
        writer,
        rel_path,
        file_bytes,
        actor=actor,
        kind="episode",
        extra={
            # surface the closed-enum outcome on the AuditRecord so FR-MEMORY-115
            # dream's patterns detector can scan rows without re-parsing bodies
            "episode_outcome": episode.outcome,
            "episode_duration_ms": episode.duration_ms,
        },
    )
    return seq, rel_path


def recall_similar(
    store: Path,
    task: str,
    *,
    k: int = 3,
    min_relevance: float = 0.65,
    backend: str = "auto",  # "auto" | "semantic" | "fts5"
) -> dict:
    """Find episodes similar to ``task``.

    Returns a dict::

        {
          "backend": "semantic" | "fts5",
          "matches": [
            {"path", "task", "approach", "outcome", "quality_score",
             "relevance", "recency", "combined_score", "last_seen_at"}, ...
          ],
          "reason": None | "no_episodes_in_store" | "no_episodes_above_min_relevance",
        }

    The ranking weights default to Park-et-al 0.4/0.3/0.3 per FR-MEMORY-113;
    when the ranking module is not yet wired (FR-MEMORY-112 standalone),
    `recency` is the placeholder constant 1.0 and the combined score
    degrades gracefully.
    """
    # Resolve backend
    if backend == "auto":
        try:
            from cyberos.core import semantic
            backend = "semantic" if semantic.available() else "fts5"
        except Exception:
            backend = "fts5"

    # Try the ranking-aware combined score first (FR-MEMORY-113); fall back
    # to FR-MEMORY-112 placeholder if ranking module unavailable.
    try:
        from cyberos.core.ranking import RecallWeights, score_hits
        from cyberos.core.decay import Exponential
        weights = RecallWeights()
        decay = Exponential()
        ranking_active = True
    except Exception:
        weights = None
        decay = None
        ranking_active = False

    hits: list[dict] = _list_episodes(store, query=task, backend=backend, k=k * 4)
    if not hits:
        # Distinguish "store has 0 episodes" from "store has episodes but
        # none match the query". The former is `no_episodes_in_store`; the
        # latter is `no_episodes_above_min_relevance` (per FR-MEMORY-112 §1 #16).
        episodes_dir = store / "memories" / "episodes"
        any_episodes = episodes_dir.is_dir() and any(episodes_dir.rglob("*.md"))
        reason = (
            "no_episodes_above_min_relevance" if any_episodes else "no_episodes_in_store"
        )
        return {"backend": backend, "matches": [], "reason": reason}

    # Filter by min_relevance
    hits = [h for h in hits if h["relevance"] >= min_relevance]
    if not hits:
        return {
            "backend": backend,
            "matches": [],
            "reason": "no_episodes_above_min_relevance",
        }

    enriched = []
    for h in hits:
        qs = h.get("quality_score")
        if ranking_active:
            qs_use = float(qs) if qs is not None else 0.5
            # Compute recency from last_seen_at if present (else 1.0)
            recency_val = 1.0
            last_seen_iso = h.get("last_seen_at")
            if last_seen_iso:
                try:
                    from datetime import datetime, timezone
                    last_seen = datetime.fromisoformat(last_seen_iso.replace("Z", "+00:00"))
                    now = datetime.now(timezone.utc)
                    hours_old = max(0.0, (now - last_seen).total_seconds() / 3600.0)
                    recency_val = decay(hours_old)
                except Exception:
                    recency_val = 1.0
            combined = (
                h["relevance"] * weights.relevance
                + qs_use * weights.importance
                + recency_val * weights.recency
            )
        else:
            qs_use = float(qs) if qs is not None else 0.5
            recency_val = 1.0
            combined = h["relevance"] * 0.4 + qs_use * 0.3 + recency_val * 0.3
        enriched.append({
            **h,
            "recency": round(recency_val, 4),
            "combined_score": round(combined, 4),
        })

    enriched.sort(key=lambda x: x["combined_score"], reverse=True)
    return {"backend": backend, "matches": enriched[:k], "reason": None}


def _list_episodes(store: Path, *, query: str, backend: str, k: int) -> list[dict]:
    """Return raw hits (relevance + frontmatter extras) for episodes matching `query`.

    Backend dispatch:

    * ``"semantic"`` — uses ``cyberos.core.semantic.search`` (when sentence-
      transformers installed). Results are filtered to ``kind="episode"``.
    * ``"fts5"`` — uses the FTS5 index built by ``cyberos search`` (lexical).
    * Both fall back to scanning ``memories/episodes/`` on disk when the
      index isn't available, with a heuristic relevance based on token
      overlap.
    """
    # Strategy: scan disk + heuristic. This is the slice-3 implementation;
    # subsequent FRs (FR-MEMORY-115 dream pipeline) will hook into the FTS5
    # and semantic indices directly for better latency at scale.
    episodes_dir = store / "memories" / "episodes"
    if not episodes_dir.is_dir():
        return []

    from cyberos.core.frontmatter import parse, parse_legacy_yaml, looks_like_yaml
    import re as _re

    query_tokens = set(_re.findall(r"[A-Za-z0-9]+", query.lower()))
    if not query_tokens:
        return []

    hits: list[dict] = []
    for md_path in episodes_dir.rglob("*.md"):
        try:
            raw = md_path.read_bytes()
            try:
                fm, body = parse(raw)
            except Exception:
                if looks_like_yaml(raw):
                    fm, body = parse_legacy_yaml(raw)
                else:
                    continue
            if fm.kind != "episode":
                continue
            doc_text = body.decode("utf-8", errors="ignore").lower()
            doc_tokens = set(_re.findall(r"[A-Za-z0-9]+", doc_text))
            if not doc_tokens:
                continue
            overlap = len(query_tokens & doc_tokens)
            relevance = overlap / max(1, len(query_tokens))
            if relevance == 0.0:
                continue
            extras = dict(fm.extra or {})
            rel_path = str(md_path.relative_to(store))
            hits.append({
                "path": rel_path,
                "task": extras.get("task", ""),
                "approach": extras.get("approach", ""),
                "outcome": extras.get("outcome", ""),
                "quality_score": extras.get("quality_score"),
                "relevance": round(float(relevance), 4),
                "last_seen_at": None,  # FR-MEMORY-120 wires this from audit chain
            })
        except Exception:
            continue
    hits.sort(key=lambda h: h["relevance"], reverse=True)
    return hits[:k]
