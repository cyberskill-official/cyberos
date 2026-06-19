"""Blend scorers ~60/30/10 deterministic / LLM-judge / human.

The production consensus from the survey: ~60% deterministic (exact match, regex,
JSON-schema, latency), ~30% LLM-judge, ~10% human-in-the-loop. Never rely on the
LLM-judge alone — it stacks scorer-side stochasticity on top of agent
stochasticity. Deterministic scorers anchor the score; the judge handles the
fuzzy bits; the human catches what automation misses (e.g. a systematic bias
toward SEO content farms over authoritative sources).

Weights renormalize over whichever components are present, so a run with no human
score still blends deterministic+judge in the right 2:1 proportion.
"""

from __future__ import annotations

import math
from collections.abc import Sequence
from dataclasses import dataclass

__all__ = ["DEFAULT_WEIGHTS", "ScoreBreakdown", "blend", "aggregate_deterministic"]

DEFAULT_WEIGHTS = {"deterministic": 0.6, "judge": 0.3, "human": 0.1}


def _clamp(x: float) -> float:
    # A non-finite score (NaN/inf from a broken upstream scorer) must fail toward 0,
    # NOT 1.0 — `max(0, min(1, nan))` evaluates to 1.0 and would silently turn a
    # broken scorer into a perfect pass. Treat it as the worst score instead.
    x = float(x)
    if not math.isfinite(x):
        return 0.0
    return max(0.0, min(1.0, x))


@dataclass
class ScoreBreakdown:
    blended: float
    components: dict[str, float]
    weights_used: dict[str, float]
    threshold: float = 0.7

    @property
    def passed(self) -> bool:
        return self.blended >= self.threshold

    def to_dict(self) -> dict:
        return {
            "blended": round(self.blended, 6),
            "passed": self.passed,
            "components": {k: round(v, 6) for k, v in self.components.items()},
            "weights_used": {k: round(v, 6) for k, v in self.weights_used.items()},
            "threshold": self.threshold,
        }


def aggregate_deterministic(scores: Sequence[float]) -> float:
    """Average a set of deterministic scorer outputs (each in [0,1]); empty -> 0.0."""
    vals = [_clamp(s) for s in scores]
    return sum(vals) / len(vals) if vals else 0.0


def blend(
    deterministic: float | Sequence[float] | None,
    judge: float | None = None,
    human: float | None = None,
    weights: dict[str, float] | None = None,
    threshold: float = 0.7,
) -> ScoreBreakdown:
    """Blend present components with renormalized weights.

    Args:
        deterministic: a single [0,1] score or a sequence to average (or None to omit).
        judge: LLM-judge overall score in [0,1] (or None).
        human: human score in [0,1] (or None).
        weights: override DEFAULT_WEIGHTS.
        threshold: pass bar for ``.passed``.

    Raises:
        ValueError: if no component is provided (a blend of nothing is meaningless).
    """
    w = weights or DEFAULT_WEIGHTS
    if any(v < 0 for v in w.values()):
        raise ValueError("blend() weights must be non-negative")
    components: dict[str, float] = {}
    if deterministic is not None:
        det = (
            aggregate_deterministic(deterministic)
            if isinstance(deterministic, Sequence) and not isinstance(deterministic, str | bytes)
            else _clamp(deterministic)
        )
        components["deterministic"] = det
    if judge is not None:
        components["judge"] = _clamp(judge)
    if human is not None:
        components["human"] = _clamp(human)

    if not components:
        raise ValueError("blend() needs at least one of deterministic/judge/human")

    present_w = {k: w.get(k, 0.0) for k in components}
    wsum = sum(present_w.values())
    if wsum <= 0:
        # all present components have zero weight -> fall back to a plain mean
        present_w = dict.fromkeys(components, 1.0)
        wsum = float(len(components))

    blended = sum(present_w[k] * components[k] for k in components) / wsum
    used = {k: present_w[k] / wsum for k in components}
    return ScoreBreakdown(
        blended=blended, components=components, weights_used=used, threshold=threshold
    )
