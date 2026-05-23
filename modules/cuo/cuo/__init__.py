"""CyberOS CUO supervisor — persona-aware orchestration above the SKILL module.

Version: see root VERSION file.

The CUO ("Chief Universal Officer") routes natural-language requests through:
    1. persona match    — which C-role best fits this request
    2. workflow match   — which of that persona's workflows
    3. chain validate   — every step's skill exists in the SKILL catalog
    4. handler dispatch — read workflow.pattern, pick Handler subclass
    5. invoke           — walk the chain via the chosen Invoker
    6. record           — emit decision row to the memory audit chain

Phase 1 (a1): steps 1–3 + dry-run mode for step 5.
Phase 2 (a2): SubprocessInvoker / select_invoker + execute_chain.
Phase 3 (a3): LLMInvoker (mock-llm + Anthropic API) + memory audit emission.
Phase 4 (a4): 5 special-case workflow handlers (time_critical / per_instance /
              multi_output / sequential_approval / persona_pair); dispatched
              by workflow `pattern:` frontmatter. Default `linear` pattern
              routes through existing execute_chain() unchanged.

See `README.md` (this module) for the comprehensive guide and Appendix A
for the normative protocol.
"""

__version__ = "0.1.0"

from cuo.core.memory_bridge import MemoryEmitResult, memory_is_available, emit_chain_result
from cuo.core.catalog import PersonaEntry, WorkflowEntry, discover_personas, discover_workflows
from cuo.core.handlers import (
    KNOWN_PATTERNS,
    Handler,
    HandlerDispatchError,
    HandlerResult,
    LinearHandler,
    MultiOutputHandler,
    PerInstanceHandler,
    PersonaPairHandler,
    SequentialApprovalHandler,
    TimeCriticalHandler,
    pattern_of,
    pick_handler,
)
from cuo.core.invoker import CompositeInvoker, Invoker, StepResult, SubprocessInvoker, select_invoker
from cuo.core.llm_invoker import LLMInvoker
from cuo.core.router import RoutingDecision, route
from cuo.core.supervisor import ChainResult, DryRunResult, dry_run_chain, execute_chain
from cuo.core.validator import ValidationResult, validate_chain

__all__ = [
    "__version__",
    # catalog
    "PersonaEntry",
    "WorkflowEntry",
    "discover_personas",
    "discover_workflows",
    # validator
    "ValidationResult",
    "validate_chain",
    # router
    "RoutingDecision",
    "route",
    # invoker (Phase 2)
    "Invoker",
    "CompositeInvoker",
    "SubprocessInvoker",
    "StepResult",
    "select_invoker",
    # supervisor
    "DryRunResult",
    "dry_run_chain",
    "ChainResult",
    "execute_chain",
    # Phase 3: LLM invoker
    "LLMInvoker",
    # Phase 3: memory bridge
    "MemoryEmitResult",
    "memory_is_available",
    "emit_chain_result",
    # Phase 4: special-case workflow handlers
    "Handler",
    "HandlerResult",
    "HandlerDispatchError",
    "KNOWN_PATTERNS",
    "pick_handler",
    "pattern_of",
    "LinearHandler",
    "TimeCriticalHandler",
    "PerInstanceHandler",
    "MultiOutputHandler",
    "SequentialApprovalHandler",
    "PersonaPairHandler",
]
