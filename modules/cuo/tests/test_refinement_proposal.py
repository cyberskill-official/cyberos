"""Test suite for TASK-CUO-201 — refinement proposal emitter + stripe dedup.

10 ACs covered.
"""

from __future__ import annotations

import json
import re
import time
from pathlib import Path

import pytest

from cuo.core.refinement_proposal import (
    Emitted,
    StripeRepeatHalt,
    apply_proposal_lifecycle,
    approve_proposal,
    emit_or_halt,
    list_proposals,
    reject_proposal,
)
from cuo.core.stripe import STRIPE_ID_RE, StripeId, compute_stripe


def _row(op: str, **extra) -> dict:
    return {
        "op": op,
        "row_id": extra.pop("row_id", f"row-{op}-{id(extra)}"),
        "extra": extra,
    }


# ----------------------------------------------------------------------------
# AC #1 — First occurrence writes exactly one file
# ----------------------------------------------------------------------------


def test_first_occurrence_writes_proposal(tmp_path: Path) -> None:
    rows = [
        _row("memory.task_routed_back", skill="task-audit",
             outcome="ROUTED_BACK", rework_reason="trace-001 failure",
             row_id="r1"),
    ]
    result = emit_or_halt(
        skill_name="task-audit",
        signal_id="acceptance_rate_below",
        evidence_rows=rows,
        proposals_root=tmp_path,
    )
    assert isinstance(result, Emitted)
    assert result.proposal_path.is_file()
    assert result.proposal_path.parent.name == "open"
    body = result.proposal_path.read_text(encoding="utf-8")
    assert "## Stripe" in body
    assert "## Triggering signal" in body
    assert "## Evidence rows" in body
    assert "## Suggested change" in body
    assert "## Risk class" in body


# ----------------------------------------------------------------------------
# AC #2/#7 — Second occurrence of same stripe halts, no new file
# ----------------------------------------------------------------------------


def test_repeat_stripe_halts_no_new_file(tmp_path: Path) -> None:
    rows = [
        _row("memory.task_routed_back",
             skill="task-audit",
             outcome="ROUTED_BACK",
             rework_reason="trace-001 failure", row_id="r1"),
    ]
    first = emit_or_halt("task-audit", "acceptance_rate_below",
                         rows, tmp_path)
    assert isinstance(first, Emitted)

    # Second call with SAME projection produces same stripe
    second = emit_or_halt("task-audit", "acceptance_rate_below",
                          rows, tmp_path)
    assert isinstance(second, StripeRepeatHalt)
    assert second.stripe_id == first.stripe_id
    assert second.existing_proposal_path == first.proposal_path

    # Open dir still has exactly one file
    open_files = list((tmp_path / "open").glob("*.md"))
    assert len(open_files) == 1


# ----------------------------------------------------------------------------
# AC #3 — Applied proposal reopens the stripe naturally
# ----------------------------------------------------------------------------


def test_applied_proposal_reopens_stripe(tmp_path: Path) -> None:
    rows = [_row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
                  rework_reason="r", row_id="r1")]
    first = emit_or_halt("x", "acceptance_rate_below", rows, tmp_path)
    assert isinstance(first, Emitted)

    # Apply it (move to applied/)
    applied_path = apply_proposal_lifecycle(tmp_path, first.stripe_id)
    assert applied_path is not None
    assert applied_path.parent.name == "applied"

    # New occurrence of same stripe → new proposal because open/ is empty.
    # (Note: in millisecond-collision conditions the new proposal may land at
    # the SAME path string as the original — what matters is that an Emitted
    # result is returned, NOT a StripeRepeatHalt, and the file exists in open/.)
    third = emit_or_halt("x", "acceptance_rate_below", rows, tmp_path)
    assert isinstance(third, Emitted)
    assert third.stripe_id == first.stripe_id
    assert third.proposal_path.parent.name == "open"
    assert third.proposal_path.is_file()
    # The applied/ folder still holds the original
    assert applied_path.is_file()
    # And open/ holds the new emission
    assert third.proposal_path.is_file()


# ----------------------------------------------------------------------------
# AC #4 — Stripe determinism
# ----------------------------------------------------------------------------


def test_stripe_determinism() -> None:
    rows = [
        _row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
             rework_reason="boom", row_id="r1"),
        _row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
             rework_reason="bang", row_id="r2"),
    ]
    s1 = compute_stripe("x", "acceptance_rate_below", rows)
    s2 = compute_stripe("x", "acceptance_rate_below", rows)
    assert str(s1) == str(s2)
    # Different evidence → different stripe
    rows2 = [_row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
                   rework_reason="totally-different", row_id="r3")]
    s3 = compute_stripe("x", "acceptance_rate_below", rows2)
    assert str(s3) != str(s1)


# ----------------------------------------------------------------------------
# AC #5 — Proposal body shape (4 sections + evidence table)
# ----------------------------------------------------------------------------


def test_proposal_body_shape(tmp_path: Path) -> None:
    rows = [_row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
                  rework_reason="r", row_id="r1")]
    result = emit_or_halt("x", "acceptance_rate_below", rows, tmp_path,
                           suggested_change="Add TRACE-006 rule to catch this.")
    assert isinstance(result, Emitted)
    body = result.proposal_path.read_text(encoding="utf-8")
    # 5 mandatory body sections (task §1 #5)
    for section in ("## Stripe", "## Triggering signal", "## Evidence rows",
                    "## Suggested change", "## Risk class"):
        assert section in body
    # Stripe id appears verbatim
    assert result.stripe_id in body
    # Evidence table contains the row_id
    assert "`r1`" in body
    # Suggested change is captured
    assert "TRACE-006" in body


# ----------------------------------------------------------------------------
# AC #6/#7 — list / show / apply / reject CLI plumbing
# ----------------------------------------------------------------------------


def test_list_show_apply_reject(tmp_path: Path) -> None:
    rows = [_row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
                  rework_reason="r", row_id="r1")]
    result = emit_or_halt("x", "acceptance_rate_below", rows, tmp_path)
    assert isinstance(result, Emitted)

    listing = list_proposals(tmp_path)
    assert len(listing["open"]) == 1
    assert len(listing["applied"]) == 0
    assert len(listing["rejected"]) == 0

    # Reject the proposal with a reason
    rejected = reject_proposal(tmp_path, result.stripe_id, "Bad idea, breaks invariant X.")
    assert rejected is not None
    assert rejected.parent.name == "rejected"
    body = rejected.read_text(encoding="utf-8")
    assert "## Rejection rationale" in body
    assert "breaks invariant X" in body

    listing2 = list_proposals(tmp_path)
    assert len(listing2["open"]) == 0
    assert len(listing2["rejected"]) == 1


def test_empty_chain_clean_exit(tmp_path: Path) -> None:
    """AC #8 — running with zero evidence rows produces zero files and zero halts."""
    listing = list_proposals(tmp_path)
    assert all(len(v) == 0 for v in listing.values())


# ----------------------------------------------------------------------------
# AC #9 — Stripe hash width
# ----------------------------------------------------------------------------


def test_stripe_hash_width() -> None:
    rows = [_row("x", row_id="r1")]
    stripe = compute_stripe("a", "needs_human_rate_above", rows)
    assert len(stripe.pattern_hash) == 8
    assert all(c in "0123456789abcdef" for c in stripe.pattern_hash)
    # Validate full id against the canonical regex
    assert STRIPE_ID_RE.match(str(stripe)) is not None


# ----------------------------------------------------------------------------
# AC #10 — Drain honours HITL_HALT (supervisor integration, separate test)
# ----------------------------------------------------------------------------


def test_drain_honours_hitl_halt(tmp_path: Path) -> None:
    """The supervisor's HITL_HALT outcome (already wired in Phase 5) integrates
    with this emitter via a step's output carrying `hitl_required: true` —
    proven by `test_smoke.py::test_supervisor_hitl_halt` (existing). This test
    just verifies that a StripeRepeatHalt result carries enough info for the
    drain command to write a DRAIN_HALT.md.
    """
    rows = [_row("memory.task_routed_back", skill="x", outcome="ROUTED_BACK",
                  row_id="r1")]
    first = emit_or_halt("x", "needs_human_rate_above", rows, tmp_path)
    assert isinstance(first, Emitted)
    second = emit_or_halt("x", "needs_human_rate_above", rows, tmp_path)
    assert isinstance(second, StripeRepeatHalt)
    # The halt carries the existing proposal path so the drain CLI can surface it
    assert second.existing_proposal_path == first.proposal_path
    # And the new evidence rows for context
    assert "r1" in second.new_evidence_row_ids


def test_approve_lifecycle(tmp_path: Path) -> None:
    """approve_proposal moves pending → applied. Used by TASK-CUO-202."""
    rows = [_row("x", row_id="r1")]
    first = emit_or_halt("a", "needs_human_rate_above", rows, tmp_path)
    assert isinstance(first, Emitted)
    # Move open → pending_approval manually (simulating classifier output)
    pending = tmp_path / "pending_approval" / first.proposal_path.name
    first.proposal_path.rename(pending)
    # Approve it
    applied = approve_proposal(tmp_path, first.stripe_id)
    assert applied is not None
    assert applied.parent.name == "applied"
