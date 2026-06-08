"""FR-CUO-101 LangGraph-style routing supervisor public API."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .audit import MemoryAuditSink, build_aux_row
from .checkpointer import InMemoryCheckpointer
from .graph import GraphContract, assert_closed_topology, build_graph
from .litellm_proxy import LiteLLMProxy
from .nodes.branch import choose_path
from .nodes.invoke import SkillInvoker, invoke_candidate
from .nodes.llm_cascade import run_llm_cascade
from .nodes.parse import parse_query
from .nodes.record import record_decision
from .nodes.rule_score import rule_score
from .persona import PersonaError
from .state import Candidate, InvocationResult, LlmRoutingPick, RoutingDecision
from .transparency import build_transparency


class Supervisor:
    """Route one natural-language request through the FR-CUO-101 state machine."""

    def __init__(
        self,
        *,
        cuo_root: Path,
        audit_sink: MemoryAuditSink | None = None,
        checkpointer: InMemoryCheckpointer | None = None,
        llm_proxy: LiteLLMProxy | None = None,
        invoker: SkillInvoker | None = None,
    ) -> None:
        self.cuo_root = Path(cuo_root)
        self.audit_sink = audit_sink or MemoryAuditSink()
        self.checkpointer = checkpointer or InMemoryCheckpointer()
        self.llm_proxy = llm_proxy
        self.invoker = invoker
        assert_closed_topology(GraphContract())
        self.graph = build_graph()

    def route(
        self,
        query: str,
        *,
        tenant_id: str = "tenant-0",
        subject_id: str = "subject-0",
        persona_key: str = "genie",
        agent_persona: str | None = None,
        agent_persona_jwt_iss: str = "local-dev",
        invoke: bool = True,
        record: bool = True,
        request_id: str | None = None,
        trace_id: str | None = None,
    ) -> RoutingDecision:
        agent_persona = agent_persona or f"cuo-{persona_key}@1.0.0"
        try:
            state = parse_query(
                query,
                tenant_id=tenant_id,
                subject_id=subject_id,
                persona_key=persona_key,
                agent_persona=agent_persona,
                request_id=request_id,
                trace_id=trace_id,
            )
        except PersonaError as exc:
            rid = request_id or "invalid-request"
            trace = (trace_id or "0" * 32)[:32]
            if record:
                kind = "cuo.persona_unknown" if "unknown persona" in str(exc) else "cuo.persona_mismatch"
                self.audit_sink.emit(build_aux_row(kind, persona_key=persona_key, error=str(exc), request_id=rid))
            transparency = build_transparency(
                chosen=None,
                confidence=0.0,
                alternatives=[],
                path_taken="defer",
                llm_used=False,
            )
            return RoutingDecision(
                routed=False,
                confidence=0.0,
                path_taken="defer",
                reason=str(exc),
                transparency=transparency,
                request_id=rid,
                trace_id=trace,
            )

        self.checkpointer.save(state["request_id"], tenant_id, {"node": "parse", **_serializable_state(state)})
        state["rule_scores"] = rule_score(state["normalized_query"], cuo_root=self.cuo_root, persona_key=persona_key)
        self.checkpointer.save(state["request_id"], tenant_id, {"node": "rule_score", **_serializable_state(state)})

        path_taken, chosen = choose_path(state["rule_scores"], cascade_taken=False)
        llm_pick: LlmRoutingPick | None = None
        llm_used = False
        if path_taken == "cascade":
            llm_used = True
            if self.llm_proxy is None:
                state["errors"].append("gateway_error:no_llm_proxy")
            else:
                llm_pick, err = run_llm_cascade(
                    query=state["normalized_query"],
                    persona_key=persona_key,
                    rule_scores=state["rule_scores"],
                    proxy=self.llm_proxy,
                    request_id=state["request_id"],
                )
                if err:
                    state["errors"].append(err)
                    if record and err == "timeout":
                        self.audit_sink.emit(build_aux_row("cuo.llm_cascade_timeout", request_id=state["request_id"]))
            state["cascade_taken"] = True
            if llm_pick is not None:
                state["llm_pick"] = llm_pick
            path_taken, chosen = choose_path(
                state["rule_scores"],
                cascade_taken=True,
                llm_pick=llm_pick,
            )

        invocation_result = None
        if path_taken in ("auto", "cascade_then_auto"):
            invocation_result = invoke_candidate(
                chosen,
                persona_key=persona_key,
                invoke=invoke,
                invoker=self.invoker,
            )
            if invocation_result and invocation_result.status == "blocked":
                path_taken = "ask"
                state["errors"].append(invocation_result.stderr)
                if record:
                    self.audit_sink.emit(
                        build_aux_row(
                            "cuo.persona_defer_block",
                            persona_key=persona_key,
                            operation=(chosen.operation if chosen else None),
                            request_id=state["request_id"],
                        )
                    )

        self.checkpointer.save(state["request_id"], tenant_id, {"node": "branch", **_serializable_state(state), "path_taken": path_taken})
        record_decision(
            sink=self.audit_sink,
            record=record,
            tenant_id=tenant_id,
            subject_id=subject_id,
            persona_key=persona_key,
            persona_version=state["persona_version"],
            agent_persona_jwt_iss=agent_persona_jwt_iss,
            query=state["normalized_query"],
            rule_scores=state["rule_scores"],
            path_taken=path_taken,
            llm_pick=llm_pick,
            invocation_result=invocation_result,
            request_id=state["request_id"],
            trace_id=state["trace_id"],
            ts_ns_start=state["ts_ns_start"],
            cuo_state_v=state["cuo_state_v"],
        )
        self.checkpointer.save(state["request_id"], tenant_id, {"node": "record", **_serializable_state(state), "path_taken": path_taken})

        alternatives = _alternatives(state["rule_scores"], chosen)
        routed = path_taken in ("auto", "cascade_then_auto") and not (
            invocation_result and invocation_result.status in {"blocked", "failed", "not_configured"}
        )
        if path_taken in ("ask", "cascade_then_ask", "defer"):
            routed = False
        confidence = chosen.confidence if chosen else 0.0
        transparency = build_transparency(
            chosen=chosen,
            confidence=confidence,
            alternatives=alternatives,
            path_taken=path_taken,
            llm_used=llm_used,
        )
        return RoutingDecision(
            routed=routed,
            skill_name=chosen.skill_name if chosen else None,
            arguments=chosen.arguments if chosen else {},
            confidence=confidence,
            path_taken=path_taken,
            alternatives=alternatives,
            reason=";".join(state["errors"]) or None,
            llm_used=llm_used,
            invocation_result=invocation_result,
            transparency=transparency,
            request_id=state["request_id"],
            trace_id=state["trace_id"],
        )


def run_supervisor(query: str, *, cuo_root: Path, **kwargs: Any) -> RoutingDecision:
    return Supervisor(cuo_root=cuo_root).route(query, **kwargs)


def _alternatives(scores: list[Candidate], chosen: Candidate | None) -> list[Candidate]:
    return [c for c in scores if chosen is None or c.skill_name != chosen.skill_name][:3]


def _serializable_state(state: dict[str, Any]) -> dict[str, Any]:
    out: dict[str, Any] = {}
    for k, v in state.items():
        if k == "rule_scores":
            out[k] = [c.model_dump(mode="json") for c in v]
        elif k == "llm_pick" and v is not None:
            out[k] = v.model_dump(mode="json")
        elif k == "invocation_result" and v is not None:
            out[k] = v.model_dump(mode="json")
        else:
            out[k] = v
    return out


__all__ = [
    "Candidate",
    "InMemoryCheckpointer",
    "InvocationResult",
    "LiteLLMProxy",
    "MemoryAuditSink",
    "RoutingDecision",
    "Supervisor",
    "build_graph",
    "run_supervisor",
]
