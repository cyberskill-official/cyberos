"""Tests for TASK-CUO-204 dream_loop - the idle-gated orchestrator.

These prove the safety properties deterministically with injected fakes: disabled by default, only runs
when idle, a denylisted target never reaches the applier, only low-risk in-envelope green changes apply,
major/safety changes and red gates halt for a human, and the per-window cap is honoured.
"""

from __future__ import annotations

from types import SimpleNamespace

from cuo.core.dream_loop import run_dream_cycle
from cuo.core.evolution_envelope import EvolutionEnvelope


def _env(enabled=True, allow=("modules/*/skills/*/SKILL.md",), deny=("*auth*", "*/audit*"), maxc=5):
    return EvolutionEnvelope(
        allowlist=list(allow),
        denylist=list(deny),
        enabled=enabled,
        idle_window_minutes=30,
        max_changes_per_window=maxc,
        max_wall_clock_seconds=600,
    )


def _prop(target):
    return SimpleNamespace(target=target)


def _cls(auto=True, risk="minor"):
    return SimpleNamespace(will_auto_apply=auto, risk_class=risk)


def _applied():
    return SimpleNamespace(outcome="AUTO_APPLIED")


def _audit_sink():
    kinds: list[str] = []
    return kinds, (lambda kind, body: kinds.append(kind))


def test_disabled_by_default_does_nothing():
    kinds, audit = _audit_sink()
    r = run_dream_cycle(
        _env(enabled=False),
        propose_fn=lambda: [_prop("modules/obs/skills/t/SKILL.md")],
        classify_fn=lambda p: _cls(),
        apply_fn=lambda p: _applied(),
        idle_fn=lambda: True,
        audit_fn=audit,
        env={},
    )
    assert r.status == "disabled"
    assert r.applied == 0
    assert kinds == []  # nothing ran, nothing audited


def test_does_not_run_when_not_idle():
    r = run_dream_cycle(
        _env(enabled=True),
        propose_fn=lambda: [_prop("modules/obs/skills/t/SKILL.md")],
        classify_fn=lambda p: _cls(),
        apply_fn=lambda p: _applied(),
        idle_fn=lambda: False,
        env={},
    )
    assert r.status == "not_idle"
    assert r.applied == 0


def test_denylisted_target_halts_and_applier_is_never_called():
    kinds, audit = _audit_sink()
    apply_calls = []

    def apply_fn(p):
        apply_calls.append(p)
        return _applied()

    r = run_dream_cycle(
        _env(enabled=True),
        propose_fn=lambda: [_prop("services/auth/src/rbac.rs")],
        classify_fn=lambda p: _cls(),
        apply_fn=apply_fn,
        idle_fn=lambda: True,
        audit_fn=audit,
        env={},
    )
    assert r.applied == 0
    assert r.halted_hitl == 1
    assert apply_calls == []  # a denylisted target must never reach the applier
    assert "cuo.dream_halted_hitl" in kinds


def test_low_risk_in_envelope_green_is_applied():
    kinds, audit = _audit_sink()
    r = run_dream_cycle(
        _env(enabled=True),
        propose_fn=lambda: [_prop("modules/obs/skills/triage/SKILL.md")],
        classify_fn=lambda p: _cls(auto=True, risk="minor"),
        apply_fn=lambda p: _applied(),
        idle_fn=lambda: True,
        audit_fn=audit,
        env={},
    )
    assert r.applied == 1
    assert r.halted_hitl == 0
    assert "cuo.dream_started" in kinds
    assert "cuo.dream_applied" in kinds


def test_major_or_safety_change_halts_not_applies():
    r = run_dream_cycle(
        _env(enabled=True),
        propose_fn=lambda: [
            _prop("modules/obs/skills/a/SKILL.md"),
            _prop("modules/obs/skills/b/SKILL.md"),
        ],
        classify_fn=lambda p: _cls(auto=False, risk="safety"),
        apply_fn=lambda p: _applied(),
        idle_fn=lambda: True,
        env={},
    )
    assert r.applied == 0
    assert r.halted_hitl == 2


def test_red_test_gate_does_not_apply():
    r = run_dream_cycle(
        _env(enabled=True),
        propose_fn=lambda: [_prop("modules/obs/skills/triage/SKILL.md")],
        classify_fn=lambda p: _cls(),
        apply_fn=lambda p: SimpleNamespace(outcome="TEST_GATE_FAILED"),
        idle_fn=lambda: True,
        env={},
    )
    assert r.applied == 0
    assert r.gate_failed == 1


def test_max_changes_per_window_caps_applies():
    props = [_prop(f"modules/obs/skills/s{i}/SKILL.md") for i in range(10)]
    r = run_dream_cycle(
        _env(enabled=True, maxc=3),
        propose_fn=lambda: props,
        classify_fn=lambda p: _cls(),
        apply_fn=lambda p: _applied(),
        idle_fn=lambda: True,
        env={},
    )
    assert r.applied == 3


def test_kill_switch_env_disables_even_when_enabled():
    r = run_dream_cycle(
        _env(enabled=True),
        propose_fn=lambda: [_prop("modules/obs/skills/triage/SKILL.md")],
        classify_fn=lambda p: _cls(),
        apply_fn=lambda p: _applied(),
        idle_fn=lambda: True,
        env={"CYBEROS_DREAM_KILL": "1"},
    )
    assert r.status == "disabled"
    assert r.applied == 0
