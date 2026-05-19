"""TimeCriticalHandler — bypass scheduler queueing; log SLA-breach event if exceeded.

Per FR-CUO-106 §1.3 + DEC-2382. Affects 3 workflows:
    - chief-privacy-officer/breach-response-cycle              (sla_minutes: 240 = 4h)
    - chief-communications-officer/per-crisis-response         (sla_minutes: 120 = 2h)
    - chief-trust-officer/per-trust-incident-update  (sla_minutes: 240)

The handler invokes the chain synchronously (no scheduling/batching/work-stealing).
It tracks total_duration_ms and, if it exceeds the declared SLA, emits a
breach event in `extra_audit_kinds` for the memory bridge to record.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from cuo.core.handlers.base import Handler, HandlerResult

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker


class TimeCriticalHandler(Handler):
    handler_kind = "TimeCriticalHandler"

    def __init__(self, sla_minutes: int):
        if sla_minutes <= 0:
            raise ValueError(f"sla_minutes must be positive, got {sla_minutes}")
        self.sla_minutes = sla_minutes

    @classmethod
    def from_workflow(cls, workflow: "WorkflowEntry") -> "TimeCriticalHandler":
        """Build a handler from the workflow's `sla_minutes` frontmatter field."""
        fm = getattr(workflow, "frontmatter", None) or {}
        sla = fm.get("sla_minutes")
        if not isinstance(sla, int) or sla <= 0:
            raise ValueError(
                f"workflow {workflow.workflow_id} declares pattern: time_critical "
                f"but sla_minutes is missing or invalid (got {sla!r})"
            )
        return cls(sla_minutes=sla)

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
        from cuo.core.supervisor import execute_chain

        # Invoke synchronously — NO scheduling layer, NO queueing
        chain_result = execute_chain(
            persona=persona,
            workflow_slug=workflow.workflow_id.split("/")[-1],
            skill_root=skill_root,
            output_dir=output_dir,
            inputs=inputs,
            invoker=invoker,
        )

        sla_ms = self.sla_minutes * 60 * 1000
        extra_audit: list[dict] = []
        notes: list[str] = []
        if chain_result.total_duration_ms > sla_ms:
            severity = (chain_result.total_duration_ms - sla_ms) / sla_ms
            extra_audit.append({
                "kind": "cuo.time_critical_sla_breach",
                "workflow_id": chain_result.workflow_id,
                "sla_minutes": self.sla_minutes,
                "actual_ms": chain_result.total_duration_ms,
                "breach_severity": round(severity, 3),
            })
            notes.append(
                f"SLA breach: actual {chain_result.total_duration_ms}ms > "
                f"sla {sla_ms}ms (severity {severity:.2f})"
            )

        return HandlerResult(
            handler_kind=self.handler_kind,
            chain_result=chain_result,
            extra_audit_kinds=extra_audit,
            notes=notes,
        )
