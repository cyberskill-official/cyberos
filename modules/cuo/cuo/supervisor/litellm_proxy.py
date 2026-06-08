"""LiteLLM-shaped client that forwards through AI Gateway FR-AI-008 only."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

import httpx


GatewayTransport = Callable[[dict[str, Any], dict[str, str], float], dict[str, Any]]


class GatewayError(RuntimeError):
    pass


@dataclass
class LiteLLMProxy:
    """A minimal LiteLLM-like interface backed by the AI Gateway.

    No provider SDK imports are allowed in this module. Tests AST-walk the
    package for direct imports of boto3, anthropic, and openai.
    """

    ai_gateway_url: str = "http://127.0.0.1:8088"
    timeout_s: float = 3.0
    transport: GatewayTransport | None = None

    def completion(self, *, messages: list[dict[str, str]], request_id: str) -> dict[str, Any]:
        payload = {
            "model_alias": "chat.smart",
            "messages": messages,
            "response_format": {"type": "json_object"},
            "metadata": {"cuo_request_id": request_id},
        }
        headers = {
            "Content-Type": "application/json",
            "X-CUO-Decision-Id": request_id,
        }
        if self.transport is not None:
            return self.transport(payload, headers, self.timeout_s)
        try:
            with httpx.Client(timeout=self.timeout_s) as client:
                resp = client.post(f"{self.ai_gateway_url.rstrip('/')}/v1/ai/chat", json=payload, headers=headers)
                resp.raise_for_status()
                return resp.json()
        except httpx.TimeoutException as exc:
            raise TimeoutError("cuo llm cascade timeout") from exc
        except httpx.HTTPError as exc:
            raise GatewayError(str(exc)) from exc
