"""workflow_evolution — TASK-CUO-203 Wave 4 of the harness.

Aggregates per-workflow outcome distribution across the memory audit chain
window, trips workflow-level threshold signals (`routed_back_rate_above`,
`hitl_halt_rate_above`, `repeat_phase_failure_above`, `chain_length_efficiency_below`),
emits workflow_refinement proposals through the same TASK-CUO-201 stripe machinery.

Workflow stripe format: `<persona>/<workflow_slug>:<signal_id>:<8-hex>` —
disjoint from skill stripes (`<skill>:<signal>:<hex>`) by the `/` separator.
"""

from __future__ import annotations

import hashlib
import json
import re
from dataclasses import dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path
from typing import Optional

from cuo.core.harness import WorkflowMetrics
from cuo.core.refinement_proposal import (
    Emitted,
    EmissionResult,
    StripeRepeatHalt,
    emit_or_halt,
)


# Default workflow-level threshold signals (mirroring SKILL.md's self_audit.anomaly_signals).
DEFAULT_WORKFLOW_SIGNALS = {
    "routed_back_rate_above": {"threshold": 0.3},
    "hitl_halt_rate_above": {"threshold": 0.1},
    "repeat_phase_failure_above": {"threshold": 3},
    "chain_length_efficiency_below": {"threshold": 0.7},
}


@dataclass
class WorkflowEvolutionReport:
    """Aggregate metrics + list of workflows whose signals tripped."""
    workflow_metrics: dict[str, WorkflowMetrics] = field(default_factory=dict)
    tripped: list[tuple[str, str, float, float]] = field(default_factory=list)
    # tripped entries are (workflow_id, signal_id, value, threshold)

    @property
    def has_tripped_signals(self) -> bool:
        return bool(self.tripped)


def compute_workflow_metrics(
    rows: list[dict],
    *,
    workflow_filter: Optional[str] = None,
) -> dict[str, WorkflowMetrics]:
    """Aggregate audit rows into per-workflow outcome distributions.

    Inputs: a list of `workflow_complete` rows (or compatible). Each row's
    `extra.workflow_id` keys the bucket; `extra.outcome` increments the
    appropriate counter.
    """
    out: dict[str, WorkflowMetrics] = {}
    for r in rows:
        op = r.get("op", "")
        extra = r.get("extra") or {}
        if op != "workflow_complete":
            continue
        wf_id = extra.get("workflow_id")
        if not wf_id:
            continue
        if workflow_filter and wf_id != workflow_filter:
            continue
        wm = out.setdefault(wf_id, WorkflowMetrics(workflow_id=wf_id))
        wm.total_runs += 1
        outcome = extra.get("outcome", "")
        if outcome in ("COMPLETED", "done"):
            wm.completed += 1
        elif outcome == "ROUTED_BACK":
            wm.routed_back += 1
        elif outcome == "HITL_HALT":
            wm.hitl_halt += 1
        elif outcome == "FAILED":
            wm.failed += 1
    return out


def evaluate_workflow_signals(
    metrics: dict[str, WorkflowMetrics],
    rows: list[dict],
    signals: Optional[dict] = None,
) -> list[tuple[str, str, float, float, list[dict]]]:
    """For each workflow + each declared signal, evaluate and return tripped breaches.

    Returns list of (workflow_id, signal_id, value, threshold, evidence_rows).
    """
    if signals is None:
        signals = DEFAULT_WORKFLOW_SIGNALS
    tripped: list[tuple[str, str, float, float, list[dict]]] = []

    for wf_id, wm in metrics.items():
        for sig_id, spec in signals.items():
            threshold = float(spec.get("threshold", 0))
            value, evidence = _eval_workflow_signal(sig_id, wm, rows, wf_id)
            tripped_now = _check_threshold(sig_id, value, threshold)
            if tripped_now:
                tripped.append((wf_id, sig_id, value, threshold, evidence))
    return tripped


def _eval_workflow_signal(
    sig_id: str,
    wm: WorkflowMetrics,
    rows: list[dict],
    wf_id: str,
) -> tuple[float, list[dict]]:
    """Compute the (value, evidence) for one workflow + one signal."""
    if sig_id == "routed_back_rate_above":
        value = wm.rework_rate
        evidence = [
            r for r in rows
            if (r.get("extra") or {}).get("workflow_id") == wf_id
            and (r.get("extra") or {}).get("outcome") == "ROUTED_BACK"
        ]
        return value, evidence
    if sig_id == "hitl_halt_rate_above":
        if wm.total_runs == 0:
            return 0.0, []
        value = wm.hitl_halt / wm.total_runs
        evidence = [
            r for r in rows
            if (r.get("extra") or {}).get("workflow_id") == wf_id
            and (r.get("extra") or {}).get("outcome") == "HITL_HALT"
        ]
        return value, evidence
    if sig_id == "repeat_phase_failure_above":
        # Count distinct FRs that failed at the same phase across this workflow.
        phase_failures: dict[str, set] = {}
        for r in rows:
            extra = r.get("extra") or {}
            if extra.get("workflow_id") != wf_id:
                continue
            if extra.get("outcome") not in ("ROUTED_BACK", "FAILED"):
                continue
            phase = extra.get("failed_phase") or extra.get("rework_phase", "")
            if not phase:
                continue
            phase_failures.setdefault(phase, set()).add(extra.get("task_id", ""))
        max_distinct = max((len(s) for s in phase_failures.values()), default=0)
        evidence = [
            r for r in rows
            if (r.get("extra") or {}).get("workflow_id") == wf_id
            and (r.get("extra") or {}).get("outcome") in ("ROUTED_BACK", "FAILED")
        ]
        return float(max_distinct), evidence
    if sig_id == "chain_length_efficiency_below":
        # Average steps_run / chain_length across failed runs in this workflow.
        # Lower-is-worse signal — trip when value is LOW.
        runs = [
            r for r in rows
            if (r.get("extra") or {}).get("workflow_id") == wf_id
            and (r.get("extra") or {}).get("outcome") in ("ROUTED_BACK", "FAILED")
        ]
        if not runs:
            return 1.0, []
        ratios = [
            (r.get("extra") or {}).get("steps_run", 0)
            / max(1, (r.get("extra") or {}).get("chain_length", 1))
            for r in runs
        ]
        return sum(ratios) / len(ratios), runs
    return 0.0, []


def _check_threshold(sig_id: str, value: float, threshold: float) -> bool:
    """Direction-aware threshold comparison."""
    # `*_below` signals trip when value < threshold (lower-is-worse)
    if sig_id.endswith("_below"):
        return value < threshold
    # `*_above` signals trip when value > threshold (higher-is-worse)
    return value > threshold


def compute_workflow_stripe(
    workflow_id: str,
    signal_id: str,
    failure_pattern: list[dict],
) -> str:
    """Stripe id format: `<persona>/<workflow_slug>:<signal_id>:<8-hex>`.

    workflow_id is expected to be in the form `<persona>/<workflow_slug>`.
    The pattern_hash projects the failure pattern (set of phase names,
    rework reasons, etc.) into a deterministic 8-hex digest.
    """
    projection = sorted({
        ((r.get("extra") or {}).get("failed_phase", "")
         or (r.get("extra") or {}).get("rework_phase", ""),
         (r.get("extra") or {}).get("rework_reason", "")[:50])
        for r in failure_pattern
    })
    canon = json.dumps(projection, sort_keys=True, separators=(",", ":"))
    digest = hashlib.sha256(canon.encode("utf-8")).hexdigest()[:8]
    return f"{workflow_id}:{signal_id}:{digest}"


def emit_workflow_proposal(
    workflow_id: str,
    signal_id: str,
    value: float,
    threshold: float,
    evidence_rows: list[dict],
    proposals_root: Path,
    *,
    risk_class: str = "minor",
    memory_root: Optional[Path] = None,
) -> EmissionResult:
    """Emit a workflow_refinement proposal through the TASK-CUO-201 emitter.

    Reuses emit_or_halt with kind=workflow_refinement. The stripe is
    workflow-shaped (`<persona>/<wf>:<sig>:<hex>`) so it's disjoint from
    skill stripes.
    """
    suggested_change = (
        f"Workflow `{workflow_id}` tripped `{signal_id}` "
        f"(value={value:.3f}, threshold={threshold:.3f}). "
        f"Consider editing the `skill_chain:` block to address the failing "
        f"phase. Default classifier bucket: see TASK-CUO-203 §1 #6."
    )

    # Build the synthetic stripe id we want to use
    stripe_id = compute_workflow_stripe(workflow_id, signal_id, evidence_rows)

    return emit_or_halt(
        skill_name=workflow_id,  # acts as `scope` in stripe id
        signal_id=signal_id,
        evidence_rows=evidence_rows,
        proposals_root=proposals_root,
        risk_class=risk_class,
        suggested_change=suggested_change,
        kind="workflow_refinement",
        memory_root=memory_root,
        actor="cuo-workflow-harness",
    )
