"""Tests for TASK-CUO-204 dream_runner - the safety-enforcing operator layer.

These prove the enablement locks deterministically with injected fakes: off does nothing; propose runs the
gates but applies nothing and never even calls the real applier; auto-apply happens ONLY with mode=auto +
the explicit opt-in + a dream branch; the kill switch overrides; and the audit trail is written.
"""

from __future__ import annotations

import json
from types import SimpleNamespace

from cuo.core.dream_runner import (
    RunResult,
    choose_apply_fn,
    run_dream_safely,
)
from cuo.core.evolution_envelope import EvolutionEnvelope

ALLOW = ("modules/*/skills/*/SKILL.md",)
DENY = ("*auth*", "*dream*")
TARGET = "modules/obs/skills/triage/SKILL.md"


def _env(mode="propose", enabled=True):
    return EvolutionEnvelope(
        allowlist=list(ALLOW),
        denylist=list(DENY),
        enabled=enabled,
        mode=mode,
        idle_window_minutes=30,
        max_changes_per_window=5,
        max_wall_clock_seconds=600,
    )


def _prop(target=TARGET):
    return SimpleNamespace(target=target)


def _low_risk(_p):
    return SimpleNamespace(will_auto_apply=True, risk_class="minor")


class _ApplySpy:
    def __init__(self, outcome="AUTO_APPLIED"):
        self.calls = 0
        self.outcome = outcome

    def __call__(self, _p):
        self.calls += 1
        return SimpleNamespace(outcome=self.outcome)


# ---- effective_mode on the envelope ----------------------------------------------------

def test_effective_mode_resolves_under_enable_and_kill():
    assert _env(mode="propose").effective_mode({}) == "propose"
    assert _env(mode="auto").effective_mode({}) == "auto"
    assert _env(mode="propose", enabled=False).effective_mode({}) == "off"
    assert _env(mode="auto").effective_mode({"CYBEROS_DREAM_KILL": "1"}) == "off"
    assert _env(mode="banana").effective_mode({}) == "off"  # unknown mode is treated as off


# ---- the apply-binding lock matrix -----------------------------------------------------

def test_choose_apply_fn_only_arms_with_every_lock():
    real = _ApplySpy()
    # auto + opt-in + dream branch + real applier => armed
    fn, armed, _ = choose_apply_fn("auto", allow_auto_apply=True, branch="auto/dream", real_apply_fn=real)
    assert armed is True and fn is real
    # any single lock missing => dry run (not the real fn)
    for kwargs in [
        dict(mode="propose", allow_auto_apply=True, branch="auto/dream", real_apply_fn=real),
        dict(mode="auto", allow_auto_apply=False, branch="auto/dream", real_apply_fn=real),
        dict(mode="auto", allow_auto_apply=True, branch="main", real_apply_fn=real),
        dict(mode="auto", allow_auto_apply=True, branch="auto/dream", real_apply_fn=None),
    ]:
        fn, armed, _ = choose_apply_fn(**kwargs)
        assert armed is False and fn is not real


# ---- end-to-end runner behaviour -------------------------------------------------------

def test_off_mode_does_nothing():
    spy = _ApplySpy()
    res = run_dream_safely(
        _env(enabled=False),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        env={},
    )
    assert isinstance(res, RunResult)
    assert res.mode == "off"
    assert res.report.status == "disabled"
    assert res.report.applied == 0
    assert spy.calls == 0


def test_propose_mode_records_but_never_applies_or_calls_real_applier():
    spy = _ApplySpy(outcome="AUTO_APPLIED")  # would auto-apply if it were ever called
    res = run_dream_safely(
        _env(mode="propose"),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        idle_fn=lambda: True,
        env={},
    )
    assert res.mode == "propose"
    assert res.auto_apply_armed is False
    assert res.report.applied == 0, "propose mode must apply nothing"
    assert spy.calls == 0, "the real applier must never be called in propose mode"
    assert res.report.seen == 1
    assert res.report.halted_hitl == 1  # the would-apply proposal is recorded for review


def test_auto_without_optin_stays_dry():
    spy = _ApplySpy()
    res = run_dream_safely(
        _env(mode="auto"),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        allow_auto_apply=False,
        branch="auto/dream",
        env={},
    )
    assert res.auto_apply_armed is False
    assert res.report.applied == 0
    assert spy.calls == 0


def test_auto_optin_but_wrong_branch_stays_dry():
    spy = _ApplySpy()
    res = run_dream_safely(
        _env(mode="auto"),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        allow_auto_apply=True,
        branch="main",
        env={},
    )
    assert res.auto_apply_armed is False
    assert res.report.applied == 0
    assert spy.calls == 0
    assert any("branch" in n for n in res.notes)


def test_auto_fully_armed_applies_via_real_applier():
    spy = _ApplySpy(outcome="AUTO_APPLIED")
    res = run_dream_safely(
        _env(mode="auto"),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        allow_auto_apply=True,
        branch="auto/dream",
        env={},
    )
    assert res.auto_apply_armed is True
    assert res.report.applied == 1
    assert spy.calls == 1


def test_kill_switch_forces_off_even_in_auto():
    spy = _ApplySpy()
    res = run_dream_safely(
        _env(mode="auto"),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        allow_auto_apply=True,
        branch="auto/dream",
        env={"CYBEROS_DREAM_KILL": "1"},
    )
    assert res.mode == "off"
    assert res.report.applied == 0
    assert spy.calls == 0


def test_denylisted_target_never_applies_even_when_armed():
    spy = _ApplySpy()
    res = run_dream_safely(
        _env(mode="auto"),
        propose_fn=lambda: [_prop("services/auth/src/rbac.rs")],
        classify_fn=_low_risk,
        real_apply_fn=spy,
        allow_auto_apply=True,
        branch="auto/dream",
        env={},
    )
    assert res.report.applied == 0
    assert res.report.halted_hitl == 1
    assert spy.calls == 0  # the envelope stops it before the applier


def test_audit_trail_is_written(tmp_path):
    from cuo.core.dream_runner import _jsonl_audit

    log = tmp_path / "dream-audit.jsonl"
    res = run_dream_safely(
        _env(mode="propose"),
        propose_fn=lambda: [_prop()],
        classify_fn=_low_risk,
        audit_fn=_jsonl_audit(log),
        env={},
    )
    assert res.report.status == "ran"
    rows = [json.loads(line) for line in log.read_text().splitlines()]
    kinds = {r["kind"] for r in rows}
    assert "cuo.dream_started" in kinds
    assert "cuo.dream_proposal" in kinds
