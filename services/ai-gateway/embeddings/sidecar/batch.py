"""Batch request/response models for the BGE-M3 sidecar."""

from __future__ import annotations

from typing import Literal

from pydantic import BaseModel, Field, model_validator

MAX_TOKENS = 8192


class EmbedRequest(BaseModel):
    texts: list[str] = Field(min_length=1, max_length=32)
    tenant_id: str
    tenant_ids: list[str] | None = None
    task: Literal["passage", "code"] = "passage"

    @model_validator(mode="after")
    def tenant_ids_match_texts(self) -> "EmbedRequest":
        if self.tenant_ids is not None and len(self.tenant_ids) != len(self.texts):
            raise ValueError("tenant_ids length must match texts length")
        return self


class EmbedResponse(BaseModel):
    embeddings: list[list[float]]
    model_name: str
    model_sha256: str
    sidecar_version: str
    device: Literal["cuda", "cpu"]
    elapsed_ms: int


def apply_task_prefix(text: str, task: str) -> str:
    if task == "code":
        return f"Code: {text}"
    return text


def rough_token_count(text: str) -> int:
    return max(1, len(text.split()))
