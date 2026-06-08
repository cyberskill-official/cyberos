"""Supervisor graph topology.

When `langgraph` is installed, `build_graph` uses its `StateGraph`. The local
fallback keeps tests deterministic in development environments that have not
installed optional FR-CUO-101 dependencies yet.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from typing import Any, Callable

try:  # pragma: no cover - exercised when optional dependency is installed
    from langgraph.graph import StateGraph as _StateGraph
except ModuleNotFoundError:  # local compatibility shim
    class _StateGraph:  # type: ignore[no-redef]
        def __init__(self, state_type: type) -> None:
            self.state_type = state_type
            self.nodes: dict[str, Callable[..., Any]] = {}
            self.edges: list[tuple[str, str]] = []
            self.conditional_edges: dict[str, Any] = {}
            self.entry_point: str | None = None

        def add_node(self, name: str, func: Callable[..., Any]) -> None:
            self.nodes[name] = func

        def add_edge(self, source: str, target: str) -> None:
            self.edges.append((source, target))

        def add_conditional_edges(self, source: str, router: Callable[..., str], mapping: dict[str, str]) -> None:
            self.conditional_edges[source] = (router, dict(mapping))

        def set_entry_point(self, name: str) -> None:
            self.entry_point = name

        def compile(self) -> "_StateGraph":
            return self

from .state import CuoState


GRAPH_NODE_NAMES = ("parse", "rule_score", "branch", "invoke", "record")


@dataclass(frozen=True)
class GraphContract:
    nodes: tuple[str, ...] = GRAPH_NODE_NAMES
    transitions: tuple[tuple[str, str], ...] = (
        ("parse", "rule_score"),
        ("rule_score", "branch"),
        ("branch", "invoke"),
        ("branch", "record"),
        ("invoke", "record"),
    )
    conditional_paths: dict[str, str] = field(default_factory=lambda: {
        "auto": "invoke",
        "ask": "record",
        "cascade": "record",
        "defer": "record",
        "cascade_then_auto": "invoke",
        "cascade_then_ask": "record",
    })


def build_graph() -> Any:
    graph = _StateGraph(CuoState)
    for name in GRAPH_NODE_NAMES:
        graph.add_node(name, lambda state, _name=name: state)
    graph.set_entry_point("parse")
    graph.add_edge("parse", "rule_score")
    graph.add_edge("rule_score", "branch")
    graph.add_conditional_edges("branch", lambda state: state.get("path_taken", "defer"), GraphContract().conditional_paths)
    graph.add_edge("invoke", "record")
    return graph.compile()


def assert_closed_topology(contract: GraphContract | None = None) -> None:
    c = contract or GraphContract()
    if tuple(c.nodes) != GRAPH_NODE_NAMES:
        raise ValueError("FR-CUO-101 graph must have exactly parse/rule_score/branch/invoke/record")
    allowed = set(GRAPH_NODE_NAMES)
    for source, target in c.transitions:
        if source not in allowed or target not in allowed:
            raise ValueError(f"transition outside closed topology: {source}->{target}")
