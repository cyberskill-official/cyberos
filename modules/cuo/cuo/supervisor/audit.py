"""Canonical CUO routing audit row builder."""

from __future__ import annotations

import hashlib
import re
import time
from typing import Any

from .state import Candidate, InvocationResult, LlmRoutingPick, PathTaken


_EMAIL_RE = re.compile(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b")
_PHONE_RE = re.compile(r"(?<!\d)(?:\+?\d[\d .-]{7,}\d)(?!\d)")


def hash16(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()[:16]


def scrub_query(query: str) -> str:
    query = _EMAIL_RE.sub("[redacted-email]", query)
    return _PHONE_RE.sub("[redacted-phone]", query)


class MemoryAuditSink:
    """Small injectable sink; production can replace with the memory writer."""

    def __init__(self) -> None:
        self.rows: list[dict[str, Any]] = []

    def emit(self, row: dict[str, Any]) -> None:
        self.rows.append(row)


def build_routing_decision_row(
    *,
    tenant_id: str,
    subject_id: str,
    persona_key: str,
    persona_version: str,
    agent_persona_jwt_iss: str,
    query: str,
    rule_scores: list[Candidate],
    path_taken: PathTaken,
    llm_pick: LlmRoutingPick | None,
    invocation_result: InvocationResult | None,
    request_id: str,
    trace_id: str,
    ts_ns_start: int,
    cuo_state_v: int,
) -> dict[str, Any]:
    ts_ns_end = time.time_ns()
    return {
        "row_kind": "cuo.routing_decision",
        "tenant_id": tenant_id,
        "subject_id_hash16": hash16(subject_id),
        "persona_key": persona_key,
        "persona_version": persona_version,
        "agent_persona_jwt_iss": agent_persona_jwt_iss,
        "query": scrub_query(query),
        "rule_scores": [c.model_dump(mode="json") for c in rule_scores[:3]],
        "path_taken": path_taken,
        "llm_pick": llm_pick.model_dump(mode="json") if llm_pick else None,
        "invocation_result": invocation_result.model_dump(mode="json") if invocation_result else None,
        "cuo_state_v": cuo_state_v,
        "request_id": request_id,
        "trace_id": trace_id,
        "ts_ns_start": ts_ns_start,
        "ts_ns_end": ts_ns_end,
        "next_step": None,
    }


def build_aux_row(kind: str, **payload: Any) -> dict[str, Any]:
    row = {"row_kind": kind, "ts_ns": time.time_ns()}
    row.update(payload)
    return row
