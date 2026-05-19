"""harness_signals — per-signal evaluation functions for FR-CUO-200.

Every SKILL.md declares a `self_audit.anomaly_signals` block + a
`human_fine_tune.signals_to_initiate` list. This module evaluates each
declared signal against a set of windowed audit rows and returns a
(tripped, value, evidence) tuple.

Each signal function MUST:
  * Take (rows: list[dict], threshold: dict) — `rows` is pre-filtered to
    the analysis window + scope (skill or workflow); `threshold` is the
    raw frontmatter value (may be a dict with `threshold`/`window`/etc.
    keys or a bare numeric).
  * Return (tripped: bool, value: float, evidence_rows: list[dict]) —
    `evidence_rows` is the subset of `rows` that contributed to the value.

Signal taxonomy (per SKILL.md frontmatter schema):

  - confidence_low_streak       — N consecutive rows below confidence X
  - user_correction_streak      — N consecutive rows with operator override
  - rule_reversal_streak        — N consecutive runs where the audit verdict
                                  reversed (PASS → FAIL or vice versa)
  - needs_human_rate_above      — fraction of rows with needs_human verdict
                                  exceeds X over the window
  - deterministic_drift         — same input produced different output ≥ N
                                  times (only for skills with determinism:reproducible:true)
  - acceptance_rate_below       — fraction of rows with status=done divided
                                  by total terminal-status rows falls below X
  - hitl_pause_rate_above       — fraction of workflow runs whose outcome was
                                  HITL_HALT exceeds X
  - drift_signal_count_above    — total count of "drift" signals (across all
                                  types) exceeds N

Added 2026-05-19 for FR-CUO-200.
"""

from __future__ import annotations

from typing import Any


def _to_threshold(spec: Any, key: str = "threshold") -> float:
    """Extract the numeric threshold from a YAML threshold spec.

    Spec shapes:
      * bare number → that number
      * dict with key → spec[key]
      * dict-style `{threshold: N, window: M}` (common) → spec['threshold']
    Default key 'threshold' covers most cases; the caller can pass 'window'
    for streak-length lookups.
    """
    if isinstance(spec, (int, float)):
        return float(spec)
    if isinstance(spec, dict) and key in spec:
        return float(spec[key])
    return 0.0


def confidence_low_streak(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Count consecutive rows with confidence below the low threshold.

    threshold spec: `{threshold: N, window: M}` — N = streak length needed
    to trip; M = how far back to consider (ignored here, caller pre-filters).
    """
    streak_required = int(_to_threshold(threshold, "threshold"))
    if streak_required <= 0:
        return (False, 0.0, [])

    streak = 0
    max_streak = 0
    evidence: list[dict] = []
    current_streak_evidence: list[dict] = []
    confidence_floor = 0.5  # constant — "low confidence" semantics

    for row in rows:
        conf = _row_confidence(row)
        if conf is not None and conf < confidence_floor:
            streak += 1
            current_streak_evidence.append(row)
            if streak > max_streak:
                max_streak = streak
                evidence = list(current_streak_evidence)
        else:
            streak = 0
            current_streak_evidence = []

    return (max_streak >= streak_required, float(max_streak), evidence)


def user_correction_streak(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Count consecutive rows where the operator corrected output.

    A "correction" row is `op == "view"` AND `extra.correction_to` is set —
    a memory write that supersedes a prior agent-emitted row.
    """
    streak_required = int(_to_threshold(threshold, "threshold"))
    if streak_required <= 0:
        return (False, 0.0, [])

    streak = 0
    max_streak = 0
    evidence: list[dict] = []
    current: list[dict] = []
    for row in rows:
        extra = row.get("extra") or {}
        is_correction = row.get("op") == "view" and "correction_to" in extra
        if is_correction:
            streak += 1
            current.append(row)
            if streak > max_streak:
                max_streak = streak
                evidence = list(current)
        else:
            streak = 0
            current = []
    return (max_streak >= streak_required, float(max_streak), evidence)


def rule_reversal_streak(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Detect audit verdict reversals (PASS ↔ FAIL on the same skill)."""
    streak_required = int(_to_threshold(threshold, "threshold"))
    if streak_required <= 0:
        return (False, 0.0, [])

    audit_rows = [r for r in rows if (r.get("extra") or {}).get("audit_verdict")]
    if len(audit_rows) < 2:
        return (False, 0.0, [])

    reversals = 0
    evidence: list[dict] = []
    prev_verdict = None
    for r in audit_rows:
        verdict = (r.get("extra") or {}).get("audit_verdict")
        if prev_verdict is not None and verdict != prev_verdict:
            reversals += 1
            evidence.append(r)
        prev_verdict = verdict
    return (reversals >= streak_required, float(reversals), evidence)


def needs_human_rate_above(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Fraction of rows tagged as `needs_human` exceeds threshold."""
    t = _to_threshold(threshold, "threshold")
    if t <= 0 or not rows:
        return (False, 0.0, [])
    needs_human = [
        r for r in rows
        if (r.get("extra") or {}).get("audit_verdict") == "needs_human"
        or (r.get("extra") or {}).get("hitl_required") is True
    ]
    rate = len(needs_human) / len(rows)
    return (rate > t, rate, needs_human)


def deterministic_drift(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Count distinct outputs for the same (skill, input_hash) — drift."""
    t = int(_to_threshold(threshold, "threshold"))
    if t <= 0:
        return (False, 0.0, [])

    seen: dict[tuple, set] = {}
    evidence: list[dict] = []
    for r in rows:
        extra = r.get("extra") or {}
        skill = extra.get("skill") or r.get("path", "")
        input_hash = extra.get("input_hash")
        output_hash = extra.get("output_hash")
        if not (input_hash and output_hash):
            continue
        key = (skill, input_hash)
        outputs = seen.setdefault(key, set())
        if output_hash not in outputs and outputs:
            evidence.append(r)
        outputs.add(output_hash)

    drift_count = len(evidence)
    return (drift_count >= t, float(drift_count), evidence)


def acceptance_rate_below(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Fraction of FR runs that reached `done` divided by total terminal runs
    falls below threshold. Inverse-direction signal: trip when value is below.
    """
    t = _to_threshold(threshold, "threshold")
    terminal = [
        r for r in rows
        if (r.get("extra") or {}).get("outcome") in (
            "done", "ROUTED_BACK", "HITL_HALT", "FAILED",
        ) or r.get("op") in ("memory.fr_routed_back",)
    ]
    if not terminal:
        return (False, 0.0, [])
    done = [
        r for r in terminal
        if (r.get("extra") or {}).get("outcome") == "done"
    ]
    rate = len(done) / len(terminal)
    # Evidence = the non-done terminals (the failures dragging the rate down).
    evidence = [r for r in terminal if r not in done]
    return (rate < t, rate, evidence)


def hitl_pause_rate_above(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Fraction of workflow runs whose outcome was HITL_HALT exceeds threshold."""
    t = _to_threshold(threshold, "threshold")
    workflow_runs = [r for r in rows if r.get("kind") == "workflow_complete" or r.get("op") == "workflow_complete"]
    if not workflow_runs:
        return (False, 0.0, [])
    hitl = [
        r for r in workflow_runs
        if (r.get("extra") or {}).get("outcome") == "HITL_HALT"
    ]
    rate = len(hitl) / len(workflow_runs)
    return (rate > t, rate, hitl)


def drift_signal_count_above(rows: list[dict], threshold: dict) -> tuple[bool, float, list[dict]]:
    """Total count of drift-tagged rows (any drift type) exceeds threshold."""
    t = int(_to_threshold(threshold, "threshold"))
    drift_rows = [
        r for r in rows
        if "drift" in (r.get("op", "") or "")
        or (r.get("extra") or {}).get("drift_signal") is not None
    ]
    return (len(drift_rows) >= t, float(len(drift_rows)), drift_rows)


# Lookup table — signal_id → evaluator function. New signals land here.
SIGNAL_EVALUATORS = {
    "confidence_low_streak": confidence_low_streak,
    "user_correction_streak": user_correction_streak,
    "rule_reversal_streak": rule_reversal_streak,
    "needs_human_rate_above": needs_human_rate_above,
    "deterministic_drift": deterministic_drift,
    "acceptance_rate_below": acceptance_rate_below,
    "hitl_pause_rate_above": hitl_pause_rate_above,
    "drift_signal_count_above": drift_signal_count_above,
}


def evaluate_signal(signal_id: str, rows: list[dict], threshold: Any) -> tuple[bool, float, list[dict]]:
    """Dispatch to the correct evaluator. Unknown signals return (False, 0, [])."""
    fn = SIGNAL_EVALUATORS.get(signal_id)
    if fn is None:
        return (False, 0.0, [])
    if isinstance(threshold, (int, float)):
        threshold = {"threshold": threshold}
    return fn(rows, threshold)


def _row_confidence(row: dict) -> float | None:
    """Best-effort extraction of a row's confidence value, if present."""
    extra = row.get("extra") or {}
    for key in ("confidence", "score", "importance"):
        val = extra.get(key)
        if isinstance(val, (int, float)):
            return float(val)
    return None
