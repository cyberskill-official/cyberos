"""LinearHandler — default pattern; routes to existing execute_chain() unchanged.

185 of 194 workflows in the catalog use this pattern (the default). Existence
of this handler is purely structural: dispatch.pick_handler() returns it for
`pattern: linear` (or no pattern declared) so the supervisor's call site
doesn't need a special case.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

from cuo.core.handlers.base import Handler, HandlerResult

if TYPE_CHECKING:
    from cuo.core.catalog import PersonaEntry, WorkflowEntry
    from cuo.core.invoker import Invoker


class LinearHandler(Handler):
    """Default chain walker. Delegates to supervisor.execute_chain()."""

    handler_kind = "LinearHandler"

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
        # Lazy import — supervisor.execute_chain imports handlers indirectly
        from cuo.core.supervisor import execute_chain

        chain_result = execute_chain(
            persona=persona,
            workflow_slug=workflow.workflow_id.split("/")[-1],
            skill_root=skill_root,
            output_dir=output_dir,
            inputs=inputs,
            invoker=invoker,
        )
        return HandlerResult(
            handler_kind=self.handler_kind,
            chain_result=chain_result,
        )
