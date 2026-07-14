"""SequentialApprovalHandler — chain A halts for approver B; resumes on approval.

Per TASK-CUO-106 §1.6 + DEC-2385. Affects 1 pair today:
    - chief-ethics-officer/per-model-card-ethics-sign-off
        gates chief-ai-officer/per-model-card-release

The handler reads `workflow.frontmatter.gates` (list of `{approver_persona,
approver_workflow}`). For each gate, it runs the approver's chain first. If
the approver chain's outcome is NOT COMPLETED, the gated chain is BLOCKED
and we emit `cuo.sequential_approval_halted`. If the approver chain succeeds,
we run the gated chain and emit `cuo.sequential_approval_resumed`.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

from cuo.core.handlers.base import Handler, HandlerResult

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker


class SequentialApprovalHandler(Handler):
    handler_kind = "SequentialApprovalHandler"

    def __init__(self, gates: list[dict[str, Any]]):
        if not isinstance(gates, list):
            raise TypeError(f"gates must be a list, got {type(gates).__name__}")
        if not gates:
            raise ValueError("gates cannot be empty")
        for g in gates:
            if not isinstance(g, dict):
                raise TypeError(f"each gate must be a dict, got {type(g).__name__}")
            for k in ("approver_persona", "approver_workflow"):
                if k not in g:
                    raise ValueError(f"gate missing required key: {k!r}")
        self.gates = gates

    @classmethod
    def from_workflow(cls, workflow: "WorkflowEntry") -> "SequentialApprovalHandler":
        fm = getattr(workflow, "frontmatter", None) or {}
        gates = fm.get("gates")
        if not isinstance(gates, list) or not gates:
            raise ValueError(
                f"workflow {workflow.workflow_id} declares pattern: sequential_approval "
                f"but gates is missing or empty (got {gates!r})"
            )
        return cls(gates=gates)

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
        from cuo.core.catalog import discover_personas
        from cuo.core.supervisor import ChainResult, execute_chain

        # Resolve the approver persona from the catalog
        cuo_root = persona.persona_dir.parent
        all_personas = discover_personas(cuo_root)
        personas_by_slug = {p.slug: p for p in all_personas}

        extra_audit: list[dict] = []
        notes: list[str] = []

        # Walk each gate sequentially — if any fail, halt
        for gate in self.gates:
            approver_slug = gate["approver_persona"]
            approver_workflow_slug = gate["approver_workflow"]

            approver_persona = personas_by_slug.get(approver_slug)
            if approver_persona is None:
                # Approver persona missing → cannot validate the gate → halt
                halt_result = ChainResult(
                    workflow_id=f"{persona.slug}/{workflow.workflow_id.split('/')[-1]}",
                    outcome="BLOCKED",
                    validation=None,  # type: ignore[arg-type]
                    output_dir=output_dir,
                    invoker_kind=type(invoker).__name__ if invoker else "",
                    notes=[f"gate approver persona {approver_slug!r} not found in catalog"],
                )
                extra_audit.append({
                    "kind": "cuo.sequential_approval_halted",
                    "workflow_id": halt_result.workflow_id,
                    "gate_index": self.gates.index(gate),
                    "approver_persona": approver_slug,
                    "approver_workflow": approver_workflow_slug,
                    "halt_reason": "approver_persona_not_found",
                })
                return HandlerResult(
                    handler_kind=self.handler_kind,
                    chain_result=halt_result,
                    extra_audit_kinds=extra_audit,
                    notes=notes + [f"halted: approver {approver_slug!r} not found"],
                )

            # Run the approver chain
            approver_dir = output_dir / f"gate-{self.gates.index(gate):02d}-approver"
            approver_result = execute_chain(
                persona=approver_persona,
                workflow_slug=approver_workflow_slug,
                skill_root=skill_root,
                output_dir=approver_dir,
                inputs=inputs,
                invoker=invoker,
            )

            if approver_result.outcome != "COMPLETED":
                # Approver chain failed → gated chain blocked
                halt_result = ChainResult(
                    workflow_id=f"{persona.slug}/{workflow.workflow_id.split('/')[-1]}",
                    outcome="BLOCKED",
                    validation=approver_result.validation,
                    output_dir=output_dir,
                    invoker_kind=approver_result.invoker_kind,
                    notes=[
                        f"halted by approver gate: {approver_slug}/{approver_workflow_slug} "
                        f"returned {approver_result.outcome}"
                    ],
                )
                extra_audit.append({
                    "kind": "cuo.sequential_approval_halted",
                    "workflow_id": halt_result.workflow_id,
                    "gate_index": self.gates.index(gate),
                    "approver_persona": approver_slug,
                    "approver_workflow": approver_workflow_slug,
                    "approver_outcome": approver_result.outcome,
                    "halt_reason": "approver_chain_did_not_complete",
                })
                return HandlerResult(
                    handler_kind=self.handler_kind,
                    chain_result=halt_result,
                    extra_audit_kinds=extra_audit,
                    notes=halt_result.notes,
                )

            # Approver succeeded — log the resume signal
            extra_audit.append({
                "kind": "cuo.sequential_approval_resumed",
                "workflow_id": f"{persona.slug}/{workflow.workflow_id.split('/')[-1]}",
                "gate_index": self.gates.index(gate),
                "approver_persona": approver_slug,
                "approver_workflow": approver_workflow_slug,
                "approver_duration_ms": approver_result.total_duration_ms,
            })
            notes.append(
                f"gate {self.gates.index(gate)} cleared by {approver_slug}/{approver_workflow_slug}"
            )

        # All gates cleared — run the gated chain
        gated_dir = output_dir / "gated-chain"
        gated_result = execute_chain(
            persona=persona,
            workflow_slug=workflow.workflow_id.split("/")[-1],
            skill_root=skill_root,
            output_dir=gated_dir,
            inputs=inputs,
            invoker=invoker,
        )

        return HandlerResult(
            handler_kind=self.handler_kind,
            chain_result=gated_result,
            extra_audit_kinds=extra_audit,
            notes=notes,
        )
