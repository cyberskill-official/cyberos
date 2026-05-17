"""PerInstanceHandler — iterate chain once per instance_descriptor entry; fan-in summary.

Per FR-CUO-106 §1.4 + DEC-2383. Affects 1 workflow today:
    - chief-sales-officer/quarterly-account-plan (10–20 top-tier accounts per quarter)

The handler reads `workflow.frontmatter.instance_descriptor` (a list of dicts).
For each instance, it invokes `execute_chain()` with `inputs.merged(instance)`.
It collects all ChainResults, then builds a summary ChainResult with
outcome = COMPLETED_BATCH | PARTIAL_BATCH | FAILED_BATCH and `per_instance: [...]`.
"""

from __future__ import annotations

from copy import deepcopy
from dataclasses import field
from pathlib import Path
from typing import TYPE_CHECKING, Any

from cuo.core.handlers.base import Handler, HandlerResult

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker


class PerInstanceHandler(Handler):
    handler_kind = "PerInstanceHandler"

    def __init__(self, instance_descriptor: list[dict[str, Any]]):
        if not isinstance(instance_descriptor, list):
            raise TypeError(
                f"instance_descriptor must be a list, got {type(instance_descriptor).__name__}"
            )
        if not instance_descriptor:
            raise ValueError("instance_descriptor cannot be empty")
        self.instance_descriptor = instance_descriptor

    @classmethod
    def from_workflow(cls, workflow: "WorkflowEntry") -> "PerInstanceHandler":
        fm = getattr(workflow, "frontmatter", None) or {}
        descriptor = fm.get("instance_descriptor")
        if not isinstance(descriptor, list) or not descriptor:
            raise ValueError(
                f"workflow {workflow.workflow_id} declares pattern: per_instance "
                f"but instance_descriptor is missing or empty (got {descriptor!r})"
            )
        return cls(instance_descriptor=descriptor)

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
        from cuo.core.supervisor import ChainResult, execute_chain

        base_inputs = dict(inputs or {})
        workflow_slug = workflow.workflow_id.split("/")[-1]

        per_instance: list[ChainResult] = []
        extra_audit: list[dict] = []
        succeeded = 0
        failed = 0

        for idx, instance in enumerate(self.instance_descriptor):
            merged_inputs = {**base_inputs, **(instance if isinstance(instance, dict) else {})}
            instance_dir = output_dir / f"instance-{idx:03d}"
            cr = execute_chain(
                persona=persona,
                workflow_slug=workflow_slug,
                skill_root=skill_root,
                output_dir=instance_dir,
                inputs=merged_inputs,
                invoker=invoker,
            )
            per_instance.append(cr)
            extra_audit.append({
                "kind": "cuo.per_instance_iteration",
                "workflow_id": cr.workflow_id,
                "instance_idx": idx,
                "instance_keys": sorted(instance.keys()) if isinstance(instance, dict) else [],
                "outcome": cr.outcome,
                "duration_ms": cr.total_duration_ms,
            })
            if cr.outcome == "COMPLETED":
                succeeded += 1
            else:
                failed += 1

        # Build batch summary
        if failed == 0:
            batch_outcome = "COMPLETED_BATCH"
        elif succeeded == 0:
            batch_outcome = "FAILED_BATCH"
        else:
            batch_outcome = "PARTIAL_BATCH"

        # Use the first instance's ChainResult as the template, then overwrite
        # outcome + notes to represent the batch
        first = per_instance[0]
        # We pass the same ValidationResult since chain structure is invariant
        summary = ChainResult(
            workflow_id=first.workflow_id,
            outcome=batch_outcome,
            validation=first.validation,
            step_results=[],   # no single per-step list for a batch
            output_dir=output_dir,
            invoker_kind=first.invoker_kind,
            notes=[
                f"per_instance batch: {len(per_instance)} instances "
                f"({succeeded} succeeded, {failed} failed)"
            ],
        )

        extra_audit.append({
            "kind": "cuo.per_instance_summary",
            "workflow_id": summary.workflow_id,
            "n_instances": len(per_instance),
            "n_succeeded": succeeded,
            "n_failed": failed,
            "batch_outcome": batch_outcome,
            "total_duration_ms": sum(cr.total_duration_ms for cr in per_instance),
        })

        return HandlerResult(
            handler_kind=self.handler_kind,
            chain_result=summary,
            per_instance=per_instance,
            extra_audit_kinds=extra_audit,
            notes=summary.notes,
        )
