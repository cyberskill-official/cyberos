"""Phase 4 — special-case workflow handlers.

Five patterns surfaced during Sessions D–N (catalog completion) that the default
left-to-right chain walker cannot handle correctly. Each pattern gets a Handler
subclass dispatched by the workflow's `pattern:` frontmatter field.

Patterns (enum):
    linear              — default; routes to existing execute_chain() unchanged
    time_critical       — bypass scheduler; SLA-breach audit if exceeded
    per_instance        — iterate chain ×N per instance_descriptor; fan-in summary
    multi_output        — chain runs once; final output fanned out per recipient
    sequential_approval — chain A halts for approver B; resumes on approval audit
    persona_pair        — interleaved chains with shared artefact ownership

Spec: TASK-CUO-106 (docs/tasks/cuo/).
"""

from cuo.core.handlers.base import Handler, HandlerResult
from cuo.core.handlers.dispatch import (
    KNOWN_PATTERNS,
    HandlerDispatchError,
    pick_handler,
    pattern_of,
)
from cuo.core.handlers.linear import LinearHandler
from cuo.core.handlers.time_critical import TimeCriticalHandler
from cuo.core.handlers.per_instance import PerInstanceHandler
from cuo.core.handlers.multi_output import MultiOutputHandler
from cuo.core.handlers.sequential_approval import SequentialApprovalHandler
from cuo.core.handlers.persona_pair import PersonaPairHandler

__all__ = [
    "Handler",
    "HandlerResult",
    "KNOWN_PATTERNS",
    "HandlerDispatchError",
    "pick_handler",
    "pattern_of",
    "LinearHandler",
    "TimeCriticalHandler",
    "PerInstanceHandler",
    "MultiOutputHandler",
    "SequentialApprovalHandler",
    "PersonaPairHandler",
]
