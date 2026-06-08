"""State and schema contracts for the FR-CUO-101 routing supervisor."""

from __future__ import annotations

from typing import Any, Literal, NotRequired, TypedDict

from pydantic import BaseModel, ConfigDict, Field, field_validator


CUO_STATE_V = 1
CASCADE_THRESHOLD_LOW = 0.10
CASCADE_THRESHOLD_HIGH = 0.50
ASK_THRESHOLD = 0.70

PathTaken = Literal[
    "auto",
    "ask",
    "cascade",
    "defer",
    "cascade_then_auto",
    "cascade_then_ask",
]


class Candidate(BaseModel):
    """A scored route candidate from the rule scorer or LLM cascade."""

    model_config = ConfigDict(extra="forbid")

    skill_name: str
    confidence: float = Field(ge=0.0, le=1.0)
    arguments: dict[str, Any] = Field(default_factory=dict)
    score_components: dict[str, Any] = Field(default_factory=dict)
    persona_slug: str | None = None
    workflow_slug: str | None = None
    operation: str | None = None


class LlmRoutingPick(BaseModel):
    """Structured-only LLM cascade response."""

    model_config = ConfigDict(extra="forbid")

    skill_name: str
    arguments: dict[str, Any] = Field(default_factory=dict)
    rationale: str = Field(min_length=1, max_length=500)
    confidence: float = Field(ge=0.0, le=1.0)

    @field_validator("arguments")
    @classmethod
    def _arguments_must_be_jsonish(cls, value: dict[str, Any]) -> dict[str, Any]:
        # Pydantic has already rejected non-dict; keep values primitive enough
        # for audit rows and CLI JSON without teaching the supervisor about
        # arbitrary object serialisation.
        def ok(v: Any) -> bool:
            if v is None or isinstance(v, (str, int, float, bool)):
                return True
            if isinstance(v, list):
                return all(ok(x) for x in v)
            if isinstance(v, dict):
                return all(isinstance(k, str) and ok(x) for k, x in v.items())
            return False

        if not ok(value):
            raise ValueError("arguments must be JSON-serialisable primitives")
        return value


class InvocationResult(BaseModel):
    """Captured result from optional skill invocation."""

    model_config = ConfigDict(extra="forbid")

    status: Literal["skipped", "succeeded", "failed", "blocked", "not_configured"]
    stdout: str = ""
    stderr: str = ""
    exit_code: int | None = None
    duration_ms: int = 0


class Transparency(BaseModel):
    """EU AI Act Art. 13 disclosure shape returned to callers."""

    model_config = ConfigDict(extra="forbid")

    skill_chosen: str | None
    confidence: float
    alternatives: list[dict[str, Any]]
    path_taken: PathTaken
    llm_used: bool


class RoutingDecision(BaseModel):
    """Public return value from the supervisor route call."""

    model_config = ConfigDict(extra="forbid")

    routed: bool
    skill_name: str | None = None
    arguments: dict[str, Any] = Field(default_factory=dict)
    confidence: float = Field(ge=0.0, le=1.0)
    path_taken: PathTaken
    alternatives: list[Candidate] = Field(default_factory=list)
    reason: str | None = None
    llm_used: bool = False
    invocation_result: InvocationResult | None = None
    transparency: Transparency
    next_step: None = None
    request_id: str
    trace_id: str


class CuoState(TypedDict):
    """LangGraph-compatible state dictionary.

    The FR-CUO-101 state is deliberately plain JSON-compatible data so it can
    be checkpointed in memory now and Postgres in FR-CUO-102.
    """

    cuo_state_v: int
    query: str
    normalized_query: str
    tenant_id: str
    subject_id: str
    persona_key: str
    persona_version: str
    request_id: str
    trace_id: str
    ts_ns_start: int
    cascade_taken: bool
    rule_scores: list[Candidate]
    path_taken: NotRequired[PathTaken]
    llm_pick: NotRequired[LlmRoutingPick]
    invocation_result: NotRequired[InvocationResult]
    audit_rows: list[dict[str, Any]]
    errors: list[str]
