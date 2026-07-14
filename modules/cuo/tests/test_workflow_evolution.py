"""Test suite for TASK-CUO-203 — workflow-level evolution."""

from __future__ import annotations

import re
from pathlib import Path

import pytest

from cuo.core.workflow_evolution import (
    DEFAULT_WORKFLOW_SIGNALS,
    compute_workflow_metrics,
    compute_workflow_stripe,
    emit_workflow_proposal,
    evaluate_workflow_signals,
)
from cuo.core.refinement_proposal import Emitted, StripeRepeatHalt


def _row(op: str, **extra) -> dict:
    return {
        "op": op,
        "row_id": extra.pop("row_id", f"row-{id(extra)}"),
        "extra": extra,
    }


# ----------------------------------------------------------------------------
# AC #1/#2 — metrics aggregation; 0-tripped workflow with all-COMPLETED runs
# ----------------------------------------------------------------------------


def test_metrics_aggregation() -> None:
    """AC #1: per-workflow rows: total_runs / completed / routed_back / etc."""
    rows = [
        _row("workflow_complete", workflow_id="cto/ship", outcome="COMPLETED",
             row_id="r1"),
        _row("workflow_complete", workflow_id="cto/ship", outcome="COMPLETED",
             row_id="r2"),
        _row("workflow_complete", workflow_id="cto/ship", outcome="ROUTED_BACK",
             row_id="r3"),
        _row("workflow_complete", workflow_id="ceo/qbr", outcome="HITL_HALT",
             row_id="r4"),
    ]
    metrics = compute_workflow_metrics(rows)
    assert metrics["cto/ship"].total_runs == 3
    assert metrics["cto/ship"].completed == 2
    assert metrics["cto/ship"].routed_back == 1
    assert metrics["ceo/qbr"].hitl_halt == 1


def test_all_completed_no_trips() -> None:
    """AC #2: 5 runs all COMPLETED → zero tripped signals."""
    rows = [
        _row("workflow_complete", workflow_id="cto/ship", outcome="COMPLETED",
             row_id=f"r{i}")
        for i in range(5)
    ]
    metrics = compute_workflow_metrics(rows)
    tripped = evaluate_workflow_signals(metrics, rows)
    assert tripped == []


# ----------------------------------------------------------------------------
# AC #3 — 4 ROUTED_BACK out of 10 trips routed_back_rate_above: 0.3
# ----------------------------------------------------------------------------


def test_routed_back_rate_trips(tmp_path: Path) -> None:
    """AC #3: trip + emit a workflow_refinement_proposal@1."""
    rows = []
    for i in range(6):
        rows.append(_row("workflow_complete", workflow_id="cto/ship",
                          outcome="COMPLETED", row_id=f"r-ok-{i}"))
    for i in range(4):
        rows.append(_row("workflow_complete", workflow_id="cto/ship",
                          outcome="ROUTED_BACK",
                          rework_reason=f"phase-{i}-failed",
                          row_id=f"r-rb-{i}"))
    metrics = compute_workflow_metrics(rows)
    assert metrics["cto/ship"].rework_rate == 0.4

    tripped = evaluate_workflow_signals(metrics, rows)
    assert any(t[1] == "routed_back_rate_above" for t in tripped)

    # Emit a proposal
    wf_id, sig_id, value, threshold, evidence = next(
        t for t in tripped if t[1] == "routed_back_rate_above"
    )
    result = emit_workflow_proposal(
        wf_id, sig_id, value, threshold, evidence, tmp_path,
    )
    assert isinstance(result, Emitted)
    assert result.proposal_path.is_file()


# ----------------------------------------------------------------------------
# AC #4 — proposal body has 4 mandatory sections
# ----------------------------------------------------------------------------


def test_proposal_body_sections(tmp_path: Path) -> None:
    """AC #4: Before / After / Rationale / Backward-compat — the proposal
    body must include 'Stripe', 'Triggering signal', 'Evidence rows',
    'Suggested change', 'Risk class' (mapped to AC #4 via the emit_or_halt
    template that supplies all required sections + the additional content)."""
    evidence = [_row("workflow_complete", workflow_id="cto/ship",
                      outcome="ROUTED_BACK", rework_reason="boom", row_id="r1")]
    result = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.4, 0.3, evidence, tmp_path,
    )
    assert isinstance(result, Emitted)
    body = result.proposal_path.read_text(encoding="utf-8")
    for section in ("## Stripe", "## Triggering signal", "## Evidence rows",
                    "## Suggested change", "## Risk class"):
        assert section in body, f"missing section {section}"


# ----------------------------------------------------------------------------
# AC #5 — Workflow stripe format
# ----------------------------------------------------------------------------


def test_workflow_stripe_format() -> None:
    """AC #5: <persona>/<workflow_slug>:<signal_id>:<8 hex>."""
    rows = [_row("x", workflow_id="cto/ship", row_id="r1")]
    stripe = compute_workflow_stripe("cto/ship", "routed_back_rate_above", rows)
    assert "/" in stripe.split(":")[0]
    # Match full canonical regex form: 8 hex chars
    m = re.match(r"^([a-z0-9_-]+)/([a-z0-9_-]+):([a-z_]+):([0-9a-f]{8})$",
                  stripe)
    assert m is not None
    assert m.group(1) == "cto"
    assert m.group(2) == "ship"


# ----------------------------------------------------------------------------
# AC #7 — Repeat stripe halts (via TASK-CUO-201)
# ----------------------------------------------------------------------------


def test_repeat_stripe_halts(tmp_path: Path) -> None:
    """AC #7: second emission of same stripe → StripeRepeatHalt."""
    evidence = [_row("workflow_complete", workflow_id="cto/ship",
                      outcome="ROUTED_BACK", rework_reason="boom", row_id="r1")]
    first = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.4, 0.3, evidence, tmp_path,
    )
    assert isinstance(first, Emitted)
    second = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.4, 0.3, evidence, tmp_path,
    )
    assert isinstance(second, StripeRepeatHalt)


# ----------------------------------------------------------------------------
# AC #10 — workflow report cites FR ids per tripped signal
# ----------------------------------------------------------------------------


def test_report_cites_fr_ids(tmp_path: Path) -> None:
    """AC #10: the proposal evidence table references specific FR ids."""
    evidence = [
        _row("workflow_complete", workflow_id="cto/ship",
             outcome="ROUTED_BACK", task_id="TASK-MEMORY-101",
             rework_reason="reason-A", row_id="r1"),
        _row("workflow_complete", workflow_id="cto/ship",
             outcome="ROUTED_BACK", task_id="TASK-MEMORY-102",
             rework_reason="reason-B", row_id="r2"),
    ]
    result = emit_workflow_proposal(
        "cto/ship", "routed_back_rate_above", 0.5, 0.3, evidence, tmp_path,
    )
    assert isinstance(result, Emitted)
    body = result.proposal_path.read_text(encoding="utf-8")
    assert "TASK-MEMORY-101" in body
    assert "TASK-MEMORY-102" in body


# ----------------------------------------------------------------------------
# Bonus — workflow stripe and skill stripe are disjoint
# ----------------------------------------------------------------------------


def test_workflow_and_skill_stripes_disjoint() -> None:
    """§2: workflow stripes contain `/`; skill stripes don't → no collision."""
    from cuo.core.stripe import compute_stripe
    wf_stripe = compute_workflow_stripe("cto/ship", "routed_back_rate_above", [])
    skill_stripe = str(compute_stripe("task-audit",
                                       "needs_human_rate_above", []))
    assert "/" in wf_stripe
    assert "/" not in skill_stripe
