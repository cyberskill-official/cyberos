"""Record node."""

from __future__ import annotations

from ..audit import MemoryAuditSink, build_routing_decision_row
from ..state import Candidate, InvocationResult, LlmRoutingPick, PathTaken


def record_decision(
    *,
    sink: MemoryAuditSink,
    record: bool,
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
) -> dict | None:
    if not record:
        return None
    row = build_routing_decision_row(
        tenant_id=tenant_id,
        subject_id=subject_id,
        persona_key=persona_key,
        persona_version=persona_version,
        agent_persona_jwt_iss=agent_persona_jwt_iss,
        query=query,
        rule_scores=rule_scores,
        path_taken=path_taken,
        llm_pick=llm_pick,
        invocation_result=invocation_result,
        request_id=request_id,
        trace_id=trace_id,
        ts_ns_start=ts_ns_start,
        cuo_state_v=cuo_state_v,
    )
    sink.emit(row)
    return row
