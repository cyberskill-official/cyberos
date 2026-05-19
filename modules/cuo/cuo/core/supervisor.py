"""Supervisor — walks workflow chains.

Phase 1: dry-run mode (`dry_run_chain`). Prints what would be invoked without
actually executing.

Phase 2 (added 2026-05-18): actual chain execution via pluggable `Invoker`
(`execute_chain`). Defaults to `MockInvoker` (deterministic placeholder
output) when no `cyberos-skill` binary is available; uses `SubprocessInvoker`
otherwise. Each step's output is persisted to the workflow output dir for
hand-off to the next step.

Phase 3 (added 2026-05-18): emit decision rows to the memory audit chain per
AGENTS.md §6 + §11. The chain output already includes per-step output hashes
ready to be sealed into chain leaves.

Phase 4 (added 2026-05-18): special-case handlers for time-critical /
per-instance / multi-output / sequential-approval / persona-pair workflows.

Phase 5 (added 2026-05-19 — STATUS-WAVE): condition-aware step evaluation +
failure-routing rework branch + observability spans. This is what makes
`chief-technology-officer/ship-feature-requests` v2.0.0 fully usable:

  * Conditional steps (`condition: "mode == \"implement\""`, `condition:
    "step 3 ran"`, etc.) are honoured — steps whose condition evaluates
    False are SKIPPED, not invoked. The hand-off map carries `step_<N>_ran`
    booleans + named field references for downstream conditions.

  * Failure routing — when any step returns FAILED, the supervisor scans
    the chain for a rework branch (the last `backlog-state-update-author`
    step whose `transition` literal starts with "any-stage" OR contains
    "ready_to_implement"). If found, it's invoked with a synthesized
    rework outcome, applying the BACKLOG.md status flip and emitting
    memory.fr_routed_back. The chain outcome becomes ROUTED_BACK.

  * Observability spans — every step invocation emits a structured log
    line `{event: skill.invoke, span_id, step, skill, status, duration_ms}`
    via the `cyberos.cuo.spans` logger. A run-level `fr_routed_back_count`
    is maintained in-process; production OTel exporters tracked under
    FR-OBS-001..003.
"""

from __future__ import annotations

import logging
import os
import re
import secrets
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from cuo.core.catalog import PersonaEntry, WorkflowEntry, discover_workflows
from cuo.core.invoker import Invoker, MockInvoker, StepResult, select_invoker
from cuo.core.validator import ValidationResult, validate_chain


# Structured-logging channel for span events. Production deployments can
# attach a JSON handler + OTel exporter; the default is stderr text.
_SPANS_LOGGER = logging.getLogger("cyberos.cuo.spans")

# In-process counter for rework-routed FRs. Surfaced in ChainResult.notes
# and emitted as a memory.fr_routed_back aux row by emit_chain_result.
_REWORK_COUNTER: dict[str, int] = {"fr_routed_back": 0}


@dataclass
class DryRunResult:
    """Outcome of a dry-run walk through a workflow's skill_chain."""

    workflow_id: str
    validation: ValidationResult
    step_plan: list[str] = field(default_factory=list)
    notes: list[str] = field(default_factory=list)

    @property
    def runnable(self) -> bool:
        """True iff every step's skill is found (no MISSING, no planned: gaps)."""
        return self.validation.valid

    def __repr__(self) -> str:
        flag = "RUNNABLE" if self.runnable else "BLOCKED"
        return f"DryRunResult({self.workflow_id!r}, {flag}, steps={len(self.step_plan)})"


def _find_workflow(persona: PersonaEntry, workflow_slug: str) -> WorkflowEntry | None:
    """Locate a workflow by slug within a persona's workflows folder."""
    for wf in discover_workflows(persona):
        if wf.slug == workflow_slug:
            return wf
    return None


def dry_run_chain(
    persona: PersonaEntry,
    workflow_slug: str,
    skill_root: Path,
    *,
    inputs: dict | None = None,
) -> DryRunResult:
    """Plan (but do not execute) the skill chain for a workflow.

    Args:
        persona: the PersonaEntry owning the workflow.
        workflow_slug: the workflow's slug (filename without `.md`).
        skill_root: path to `skill/` for chain validation.
        inputs: optional dict of workflow-input → file-path bindings. Phase 1
            uses these only for display in the step plan.

    Returns:
        DryRunResult with:
          - validation: per-step skill-existence check
          - step_plan: human-readable list of "step N: would invoke <skill> with inputs_from <X>"
          - notes: surfaced warnings (planned: gaps, missing skills, etc.)
    """
    wf = _find_workflow(persona, workflow_slug)
    if wf is None:
        result = DryRunResult(workflow_id=f"{persona.slug}/{workflow_slug}", validation=ValidationResult(
            workflow_id=f"{persona.slug}/{workflow_slug}", valid=False,
            notes=[f"workflow file not found in {persona.workflows_dir}"]
        ))
        return result

    validation = validate_chain(wf, skill_root)
    result = DryRunResult(workflow_id=wf.workflow_id, validation=validation)

    # Build the step plan — one human-readable line per chain step.
    for step in wf.skill_chain:
        if not isinstance(step, dict):
            result.notes.append(f"malformed step: {step!r}")
            continue
        step_num = step.get("step", "?")
        skill = step.get("skill", "<no skill>")
        inputs_from = step.get("inputs_from", "")
        outputs_to = step.get("outputs_to", "")

        # Decorate skill name with status.
        if isinstance(skill, str) and skill.startswith("planned:"):
            marker = "PLANNED"
        elif isinstance(skill, str) and skill in validation.found_skills:
            marker = "FOUND"
        elif isinstance(skill, str) and skill in validation.missing_skills:
            marker = "MISSING"
        else:
            marker = "UNKNOWN"

        plan_line = f"step {step_num} [{marker}]: would invoke `{skill}`"
        if inputs_from:
            plan_line += f" — inputs_from={inputs_from}"
        if outputs_to:
            plan_line += f" → outputs_to={outputs_to}"
        result.step_plan.append(plan_line)

    # Surface operator-facing notes.
    if validation.missing_skills:
        result.notes.append(
            f"MISSING_SKILL_REQUEST: {len(validation.missing_skills)} skill(s) not in catalog: "
            + ", ".join(validation.missing_skills)
        )
    if validation.planned_skills:
        result.notes.append(
            f"PLANNED gaps: {len(validation.planned_skills)} skill(s) declared planned: per AGENTS.md §0.8: "
            + ", ".join(validation.planned_skills)
        )
    if wf.escalates_to:
        result.notes.append(
            f"escalates_to: {len(wf.escalates_to)} cross-persona escalation declarations — "
            "Phase 4 supervisor will walk these mid-chain"
        )
    if wf.consults:
        result.notes.append(
            f"consults: {len(wf.consults)} cross-persona consult declarations"
        )

    return result


# ---------------------------------------------------------------------------
# Phase 2 — actual chain execution
# ---------------------------------------------------------------------------


@dataclass
class ChainResult:
    """Outcome of executing a workflow's full chain (Phase 2)."""

    workflow_id: str
    outcome: str  # "COMPLETED" | "HALTED_HITL" | "FAILED" | "PARTIAL" | "BLOCKED"
    validation: ValidationResult
    step_results: list[StepResult] = field(default_factory=list)
    output_dir: Path | None = None
    invoker_kind: str = ""
    notes: list[str] = field(default_factory=list)

    @property
    def total_duration_ms(self) -> int:
        return sum(s.duration_ms for s in self.step_results)

    def __repr__(self) -> str:
        return (
            f"ChainResult({self.workflow_id!r}, {self.outcome}, "
            f"{len(self.step_results)} steps, {self.total_duration_ms}ms)"
        )


def execute_chain(
    persona: PersonaEntry,
    workflow_slug: str,
    skill_root: Path,
    output_dir: Path,
    *,
    inputs: dict | None = None,
    invoker: Invoker | None = None,
    stop_on_failure: bool = True,
) -> ChainResult:
    """Execute (not just plan) the skill chain for a workflow.

    Walks each step in `workflow.skill_chain`, invoking the named skill via
    the supplied `invoker`. Persists per-step output to `output_dir/stepNN_<skill>.json`
    so the next step's `inputs_from` references can pick them up.

    Args:
        persona: the PersonaEntry owning the workflow.
        workflow_slug: workflow's slug (filename without `.md`).
        skill_root: path to `skill/` for validation + invocation.
        output_dir: directory where per-step output JSON files are written.
        inputs: optional dict of initial workflow inputs (passed to step 1).
        invoker: Invoker instance; defaults to `select_invoker("auto")`.
        stop_on_failure: if True (default), abort the chain on first FAILED step.
            If False, continue and collect all results (useful for triage).

    Returns:
        ChainResult with per-step outcomes + overall status. Validates the
        chain first; if validation fails (MISSING or PLANNED skills), returns
        outcome=BLOCKED without invoking anything.
    """
    if invoker is None:
        invoker = select_invoker("auto")
    invoker_kind = type(invoker).__name__

    wf = _find_workflow(persona, workflow_slug)
    if wf is None:
        validation = ValidationResult(
            workflow_id=f"{persona.slug}/{workflow_slug}",
            valid=False,
            notes=[f"workflow file not found in {persona.workflows_dir}"],
        )
        return ChainResult(
            workflow_id=validation.workflow_id,
            outcome="BLOCKED",
            validation=validation,
            output_dir=output_dir,
            invoker_kind=invoker_kind,
        )

    validation = validate_chain(wf, skill_root)
    if not validation.valid:
        # MISSING or PLANNED gap — refuse to execute per AGENTS.md §1.4
        # (emit MISSING_SKILL_REQUEST equivalent).
        return ChainResult(
            workflow_id=wf.workflow_id,
            outcome="BLOCKED",
            validation=validation,
            output_dir=output_dir,
            invoker_kind=invoker_kind,
            notes=[
                f"validation failed: {len(validation.missing_skills)} missing + "
                f"{len(validation.planned_skills)} planned. Refusing to execute."
            ],
        )

    output_dir.mkdir(parents=True, exist_ok=True)

    # Build a hand-off map: `inputs_from` references resolve from this map.
    # Initial inputs (passed by caller) seed it; step outputs populate the rest.
    # The map also tracks "step_<N>_ran" booleans so downstream `condition:`
    # clauses like `"step 3 ran"` can be evaluated (Phase 5).
    hand_off: dict = dict(inputs or {})
    step_results: list[StepResult] = []
    outcome = "COMPLETED"

    # Run-level span id for log correlation (Phase 5 observability).
    run_span_id = secrets.token_hex(6)
    _SPANS_LOGGER.info(
        "workflow.start",
        extra={
            "event": "workflow.start", "span_id": run_span_id,
            "workflow_id": wf.workflow_id, "invoker": invoker_kind,
            "input_keys": sorted(hand_off.keys()),
        },
    )

    for step_spec in wf.skill_chain:
        if not isinstance(step_spec, dict):
            continue
        step_num = step_spec.get("step", len(step_results) + 1)
        skill_name = step_spec.get("skill", "")
        if not isinstance(skill_name, str) or not skill_name or skill_name.startswith("planned:"):
            continue

        # ── Phase 5: condition evaluation ────────────────────────────────
        # Steps with `condition: "<expr>"` are SKIPPED when the expression
        # evaluates to False against the running hand-off map. Expressions
        # reference: prior steps (`"step 3 ran"`), workflow inputs
        # (`'mode == "implement"'`), or step output fields
        # (`"context_map.files_outside_immediate_domain > 3"`).
        condition = step_spec.get("condition")
        if condition and not _eval_condition(condition, hand_off, step_results):
            skipped = StepResult(
                step=step_num, skill=skill_name, status="SKIPPED",
                notes=[f"condition false: {condition!r}"],
            )
            step_results.append(skipped)
            hand_off[f"step_{step_num}_ran"] = False
            _SPANS_LOGGER.info(
                "skill.skip",
                extra={
                    "event": "skill.skip", "span_id": run_span_id,
                    "step": step_num, "skill": skill_name,
                    "condition": str(condition),
                },
            )
            continue

        # Resolve inputs for this step from the hand-off map.
        step_inputs = _resolve_step_inputs(step_spec.get("inputs_from"), hand_off)

        # ── Phase 5: span emission ───────────────────────────────────────
        step_span_id = secrets.token_hex(4)
        t_step_0 = time.monotonic_ns()
        _SPANS_LOGGER.info(
            "skill.invoke",
            extra={
                "event": "skill.invoke", "span_id": run_span_id,
                "step_span_id": step_span_id, "step": step_num,
                "skill": skill_name, "input_keys": sorted(step_inputs.keys()),
            },
        )

        step_result = invoker.invoke(
            skill_name=skill_name,
            inputs=step_inputs,
            skill_root=skill_root,
            output_dir=output_dir,
            step_num=step_num,
        )
        step_results.append(step_result)

        _SPANS_LOGGER.info(
            "skill.complete",
            extra={
                "event": "skill.complete", "span_id": run_span_id,
                "step_span_id": step_span_id, "step": step_num,
                "skill": skill_name, "status": step_result.status,
                "duration_ms": step_result.duration_ms,
            },
        )

        # Mark this step as ran (regardless of OK/FAILED — the work was attempted).
        hand_off[f"step_{step_num}_ran"] = step_result.status in ("OK", "MOCKED")

        # ── Phase 5: post-author appliers ────────────────────────────────
        # For skills with file-side-effect contracts (backlog-state-update-author,
        # coverage-gate-author), the LLM's JSON output describes WHAT to do; we
        # actually APPLY it here. This bridges the LLM-prompt-only architecture
        # to real filesystem / subprocess work without giving the LLM tool access.
        if step_result.status in ("OK", "MOCKED"):
            try:
                from cuo.core.applier import apply_step_side_effect
                apply_step_side_effect(skill_name, step_result, hand_off, run_span_id)
            except ImportError:
                pass  # applier module is optional; absent → no side effects

        # Populate the hand-off map with this step's output (named by outputs_to).
        outputs_to = step_spec.get("outputs_to")
        if outputs_to and step_result.status in ("OK", "MOCKED"):
            if isinstance(outputs_to, str):
                hand_off[outputs_to] = step_result.output_path or step_result.output
            elif isinstance(outputs_to, dict):
                # Some workflows declare multi-output: outputs_to: {name1: ..., name2: ...}
                # Map each to the same step output for now (single artefact, multiple refs).
                for ref in outputs_to.values():
                    if isinstance(ref, str):
                        hand_off[ref] = step_result.output_path or step_result.output

        # ── Phase 5: HITL escalation signal ──────────────────────────────
        # Any step's LLM output (or applier amendment) MAY include a
        # top-level `escalation_signal` or `hitl_required: true` field.
        # When present and truthy, the supervisor halts the chain with
        # outcome=HITL_HALT and the drain loop stops (vs. ROUTED_BACK
        # which is a soft "try again" signal).
        step_output = step_result.output if isinstance(step_result.output, dict) else {}
        if step_output.get("hitl_required") or step_output.get("escalation_signal"):
            esc_reason = (
                step_output.get("escalation_signal")
                or step_output.get("hitl_reason")
                or f"step {step_num} signalled hitl_required"
            )
            _SPANS_LOGGER.warning(
                "hitl.halt",
                extra={
                    "event": "hitl.halt", "span_id": run_span_id,
                    "step": step_num, "skill": skill_name,
                    "escalation_reason": str(esc_reason),
                },
            )
            outcome = "HITL_HALT"
            step_result.notes.append(f"hitl-halt: {esc_reason}")
            break

        if step_result.status == "FAILED":
            outcome = "FAILED" if stop_on_failure else "PARTIAL"
            if stop_on_failure:
                break

    # ── Phase 5: failure → rework branch ──────────────────────────────────
    # When the forward path failed AND the workflow chain contains at least
    # one backlog-state-update-author step (i.e. it's a lifecycle workflow,
    # not just any chain), synthesize a rework call. The synthesized invocation
    # passes transition_kind=rework + the failure reason; the applier picks
    # this up and flips BACKLOG.md status back to ready_to_implement.
    if outcome == "FAILED" and _chain_has_backlog_update(wf):
        # The rework call doesn't reuse a step from the workflow — it's a
        # synthesized invocation triggered by any forward-path failure.
        rework_inputs = _build_rework_inputs_from_failure(
            step_results, hand_off,
        )
        rework_span_id = secrets.token_hex(4)
        _SPANS_LOGGER.info(
            "rework.branch",
            extra={
                "event": "rework.branch", "span_id": run_span_id,
                "step_span_id": rework_span_id,
                "rework_reason": rework_inputs.get("rework_reason"),
                "fr_id": rework_inputs.get("fr_id"),
            },
        )
        rework_result = invoker.invoke(
            skill_name="backlog-state-update-author",
            inputs=rework_inputs,
            skill_root=skill_root,
            output_dir=output_dir,
            step_num=99,  # synthesized; not a real chain step
        )
        step_results.append(rework_result)
        if rework_result.status in ("OK", "MOCKED"):
            try:
                from cuo.core.applier import apply_step_side_effect
                # Inject the rework metadata into the result so the applier
                # sees `transition_kind: rework` even when the mock invoker
                # returns generic mock output.
                if isinstance(rework_result.output, dict):
                    rework_result.output.setdefault("fr_id", rework_inputs.get("fr_id"))
                    rework_result.output.setdefault("new_status", "ready_to_implement")
                    rework_result.output.setdefault("transition_kind", "rework")
                    rework_result.output.setdefault("rework_reason",
                                                   rework_inputs.get("rework_reason", ""))
                apply_step_side_effect(
                    "backlog-state-update-author", rework_result,
                    hand_off, run_span_id,
                )
            except ImportError:
                pass
            _REWORK_COUNTER["fr_routed_back"] += 1
            outcome = "ROUTED_BACK"

    if outcome == "COMPLETED" and any(s.status == "FAILED" for s in step_results):
        outcome = "PARTIAL"

    _SPANS_LOGGER.info(
        "workflow.complete",
        extra={
            "event": "workflow.complete", "span_id": run_span_id,
            "workflow_id": wf.workflow_id, "outcome": outcome,
            "steps_run": sum(1 for s in step_results if s.status != "SKIPPED"),
            "steps_skipped": sum(1 for s in step_results if s.status == "SKIPPED"),
            "fr_routed_back_count": _REWORK_COUNTER["fr_routed_back"],
        },
    )

    return ChainResult(
        workflow_id=wf.workflow_id,
        outcome=outcome,
        validation=validation,
        step_results=step_results,
        output_dir=output_dir,
        invoker_kind=invoker_kind,
    )


# ---------------------------------------------------------------------------
# Phase 5 helpers — condition eval, rework branch detection, span emission
# ---------------------------------------------------------------------------


def _eval_condition(expr: str, hand_off: dict, step_results: list[StepResult]) -> bool:
    """Evaluate a workflow-step `condition:` expression against the hand-off map.

    Supported forms (from ship-feature-requests + sibling workflows):
      * `"step N ran"` → True iff step N completed with status in {OK, MOCKED}
      * `'mode == "implement"'` → standard Python comparison against hand_off['mode']
      * `"<field>.<subfield> > <value>"` → simple comparison; resolves dotted
        attributes against hand_off (e.g. context_map.files_outside_immediate_domain > 3)
      * Combinations via `and` / `or` (literal Python boolean operators).
      * Returns True (= run the step) when the expression is malformed —
        better to run unconditionally than to skip silently.
    """
    if not isinstance(expr, str) or not expr.strip():
        return True
    expr = expr.strip()

    # Fast-path: `"step N ran"` literal
    m = re.fullmatch(r"step\s+(\d+)\s+ran", expr)
    if m:
        target = int(m.group(1))
        return hand_off.get(f"step_{target}_ran", False) is True

    # Build a restricted eval namespace.
    safe_globals = {"__builtins__": {}}
    safe_locals: dict[str, Any] = {}

    # Workflow inputs land directly as locals (e.g. `mode`).
    for k, v in hand_off.items():
        if isinstance(k, str) and k.isidentifier():
            safe_locals[k] = v

    # `step_N_ran` booleans land too.
    for sr in step_results:
        safe_locals[f"step_{sr.step}_ran"] = (sr.status in ("OK", "MOCKED"))

    # Dotted-field access (e.g. `context_map.files_outside_immediate_domain`) —
    # the LHS resolves via the hand-off map's stored dict outputs. Wrap them
    # in a tiny attr-access shim.
    class _AttrDict:
        def __init__(self, payload: Any):
            self._payload = payload
        def __getattr__(self, name: str) -> Any:
            if isinstance(self._payload, dict):
                if name in self._payload:
                    val = self._payload[name]
                    return _AttrDict(val) if isinstance(val, (dict, list)) else val
            return None  # missing attribute → falsy

    for k, v in list(safe_locals.items()):
        if isinstance(v, (dict, list)):
            safe_locals[k] = _AttrDict(v)

    try:
        return bool(eval(expr, safe_globals, safe_locals))  # noqa: S307
    except Exception:  # noqa: BLE001
        # Malformed condition → run the step (safer than silent skip).
        return True


def _chain_has_backlog_update(wf: WorkflowEntry) -> bool:
    """True if the workflow's skill_chain contains at least one
    backlog-state-update-author step — used to gate rework synthesis."""
    for step_spec in wf.skill_chain:
        if isinstance(step_spec, dict) and step_spec.get("skill") == "backlog-state-update-author":
            return True
    return False


def _build_rework_inputs_from_failure(
    step_results: list[StepResult],
    hand_off: dict,
) -> dict:
    """Synthesize the rework call's input bundle from forward-path failure.

    Returns a dict carrying:
      * `fr_id` — the FR being shipped (from hand_off; falls back to "unknown").
      * `transition_kind: "rework"` — signals the applier to flip status
        to ready_to_implement and increment routed_back_count.
      * `new_status: "ready_to_implement"` — the target lifecycle state.
      * `rework_reason` — string built from the failing step(s) for the
        memory.fr_routed_back aux row payload.
      * `outcome` — the last failed StepResult's output, so the LLM/applier
        knows which artefact caused the rework.
      * `synthesized_rework: True` — distinguishes from natural-flow rework
        steps (none exist today, but future workflows may declare them).
    """
    failed_steps = [s for s in step_results if s.status == "FAILED"]
    failed_skill = failed_steps[-1].skill if failed_steps else "unknown"
    failed_notes = ", ".join(failed_steps[-1].notes[:2]) if failed_steps else ""
    rework_reason = (
        f"forward path failed at step '{failed_skill}'"
        + (f": {failed_notes}" if failed_notes else "")
    )

    return {
        "fr_id": hand_off.get("fr_id", "unknown"),
        "transition_kind": "rework",
        "new_status": "ready_to_implement",
        "rework_reason": rework_reason,
        "outcome": failed_steps[-1].output if failed_steps else {},
        "synthesized_rework": True,
    }


def get_rework_counter() -> int:
    """Return the in-process count of rework-routed FRs since process start.

    Surfaced via `cyberos-cuo execute --explain` and emitted as a
    memory.fr_routed_back_count aux row by emit_chain_result.
    """
    return _REWORK_COUNTER["fr_routed_back"]


def reset_rework_counter() -> None:
    """Reset the in-process rework counter — test-only helper."""
    _REWORK_COUNTER["fr_routed_back"] = 0


def _resolve_step_inputs(inputs_from, hand_off: dict) -> dict:
    """Resolve a step's `inputs_from` declaration against the running hand-off map.

    `inputs_from` shapes seen in real workflows:
      - str:  the name of a single upstream output, e.g. "srs_draft"
      - dict: {input_name: upstream_name, ...} — multiple named inputs
      - None: step has no upstream inputs (uses initial workflow inputs)
    """
    if inputs_from is None:
        return dict(hand_off)
    if isinstance(inputs_from, str):
        return {inputs_from: hand_off.get(inputs_from)} if inputs_from in hand_off else {}
    if isinstance(inputs_from, dict):
        resolved: dict = {}
        for input_name, upstream_name in inputs_from.items():
            if isinstance(upstream_name, str) and upstream_name in hand_off:
                resolved[input_name] = hand_off[upstream_name]
            else:
                resolved[input_name] = upstream_name
        return resolved
    return dict(hand_off)
