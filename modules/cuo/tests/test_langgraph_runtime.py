from __future__ import annotations

import pytest

from cuo.core.langgraph_runtime import (
    ChainNode,
    InMemoryCheckpointer,
    LiteLLMRouter,
    ModelCandidate,
    RollbackStep,
    build_trace_row,
    execute_with_rollback,
    postgres_checkpoint_insert_sql,
    topological_walk,
)


def test_litellm_router_escalates_between_010_and_050():
    router = LiteLLMRouter()
    assert router.route([ModelCandidate("anthropic", "claude", 0.09)]).outcome == "no_match"
    assert router.route([ModelCandidate("anthropic", "claude", 0.30)]).outcome == "escalate"
    decision = router.route([ModelCandidate("bedrock", "sonnet", 0.80, cost_rank=1)])
    assert decision.outcome == "accept"
    assert decision.provider == "bedrock"


def test_checkpointer_and_replay_trace_contract():
    cp = InMemoryCheckpointer()
    first = cp.save("run-1", "tenant-1", {"step": 1})
    second = cp.save("run-1", "tenant-1", {"step": 2})
    assert first.seq == 1
    assert cp.latest("run-1") == second

    row = build_trace_row(
        run_id="run-1",
        prompt="ship TASK-CUO-103",
        model="claude-sonnet",
        temperature=0.2,
        seed=42,
        response_hash="abc",
    )
    assert row.row_kind == "cuo.trace.replay"
    assert row.seed == 42
    assert row.temperature == 0.2
    assert "INSERT INTO langgraph_checkpoints" in postgres_checkpoint_insert_sql()
    with pytest.raises(ValueError):
        postgres_checkpoint_insert_sql("bad;drop")


def test_topological_walk_composite_audit_and_cycle_detection():
    nodes = [
        ChainNode("review", ("impl",)),
        ChainNode("spec"),
        ChainNode("impl", ("spec",)),
    ]
    ordered, audit = topological_walk(nodes, run_id="r")
    assert [node.step_id for node in ordered] == ["spec", "impl", "review"]
    assert audit.row_kind == "cuo.chain_walk.completed"
    assert len(audit.sub_rows) == 3

    with pytest.raises(ValueError, match="cycle"):
        topological_walk([ChainNode("a", ("b",)), ChainNode("b", ("a",))])


def test_execute_with_rollback_preserves_partial_audit():
    events: list[str] = []

    def ok(name: str):
        def run():
            events.append(f"run:{name}")
            return name
        return run

    def rb(name: str):
        def run(_result):
            events.append(f"rollback:{name}")
        return run

    def fail():
        events.append("run:fail")
        raise RuntimeError("boom")

    report = execute_with_rollback([
        RollbackStep("one", ok("one"), rb("one")),
        RollbackStep("two", ok("two"), rb("two")),
        RollbackStep("fail", fail, rb("fail")),
    ])
    assert report.outcome == "rolled_back"
    assert report.completed_steps == ["one", "two"]
    assert report.rolled_back_steps == ["two", "one"]
    assert events == ["run:one", "run:two", "run:fail", "rollback:two", "rollback:one"]
    assert report.audit_rows[-1]["row_kind"] == "cuo.chain.failed"
