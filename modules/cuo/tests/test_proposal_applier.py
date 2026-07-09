"""Test suite for FR-CUO-202 — auto-bump applier."""

from __future__ import annotations

import textwrap
from pathlib import Path

import pytest

from cuo.core.proposal_applier import (
    ApplyResult,
    Classification,
    apply_proposal,
    classify_proposal,
)
from cuo.core.version_bump import bump_version, read_version


@pytest.fixture()
def env(tmp_path: Path) -> dict:
    """Build a minimal cyberos-root with skill_root + docs/proposals/ + CHANGELOG."""
    root = tmp_path / "cyberos"
    skill_root = root / "modules" / "skill"
    proposals_root = root / "docs" / "proposals"
    skill_root.mkdir(parents=True)
    proposals_root.mkdir(parents=True)
    (proposals_root / "open").mkdir()
    (proposals_root / "applied").mkdir()
    (proposals_root / "rejected").mkdir()
    (proposals_root / "pending_approval").mkdir()
    (root / "CHANGELOG.md").write_text("# CHANGELOG\n", encoding="utf-8")

    # Skill A: no review_required flags
    a = skill_root / "test-skill-a"
    a.mkdir()
    (a / "SKILL.md").write_text(textwrap.dedent("""\
        ---
        name: test-skill-a
        license: Apache-2.0
        metadata:
          version: 1.2.3
        human_fine_tune:
          review_required:
            on_minor_bump: false
            on_major_bump: true
            on_rubric_rule_added: true
            on_rubric_rule_removed: true
            on_safety_change: true
        ---

        # body
    """), encoding="utf-8")

    # Skill B: ON for minor bumps too
    b = skill_root / "test-skill-b"
    b.mkdir()
    (b / "SKILL.md").write_text(textwrap.dedent("""\
        ---
        name: test-skill-b
        license: Apache-2.0
        metadata:
          version: 0.4.0
        human_fine_tune:
          review_required:
            on_minor_bump: true
            on_major_bump: true
            on_safety_change: true
        ---

        # body
    """), encoding="utf-8")

    return {
        "root": root, "skill_root": skill_root, "proposals_root": proposals_root,
    }


def _write_proposal(env: dict, *, skill_name: str, body: str,
                     risk_class: str = "minor") -> Path:
    """Helper: write a refinement_proposal@1 file to open/."""
    open_dir = env["proposals_root"] / "open"
    stripe = f"{skill_name}:test_signal:abcd1234"
    path = open_dir / f"{stripe}-20260519T010101001Z.md"
    path.write_text(textwrap.dedent(f"""\
        ---
        template: refinement_proposal@1
        stripe_id: {stripe}
        kind: skill_refinement
        skill_name: {skill_name}
        signal_id: test_signal
        risk_class: {risk_class}
        ---

        # Refinement proposal

        ## Stripe

        `{stripe}`

        ## Suggested change

        {body}

        ## Risk class

        **{risk_class}**
    """), encoding="utf-8")
    return path


# ----------------------------------------------------------------------------
# AC #1 — cosmetic auto-applies, patch bump
# ----------------------------------------------------------------------------


def test_cosmetic_auto_applies(env: dict) -> None:
    """AC #1 + #8: cosmetic diff → patch bump, auto-apply."""
    proposal = _write_proposal(env, skill_name="test-skill-a",
                                body="Fix typo in description.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "AUTO_APPLIED"
    assert result.classification.bucket in ("cosmetic", "wording_polish")
    assert result.classification.bump_level == "patch"
    # Version bumped from 1.2.3 → 1.2.4
    assert read_version(env["skill_root"] / "test-skill-a" / "SKILL.md") == "1.2.4"
    # Proposal moved to applied/
    assert (env["proposals_root"] / "applied" / proposal.name).is_file()
    assert not proposal.exists()


# ----------------------------------------------------------------------------
# AC #2 — rule_addition queues (on_rubric_rule_added: true)
# ----------------------------------------------------------------------------


def test_rule_addition_queues(env: dict) -> None:
    """AC #2: rule_addition + on_rubric_rule_added:true → queue."""
    proposal = _write_proposal(env, skill_name="test-skill-a",
                                body="add rule TRACE-006 catching missing ACs.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "QUEUED"
    assert result.classification.bucket == "rule_addition"
    # Target skill UNCHANGED
    assert read_version(env["skill_root"] / "test-skill-a" / "SKILL.md") == "1.2.3"
    # Proposal moved to pending_approval/
    assert (env["proposals_root"] / "pending_approval" / proposal.name).is_file()


# ----------------------------------------------------------------------------
# AC #3 — safety_class NEVER auto-applies
# ----------------------------------------------------------------------------


def test_safety_class_never_auto(env: dict) -> None:
    """AC #3: risk_class: safety → never auto regardless of bucket."""
    # Cosmetic body but risk_class: safety
    proposal = _write_proposal(env, skill_name="test-skill-a",
                                body="Fix typo in description.",
                                risk_class="safety")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "QUEUED"
    assert result.classification.bucket == "safety_class"
    assert not result.classification.will_auto_apply


# ----------------------------------------------------------------------------
# AC #4 — pre-apply test gate (presence proxy in v1)
# ----------------------------------------------------------------------------


def test_test_gate_skip_when_no_trigger_tests(env: dict) -> None:
    """AC #4: pre-apply test gate runs; if no TRIGGER_TESTS.md, gate is no-op."""
    proposal = _write_proposal(env, skill_name="test-skill-a",
                                body="Fix typo.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    # Auto-applied because no TRIGGER_TESTS.md = gate proxy-OK
    assert result.outcome == "AUTO_APPLIED"


# ----------------------------------------------------------------------------
# AC #5/#6 — audit row emission (memory module absent → no-op, tests pass)
# ----------------------------------------------------------------------------


def test_audit_rows_emitted(env: dict) -> None:
    """AC #5/#6: applier emits cuo.proposal_applied / cuo.proposal_queued aux rows.
    In test env without .cyberos/memory/store the emit is opportunistic and silent —
    the apply must still succeed."""
    proposal = _write_proposal(env, skill_name="test-skill-a", body="Fix typo.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "AUTO_APPLIED"
    # CHANGELOG entry was appended
    cl = (env["root"] / "CHANGELOG.md").read_text(encoding="utf-8")
    assert "test-skill-a" in cl
    assert "1.2.3" in cl and "1.2.4" in cl


# ----------------------------------------------------------------------------
# AC #7 — classify is read-only
# ----------------------------------------------------------------------------


def test_classify_is_read_only(env: dict) -> None:
    """AC #7: classify_proposal MUST NOT mutate any file."""
    proposal = _write_proposal(env, skill_name="test-skill-a", body="Fix typo.")
    before_skill = (env["skill_root"] / "test-skill-a" / "SKILL.md").read_text()
    before_proposal = proposal.read_text(encoding="utf-8")
    classification = classify_proposal(proposal, env["skill_root"])
    after_skill = (env["skill_root"] / "test-skill-a" / "SKILL.md").read_text()
    after_proposal = proposal.read_text(encoding="utf-8")
    assert before_skill == after_skill
    assert before_proposal == after_proposal
    assert isinstance(classification, Classification)


# ----------------------------------------------------------------------------
# AC #8 — bump-level table
# ----------------------------------------------------------------------------


def test_bump_levels() -> None:
    """AC #8: bump_version respects semver mapping."""
    assert bump_version("1.2.3", "patch") == "1.2.4"
    assert bump_version("1.2.3", "minor") == "1.3.0"
    assert bump_version("1.2.3", "major") == "2.0.0"
    # Prerelease suffix dropped on bump
    assert bump_version("1.0.0-rc1", "patch") == "1.0.1"


# ----------------------------------------------------------------------------
# AC #9 — approve transactional
# ----------------------------------------------------------------------------


def test_approve_transactional(env: dict) -> None:
    """AC #9: queued proposal → approve path moves it to applied/."""
    # Queue first via rule_addition
    proposal = _write_proposal(env, skill_name="test-skill-a",
                                body="add rule TRACE-006.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "QUEUED"
    pending = env["proposals_root"] / "pending_approval" / proposal.name
    assert pending.is_file()

    # Approve via refinement_proposal.approve_proposal
    from cuo.core.refinement_proposal import approve_proposal
    stripe = "test-skill-a:test_signal:abcd1234"
    applied = approve_proposal(env["proposals_root"], stripe)
    assert applied is not None
    assert applied.parent.name == "applied"
    assert not pending.exists()


# ----------------------------------------------------------------------------
# AC #10 — post-apply list reflects new version
# ----------------------------------------------------------------------------


def test_post_apply_list(env: dict) -> None:
    """AC #10: after apply, list_proposals shows the applied entry."""
    proposal = _write_proposal(env, skill_name="test-skill-a", body="Fix typo.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "AUTO_APPLIED"

    from cuo.core.refinement_proposal import list_proposals
    listing = list_proposals(env["proposals_root"])
    assert len(listing["open"]) == 0
    assert len(listing["applied"]) == 1
    assert len(listing["rejected"]) == 0
    # The applied entry exists
    assert listing["applied"][0].name == proposal.name


# ----------------------------------------------------------------------------
# Bonus — on_minor_bump: true forces queue
# ----------------------------------------------------------------------------


def test_skill_b_on_minor_bump_true_queues(env: dict) -> None:
    """test-skill-b has on_minor_bump: true → threshold_tune queues."""
    proposal = _write_proposal(env, skill_name="test-skill-b",
                                body="tune the threshold from 3 to 5.")
    result = apply_proposal(proposal, env["skill_root"],
                             proposals_root=env["proposals_root"])
    assert result.outcome == "QUEUED"
    assert result.classification.bump_level == "minor"
    assert "on_minor_bump" in " ".join(result.classification.review_required_reasons)
