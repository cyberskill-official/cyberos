"""Invoke node."""

from __future__ import annotations

import time
from typing import Callable

from ..persona import get_persona
from ..state import Candidate, InvocationResult

SkillInvoker = Callable[[Candidate], InvocationResult]


def invoke_candidate(
    candidate: Candidate | None,
    *,
    persona_key: str,
    invoke: bool,
    invoker: SkillInvoker | None,
) -> InvocationResult | None:
    if candidate is None:
        return None
    if not invoke:
        return InvocationResult(status="skipped")
    persona = get_persona(persona_key)
    operation = candidate.operation or str(candidate.arguments.get("operation") or "")
    if operation and operation in persona.defer_to_human_matrix:
        return InvocationResult(
            status="blocked",
            stderr=f"persona_defer_matrix:{operation}",
            exit_code=1,
        )
    if invoker is None:
        return InvocationResult(status="not_configured", stderr="no skill invoker configured", exit_code=75)
    t0 = time.monotonic_ns()
    result = invoker(candidate)
    return result.model_copy(update={"duration_ms": (time.monotonic_ns() - t0) // 1_000_000})
