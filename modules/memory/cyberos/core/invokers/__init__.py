"""
cyberos.core.invokers — pluggable LLM invokers for write-time scoring
(FR-MEMORY-114 §1 #3, §1 #5).

Mirrors the ``modules/cuo/cuo/invokers.py`` Phase-3 pattern:

* :class:`MockInvoker` — deterministic, offline. Default for tests/dev.
* :class:`AnthropicInvoker` — calls the real Anthropic API (Haiku by
  default). Gracefully fails if `anthropic` package or `ANTHROPIC_API_KEY`
  is unavailable.
* :func:`select_invoker` — env / manifest / default selection chain with
  ``CYBEROS_DISABLE_LLM=1`` escape hatch.
"""

from cyberos.core.invokers.base import ImportanceInvoker, ScoreResult
from cyberos.core.invokers.mock import MockInvoker

__all__ = ["ImportanceInvoker", "ScoreResult", "MockInvoker"]
