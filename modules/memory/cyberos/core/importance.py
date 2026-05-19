"""
cyberos.core.importance — write-time importance scoring orchestrator
(FR-MEMORY-114).

The orchestrator pattern:

1. Resolve the active :class:`ImportanceInvoker` via the priority chain
   (CLI flag → env → manifest → default) — see :func:`select_invoker`.
2. Check the SHA-256-keyed cache. Hit → return the cached score
   instantly. Miss → invoke + populate the cache (on success only;
   fallbacks are NOT cached so subsequent calls get a chance to retry
   if the API recovered).
3. Caller emits a ``memory.importance_scored`` aux audit row (FR-MEMORY-114
   §1 #8) — separate from this module to keep the orchestrator
   write-free.
"""

from __future__ import annotations

import hashlib
import os
import sqlite3
import time
from pathlib import Path
from typing import Optional

from cyberos.core.invokers.base import ImportanceInvoker, ScoreResult
from cyberos.core.invokers.mock import MockInvoker


# --- Invoker selection ----------------------------------------------------


def _default_from_env() -> str:
    """Default invoker when no env / CLI / manifest override is present."""
    return "anthropic" if os.environ.get("ANTHROPIC_API_KEY") else "mock"


def select_invoker(name: Optional[str] = None) -> ImportanceInvoker:
    """Resolve and instantiate the active ImportanceInvoker.

    Priority chain (per FR-MEMORY-114 §1 #3, #4):

    1. ``CYBEROS_DISABLE_LLM=1`` env → ALWAYS MockInvoker (escape hatch).
    2. Explicit ``name`` argument (typically from a CLI flag).
    3. ``CYBEROS_IMPORTANCE_INVOKER`` env.
    4. Default: ``anthropic`` if ``ANTHROPIC_API_KEY`` is set, else ``mock``.
    """
    if os.environ.get("CYBEROS_DISABLE_LLM") == "1":
        return MockInvoker()

    chosen = (
        name
        or os.environ.get("CYBEROS_IMPORTANCE_INVOKER")
        or _default_from_env()
    )

    if chosen == "mock":
        return MockInvoker()
    if chosen == "anthropic":
        # Lazy import — avoid the package presence check unless this
        # invoker is actually selected.
        from cyberos.core.invokers.anthropic_invoker import AnthropicInvoker
        return AnthropicInvoker()
    raise ValueError(
        f"unknown invoker {chosen!r}; expected one of: mock, anthropic"
    )


# --- ImportanceCache ------------------------------------------------------


class ImportanceCache:
    """SQLite-backed score cache keyed on ``sha256(content)``.

    Lives at ``<memory-root>/index/importance_cache.db``. Schema is single-
    table; no migrations. Slice-4 may drop + recreate if the schema
    changes (data is regenerable by re-scoring).
    """

    def __init__(self, db_path: Path) -> None:
        db_path.parent.mkdir(parents=True, exist_ok=True)
        self._con = sqlite3.connect(str(db_path), isolation_level=None)
        self._con.execute("""
            CREATE TABLE IF NOT EXISTS importance_cache (
                content_sha256 BLOB PRIMARY KEY,
                score          REAL NOT NULL,
                model          TEXT NOT NULL,
                scored_at_ns   INTEGER NOT NULL
            )
        """)

    def get(self, content_sha256: bytes) -> Optional[float]:
        cur = self._con.execute(
            "SELECT score FROM importance_cache WHERE content_sha256 = ?",
            (content_sha256,),
        )
        row = cur.fetchone()
        return row[0] if row else None

    def put(self, content_sha256: bytes, score: float, model: str) -> None:
        self._con.execute(
            "INSERT OR REPLACE INTO importance_cache "
            "(content_sha256, score, model, scored_at_ns) VALUES (?, ?, ?, ?)",
            (content_sha256, float(score), str(model), time.time_ns()),
        )

    def close(self) -> None:
        try:
            self._con.close()
        except Exception:
            pass


# --- Orchestrator --------------------------------------------------------


async def score(
    content: str,
    invoker: ImportanceInvoker,
    cache: Optional[ImportanceCache] = None,
    *,
    aux_emitter=None,
    path: str = "",
) -> ScoreResult:
    """Score ``content`` for write-time importance.

    Cache-hit path is synchronous (no invoker call); cache-miss path
    awaits the invoker. The ``aux_emitter``, if provided, is called as
    ``aux_emitter(kind: str, payload: dict)`` (positional) with the
    FR-MEMORY-114 §1 #8 payload
    shape — typically wired to the Writer's audit-row emit so the
    importance scoring leaves an audit trail.

    Returns the :class:`ScoreResult`. Cache hits report
    ``model="cache"`` and ``latency_ms=0``.
    """
    h = hashlib.sha256(content.encode("utf-8")).digest()

    if cache is not None:
        cached = cache.get(h)
        if cached is not None:
            result = ScoreResult(
                score=float(cached), latency_ms=0, model="cache",
                outcome="ok", reason=None,
            )
            if aux_emitter is not None:
                aux_emitter(
                    "memory.importance_scored",
                    {
                        "path": path,
                        "content_sha256": h.hex(),
                        "score": result.score,
                        "model": result.model,
                        "outcome": result.outcome,
                        "reason": result.reason,
                        "cache_hit": True,
                        "latency_ms": result.latency_ms,
                    },
                )
            return result

    result = await invoker.score(content)

    # Cache only successes — fallbacks must retry on next call.
    if cache is not None and result.outcome == "ok":
        cache.put(h, result.score, result.model)

    if aux_emitter is not None:
        aux_emitter(
            "memory.importance_scored",
            {
                "path": path,
                "content_sha256": h.hex(),
                "score": result.score,
                "model": result.model,
                "outcome": result.outcome,
                "reason": result.reason,
                "cache_hit": False,
                "latency_ms": result.latency_ms,
            },
        )
    return result


def score_sync(
    content: str,
    invoker: ImportanceInvoker,
    cache: Optional[ImportanceCache] = None,
    *,
    aux_emitter=None,
    path: str = "",
) -> ScoreResult:
    """Synchronous wrapper for ``score()``.

    Convenience for CLI callers that don't want to bring up an event
    loop. Internally creates one via ``asyncio.run``. If the calling
    context already has a loop running, callers should `await score()`
    directly instead.
    """
    import asyncio
    return asyncio.run(score(
        content, invoker, cache,
        aux_emitter=aux_emitter, path=path,
    ))
