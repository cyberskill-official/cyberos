"""FR-CUO-101..105 — production supervisor primitives.

The real deployment can swap the graph engine underneath this adapter. The
contract here is intentionally dependency-light and testable: confidence-band
routing, checkpoint persistence, deterministic replay traces, topological skill
walks, and per-step rollback with partial audit preservation.
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import Any, Callable


@dataclass(frozen=True)
class ModelCandidate:
    provider: str
    model: str
    confidence: float
    cost_rank: int = 0


@dataclass(frozen=True)
class RouteDecision:
    outcome: str  # accept | escalate | no_match
    provider: str | None
    model: str | None
    confidence: float
    rationale: str


class LiteLLMRouter:
    """Small LiteLLM-compatible selector for CUO graph entrypoints."""

    def __init__(self, *, escalate_low: float = 0.10, escalate_high: float = 0.50) -> None:
        self.escalate_low = escalate_low
        self.escalate_high = escalate_high

    def route(self, candidates: list[ModelCandidate]) -> RouteDecision:
        if not candidates:
            return RouteDecision("no_match", None, None, 0.0, "no candidates")
        best = sorted(candidates, key=lambda c: (-c.confidence, c.cost_rank, c.provider, c.model))[0]
        if best.confidence < self.escalate_low:
            return RouteDecision("no_match", best.provider, best.model, best.confidence, "below no-match floor")
        if self.escalate_low <= best.confidence <= self.escalate_high:
            return RouteDecision("escalate", best.provider, best.model, best.confidence, "confidence band requires human/CUO escalation")
        return RouteDecision("accept", best.provider, best.model, best.confidence, "confidence above escalation band")


@dataclass(frozen=True)
class GraphCheckpoint:
    run_id: str
    tenant_id: str
    state: dict[str, Any]
    seq: int
    ts_ns: int


class InMemoryCheckpointer:
    """Postgres-checkpointer contract implemented in memory for tests."""

    def __init__(self) -> None:
        self._rows: dict[str, list[GraphCheckpoint]] = {}

    def save(self, run_id: str, tenant_id: str, state: dict[str, Any]) -> GraphCheckpoint:
        rows = self._rows.setdefault(run_id, [])
        cp = GraphCheckpoint(run_id, tenant_id, dict(state), len(rows) + 1, time.time_ns())
        rows.append(cp)
        return cp

    def latest(self, run_id: str) -> GraphCheckpoint | None:
        rows = self._rows.get(run_id) or []
        return rows[-1] if rows else None


def postgres_checkpoint_insert_sql(table: str = "langgraph_checkpoints") -> str:
    """Return the SQL shape used by the Postgres checkpointer implementation."""
    if not table.replace("_", "").isalnum():
        raise ValueError("unsafe table name")
    return (
        f"INSERT INTO {table} (run_id, tenant_id, seq, state_json, ts_ns) "
        "VALUES ($1, $2, $3, $4, $5) "
        "ON CONFLICT (run_id, seq) DO NOTHING"
    )


@dataclass(frozen=True)
class TraceRow:
    row_kind: str
    run_id: str
    prompt: str
    model: str
    temperature: float
    seed: int
    response_hash: str
    ts_ns: int


def build_trace_row(
    *,
    run_id: str,
    prompt: str,
    model: str,
    temperature: float,
    seed: int,
    response_hash: str,
) -> TraceRow:
    return TraceRow(
        row_kind="cuo.trace.replay",
        run_id=run_id,
        prompt=prompt,
        model=model,
        temperature=temperature,
        seed=seed,
        response_hash=response_hash,
        ts_ns=time.time_ns(),
    )


@dataclass(frozen=True)
class ChainNode:
    step_id: str
    depends_on: tuple[str, ...] = ()
    payload: dict[str, Any] = field(default_factory=dict)


@dataclass(frozen=True)
class CompositeAuditRow:
    row_kind: str
    run_id: str
    ordered_step_ids: list[str]
    sub_rows: list[dict[str, Any]]


def topological_walk(nodes: list[ChainNode], *, run_id: str = "run") -> tuple[list[ChainNode], CompositeAuditRow]:
    by_id = {node.step_id: node for node in nodes}
    if len(by_id) != len(nodes):
        raise ValueError("duplicate step_id")
    ordered: list[ChainNode] = []
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(step_id: str) -> None:
        if step_id in visited:
            return
        if step_id in visiting:
            raise ValueError(f"cycle detected at {step_id}")
        node = by_id.get(step_id)
        if node is None:
            raise ValueError(f"missing dependency: {step_id}")
        visiting.add(step_id)
        for dep in node.depends_on:
            visit(dep)
        visiting.remove(step_id)
        visited.add(step_id)
        ordered.append(node)

    for node in nodes:
        visit(node.step_id)
    audit = CompositeAuditRow(
        row_kind="cuo.chain_walk.completed",
        run_id=run_id,
        ordered_step_ids=[node.step_id for node in ordered],
        sub_rows=[
            {"row_kind": "cuo.chain_walk.step", "run_id": run_id, "step_id": node.step_id}
            for node in ordered
        ],
    )
    return ordered, audit


@dataclass
class RollbackStep:
    step_id: str
    action: Callable[[], Any]
    rollback: Callable[[Any], Any]


@dataclass(frozen=True)
class RollbackReport:
    outcome: str
    completed_steps: list[str]
    rolled_back_steps: list[str]
    audit_rows: list[dict[str, Any]]
    error: str | None = None


def execute_with_rollback(steps: list[RollbackStep]) -> RollbackReport:
    completed: list[tuple[RollbackStep, Any]] = []
    audit_rows: list[dict[str, Any]] = []
    try:
        for step in steps:
            result = step.action()
            completed.append((step, result))
            audit_rows.append({"row_kind": "cuo.step.completed", "step_id": step.step_id})
    except Exception as exc:  # noqa: BLE001 - preserve failure while compensating
        rolled_back: list[str] = []
        for step, result in reversed(completed):
            step.rollback(result)
            rolled_back.append(step.step_id)
            audit_rows.append({"row_kind": "cuo.step.rolled_back", "step_id": step.step_id})
        audit_rows.append({"row_kind": "cuo.chain.failed", "error": str(exc)})
        return RollbackReport(
            outcome="rolled_back",
            completed_steps=[step.step_id for step, _ in completed],
            rolled_back_steps=rolled_back,
            audit_rows=audit_rows,
            error=str(exc),
        )
    audit_rows.append({"row_kind": "cuo.chain.completed"})
    return RollbackReport(
        outcome="completed",
        completed_steps=[step.step_id for step, _ in completed],
        rolled_back_steps=[],
        audit_rows=audit_rows,
    )
