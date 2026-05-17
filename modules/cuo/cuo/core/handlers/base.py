"""Handler ABC — every special-case handler implements `execute()` returning a HandlerResult.

The HandlerResult mirrors ChainResult (one chain, one outcome) for linear/time_critical/
multi_output/sequential_approval, OR carries a `per_instance: list[ChainResult]` for
per_instance (batch execution).
"""

from __future__ import annotations

import abc
from dataclasses import dataclass, field
from pathlib import Path
from typing import TYPE_CHECKING, Any

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker
    from cuo.core.supervisor import ChainResult


@dataclass
class HandlerResult:
    """Outcome of a Handler.execute() call.

    For linear/time_critical/multi_output/sequential_approval handlers, this
    wraps a single underlying ChainResult.

    For per_instance handlers, `chain_result` summarises the batch (outcome:
    COMPLETED_BATCH | PARTIAL_BATCH | FAILED_BATCH) and `per_instance` holds
    each instance's individual ChainResult.

    For persona_pair handlers, `chain_result` is the primary persona's chain
    after re-integration of the peer's contribution; `peer_chain_result`
    holds the peer's chain.
    """

    handler_kind: str  # class name, e.g. "TimeCriticalHandler"
    chain_result: "ChainResult"
    per_instance: list["ChainResult"] = field(default_factory=list)
    peer_chain_result: "ChainResult | None" = None
    extra_audit_kinds: list[dict[str, Any]] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    def __repr__(self) -> str:
        bits = [f"handler={self.handler_kind!r}", f"outcome={self.chain_result.outcome}"]
        if self.per_instance:
            bits.append(f"per_instance={len(self.per_instance)}")
        if self.peer_chain_result:
            bits.append(f"peer={self.peer_chain_result.outcome}")
        return f"HandlerResult({', '.join(bits)})"


class Handler(abc.ABC):
    """Base class for Phase-4 workflow Handlers.

    Subclasses MUST implement `execute()`. The dispatcher reads workflow
    frontmatter, picks the matching Handler, and calls `execute()`.
    """

    handler_kind: str = "Handler"  # subclasses override

    @abc.abstractmethod
    def execute(
        self,
        persona: "PersonaEntry",
        workflow: "WorkflowEntry",
        skill_root: Path,
        output_dir: Path,
        *,
        inputs: dict | None = None,
        invoker: "Invoker | None" = None,
    ) -> HandlerResult:
        """Execute the workflow per this Handler's pattern semantics."""
        raise NotImplementedError

    def __repr__(self) -> str:
        return f"{self.__class__.__name__}()"
