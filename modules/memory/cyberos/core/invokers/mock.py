"""
cyberos.core.invokers.mock — deterministic, offline ImportanceInvoker.

Used by tests (CI without API keys) and by `--invoker mock` deployments
that don't have / don't want LLM calls. The mock's score is derived from
``sha256(content)[:8]`` so the same content always produces the same
score — which is exactly what we want for cache-invalidation tests AND
for reproducible dream-pipeline behaviour (FR-MEMORY-115).

Per FR-MEMORY-114 implementation note: the mock score is clamped to
``[0.1, 0.95]`` so it never produces literal 0.0 or 1.0 — those values
would conflate with the fallback / pinned-by-operator paths.
"""

from __future__ import annotations

import hashlib
import time

from cyberos.core.invokers.base import ScoreResult


class MockInvoker:
    """Deterministic, offline. ``score(content)`` is a pure function of
    ``sha256(content)``."""

    async def score(self, content: str) -> ScoreResult:
        t0 = time.perf_counter()
        h = hashlib.sha256(content.encode("utf-8")).hexdigest()
        raw = int(h[:8], 16) / 0xFFFFFFFF
        score = 0.1 + raw * 0.85  # clamp into [0.1, 0.95]
        latency_ms = int((time.perf_counter() - t0) * 1000)
        return ScoreResult(
            score=score, latency_ms=latency_ms, model="mock",
            outcome="ok", reason=None,
        )
