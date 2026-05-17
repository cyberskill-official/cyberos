"""Dispatch — read workflow `pattern:` frontmatter, return matching Handler subclass.

Per FR-CUO-106 §1.2 + DEC-2387:
    - Read workflow.frontmatter.pattern (default 'linear')
    - Return matching Handler subclass instance
    - linear pattern → LinearHandler (= existing execute_chain)
    - all others → dedicated Handler subclass
"""

from __future__ import annotations

from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from cuo.core.catalog import WorkflowEntry
    from cuo.core.handlers.base import Handler


# Closed enum per FR-CUO-106 DEC-2381 (cardinality 6)
KNOWN_PATTERNS = frozenset({
    "linear",
    "time_critical",
    "per_instance",
    "multi_output",
    "sequential_approval",
    "persona_pair",
})


class HandlerDispatchError(ValueError):
    """Raised when workflow.pattern is not in KNOWN_PATTERNS."""


def pattern_of(workflow: "WorkflowEntry") -> str:
    """Extract the workflow's declared pattern (default 'linear').

    A WorkflowEntry's source-of-truth for special-case handling is its
    workflow file's YAML frontmatter `pattern:` field. We look in this
    order:
      1. workflow.pattern attribute (if catalog parsing populated it)
      2. workflow.frontmatter dict 'pattern' key (if discovered via dict)
      3. Default 'linear'
    """
    # Try attribute access first
    pattern = getattr(workflow, "pattern", None)
    if isinstance(pattern, str) and pattern:
        return pattern
    # Try dict-style frontmatter
    fm = getattr(workflow, "frontmatter", None)
    if isinstance(fm, dict):
        p = fm.get("pattern")
        if isinstance(p, str) and p:
            return p
    return "linear"


def pick_handler(workflow: "WorkflowEntry") -> "Handler":
    """Return a Handler instance for `workflow`'s pattern.

    Raises HandlerDispatchError if the pattern is unrecognised.
    """
    # Lazy imports — avoid circular dep on handlers/__init__.py at module load
    from cuo.core.handlers.linear import LinearHandler
    from cuo.core.handlers.time_critical import TimeCriticalHandler
    from cuo.core.handlers.per_instance import PerInstanceHandler
    from cuo.core.handlers.multi_output import MultiOutputHandler
    from cuo.core.handlers.sequential_approval import SequentialApprovalHandler
    from cuo.core.handlers.persona_pair import PersonaPairHandler

    pattern = pattern_of(workflow)
    if pattern not in KNOWN_PATTERNS:
        raise HandlerDispatchError(
            f"Unknown workflow pattern {pattern!r} in {workflow.workflow_id}; "
            f"expected one of {sorted(KNOWN_PATTERNS)}"
        )

    if pattern == "linear":
        return LinearHandler()
    if pattern == "time_critical":
        return TimeCriticalHandler.from_workflow(workflow)
    if pattern == "per_instance":
        return PerInstanceHandler.from_workflow(workflow)
    if pattern == "multi_output":
        return MultiOutputHandler.from_workflow(workflow)
    if pattern == "sequential_approval":
        return SequentialApprovalHandler.from_workflow(workflow)
    if pattern == "persona_pair":
        return PersonaPairHandler.from_workflow(workflow)
    # Unreachable given the frozenset check above
    raise HandlerDispatchError(f"unhandled pattern {pattern!r}")
