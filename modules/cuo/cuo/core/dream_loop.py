"""dream_loop - FR-CUO-204 idle-time autonomous evolution, fenced by the evolution envelope.

When the machine is idle and the loop is explicitly enabled, this runs the existing FR-CUO-201/202/203
propose cycle against the golden sets and applies ONLY changes that clear all three gates:

  1. in-envelope - the target path is on the allowlist and off the denylist (`evolution_envelope`);
  2. low-risk    - the content classifier auto-applies it and it is not a safety change (`proposal_applier`);
  3. gate-green  - the AWH test gate passes (run inside the apply step).

Anything that fails a gate - a denylisted target, a major or safety change, a red gate - is recorded as a
human-in-the-loop halt and is NOT applied. The loop is DISABLED BY DEFAULT (the envelope's `enabled` flag
is false and a kill env overrides it), bounded per window (max changes and max wall-clock), and every
action emits a memory audit row.

The dependencies are injected so the safety logic is deterministically testable without a live harness:

  propose_fn()        -> iterable of proposals, each exposing `.target` (a path string)
  classify_fn(prop)   -> object with `.will_auto_apply` (bool) and `.risk_class` (str)
  apply_fn(prop)      -> object with `.outcome` in {AUTO_APPLIED, QUEUED, TEST_GATE_FAILED, ERRORED}
  idle_fn()           -> bool (True when no human is active)
  audit_fn(kind, body)-> None (defaults to a no-op; production binds the memory-audit emitter)
  now_fn()            -> float monotonic seconds (defaults to time.monotonic)

Production binds these to the real `cuo.core` functions; tests inject fakes. This module never wires
itself to a scheduler - enabling and scheduling the loop is a deliberate operator action.
"""

from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import Callable, Iterable, Optional

from cuo.core.evolution_envelope import EvolutionEnvelope

# The five memory audit kinds FR-CUO-204 emits.
AUDIT_STARTED = "cuo.dream_started"
AUDIT_PROPOSAL = "cuo.dream_proposal"
AUDIT_APPLIED = "cuo.dream_applied"
AUDIT_HALTED = "cuo.dream_halted_hitl"
AUDIT_REVERTED = "cuo.dream_reverted"


@dataclass
class DreamReport:
    """The outcome of one dream cycle."""

    status: str  # "disabled" | "not_idle" | "ran"
    reason: str = ""
    seen: int = 0
    applied: int = 0
    halted_hitl: int = 0
    gate_failed: int = 0
    reverted: int = 0
    actions: list = field(default_factory=list)  # list of (action, target, reason)
    notes: list = field(default_factory=list)


def run_dream_cycle(
    envelope: EvolutionEnvelope,
    *,
    propose_fn: Callable[[], Iterable],
    classify_fn: Callable[[object], object],
    apply_fn: Callable[[object], object],
    idle_fn: Callable[[], bool],
    audit_fn: Optional[Callable[[str, dict], None]] = None,
    now_fn: Callable[[], float] = time.monotonic,
    env: Optional[dict] = None,
) -> DreamReport:
    """Run one idle-time evolution cycle under the envelope. Applies nothing unless all gates pass."""
    audit = audit_fn or (lambda kind, body: None)

    # Gate 0a: the loop must be explicitly enabled and not kill-switched.
    if not envelope.is_enabled(env):
        return DreamReport(
            status="disabled",
            reason="dream loop disabled (envelope.enabled is false or the kill switch is set)",
        )

    # Gate 0b: only dream when no human is active.
    if not idle_fn():
        return DreamReport(
            status="not_idle",
            reason="system not idle; the loop runs only when no human is active",
        )

    audit(AUDIT_STARTED, {"max_changes": envelope.max_changes_per_window})
    report = DreamReport(status="ran")
    deadline = now_fn() + envelope.max_wall_clock_seconds

    for prop in propose_fn():
        if report.applied >= envelope.max_changes_per_window:
            report.notes.append("max_changes_per_window reached; stopping")
            break
        if now_fn() >= deadline:
            report.notes.append("max_wall_clock_seconds reached; stopping")
            break

        report.seen += 1
        target = str(getattr(prop, "target", "") or "(unknown)")
        audit(AUDIT_PROPOSAL, {"target": target})

        # Gate 1: the envelope (path-based). Denylisted or unrecognised targets halt for a human.
        verdict = envelope.classify_target(target)
        if not verdict.allowed:
            _halt(report, audit, target, verdict.reason, gate="envelope")
            continue

        # Gate 2: the content risk classifier (FR-CUO-202). Only auto-applicable, non-safety changes pass.
        classification = classify_fn(prop)
        will_auto = bool(getattr(classification, "will_auto_apply", False))
        risk_class = str(getattr(classification, "risk_class", "minor"))
        if not will_auto or risk_class == "safety":
            _halt(report, audit, target, f"not low-risk (risk_class={risk_class}); queued for human", gate="risk_class")
            continue

        # Gate 3: the AWH test gate, run inside the applier.
        result = apply_fn(prop)
        outcome = str(getattr(result, "outcome", "ERRORED"))
        if outcome == "AUTO_APPLIED":
            report.applied += 1
            report.actions.append(("applied", target, "all three gates green"))
            audit(AUDIT_APPLIED, {"target": target})
        elif outcome == "TEST_GATE_FAILED":
            report.gate_failed += 1
            report.actions.append(("halted_hitl", target, "test gate failed"))
            audit(AUDIT_HALTED, {"target": target, "reason": "test gate failed", "gate": "test"})
        else:  # QUEUED or ERRORED - never silently applied
            _halt(report, audit, target, f"applier outcome {outcome}", gate="applier")

    return report


def _halt(report: DreamReport, audit, target: str, reason: str, *, gate: str) -> None:
    report.halted_hitl += 1
    report.actions.append(("halted_hitl", target, reason))
    audit(AUDIT_HALTED, {"target": target, "reason": reason, "gate": gate})
