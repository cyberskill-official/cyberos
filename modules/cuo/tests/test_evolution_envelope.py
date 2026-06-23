"""Tests for FR-CUO-204 evolution_envelope - the path-based safety boundary.

These prove the boundary is safe by construction: denylist beats allowlist, unknown targets are denied by
default, the named security invariants are always denied, and the loop is disabled by default with a kill
switch that overrides an enabled config.
"""

from __future__ import annotations

from pathlib import Path

from cuo.core.evolution_envelope import EvolutionEnvelope


def _env(allow, deny, enabled=False):
    return EvolutionEnvelope(
        allowlist=list(allow),
        denylist=list(deny),
        enabled=enabled,
        idle_window_minutes=30,
        max_changes_per_window=5,
        max_wall_clock_seconds=600,
    )


def test_denylist_beats_allowlist():
    e = _env(allow=["services/*"], deny=["*auth*"])
    v = e.classify_target("services/auth/src/rbac.rs")
    assert not v.allowed
    assert "denylist" in v.reason


def test_allowlisted_target_is_allowed():
    e = _env(allow=["modules/*/skills/*/SKILL.md"], deny=["*auth*"])
    v = e.classify_target("modules/obs/skills/triage/SKILL.md")
    assert v.allowed
    assert v.matched == "modules/*/skills/*/SKILL.md"


def test_unknown_target_is_default_denied():
    e = _env(allow=["modules/*/skills/*/SKILL.md"], deny=["*auth*"])
    v = e.classify_target("services/memory/src/something_else.rs")
    assert not v.allowed
    assert "default-deny" in v.reason


def test_security_invariants_are_always_denied_even_with_permissive_allowlist():
    # Allow everything, then prove the denylist still refuses the invariants.
    e = _env(
        allow=["*"],
        deny=["*auth*", "*/audit*", "*cross_tenant*", "*cost_ledger*", "*pii*", "*rls*", "*redact*"],
    )
    for bad in [
        "services/memory/src/audit_chain.rs",
        "services/ai-gateway/src/cross_tenant_check.rs",
        "services/ai-gateway/src/cost_ledger/mod.rs",
        "modules/ai/pii_plugin.py",
        "db/policies/rls_tenant.sql",
        "services/ai-gateway/src/redact/mod.rs",
        "services/auth/src/rbac.rs",
    ]:
        assert not e.classify_target(bad).allowed, f"should be denied: {bad}"


def test_disabled_by_default():
    e = _env(allow=["*"], deny=[], enabled=False)
    assert e.is_enabled(env={}) is False


def test_kill_switch_overrides_enabled_config():
    e = _env(allow=["*"], deny=[], enabled=True)
    assert e.is_enabled(env={}) is True
    assert e.is_enabled(env={"CYBEROS_DREAM_KILL": "1"}) is False
    assert e.is_enabled(env={"CYBEROS_DREAM_KILL": "true"}) is False


def test_missing_config_is_safe_disabled_deny_everything():
    e = EvolutionEnvelope.load(Path("/no/such/dream.yaml"))
    assert e.enabled is False
    assert e.is_enabled(env={}) is False
    assert not e.classify_target("modules/obs/skills/triage/SKILL.md").allowed  # empty allowlist -> deny


def test_shipped_config_is_propose_mode_and_denies_invariants():
    cfg = Path(__file__).resolve().parents[1] / "config" / "dream.yaml"
    e = EvolutionEnvelope.load(cfg)
    # Shipped enabled, but in propose mode: the loop runs and records, and CANNOT auto-apply.
    assert e.mode == "propose"
    assert e.effective_mode(env={}) == "propose"
    # The kill switch still forces it off regardless of config.
    assert e.effective_mode(env={"CYBEROS_DREAM_KILL": "1"}) == "off"
    # A sampling of real invariant paths must be denied.
    assert not e.classify_target("services/auth/src/x.rs").allowed
    assert not e.classify_target("services/ai-gateway/src/cost_ledger/mod.rs").allowed
    assert not e.classify_target("services/ai-gateway/src/zdr.rs").allowed
    assert not e.classify_target("deploy/vps/.env.local").allowed
    # The loop must never modify its own safety machinery (self-protection denylist).
    assert not e.classify_target("modules/cuo/config/dream.yaml").allowed
    assert not e.classify_target("modules/cuo/cuo/core/evolution_envelope.py").allowed
    # The verification harness and operator scripts are off-limits too.
    assert not e.classify_target("tools/awh/awh_gate.py").allowed
    assert not e.classify_target("scripts/mcp_call.sh").allowed
    # An allowlisted skill body is allowed (path-wise; the content gates still apply downstream).
    assert e.classify_target("modules/obs/skills/triage/SKILL.md").allowed
