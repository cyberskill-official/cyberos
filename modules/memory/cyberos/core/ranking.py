"""
cyberos.core.ranking — Park-et-al combined-score recall ranking
(TASK-MEMORY-113 §1 #1, #7, #10).

Pure-function module (no I/O, no ``datetime.now()`` inside ``score_hits()``)
so two callers — live recall paths and TASK-MEMORY-115's batch dream pipeline
— can share the same scoring engine deterministically.

The combined score is the Park-et-al ("Generative Agents", 2023) form::

    combined_score = relevance · w_r + importance · w_i + recency · w_t

Defaults: ``w_r=0.4, w_i=0.3, w_t=0.3``. Weights MUST sum to 1.0 ±1e-6
(constructor-enforced + walker-enforced via ``manifest-recall-weights-
sum-to-one`` invariant once TASK-MEMORY-113 wires the manifest validation).

Absent ``importance`` on a hit's frontmatter is treated as 0.5 (the
neutral midpoint, per DEC-181 / DEC-192). Absent ``last_seen_at`` →
recency=1.0 (treat as fresh, per TASK-MEMORY-113 §1 #6).
"""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import datetime, timezone
from typing import Iterable, Optional

from cyberos.core.decay import DecayProfile, Exponential, hours_between


@dataclass(frozen=True)
class RecallWeights:
    """The (w_r, w_i, w_t) triple used in the combined-score formula.

    Constructor enforces:
    * Each weight in ``[0.0, 1.0]``
    * Sum of weights == 1.0 ±1e-6
    """

    relevance: float = 0.4
    importance: float = 0.3
    recency: float = 0.3

    def __post_init__(self) -> None:
        for name, v in (
            ("relevance", self.relevance),
            ("importance", self.importance),
            ("recency", self.recency),
        ):
            if not isinstance(v, (int, float)):
                raise ValueError(f"{name} must be a number; got {type(v).__name__}")
            if not (0.0 <= float(v) <= 1.0):
                raise ValueError(f"{name} must be in [0.0, 1.0]; got {v}")
        total = float(self.relevance) + float(self.importance) + float(self.recency)
        if abs(total - 1.0) > 1e-6:
            raise ValueError(
                f"weights must sum to 1.0 ±1e-6; got {total} "
                f"(relevance={self.relevance}, importance={self.importance}, "
                f"recency={self.recency})"
            )

    @classmethod
    def relevance_only(cls) -> "RecallWeights":
        """For internal tools that need raw similarity order."""
        return cls(relevance=1.0, importance=0.0, recency=0.0)


@dataclass
class ScoredHit:
    """A single ranked hit with the four scalars annotated.

    Downstream CLIs / REST endpoints / TASK-MEMORY-115 dream all rely on
    these annotations to explain why a hit ranked where it did
    (TASK-MEMORY-113 §1 #8).
    """

    path: str
    relevance: float
    importance: float
    recency: float
    combined_score: float
    frontmatter: dict
    last_seen_at: Optional[datetime] = None
    body_text: str = ""


def _coerce_to_dt(value) -> Optional[datetime]:
    if value is None:
        return None
    if isinstance(value, datetime):
        return value
    if isinstance(value, str):
        try:
            return datetime.fromisoformat(value.replace("Z", "+00:00"))
        except ValueError:
            return None
    return None


def score_hits(
    hits: Iterable,
    weights: RecallWeights,
    decay: DecayProfile,
    *,
    now: Optional[datetime] = None,
) -> list[ScoredHit]:
    """Score and sort hits by combined score.

    Pure function. ``now`` is injected so tests can run deterministically
    against a snapshot timestamp; downstream callers should pass
    ``datetime.now(timezone.utc)`` explicitly.

    ``hits`` may be any iterable of objects with these attributes / keys
    (duck-typed):

    * ``path`` (str)
    * ``relevance`` (float)
    * ``frontmatter`` (dict) — looked up for ``importance``
    * ``last_seen_at`` (datetime or ISO string or None)
    * ``body_text`` (str, optional)

    Returns a list of :class:`ScoredHit`, sorted by ``combined_score``
    descending. Stable sort preserves insertion order on ties.
    """
    if now is None:
        now = datetime.now(timezone.utc)
    out: list[ScoredHit] = []
    for h in hits:
        # Duck-typed accessor for object-or-dict hits
        def _get(obj, key, default=None):
            if isinstance(obj, dict):
                return obj.get(key, default)
            return getattr(obj, key, default)

        rel = float(_get(h, "relevance", 0.0))
        fm = _get(h, "frontmatter", {}) or {}
        importance_raw = fm.get("importance", 0.5)
        try:
            imp = float(importance_raw) if importance_raw is not None else 0.5
        except (TypeError, ValueError):
            imp = 0.5
        last_seen = _coerce_to_dt(_get(h, "last_seen_at"))
        delta = hours_between(now, last_seen)
        rec = decay(delta) if delta is not None else 1.0
        combined = rel * weights.relevance + imp * weights.importance + rec * weights.recency
        out.append(ScoredHit(
            path=str(_get(h, "path", "")),
            relevance=rel,
            importance=imp,
            recency=rec,
            combined_score=combined,
            frontmatter=fm,
            last_seen_at=last_seen,
            body_text=str(_get(h, "body_text", "")),
        ))
    out.sort(key=lambda s: s.combined_score, reverse=True)
    return out
