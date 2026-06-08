"""LLM cascade helper."""

from __future__ import annotations

import json
from typing import Any

from pydantic import ValidationError

from ..litellm_proxy import GatewayError, LiteLLMProxy
from ..persona import get_persona
from ..state import Candidate, LlmRoutingPick


def run_llm_cascade(
    *,
    query: str,
    persona_key: str,
    rule_scores: list[Candidate],
    proxy: LiteLLMProxy,
    request_id: str,
) -> tuple[LlmRoutingPick | None, str | None]:
    persona = get_persona(persona_key)
    messages = [
        {"role": "system", "content": persona.system_prompt},
        {
            "role": "user",
            "content": json.dumps(
                {
                    "query": query,
                    "top_candidates": [c.model_dump(mode="json") for c in rule_scores[:5]],
                    "schema": "LlmRoutingPick{skill_name,arguments,rationale,confidence}",
                },
                sort_keys=True,
            ),
        },
    ]
    last_error: str | None = None
    for _ in range(2):
        try:
            raw = proxy.completion(messages=messages, request_id=request_id)
        except TimeoutError:
            return None, "timeout"
        except GatewayError:
            return None, "gateway_error"
        payload: Any = raw
        if "choices" in raw:
            content = raw["choices"][0].get("message", {}).get("content", "{}")
            try:
                payload = json.loads(content)
            except json.JSONDecodeError as exc:
                last_error = f"schema_violation:{exc}"
                continue
        try:
            return LlmRoutingPick.model_validate(payload), None
        except ValidationError as exc:
            last_error = f"schema_violation:{exc.errors()[0]['type']}"
    return None, last_error or "schema_violation"
