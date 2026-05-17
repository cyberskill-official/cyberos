"""Supervisor — walks workflow chains.

Phase 1: dry-run mode (`dry_run_chain`). Prints what would be invoked without
actually executing.

Phase 2 (added 2026-05-18): actual chain execution via pluggable `Invoker`
(`execute_chain`). Defaults to `MockInvoker` (deterministic placeholder
output) when no `cyberos-skill` binary is available; uses `SubprocessInvoker`
otherwise. Each step's output is persisted to the workflow output dir for
hand-off to the next step.

Phase 3 (planned): emit decision rows to the BRAIN audit chain per AGENTS.md
§6 + §11. The chain output already includes per-step output hashes ready to
be sealed into chain leaves.

Phase 4 (planned): special-case handlers for time-critical / per-instance /
multi-output / sequential-approval / persona-pair workflows.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path

from cuo.core.catalog import PersonaEntry, WorkflowEntry, discover_workflows
from cuo.core.invoker import Invoker, MockInvoker, StepResult, select_invoker
from cuo.core.validator import ValidationResult, validate_chain


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
    hand_off: dict = dict(inputs or {})
    step_results: list[StepResult] = []
    outcome = "COMPLETED"

    for step_spec in wf.skill_chain:
        if not isinstance(step_spec, dict):
            continue
        step_num = step_spec.get("step", len(step_results) + 1)
        skill_name = step_spec.get("skill", "")
        if not isinstance(skill_name, str) or not skill_name or skill_name.startswith("planned:"):
            continue

        # Resolve inputs for this step from the hand-off map.
        step_inputs = _resolve_step_inputs(step_spec.get("inputs_from"), hand_off)

        step_result = invoker.invoke(
            skill_name=skill_name,
            inputs=step_inputs,
            skill_root=skill_root,
            output_dir=output_dir,
            step_num=step_num,
        )
        step_results.append(step_result)

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

        if step_result.status == "FAILED":
            outcome = "FAILED" if stop_on_failure else "PARTIAL"
            if stop_on_failure:
                break

    if outcome == "COMPLETED" and any(s.status == "FAILED" for s in step_results):
        outcome = "PARTIAL"

    return ChainResult(
        workflow_id=wf.workflow_id,
        outcome=outcome,
        validation=validation,
        step_results=step_results,
        output_dir=output_dir,
        invoker_kind=invoker_kind,
    )


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
