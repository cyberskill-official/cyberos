"""
cyberos.core.invokers.base — ImportanceInvoker Protocol + ScoreResult.

Pure Protocol; concrete implementations live alongside (MockInvoker,
AnthropicInvoker). Slice-4+ can add OpenAI / Ollama / local-llama
implementations without changing the call sites.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Literal, Optional, Protocol, runtime_checkable


@dataclass(frozen=True)
class ScoreResult:
    """The typed result returned by every ImportanceInvoker call.

    Attributes
    ----------
    score
        Float in ``[0.0, 1.0]``. On fallback paths this is always 0.5
        (the neutral midpoint, per FR-MEMORY-114 §1 #9).
    latency_ms
        Wall-clock duration of the invocation (or 0 on cache hits).
    model
        Free-form identifier — ``"mock"``, ``"claude-haiku-4-5"``, ``"cache"``.
    outcome
        ``"ok"`` on success; ``"fallback"`` when the call failed and a
        neutral score was substituted.
    reason
        Populated only when ``outcome == "fallback"``. Carries a short
        structured tag (``"timeout"``, ``"parse_error:..."``,
        ``"api_error:..."``, etc.) for operator diagnostics.
    """

    score: float
    latency_ms: int
    model: str
    outcome: Literal["ok", "fallback"]
    reason: Optional[str] = None


@runtime_checkable
class ImportanceInvoker(Protocol):
    """Async callable returning a :class:`ScoreResult` for the given content.

    Implementations MUST clamp output to ``[0.0, 1.0]`` and SHOULD return
    a fallback (score=0.5, outcome="fallback") rather than raising on
    timeout / parse / API errors.
    """

    async def score(self, content: str) -> ScoreResult: ...
