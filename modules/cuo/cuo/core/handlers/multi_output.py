"""MultiOutputHandler — chain runs once; final-step output fans out per recipient.

Per TASK-CUO-106 §1.5 + DEC-2384. Affects 1 workflow today:
    - chief-legal-officer/quarterly-regulatory-cycle (1 source artefact → N regulator filings)

The handler invokes the chain ONCE end-to-end. After completion, it reads
`workflow.frontmatter.output_recipients` (list of `{recipient_id, format,
delivery_method}`). For each recipient, it produces a per-recipient delivery
artefact + emits a `cuo.multi_output_fanout` memory row.

This handler does NOT re-render the chain's final output — fan-out is purely
a delivery / packaging step. Each recipient gets the same source artefact but
packaged in their declared format and delivered via their declared method.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import TYPE_CHECKING, Any

from cuo.core.handlers.base import Handler, HandlerResult

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker


class MultiOutputHandler(Handler):
    handler_kind = "MultiOutputHandler"

    def __init__(self, output_recipients: list[dict[str, Any]]):
        if not isinstance(output_recipients, list):
            raise TypeError(
                f"output_recipients must be a list, got {type(output_recipients).__name__}"
            )
        if not output_recipients:
            raise ValueError("output_recipients cannot be empty")
        for r in output_recipients:
            if not isinstance(r, dict):
                raise TypeError(f"each recipient must be a dict, got {type(r).__name__}")
            for k in ("recipient_id", "format", "delivery_method"):
                if k not in r:
                    raise ValueError(f"recipient missing required key: {k!r}")
        self.output_recipients = output_recipients

    @classmethod
    def from_workflow(cls, workflow: "WorkflowEntry") -> "MultiOutputHandler":
        fm = getattr(workflow, "frontmatter", None) or {}
        recipients = fm.get("output_recipients")
        if not isinstance(recipients, list) or not recipients:
            raise ValueError(
                f"workflow {workflow.workflow_id} declares pattern: multi_output "
                f"but output_recipients is missing or empty (got {recipients!r})"
            )
        return cls(output_recipients=recipients)

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

        # 1. Run the chain ONCE end-to-end
        chain_result = execute_chain(
            persona=persona,
            workflow_slug=workflow.workflow_id.split("/")[-1],
            skill_root=skill_root,
            output_dir=output_dir,
            inputs=inputs,
            invoker=invoker,
        )

        # 2. Locate the final step's output artefact
        if not chain_result.step_results:
            return HandlerResult(
                handler_kind=self.handler_kind,
                chain_result=chain_result,
                notes=["no chain steps produced output; nothing to fan out"],
            )
        final = chain_result.step_results[-1]
        source_artefact = final.output_path

        # 3. Fan out — write per-recipient envelope + emit one memory row each
        fanout_dir = output_dir / "fanout"
        fanout_dir.mkdir(parents=True, exist_ok=True)
        extra_audit: list[dict] = []
        notes: list[str] = []

        for recipient in self.output_recipients:
            envelope_path = fanout_dir / f"{recipient['recipient_id']}.{recipient['format']}.json"
            envelope = {
                "recipient_id": recipient["recipient_id"],
                "format": recipient["format"],
                "delivery_method": recipient["delivery_method"],
                "source_artefact": str(source_artefact) if source_artefact else None,
                "source_content_hash": final.output_hash if final else None,
                "workflow_id": chain_result.workflow_id,
            }
            envelope_path.write_text(json.dumps(envelope, indent=2), encoding="utf-8")

            extra_audit.append({
                "kind": "cuo.multi_output_fanout",
                "workflow_id": chain_result.workflow_id,
                "recipient_id": recipient["recipient_id"],
                "format": recipient["format"],
                "delivery_method": recipient["delivery_method"],
                "envelope_path": str(envelope_path),
                "source_content_hash": final.output_hash if final else None,
            })
            notes.append(
                f"fanout to {recipient['recipient_id']} via {recipient['delivery_method']}"
            )

        return HandlerResult(
            handler_kind=self.handler_kind,
            chain_result=chain_result,
            extra_audit_kinds=extra_audit,
            notes=notes,
        )
