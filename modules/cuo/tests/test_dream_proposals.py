"""Tests for the FR-CUO-201/202 binding into the dream runner (propose mode).

These prove the proposal feed maps each open refinement proposal to its target SKILL.md (repo-relative, so
the path envelope matches), the real classifier is wired, and propose mode surfaces and records candidates
while applying nothing.
"""

from __future__ import annotations

from pathlib import Path

from cuo.core.dream_runner import build_refinement_bindings, run_dream_safely
from cuo.core.evolution_envelope import EvolutionEnvelope


def _make_repo(tmp_path: Path):
    repo = tmp_path
    skill_root = repo / "modules" / "test" / "skills"
    (skill_root / "demo").mkdir(parents=True)
    (skill_root / "demo" / "SKILL.md").write_text(
        "---\nname: demo\nmetadata:\n  version: 1.0.0\n---\n\nbody\n", encoding="utf-8"
    )
    proposals = repo / "proposals"
    (proposals / "open").mkdir(parents=True)
    return repo, skill_root, proposals


def _write_proposal(proposals: Path, name: str, *, skill_name: str, risk_class: str, body: str):
    (proposals / "open" / name).write_text(
        f"---\nskill_name: {skill_name}\nrisk_class: {risk_class}\n---\n\n## Suggested change\n\n{body}\n",
        encoding="utf-8",
    )


def _env(mode="propose"):
    return EvolutionEnvelope(
        allowlist=["modules/*/skills/*/SKILL.md"],
        denylist=["*auth*", "*dream*"],
        enabled=True,
        mode=mode,
        idle_window_minutes=30,
        max_changes_per_window=5,
        max_wall_clock_seconds=600,
    )


def test_propose_fn_maps_proposal_to_repo_relative_target(tmp_path):
    repo, skill_root, proposals = _make_repo(tmp_path)
    _write_proposal(proposals, "p1.md", skill_name="demo", risk_class="minor", body="Improve the wording.")
    propose_fn, _ = build_refinement_bindings(proposals, skill_root, repo_root=repo)
    props = list(propose_fn())
    assert len(props) == 1
    assert props[0].target == "modules/test/skills/demo/SKILL.md"
    assert props[0].skill_name == "demo"


def test_classify_fn_is_the_real_classifier(tmp_path):
    repo, skill_root, proposals = _make_repo(tmp_path)
    _write_proposal(proposals, "p1.md", skill_name="demo", risk_class="minor", body="Improve the wording.")
    propose_fn, classify_fn = build_refinement_bindings(proposals, skill_root, repo_root=repo)
    prop = list(propose_fn())[0]
    c = classify_fn(prop)
    # The real Classification exposes the contract the dream loop reads.
    assert hasattr(c, "will_auto_apply") and isinstance(c.will_auto_apply, bool)
    assert c.risk_class == "minor"


def test_safety_proposal_classifies_as_non_auto(tmp_path):
    repo, skill_root, proposals = _make_repo(tmp_path)
    _write_proposal(proposals, "s1.md", skill_name="demo", risk_class="safety", body="Tighten a safety rule.")
    propose_fn, classify_fn = build_refinement_bindings(proposals, skill_root, repo_root=repo)
    c = classify_fn(list(propose_fn())[0])
    assert c.will_auto_apply is False
    assert c.risk_class == "safety"


def test_propose_mode_surfaces_and_records_but_applies_nothing(tmp_path):
    repo, skill_root, proposals = _make_repo(tmp_path)
    _write_proposal(proposals, "p1.md", skill_name="demo", risk_class="minor", body="Improve the wording.")
    propose_fn, classify_fn = build_refinement_bindings(proposals, skill_root, repo_root=repo)
    res = run_dream_safely(
        _env(mode="propose"),
        propose_fn=propose_fn,
        classify_fn=classify_fn,
        env={},
    )
    assert res.mode == "propose"
    assert res.report.seen == 1, "the open proposal is surfaced"
    assert res.report.applied == 0, "propose mode applies nothing"
    # The open proposal file is untouched (read-only feed).
    assert (proposals / "open" / "p1.md").is_file()


def test_no_skill_name_is_unknown_target_and_denied(tmp_path):
    repo, skill_root, proposals = _make_repo(tmp_path)
    _write_proposal(proposals, "n1.md", skill_name="", risk_class="minor", body="Improve the wording.")
    propose_fn, _ = build_refinement_bindings(proposals, skill_root, repo_root=repo)
    prop = list(propose_fn())[0]
    assert prop.target == "(unknown)"
    # In the envelope, an unknown target is default-denied (held for a human).
    assert not _env().classify_target(prop.target).allowed


def test_missing_proposals_dir_yields_no_candidates(tmp_path):
    repo, skill_root, _ = _make_repo(tmp_path)
    propose_fn, _ = build_refinement_bindings(repo / "nope", skill_root, repo_root=repo)
    assert list(propose_fn()) == []
