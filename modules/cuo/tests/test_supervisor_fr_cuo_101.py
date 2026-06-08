from __future__ import annotations

import ast
from pathlib import Path

from cuo.supervisor import LiteLLMProxy, MemoryAuditSink, Supervisor
from cuo.supervisor.checkpointer import state_version_supported
from cuo.supervisor.graph import GRAPH_NODE_NAMES, GraphContract, assert_closed_topology, build_graph
from cuo.supervisor.persona import validate_agent_persona_claim
from cuo.supervisor.state import Candidate, InvocationResult


CUO_ROOT = Path(__file__).resolve().parent.parent


def test_supervisor_graph_topology_exactly_five_nodes() -> None:
    assert GRAPH_NODE_NAMES == ("parse", "rule_score", "branch", "invoke", "record")
    assert_closed_topology(GraphContract())
    graph = build_graph()
    assert set(graph.nodes) == set(GRAPH_NODE_NAMES)


def test_supervisor_rule_auto_path_records_once() -> None:
    sink = MemoryAuditSink()
    supervisor = Supervisor(cuo_root=CUO_ROOT, audit_sink=sink)
    decision = supervisor.route("ship feature requests", invoke=False, request_id="r1", trace_id="a" * 32)

    assert decision.path_taken == "auto"
    assert decision.routed is True
    assert decision.skill_name == "chief-technology-officer/ship-feature-requests"
    assert decision.transparency.skill_chosen == decision.skill_name
    assert len([r for r in sink.rows if r["row_kind"] == "cuo.routing_decision"]) == 1


def test_supervisor_ask_path(monkeypatch) -> None:
    monkeypatch.setattr(
        "cuo.supervisor.rule_score",
        lambda *a, **k: [
            Candidate(skill_name="chief-product-officer/quarterly-roadmap-planning", confidence=0.61),
            Candidate(skill_name="chief-technology-officer/annual-platform-roadmap", confidence=0.58),
        ],
    )
    decision = Supervisor(cuo_root=CUO_ROOT, audit_sink=MemoryAuditSink()).route(
        "ambiguous roadmap",
        invoke=False,
        record=False,
    )
    assert decision.path_taken == "ask"
    assert decision.routed is False
    assert len(decision.alternatives) == 1


def test_supervisor_defer_path(monkeypatch) -> None:
    monkeypatch.setattr("cuo.supervisor.rule_score", lambda *a, **k: [])
    decision = Supervisor(cuo_root=CUO_ROOT, audit_sink=MemoryAuditSink()).route(
        "zzzzzz",
        invoke=False,
        record=False,
    )
    assert decision.path_taken == "defer"
    assert decision.routed is False
    assert decision.transparency.alternatives == []


def test_supervisor_cascade_path_uses_structured_pick(monkeypatch) -> None:
    monkeypatch.setattr(
        "cuo.supervisor.rule_score",
        lambda *a, **k: [Candidate(skill_name="chief-product-officer/feature-prd-intake", confidence=0.30)],
    )

    def transport(payload, headers, timeout_s):
        assert headers["X-CUO-Decision-Id"] == "cascade-1"
        assert payload["model_alias"] == "chat.smart"
        assert timeout_s == 3.0
        return {
            "skill_name": "chief-technology-officer/ship-feature-requests",
            "arguments": {"persona_workflow": "chief-technology-officer/ship-feature-requests"},
            "rationale": "The request is about shipping backlog FRs.",
            "confidence": 0.82,
        }

    def invoker(_candidate: Candidate) -> InvocationResult:
        return InvocationResult(status="succeeded", stdout="ok", exit_code=0)

    sink = MemoryAuditSink()
    decision = Supervisor(
        cuo_root=CUO_ROOT,
        audit_sink=sink,
        llm_proxy=LiteLLMProxy(transport=transport),
        invoker=invoker,
    ).route("please handle this feature request", request_id="cascade-1", trace_id="b" * 32)

    assert decision.path_taken == "cascade_then_auto"
    assert decision.llm_used is True
    assert decision.routed is True
    row = next(r for r in sink.rows if r["row_kind"] == "cuo.routing_decision")
    assert row["llm_pick"]["skill_name"] == "chief-technology-officer/ship-feature-requests"
    assert row["next_step"] is None


def test_supervisor_persona_defer_matrix_blocks_destructive_auto(monkeypatch) -> None:
    monkeypatch.setattr(
        "cuo.supervisor.rule_score",
        lambda *a, **k: [
            Candidate(
                skill_name="chief-financial-officer/monthly-close",
                confidence=0.95,
                operation="invoice_emit",
            )
        ],
    )
    sink = MemoryAuditSink()
    decision = Supervisor(cuo_root=CUO_ROOT, audit_sink=sink).route(
        "emit this invoice",
        persona_key="cfo",
        agent_persona="cuo-cfo@1.0.0",
        request_id="defer-1",
        trace_id="c" * 32,
    )
    assert decision.routed is False
    assert decision.path_taken == "ask"
    assert "persona_defer_matrix" in (decision.reason or "")
    assert any(r["row_kind"] == "cuo.persona_defer_block" for r in sink.rows)


def test_supervisor_audit_row_scrubs_query_pii(monkeypatch) -> None:
    monkeypatch.setattr(
        "cuo.supervisor.rule_score",
        lambda *a, **k: [Candidate(skill_name="chief-product-officer/feature-prd-intake", confidence=0.62)],
    )
    sink = MemoryAuditSink()
    Supervisor(cuo_root=CUO_ROOT, audit_sink=sink).route(
        "draft PRD for alice@example.com",
        invoke=False,
        request_id="audit-1",
        trace_id="d" * 32,
    )
    row = next(r for r in sink.rows if r["row_kind"] == "cuo.routing_decision")
    assert row["path_taken"] == "ask"
    assert row["query"] == "draft PRD for [redacted-email]"
    assert row["subject_id_hash16"]


def test_supervisor_litellm_proxy_has_no_direct_provider_imports() -> None:
    forbidden = {"boto3", "anthropic", "openai"}
    for path in (CUO_ROOT / "cuo" / "supervisor").rglob("*.py"):
        tree = ast.parse(path.read_text(encoding="utf-8"))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imported = {alias.name.split(".", 1)[0] for alias in node.names}
                assert not (imported & forbidden), f"{path} imports {imported & forbidden}"
            if isinstance(node, ast.ImportFrom) and node.module:
                root = node.module.split(".", 1)[0]
                assert root not in forbidden, f"{path} imports {root}"


def test_supervisor_llm_cascade_timeout_falls_through_to_ask(monkeypatch) -> None:
    monkeypatch.setattr(
        "cuo.supervisor.rule_score",
        lambda *a, **k: [Candidate(skill_name="chief-product-officer/feature-prd-intake", confidence=0.30)],
    )

    def timeout_transport(_payload, _headers, _timeout_s):
        raise TimeoutError("deadline")

    sink = MemoryAuditSink()
    decision = Supervisor(
        cuo_root=CUO_ROOT,
        audit_sink=sink,
        llm_proxy=LiteLLMProxy(transport=timeout_transport),
    ).route("ambiguous", invoke=False, request_id="timeout-1", trace_id="e" * 32)

    assert decision.path_taken == "cascade_then_ask"
    assert decision.routed is False
    assert decision.llm_used is True
    assert any(r["row_kind"] == "cuo.llm_cascade_timeout" for r in sink.rows)


def test_supervisor_freeform_llm_response_is_rejected(monkeypatch) -> None:
    monkeypatch.setattr(
        "cuo.supervisor.rule_score",
        lambda *a, **k: [Candidate(skill_name="chief-product-officer/feature-prd-intake", confidence=0.30)],
    )

    def bad_transport(_payload, _headers, _timeout_s):
        return {"choices": [{"message": {"content": "plain text, not json"}}]}

    decision = Supervisor(
        cuo_root=CUO_ROOT,
        audit_sink=MemoryAuditSink(),
        llm_proxy=LiteLLMProxy(transport=bad_transport),
    ).route("ambiguous", invoke=False, record=False, request_id="bad-llm", trace_id="f" * 32)

    assert decision.path_taken == "cascade_then_ask"
    assert decision.routed is False
    assert "schema_violation" in (decision.reason or "")


def test_supervisor_state_version_window_and_persona_jwt() -> None:
    assert state_version_supported(1)
    assert state_version_supported(3)
    assert not state_version_supported(4)
    assert validate_agent_persona_claim("cuo-cto@1.2.3", "cto") == "1.2.3"


def test_supervisor_persona_jwt_mismatch_records_aux_row() -> None:
    sink = MemoryAuditSink()
    decision = Supervisor(cuo_root=CUO_ROOT, audit_sink=sink).route(
        "ship feature requests",
        persona_key="cto",
        agent_persona="cuo-cfo@1.0.0",
        request_id="jwt-1",
        trace_id="0" * 32,
    )
    assert decision.routed is False
    assert "persona_mismatch" in (decision.reason or "")
    assert sink.rows[0]["row_kind"] == "cuo.persona_mismatch"


def test_supervisor_rule_path_replay_equivalence() -> None:
    supervisor = Supervisor(cuo_root=CUO_ROOT, audit_sink=MemoryAuditSink())
    one = supervisor.route(
        "ship feature requests",
        invoke=False,
        record=False,
        request_id="same",
        trace_id="1" * 32,
    ).model_dump(mode="json")
    two = supervisor.route(
        "ship feature requests",
        invoke=False,
        record=False,
        request_id="same",
        trace_id="1" * 32,
    ).model_dump(mode="json")
    assert one == two
