"""
cyberos.core.invokers.anthropic_invoker — calls the real Anthropic API.

Uses the verbatim Ramakrushna prompt (per FR-MEMORY-114 §1 #11). Falls
back to ``score=0.5, outcome="fallback"`` on any error — timeout,
rate-limit, parse failure, transport error — preserving FR-MEMORY-114's
"the write proceeds with neutral score" guarantee.

Constructor raises (rather than fallbacking) when the prerequisites are
missing entirely (no ``anthropic`` package, no ``ANTHROPIC_API_KEY``).
This is the fail-fast path — operator gets a clear error at the seam
rather than every call silently fallbacking.
"""

from __future__ import annotations

import asyncio
import os
import re
import time

from cyberos.core.invokers.base import ScoreResult

SYSTEM_PROMPT = """Rate the importance of saving this for future interactions.
0.0 = trivial (greeting)
0.5 = moderately useful
1.0 = critical (preferences, errors, decisions)

Information: {content}
Reply with ONLY the number."""

_FLOAT_RE = re.compile(r"[-+]?\d*\.\d+|\d+")


class AnthropicInvoker:
    """Real Anthropic API caller. Reuses CUO Phase-3 graceful-import pattern."""

    def __init__(
        self,
        model: str = "claude-haiku-4-5",
        timeout_s: float = 5.0,
    ) -> None:
        # Lazy-import: only crash if the operator explicitly selected this
        # invoker. MockInvoker callers never pay this cost.
        try:
            import anthropic  # noqa: F401
        except ImportError as e:
            raise RuntimeError(
                "AnthropicInvoker selected but the `anthropic` package is not "
                "installed. Run `pip install anthropic --break-system-packages` "
                "or set CYBEROS_IMPORTANCE_INVOKER=mock (or CYBEROS_DISABLE_LLM=1)."
            ) from e
        if not os.environ.get("ANTHROPIC_API_KEY"):
            raise RuntimeError(
                "AnthropicInvoker selected but ANTHROPIC_API_KEY is unset. "
                "Export the key or set CYBEROS_DISABLE_LLM=1 to force "
                "MockInvoker."
            )
        self._model = model
        self._timeout_s = timeout_s

    async def score(self, content: str) -> ScoreResult:
        import anthropic  # noqa: WPS433 — lazy

        client = anthropic.AsyncAnthropic()
        t0 = time.perf_counter()
        try:
            resp = await asyncio.wait_for(
                client.messages.create(
                    model=self._model,
                    max_tokens=8,
                    system=SYSTEM_PROMPT.format(content=content),
                    messages=[{"role": "user", "content": "Rate this."}],
                ),
                timeout=self._timeout_s,
            )
        except asyncio.TimeoutError:
            return ScoreResult(
                score=0.5,
                latency_ms=int(self._timeout_s * 1000),
                model=self._model,
                outcome="fallback",
                reason="timeout",
            )
        except Exception as e:  # noqa: BLE001 — fail-closed on any API error
            return ScoreResult(
                score=0.5,
                latency_ms=int((time.perf_counter() - t0) * 1000),
                model=self._model,
                outcome="fallback",
                reason=f"api_error:{type(e).__name__}",
            )

        text = resp.content[0].text.strip() if resp.content else ""
        m = _FLOAT_RE.search(text)
        if not m:
            return ScoreResult(
                score=0.5,
                latency_ms=int((time.perf_counter() - t0) * 1000),
                model=self._model,
                outcome="fallback",
                reason=f"parse_error:{text!r}",
            )
        try:
            v = float(m.group())
        except ValueError:
            return ScoreResult(
                score=0.5,
                latency_ms=int((time.perf_counter() - t0) * 1000),
                model=self._model,
                outcome="fallback",
                reason=f"parse_error:{m.group()!r}",
            )
        clamped = max(0.0, min(1.0, v))
        return ScoreResult(
            score=clamped,
            latency_ms=int((time.perf_counter() - t0) * 1000),
            model=self._model,
            outcome="ok",
            reason=None,
        )
