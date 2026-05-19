"""Tests for FR-MEMORY-114 — write-time importance scoring.

Covers acceptance criteria from
`docs/feature-requests/memory/FR-MEMORY-114-write-time-importance.md`:

* AC #4 — CLI flag wins (explicit invoker name beats default)
* AC #5 — env wins over default
* AC #7 — CYBEROS_DISABLE_LLM=1 forces mock
* AC #8 — default when no API key → MockInvoker
* AC #9 — MockInvoker deterministic
* AC #10 — ScoreResult shape
* AC #11 — cache hit returns same score, no invoker call
* AC #12 — cache invalidates on content change
* AC #13 — cache survives Writer restart
* AC #15 — `memory.importance_scored` aux row emitted via aux_emitter
* AC #16 — aux row cache_hit field accurate
* AC #19 — fallback on missing API key (AnthropicInvoker constructor)
* AC #21 — verbatim Ramakrushna prompt
* AC #22 — timeout default = 5 s
"""

from __future__ import annotations

import asyncio
import hashlib
from pathlib import Path

import pytest

from cyberos.core.importance import (
    ImportanceCache,
    score as score_async,
    score_sync,
    select_invoker,
)
from cyberos.core.invokers.base import ScoreResult
from cyberos.core.invokers.mock import MockInvoker


# ---- MockInvoker -----------------------------------------------------------


def test_mock_invoker_deterministic() -> None:
    """AC #9 — same content → same score on two calls."""
    inv = MockInvoker()
    r1 = asyncio.run(inv.score("hello world"))
    r2 = asyncio.run(inv.score("hello world"))
    assert r1.score == r2.score


def test_mock_invoker_clamped_range() -> None:
    """MockInvoker should never produce literal 0.0 or 1.0 (would conflate
    with fallback / pinned paths)."""
    inv = MockInvoker()
    for s in ("a", "b", "longer content here", "even longer content with many words"):
        r = asyncio.run(inv.score(s))
        assert 0.1 <= r.score <= 0.95, f"score {r.score} for {s!r} outside [0.1, 0.95]"
        assert r.outcome == "ok"
        assert r.model == "mock"


def test_score_result_shape() -> None:
    """AC #10 — ScoreResult has the five typed fields."""
    inv = MockInvoker()
    r = asyncio.run(inv.score("x"))
    for field in ("score", "latency_ms", "model", "outcome", "reason"):
        assert hasattr(r, field)


# ---- Invoker selection chain -----------------------------------------------


def test_disable_llm_forces_mock(monkeypatch) -> None:
    """AC #7 — CYBEROS_DISABLE_LLM=1 overrides everything."""
    monkeypatch.setenv("CYBEROS_DISABLE_LLM", "1")
    monkeypatch.setenv("CYBEROS_IMPORTANCE_INVOKER", "anthropic")
    inv = select_invoker("anthropic")
    assert inv.__class__.__name__ == "MockInvoker"


def test_explicit_name_beats_env(monkeypatch) -> None:
    """AC #4 — explicit ``name`` argument wins over env."""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.delenv("CYBEROS_IMPORTANCE_INVOKER", raising=False)
    monkeypatch.setenv("ANTHROPIC_API_KEY", "fake-test-key")
    # Force mock even though anthropic would be the default
    inv = select_invoker("mock")
    assert inv.__class__.__name__ == "MockInvoker"


def test_env_wins_when_no_explicit_name(monkeypatch) -> None:
    """AC #5 — CYBEROS_IMPORTANCE_INVOKER env beats default."""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.setenv("CYBEROS_IMPORTANCE_INVOKER", "mock")
    monkeypatch.setenv("ANTHROPIC_API_KEY", "fake-test-key")
    inv = select_invoker()
    assert inv.__class__.__name__ == "MockInvoker"


def test_default_without_api_key(monkeypatch) -> None:
    """AC #8 — no env, no flag, no key → MockInvoker."""
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.delenv("CYBEROS_IMPORTANCE_INVOKER", raising=False)
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    inv = select_invoker()
    assert inv.__class__.__name__ == "MockInvoker"


def test_unknown_invoker_raises() -> None:
    with pytest.raises(ValueError, match="unknown invoker"):
        select_invoker("made_up")


def test_anthropic_missing_api_key_raises(monkeypatch) -> None:
    """AC #19 — AnthropicInvoker constructor refuses without key."""
    # Skip when the optional `anthropic` package isn't installed — the
    # constructor would raise a "package not installed" error before
    # reaching the API-key check, masking what we're trying to verify.
    pytest.importorskip("anthropic")
    monkeypatch.delenv("CYBEROS_DISABLE_LLM", raising=False)
    monkeypatch.delenv("ANTHROPIC_API_KEY", raising=False)
    with pytest.raises(RuntimeError, match="ANTHROPIC_API_KEY"):
        select_invoker("anthropic")


# ---- Verbatim prompt + timeout default -------------------------------------


def test_verbatim_ramakrushna_prompt() -> None:
    """AC #21 — the AnthropicInvoker system prompt is the article's verbatim."""
    from cyberos.core.invokers.anthropic_invoker import SYSTEM_PROMPT
    expected_anchor = (
        "Rate the importance of saving this for future interactions.\n"
        "0.0 = trivial (greeting)\n"
        "0.5 = moderately useful\n"
        "1.0 = critical (preferences, errors, decisions)"
    )
    assert expected_anchor in SYSTEM_PROMPT
    assert "Reply with ONLY the number." in SYSTEM_PROMPT


def test_anthropic_default_timeout(monkeypatch) -> None:
    """AC #22 — default timeout is 5 seconds."""
    monkeypatch.setenv("ANTHROPIC_API_KEY", "fake-test-key")
    try:
        from cyberos.core.invokers.anthropic_invoker import AnthropicInvoker
        inv = AnthropicInvoker()
    except RuntimeError as e:
        # If anthropic package isn't installed in this test env, skip rather
        # than fail — the timeout default is still a hard-coded constant.
        if "anthropic" in str(e):
            pytest.skip("anthropic package not installed in test env")
        raise
    assert inv._timeout_s == 5.0


# ---- ImportanceCache -------------------------------------------------------


def test_cache_hit_returns_cached(tmp_path: Path) -> None:
    """AC #11 — second call with same content is a cache hit."""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    r1 = score_sync("hello", inv, cache)
    r2 = score_sync("hello", inv, cache)
    assert r1.score == r2.score
    assert r2.model == "cache"
    assert r2.latency_ms == 0
    cache.close()


def test_cache_invalidates_on_content_change(tmp_path: Path) -> None:
    """AC #12 — different content → cache miss + different score."""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    r1 = score_sync("hello", inv, cache)
    r2 = score_sync("world", inv, cache)
    assert r1.score != r2.score
    assert r2.model == "mock"  # not "cache"
    cache.close()


def test_cache_survives_restart(tmp_path: Path) -> None:
    """AC #13 — close + reopen still hits."""
    db = tmp_path / "cache.db"
    cache = ImportanceCache(db)
    inv = MockInvoker()
    r1 = score_sync("hello", inv, cache)
    cache.close()

    cache2 = ImportanceCache(db)
    r2 = score_sync("hello", inv, cache2)
    assert r2.model == "cache"
    assert r2.score == r1.score
    cache2.close()


def test_cache_sha256_keying(tmp_path: Path) -> None:
    """Underlying invariant — cache key is sha256(content)."""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    asyncio.run(score_async("hello world", inv, cache))
    h = hashlib.sha256(b"hello world").digest()
    assert cache.get(h) is not None
    cache.close()


def test_cache_file_created_lazily(tmp_path: Path) -> None:
    """ImportanceCache creates the parent dir + file on construction."""
    cache = ImportanceCache(tmp_path / "nested" / "deeper" / "cache.db")
    assert (tmp_path / "nested" / "deeper" / "cache.db").exists()
    cache.close()


# ---- aux_emitter (AC #15, #16) --------------------------------------------


def test_aux_emitter_called_on_cache_miss(tmp_path: Path) -> None:
    """AC #15 — aux row emitted per call."""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    rows: list[tuple[str, dict]] = []
    score_sync("x", inv, cache, aux_emitter=lambda k, p: rows.append((k, p)),
               path="memories/facts/x.md")
    assert len(rows) == 1
    kind, payload = rows[0]
    assert kind == "memory.importance_scored"
    for key in ("path", "content_sha256", "score", "model", "outcome",
                "reason", "cache_hit", "latency_ms"):
        assert key in payload
    assert payload["cache_hit"] is False
    cache.close()


def test_aux_emitter_marks_cache_hit(tmp_path: Path) -> None:
    """AC #16 — cache_hit field flips on second call."""
    cache = ImportanceCache(tmp_path / "cache.db")
    inv = MockInvoker()
    rows: list[tuple[str, dict]] = []
    emit = lambda k, p: rows.append((k, p))  # noqa: E731
    score_sync("x", inv, cache, aux_emitter=emit, path="memories/x.md")
    score_sync("x", inv, cache, aux_emitter=emit, path="memories/x.md")
    assert rows[0][1]["cache_hit"] is False
    assert rows[1][1]["cache_hit"] is True
    assert rows[1][1]["model"] == "cache"
    cache.close()


def test_fallback_not_cached(tmp_path: Path) -> None:
    """Fallbacks are NOT cached (per importance.py implementation note)."""
    # Construct a fake invoker that always fallbacks
    class FallbackInvoker:
        async def score(self, content: str) -> ScoreResult:
            return ScoreResult(
                score=0.5, latency_ms=10, model="test",
                outcome="fallback", reason="simulated",
            )

    cache = ImportanceCache(tmp_path / "cache.db")
    inv = FallbackInvoker()
    r1 = score_sync("x", inv, cache)
    r2 = score_sync("x", inv, cache)
    # Both calls hit the invoker (cache doesn't store fallbacks)
    assert r1.outcome == "fallback"
    assert r2.outcome == "fallback"
    assert r2.model != "cache"  # didn't hit the cache
    cache.close()
