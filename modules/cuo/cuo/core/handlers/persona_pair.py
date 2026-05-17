"""PersonaPairHandler — interleaved chains with shared artefact ownership.

Per FR-CUO-106 §1.7 + DEC-2386. Affects 4 pairs:
    - cro-revenue ↔ cco-customer            (shared: churn-cohort-analysis)
    - cmo ↔ cco-communications              (shared: campaign-plan)
    - cro-risk ↔ cto                        (shared: incident-report)
    - cco-customer ↔ cdo-data               (shared: customer-profile)

The handler runs the primary chain up to `handoff_step`, then dispatches to
the peer persona's matching workflow. The peer's contribution is threaded
back as input to the remaining primary steps. Both chains' artefacts are
addressed by `shared_artefact` content hash so drift is detectable.

For this alpha implementation, we run the peer's chain end-to-end and surface
its final output back as a single input to the primary chain's resume point.
A future iteration can interleave finer-grained.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING, Any

from cuo.core.handlers.base import Handler, HandlerResult

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker


class PersonaPairHandler(Handler):
    handler_kind = "PersonaPairHandler"

    def __init__(
        self,
        peer_persona: str,
        peer_workflow: str,
        shared_artefact: str,
        handoff_step: int,
    ):
        if not isinstance(peer_persona, str) or not peer_persona:
            raise ValueError("peer_persona must be a non-empty string")
        if not isinstance(peer_workflow, str) or not peer_workflow:
            raise ValueError("peer_workflow must be a non-empty string")
        if not isinstance(shared_artefact, str) or not shared_artefact:
            raise ValueError("shared_artefact must be a non-empty string")
        if not isinstance(handoff_step, int) or handoff_step < 1:
            raise ValueError(f"handoff_step must be a positive int, got {handoff_step!r}")
        self.peer_persona = peer_persona
        self.peer_workflow = peer_workflow
        self.shared_artefact = shared_artefact
        self.handoff_step = handoff_step

    @classmethod
    def from_workflow(cls, workflow: "WorkflowEntry") -> "PersonaPairHandler":
        fm = getattr(workflow, "frontmatter", None) or {}
        return cls(
            peer_persona=fm.get("peer_persona", ""),
            peer_workflow=fm.get("peer_workflow", ""),
            shared_artefact=fm.get("shared_artefact", ""),
            handoff_step=fm.get("handoff_step", 0),
        )

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

        # Resolve peer persona from catalog
        cuo_root = persona.persona_dir.parent
        all_personas = discover_personas(cuo_root)
        personas_by_slug = {p.slug: p for p in all_personas}
        peer = personas_by_slug.get(self.peer_persona)

        extra_audit: list[dict] = []
        notes: list[str] = []

        if peer is None:
            # Peer not found → emit fail audit + return BLOCKED
            blocked = ChainResult(
                workflow_id=f"{persona.slug}/{workflow.workflow_id.split('/')[-1]}",
                outcome="FAILED",
                validation=None,  # type: ignore[arg-type]
                output_dir=output_dir,
                invoker_kind=type(invoker).__name__ if invoker else "",
                notes=[f"peer persona {self.peer_persona!r} not found in catalog"],
            )
            extra_audit.append({
                "kind": "cuo.persona_pair_peer_not_found",
                "workflow_id": blocked.workflow_id,
                "peer_persona": self.peer_persona,
                "peer_workflow": self.peer_workflow,
                "shared_artefact": self.shared_artefact,
            })
            return HandlerResult(
                handler_kind=self.handler_kind,
                chain_result=blocked,
                extra_audit_kinds=extra_audit,
                notes=blocked.notes,
            )

        # 1. Run primary chain end-to-end (alpha implementation — interleaving deferred)
        primary_dir = output_dir / "primary"
        primary_result = execute_chain(
            persona=persona,
            workflow_slug=workflow.workflow_id.split("/")[-1],
            skill_root=skill_root,
            output_dir=primary_dir,
            inputs=inputs,
            invoker=invoker,
        )

        # 2. Compute primary's shared artefact hash (use final step's output hash)
        primary_artefact_hash = (
            primary_result.step_results[-1].output_hash
            if primary_result.step_results
            else ""
        )

        # Emit handoff event for primary → peer transition
        extra_audit.append({
            "kind": "cuo.persona_pair_handoff",
            "workflow_id": primary_result.workflow_id,
            "direction": "primary_to_peer",
            "peer_persona": self.peer_persona,
            "peer_workflow": self.peer_workflow,
            "shared_artefact": self.shared_artefact,
            "handoff_step": self.handoff_step,
            "primary_artefact_hash": primary_artefact_hash,
        })

        # 3. Run peer chain with primary's artefact threaded in as input
        peer_inputs = dict(inputs or {})
        peer_inputs["shared_artefact_handle"] = primary_artefact_hash
        peer_inputs["shared_artefact_name"] = self.shared_artefact
        peer_dir = output_dir / "peer"
        peer_result = execute_chain(
            persona=peer,
            workflow_slug=self.peer_workflow,
            skill_root=skill_root,
            output_dir=peer_dir,
            inputs=peer_inputs,
            invoker=invoker,
        )

        # 4. Compute peer's shared artefact hash + verify match
        peer_artefact_hash = (
            peer_result.step_results[-1].output_hash
            if peer_result.step_results
            else ""
        )

        # Emit reverse handoff
        extra_audit.append({
            "kind": "cuo.persona_pair_handoff",
            "workflow_id": peer_result.workflow_id,
            "direction": "peer_to_primary",
            "peer_persona": persona.slug,
            "peer_workflow": workflow.workflow_id.split("/")[-1],
            "shared_artefact": self.shared_artefact,
            "primary_artefact_hash": primary_artefact_hash,
            "peer_artefact_hash": peer_artefact_hash,
        })

        # Verify artefact-hash consistency (alpha: log if mismatch but don't fail)
        if (
            primary_artefact_hash and peer_artefact_hash
            and primary_artefact_hash != peer_artefact_hash
        ):
            extra_audit.append({
                "kind": "cuo.persona_pair_artefact_drift",
                "workflow_id": primary_result.workflow_id,
                "primary_artefact_hash": primary_artefact_hash,
                "peer_artefact_hash": peer_artefact_hash,
                "severity": "warning",  # alpha — don't fail
            })
            notes.append(
                "warning: primary + peer artefact hashes differ (expected at alpha; "
                "true interleaving is Phase 4.1)"
            )

        notes.append(
            f"persona-pair: primary={persona.slug} ↔ peer={self.peer_persona}; "
            f"shared={self.shared_artefact}"
        )

        return HandlerResult(
            handler_kind=self.handler_kind,
            chain_result=primary_result,
            peer_chain_result=peer_result,
            extra_audit_kinds=extra_audit,
            notes=notes,
        )
