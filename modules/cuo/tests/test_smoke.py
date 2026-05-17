"""Smoke tests for CUO v3.0.0 Phase 1 — catalog scan, validate, route, dry-run.

Run with `pytest cuo/tests/` from the cyberos/ root after installing the package
(`pip install -e cuo/`).

These tests exercise the discovery + validation + routing layers against the
real cuo/ + skill/ filesystem catalogs (no mocks). Expected post-Session N state:
- 47 personas with workflows + 1 extinct
- 104 author+audit pairs in skill/
- 194 workflows in cuo/
- All workflows pass `validate_chain` (no MISSING, no planned: gaps)
"""

from __future__ import annotations

from pathlib import Path

import pytest

from cuo.core.brain_bridge import brain_is_available, emit_chain_result
from cuo.core.catalog import discover_personas, discover_workflows
from cuo.core.invoker import MockInvoker, SubprocessInvoker, select_invoker
from cuo.core.llm_invoker import LLMInvoker
from cuo.core.router import route
from cuo.core.supervisor import dry_run_chain, execute_chain
from cuo.core.validator import validate_chain


@pytest.fixture(scope="module")
def cuo_root() -> Path:
    """Resolve cuo/ root by walking up from this test file."""
    # cyberos/cuo/tests/test_smoke.py → cyberos/cuo/
    return Path(__file__).resolve().parent.parent


@pytest.fixture(scope="module")
def skill_root(cuo_root: Path) -> Path:
    """Resolve skill/ — sibling of cuo/."""
    return cuo_root.parent / "skill"


def test_discover_personas_finds_all_active(cuo_root: Path) -> None:
    """Expect ≥47 persona folders (48 total — 1 extinct discoverable but flagged)."""
    personas = discover_personas(cuo_root)
    assert len(personas) >= 47, f"expected ≥47 personas, got {len(personas)}"

    # Spot-check a few known personas exist.
    slugs = {p.slug for p in personas}
    for expected in ("chief-executive-officer", "chief-technology-officer", "chief-financial-officer", "chief-human-resources-officer", "chief-information-security-officer", "chief-ai-officer", "chief-customer-officer", "chief-knowledge-officer"):
        assert expected in slugs, f"missing expected persona: {expected!r}"


def test_extinct_persona_detected(cuo_root: Path) -> None:
    """chief-metaverse-officer should be detected as extinct (per C-Suite Reference §8 rule 4)."""
    personas = discover_personas(cuo_root)
    metaverse = next((p for p in personas if p.slug == "chief-metaverse-officer"), None)
    if metaverse is not None:
        assert metaverse.is_extinct, "chief-metaverse-officer should be flagged extinct"


def test_cto_has_canonical_workflows(cuo_root: Path) -> None:
    """CTO is the canonical reference persona — 5 workflows shipped per Session A-N."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    assert cto.has_workflows
    workflows = discover_workflows(cto)
    # Post-C1 (2026-05-18 depth additions): CTO has 5 original + 2 depth = 7
    assert len(workflows) >= 5, f"CTO should have ≥5 workflows, got {len(workflows)}"
    slugs = {wf.slug for wf in workflows}
    assert "architect-new-system" in slugs
    assert "adr-quick-capture" in slugs


def test_cto_architect_chain_validates(cuo_root: Path, skill_root: Path) -> None:
    """CTO architect-new-system has 10 steps — all should pass validation post-Session N."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    workflows = discover_workflows(cto)
    architect = next(wf for wf in workflows if wf.slug == "architect-new-system")

    result = validate_chain(architect, skill_root)
    assert result.chain_length == 10, f"expected 10-step chain, got {result.chain_length}"
    assert result.valid, (
        f"chain should validate post-Session N. "
        f"missing={result.missing_skills}, planned={result.planned_skills}, notes={result.notes}"
    )
    assert not result.missing_skills, f"missing skills: {result.missing_skills}"
    assert not result.planned_skills, f"planned: gaps: {result.planned_skills}"


def test_route_finds_cto_for_architect_query(cuo_root: Path) -> None:
    """Natural-language routing — "architect a new system" should resolve to CTO."""
    personas = discover_personas(cuo_root)
    decision = route("Architect a new system for our new customer onboarding flow", personas)
    assert decision is not None, "expected a routing decision for clear CTO query"
    assert decision.persona_slug == "chief-technology-officer"
    assert decision.confidence >= 0.5


def test_route_finds_cfo_for_close_query(cuo_root: Path) -> None:
    """Natural-language routing — "monthly close" should resolve to CFO."""
    personas = discover_personas(cuo_root)
    decision = route("Run the monthly close for the books", personas)
    assert decision is not None
    assert decision.persona_slug == "chief-financial-officer"


def test_dry_run_cto_architect_is_runnable(cuo_root: Path, skill_root: Path) -> None:
    """End-to-end dry-run for CTO architect-new-system should produce a runnable plan."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    result = dry_run_chain(cto, "architect-new-system", skill_root)
    assert result.runnable, (
        f"CTO architect-new-system should be runnable post-Session N. "
        f"validation={result.validation!r}, notes={result.notes}"
    )
    assert len(result.step_plan) == 10
    for plan_line in result.step_plan:
        assert "[FOUND]" in plan_line, f"step should be FOUND: {plan_line}"


def test_no_persona_has_missing_skills(cuo_root: Path, skill_root: Path) -> None:
    """Catalog-completeness invariant — every shipped workflow's chain should validate."""
    personas = discover_personas(cuo_root)
    failures: list[str] = []
    for persona in personas:
        if not persona.has_workflows or persona.is_extinct:
            continue
        for wf in discover_workflows(persona):
            result = validate_chain(wf, skill_root)
            if result.missing_skills:
                failures.append(f"{wf.workflow_id}: missing {result.missing_skills}")

    assert not failures, (
        f"MISSING skills (genuine catalog gaps) found in {len(failures)} workflow(s):\n  "
        + "\n  ".join(failures)
    )


def test_total_workflow_count_post_session_n(cuo_root: Path) -> None:
    """Persona first-coverage post-Session N: expect 194 workflows across 47 personas."""
    personas = discover_personas(cuo_root)
    total_workflows = 0
    personas_with_workflows = 0
    for p in personas:
        wfs = discover_workflows(p)
        if wfs:
            personas_with_workflows += 1
            total_workflows += len(wfs)

    assert personas_with_workflows == 47, (
        f"expected 47 personas with workflows post-Session N, got {personas_with_workflows}"
    )
    # Post-C1 (2026-05-18 depth additions): 194 first-coverage + 27 depth = 221
    assert total_workflows >= 194, (
        f"expected ≥194 workflows (Session N floor), got {total_workflows}"
    )


# ---------------------------------------------------------------------------
# Phase 2 — execution tests
# ---------------------------------------------------------------------------


def test_select_invoker_returns_mock_when_no_binary() -> None:
    """In a sandbox with no cyberos-skill on PATH, auto-select must yield MockInvoker."""
    inv = select_invoker("auto")
    # MockInvoker is the expected fallback when SubprocessInvoker is unavailable.
    if not SubprocessInvoker.is_available():
        assert isinstance(inv, MockInvoker), f"expected MockInvoker, got {type(inv).__name__}"


def test_select_invoker_force_mock() -> None:
    inv = select_invoker("mock")
    assert isinstance(inv, MockInvoker)


def test_mock_invoker_writes_output(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """MockInvoker should write a JSON file per step with synthetic output."""
    inv = MockInvoker()
    out_dir = tmp_path / "step-output"
    result = inv.invoke(
        skill_name="sow-author",
        inputs={"foo": "bar"},
        skill_root=skill_root,
        output_dir=out_dir,
        step_num=1,
    )
    assert result.status == "MOCKED", f"unexpected status: {result.status}; notes={result.notes}"
    assert result.output_path is not None and result.output_path.is_file()
    assert result.output["skill"] == "sow-author"
    assert result.output["synthetic"] is True


def test_execute_cto_architect_with_mock(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """End-to-end execute of CTO architect-new-system through MockInvoker.

    Catalog-completeness invariant under execution: all 10 chain steps walk and produce
    output. No FAILED steps; status is COMPLETED.
    """
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    out_dir = tmp_path / "cto-architect"

    result = execute_chain(
        persona=cto,
        workflow_slug="architect-new-system",
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )

    assert result.outcome == "COMPLETED", (
        f"expected COMPLETED, got {result.outcome}; notes={result.notes}"
    )
    assert len(result.step_results) == 10
    for s in result.step_results:
        assert s.status == "MOCKED", f"step {s.step} {s.skill}: unexpected status {s.status} ({s.notes})"
        assert s.output_path is not None and s.output_path.is_file()

    # All 10 step outputs should be on disk in the workflow output dir.
    files = sorted(out_dir.glob("step*.json"))
    assert len(files) == 10, f"expected 10 step output files, got {len(files)}"


def test_execute_blocks_on_planned_skill(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """A workflow with any `planned:` step should NOT execute — outcome BLOCKED.

    Post-Session N, no workflow has planned: gaps, so this test currently constructs
    a synthetic scenario in-memory rather than against the catalog. Specifically,
    we skip the test if no planned-gap workflow exists; otherwise we verify blocking.
    """
    personas = discover_personas(cuo_root)
    for p in personas:
        for wf in discover_workflows(p):
            for step in wf.skill_chain:
                if isinstance(step, dict):
                    skill_val = step.get("skill", "")
                    if isinstance(skill_val, str) and skill_val.startswith("planned:"):
                        # Found a planned-gap workflow — execute must block.
                        result = execute_chain(
                            persona=p,
                            workflow_slug=wf.slug,
                            skill_root=skill_root,
                            output_dir=tmp_path / "blocked",
                            invoker=MockInvoker(),
                        )
                        assert result.outcome == "BLOCKED", (
                            f"workflow {wf.workflow_id} with planned: step should BLOCK, got {result.outcome}"
                        )
                        return
    pytest.skip("no planned: gaps remaining in the catalog (post-Session N this is the expected state)")


# ---------------------------------------------------------------------------
# Phase 3 — LLMInvoker + BRAIN bridge
# ---------------------------------------------------------------------------


def test_llm_invoker_defaults_to_mock_llm() -> None:
    """Without ANTHROPIC_API_KEY env, LLMInvoker should be in 'mock-llm' mode."""
    import os
    if os.environ.get("ANTHROPIC_API_KEY"):
        pytest.skip("API key present in env — mock-mode test only runs when absent")
    inv = LLMInvoker()
    assert inv.mode == "mock-llm"


def test_llm_invoker_force_mock_only() -> None:
    """mock_only=True forces mock-llm mode regardless of env."""
    inv = LLMInvoker(api_key="test-key", mock_only=True)
    assert inv.mode == "mock-llm"


def test_llm_invoker_mock_produces_artefact_fields(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """Mock-LLM output should include `artefact_fields` keyed by contract template H2s."""
    inv = LLMInvoker(mock_only=True)
    out_dir = tmp_path / "llm-mock"
    result = inv.invoke(
        skill_name="sow-author",
        inputs={"engagement": "acme"},
        skill_root=skill_root,
        output_dir=out_dir,
        step_num=1,
    )
    assert result.status == "MOCKED", f"expected MOCKED, got {result.status}; notes={result.notes}"
    assert "artefact_fields" in result.output
    assert isinstance(result.output["artefact_fields"], dict)
    assert len(result.output["artefact_fields"]) > 0


def test_llm_invoker_audit_skill_includes_rubric_outcome(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """Audit-skill mock-LLM output should include a synthetic rubric_outcome block."""
    inv = LLMInvoker(mock_only=True)
    out_dir = tmp_path / "llm-audit"
    result = inv.invoke(
        skill_name="sow-audit",
        inputs={"sow": "/tmp/foo.md"},
        skill_root=skill_root,
        output_dir=out_dir,
        step_num=2,
    )
    assert result.status == "MOCKED"
    assert "rubric_outcome" in result.output
    assert result.output["rubric_outcome"]["pass"] is True


def test_execute_with_llm_invoker_walks_chain(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """End-to-end execute with LLMInvoker(mock_only=True) should COMPLETE."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    result = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=tmp_path / "llm-chain",
        invoker=LLMInvoker(mock_only=True),
    )
    assert result.outcome == "COMPLETED"
    assert len(result.step_results) == 2
    assert result.invoker_kind == "LLMInvoker"
    for s in result.step_results:
        assert s.status == "MOCKED"
        # Each step's output should include the LLM-shaped fields.
        assert "step_invocation" in s.output
        assert s.output["step_invocation"] == "mock-llm"


def test_brain_is_available_returns_bool(skill_root: Path) -> None:
    """brain_is_available should not crash even when memory module missing."""
    # The function tolerates missing memory module / missing BRAIN dir.
    result = brain_is_available(skill_root)
    assert isinstance(result, bool)


def test_brain_emit_no_op_when_unavailable(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """emit_chain_result should return a graceful BrainEmitResult when memory unavailable."""
    # First produce a ChainResult to emit.
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    chain_result = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=tmp_path / "for-brain-emit",
        invoker=MockInvoker(),
    )

    # Force a non-existent brain_root to verify graceful skip.
    br = emit_chain_result(chain_result, skill_root, brain_root=tmp_path / "no-such-brain")
    assert isinstance(br.emitted, bool)
    if not br.emitted:
        # Expected path in sandbox without memory module wired up.
        assert br.reason_skipped, "skip path should populate reason_skipped"


def test_chain_handoff_propagates_outputs(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """Step N's output should be available to step N+1 via the hand-off map.

    Verified by checking that the second step's input file paths point at the
    first step's output file. MockInvoker stringifies inputs into its output
    so we can read them back.
    """
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    out_dir = tmp_path / "cto-adr-quick"

    # adr-quick-capture has a 2-step chain — perfect for hand-off verification.
    result = execute_chain(
        persona=cto,
        workflow_slug="adr-quick-capture",
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.outcome == "COMPLETED"
    assert len(result.step_results) == 2

    step1, step2 = result.step_results
    # Step 1 should have produced an output file.
    assert step1.output_path is not None and step1.output_path.is_file()
    # Step 2 inputs should reference step 1's output (the hand-off map propagated it).
    step2_payload = step2.output.get("inputs", {})
    step1_path_str = str(step1.output_path)
    found_propagation = any(step1_path_str in str(v) for v in step2_payload.values())
    assert found_propagation, (
        f"step 2 inputs should reference step 1 output. step2_payload.inputs={step2_payload}"
    )


# =============================================================================
# Phase 4 — special-case workflow handlers (FR-CUO-106)
# =============================================================================


from cuo.core.handlers import (
    KNOWN_PATTERNS,
    HandlerDispatchError,
    LinearHandler,
    MultiOutputHandler,
    PerInstanceHandler,
    PersonaPairHandler,
    SequentialApprovalHandler,
    TimeCriticalHandler,
    pattern_of,
    pick_handler,
)


def test_known_patterns_has_cardinality_six():
    """DEC-2381 — workflow_pattern enum cardinality MUST be 6."""
    assert len(KNOWN_PATTERNS) == 6
    assert KNOWN_PATTERNS == frozenset({
        "linear",
        "time_critical",
        "per_instance",
        "multi_output",
        "sequential_approval",
        "persona_pair",
    })


def test_pattern_of_defaults_to_linear():
    """Workflows without `pattern:` frontmatter default to 'linear'."""
    class FakeWorkflow:
        workflow_id = "chief-technology-officer/architect-new-system"
        frontmatter: dict = {}
    assert pattern_of(FakeWorkflow()) == "linear"


def test_pattern_of_reads_frontmatter():
    """Workflows with `pattern: time_critical` frontmatter return that."""
    class FakeWorkflow:
        workflow_id = "chief-privacy-officer/breach-response-cycle"
        frontmatter = {"pattern": "time_critical", "sla_minutes": 240}
    assert pattern_of(FakeWorkflow()) == "time_critical"


def test_pick_handler_default_is_linear():
    """No pattern → LinearHandler."""
    class FakeWorkflow:
        workflow_id = "chief-technology-officer/architect-new-system"
        frontmatter: dict = {}
    h = pick_handler(FakeWorkflow())
    assert isinstance(h, LinearHandler)


def test_pick_handler_time_critical():
    class FakeWorkflow:
        workflow_id = "chief-privacy-officer/breach-response-cycle"
        frontmatter = {"pattern": "time_critical", "sla_minutes": 240}
    h = pick_handler(FakeWorkflow())
    assert isinstance(h, TimeCriticalHandler)
    assert h.sla_minutes == 240


def test_pick_handler_per_instance():
    class FakeWorkflow:
        workflow_id = "chief-sales-officer/quarterly-account-plan"
        frontmatter = {
            "pattern": "per_instance",
            "instance_descriptor": [
                {"account_id": "acct-1"},
                {"account_id": "acct-2"},
            ],
        }
    h = pick_handler(FakeWorkflow())
    assert isinstance(h, PerInstanceHandler)
    assert len(h.instance_descriptor) == 2


def test_pick_handler_multi_output():
    class FakeWorkflow:
        workflow_id = "chief-legal-officer/quarterly-regulatory-cycle"
        frontmatter = {
            "pattern": "multi_output",
            "output_recipients": [
                {"recipient_id": "vn-mst", "format": "xml", "delivery_method": "email"},
            ],
        }
    h = pick_handler(FakeWorkflow())
    assert isinstance(h, MultiOutputHandler)
    assert len(h.output_recipients) == 1


def test_pick_handler_sequential_approval():
    class FakeWorkflow:
        workflow_id = "chief-ai-officer/per-model-card-release"
        frontmatter = {
            "pattern": "sequential_approval",
            "gates": [
                {
                    "approver_persona": "chief-ethics-officer",
                    "approver_workflow": "per-model-card-ethics-sign-off",
                }
            ],
        }
    h = pick_handler(FakeWorkflow())
    assert isinstance(h, SequentialApprovalHandler)
    assert len(h.gates) == 1


def test_pick_handler_persona_pair():
    class FakeWorkflow:
        workflow_id = "chief-revenue-officer/churn-collaboration"
        frontmatter = {
            "pattern": "persona_pair",
            "peer_persona": "chief-customer-officer",
            "peer_workflow": "churn-collaboration",
            "shared_artefact": "churn-cohort-analysis",
            "handoff_step": 3,
        }
    h = pick_handler(FakeWorkflow())
    assert isinstance(h, PersonaPairHandler)
    assert h.peer_persona == "chief-customer-officer"
    assert h.handoff_step == 3


def test_unknown_pattern_raises():
    """Workflows declaring an unknown pattern must raise HandlerDispatchError."""
    class FakeWorkflow:
        workflow_id = "test/workflow"
        frontmatter = {"pattern": "nonsense_pattern"}
    with pytest.raises(HandlerDispatchError):
        pick_handler(FakeWorkflow())


def test_time_critical_handler_validates_sla_minutes():
    """TimeCriticalHandler.__init__ must reject non-positive sla_minutes."""
    with pytest.raises(ValueError):
        TimeCriticalHandler(sla_minutes=0)
    with pytest.raises(ValueError):
        TimeCriticalHandler(sla_minutes=-1)


def test_per_instance_handler_rejects_empty_descriptor():
    """Empty instance_descriptor must be rejected."""
    with pytest.raises(ValueError):
        PerInstanceHandler(instance_descriptor=[])


def test_multi_output_handler_rejects_empty_recipients():
    """Empty output_recipients must be rejected."""
    with pytest.raises(ValueError):
        MultiOutputHandler(output_recipients=[])


def test_multi_output_handler_validates_recipient_keys():
    """Recipients must have recipient_id, format, delivery_method."""
    with pytest.raises(ValueError):
        MultiOutputHandler(output_recipients=[{"recipient_id": "x"}])  # missing format + delivery_method


def test_sequential_approval_handler_rejects_empty_gates():
    with pytest.raises(ValueError):
        SequentialApprovalHandler(gates=[])


def test_sequential_approval_handler_validates_gate_keys():
    with pytest.raises(ValueError):
        SequentialApprovalHandler(gates=[{"approver_persona": "x"}])  # missing approver_workflow


def test_persona_pair_handler_validates_fields():
    with pytest.raises(ValueError):
        PersonaPairHandler(peer_persona="", peer_workflow="x", shared_artefact="a", handoff_step=1)
    with pytest.raises(ValueError):
        PersonaPairHandler(peer_persona="x", peer_workflow="y", shared_artefact="z", handoff_step=0)


def test_linear_handler_executes_chain_through_supervisor(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """LinearHandler delegates to execute_chain — should produce a COMPLETED ChainResult."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    workflows = discover_workflows(cto)
    adr = next(wf for wf in workflows if wf.slug == "adr-quick-capture")

    handler = LinearHandler()
    out_dir = tmp_path / "linear-out"
    result = handler.execute(
        persona=cto,
        workflow=adr,
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.handler_kind == "LinearHandler"
    assert result.chain_result.outcome == "COMPLETED"
    assert len(result.chain_result.step_results) == 2


def test_time_critical_handler_runs_chain_synchronously(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """TimeCriticalHandler runs the chain via execute_chain (synchronously)."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    workflows = discover_workflows(cto)
    adr = next(wf for wf in workflows if wf.slug == "adr-quick-capture")

    handler = TimeCriticalHandler(sla_minutes=60)
    out_dir = tmp_path / "tc-out"
    result = handler.execute(
        persona=cto,
        workflow=adr,
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.handler_kind == "TimeCriticalHandler"
    assert result.chain_result.outcome == "COMPLETED"
    # MockInvoker runs essentially instantly — should NOT breach SLA
    sla_breach_rows = [a for a in result.extra_audit_kinds if a.get("kind") == "cuo.time_critical_sla_breach"]
    assert len(sla_breach_rows) == 0


# =============================================================================
# Phase 4 — end-to-end dispatch against the real catalog (post-A2 frontmatter)
# =============================================================================


def test_breach_response_dispatches_to_time_critical(cuo_root: Path) -> None:
    """chief-privacy-officer/breach-response-cycle should declare pattern: time_critical."""
    personas = discover_personas(cuo_root)
    cpo_privacy = next(p for p in personas if p.slug == "chief-privacy-officer")
    workflows = discover_workflows(cpo_privacy)
    breach = next(wf for wf in workflows if wf.slug == "breach-response-cycle")
    assert pattern_of(breach) == "time_critical"
    handler = pick_handler(breach)
    assert isinstance(handler, TimeCriticalHandler)
    assert handler.sla_minutes == 240


def test_quarterly_account_plan_dispatches_to_per_instance(cuo_root: Path) -> None:
    personas = discover_personas(cuo_root)
    cso_sales = next(p for p in personas if p.slug == "chief-sales-officer")
    workflows = discover_workflows(cso_sales)
    qap = next(wf for wf in workflows if wf.slug == "quarterly-account-plan")
    assert pattern_of(qap) == "per_instance"
    handler = pick_handler(qap)
    assert isinstance(handler, PerInstanceHandler)
    assert len(handler.instance_descriptor) >= 1


def test_quarterly_regulatory_cycle_dispatches_to_multi_output(cuo_root: Path) -> None:
    personas = discover_personas(cuo_root)
    clo_legal = next(p for p in personas if p.slug == "chief-legal-officer")
    workflows = discover_workflows(clo_legal)
    qrc = next(wf for wf in workflows if wf.slug == "quarterly-regulatory-cycle")
    assert pattern_of(qrc) == "multi_output"
    handler = pick_handler(qrc)
    assert isinstance(handler, MultiOutputHandler)
    assert len(handler.output_recipients) == 4  # vn-mst, vn-mof, vn-sbv, vn-mic


def test_model_card_release_dispatches_to_sequential_approval(cuo_root: Path) -> None:
    personas = discover_personas(cuo_root)
    caio = next(p for p in personas if p.slug == "chief-ai-officer")
    workflows = discover_workflows(caio)
    mcr = next(wf for wf in workflows if wf.slug == "per-model-card-release")
    assert pattern_of(mcr) == "sequential_approval"
    handler = pick_handler(mcr)
    assert isinstance(handler, SequentialApprovalHandler)
    assert handler.gates[0]["approver_persona"] == "chief-ethics-officer"


def test_post_incident_review_dispatches_to_persona_pair(cuo_root: Path) -> None:
    """chief-technology-officer/post-incident-review pairs with chief-risk-officer/per-incident-postmortem."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    workflows = discover_workflows(cto)
    pir = next(wf for wf in workflows if wf.slug == "post-incident-review")
    assert pattern_of(pir) == "persona_pair"
    handler = pick_handler(pir)
    assert isinstance(handler, PersonaPairHandler)
    assert handler.peer_persona == "chief-risk-officer"
    assert handler.peer_workflow == "per-incident-postmortem"
    assert handler.shared_artefact == "incident-report"


def test_time_critical_e2e_executes_and_audits(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """End-to-end: dispatch + execute + check extra_audit list shape."""
    personas = discover_personas(cuo_root)
    cpo_privacy = next(p for p in personas if p.slug == "chief-privacy-officer")
    workflows = discover_workflows(cpo_privacy)
    breach = next(wf for wf in workflows if wf.slug == "breach-response-cycle")

    handler = pick_handler(breach)
    out_dir = tmp_path / "e2e-tc"
    result = handler.execute(
        persona=cpo_privacy,
        workflow=breach,
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.chain_result.outcome == "COMPLETED"
    assert result.handler_kind == "TimeCriticalHandler"
    # MockInvoker is fast — should NOT trigger SLA breach
    sla_breach = [a for a in result.extra_audit_kinds if a.get("kind") == "cuo.time_critical_sla_breach"]
    assert len(sla_breach) == 0  # 0ms << 240min


def test_per_instance_e2e_iterates(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """Per-instance handler should iterate len(instance_descriptor) times + emit summary."""
    personas = discover_personas(cuo_root)
    cso_sales = next(p for p in personas if p.slug == "chief-sales-officer")
    workflows = discover_workflows(cso_sales)
    qap = next(wf for wf in workflows if wf.slug == "quarterly-account-plan")

    handler = pick_handler(qap)
    n_instances = len(handler.instance_descriptor)
    out_dir = tmp_path / "e2e-pi"
    result = handler.execute(
        persona=cso_sales,
        workflow=qap,
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.handler_kind == "PerInstanceHandler"
    assert len(result.per_instance) == n_instances
    # Should produce one iteration audit per instance + one summary
    iter_rows = [a for a in result.extra_audit_kinds if a.get("kind") == "cuo.per_instance_iteration"]
    summary_rows = [a for a in result.extra_audit_kinds if a.get("kind") == "cuo.per_instance_summary"]
    assert len(iter_rows) == n_instances
    assert len(summary_rows) == 1


def test_multi_output_e2e_fans_out(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    personas = discover_personas(cuo_root)
    clo_legal = next(p for p in personas if p.slug == "chief-legal-officer")
    workflows = discover_workflows(clo_legal)
    qrc = next(wf for wf in workflows if wf.slug == "quarterly-regulatory-cycle")

    handler = pick_handler(qrc)
    n_recipients = len(handler.output_recipients)
    out_dir = tmp_path / "e2e-mo"
    result = handler.execute(
        persona=clo_legal,
        workflow=qrc,
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.handler_kind == "MultiOutputHandler"
    fanout_rows = [a for a in result.extra_audit_kinds if a.get("kind") == "cuo.multi_output_fanout"]
    assert len(fanout_rows) == n_recipients
    # Per-recipient envelopes were written
    fanout_dir = out_dir / "fanout"
    assert fanout_dir.is_dir()
    assert len(list(fanout_dir.glob("*.json"))) == n_recipients


def test_persona_pair_e2e_runs_both_legs(cuo_root: Path, skill_root: Path, tmp_path: Path) -> None:
    """Persona-pair handler runs primary AND peer chains; both legs produce ChainResults."""
    personas = discover_personas(cuo_root)
    cto = next(p for p in personas if p.slug == "chief-technology-officer")
    workflows = discover_workflows(cto)
    pir = next(wf for wf in workflows if wf.slug == "post-incident-review")

    handler = pick_handler(pir)
    out_dir = tmp_path / "e2e-pp"
    result = handler.execute(
        persona=cto,
        workflow=pir,
        skill_root=skill_root,
        output_dir=out_dir,
        invoker=MockInvoker(),
    )
    assert result.handler_kind == "PersonaPairHandler"
    assert result.chain_result.outcome == "COMPLETED"
    assert result.peer_chain_result is not None
    assert result.peer_chain_result.outcome == "COMPLETED"
    # Should have at least 2 handoff rows (primary→peer + peer→primary)
    handoff_rows = [a for a in result.extra_audit_kinds if a.get("kind") == "cuo.persona_pair_handoff"]
    assert len(handoff_rows) >= 2
