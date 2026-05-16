---
fr_id: FR-CUO-101
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 14
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; AUTHORING.md §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per AUTHORING.md §0; all 14 ISSes resolved in revision)
---

## §1 — Verdict summary

FR-CUO-101 ships the CUO Phase 2 LangGraph supervisor on top of the Phase 1 rule router. Scope: 26 §1 normative clauses covering the 5-node graph topology, confidence-band branching, LLM cascade via LiteLLM-shaped proxy routed through AI Gateway FR-AI-008, 11-persona catalogue with intrinsic defer-to-human matrix, structured Pydantic LLM output, 3-second cascade budget, BRAIN audit row per decision in every path, EU AI Act Art. 12/13/26 compliance, OTel instrumentation, CLI subcommand, dry-run mode, replay-equivalence guarantee, state versioning, forward-compat `next_step: null` stub. 17 rationale paragraphs. §3 contains: CuoState TypedDict with versioning, StateGraph construction with conditional edges, branch-node logic with the four-band decision tree, LLM cascade node with timeout + retry + hallucination rejection, LiteLLM proxy module asserting no direct provider imports, persona catalogue with defer matrix, audit row builder, CLI entry point. 31 ACs. §10 has 32 failure-mode rows. §11 has 24 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — LLM cascade infinite loop risk
First-pass `branch → cascade → branch` edge could re-enter cascade indefinitely on persistently-low LLM confidence. Resolved: §1 #5 + DEC-162 + `cascade_taken: bool` flag in state + branch-node logic that forces `ask` on second visit; AC #7 enforces.

### ISS-002 — Direct provider import bypassing AI Gateway
First-pass had no architectural enforcement that CUO routes through the gateway. Resolved: §1 #11 + DEC-161 + AST-walker test `test_supervisor_litellm_routes_via_gateway` that fails CI on any `import boto3 | import anthropic | import openai` in the supervisor package; AC #15.

### ISS-003 — Defer-to-human matrix overridable via config
First-pass allowed per-tenant matrix overrides; would defeat EU AI Act Art. 26. Resolved: §1 #19 + DEC-164 — matrix is intrinsic to the persona (code, not config); ADR-required to change.

### ISS-004 — LLM freeform response silently invoked
First-pass would regex-parse LLM output; risk of acting on hallucinated free text. Resolved: §1 #6 + DEC-163 — Pydantic `LlmRoutingPick` schema with hard validation + 1-retry-then-fallback to ask; AC #9.

### ISS-005 — Hallucinated skill name silently invoked
First-pass trusted the LLM's `skill_name`. Resolved: §1 #6 — explicit `skill_name not in catalog` check after Pydantic validation; AC #10.

### ISS-006 — No timeout budget for LLM cascade
First-pass would wait indefinitely on slow LLM. Resolved: §1 #6 + DEC-169 — hard 3-second `asyncio.wait_for` budget; timeout → fall through to ask + emit `cuo.llm_cascade_timeout` row; AC #8.

### ISS-007 — Defer path emits no audit row
First-pass only emitted on successful invocation. Defer is itself an AI decision. Resolved: §1 #8 + DEC-165 — emit `cuo.routing_decision` on EVERY path (auto, ask, cascade, defer); AC #17.

### ISS-008 — Destructive-skill bypass at high confidence
First-pass would auto-invoke any skill at confidence ≥ 0.70. Resolved: §1 #7 + DEC-170 — `destructive: true` annotation in skill catalog triggers capability broker FR-SKILL-104 Elicitation flow; supervisor refuses to override regardless of confidence; AC #12.

### ISS-009 — State schema version absent
First-pass had no versioning; replay across schema changes would break. Resolved: §1 #12 + DEC-167 + `cuo_state_v: int` field with ±2 tolerance; AC #16.

### ISS-010 — `next_step` field shape ambiguity
First-pass omitted the field at slice 2; FR-CUO-104 would have to handle both shapes. Resolved: §1 #25 + #26 + AC #18 — field is present-but-null at slice 2; one shape forever.

### ISS-011 — Persona JWT not validated at supervisor entry
First-pass trusted the request envelope; a `genie` JWT could trigger `cfo` routing. Resolved: §1 #10 + DEC-166 + parse-node validation; AC #13.

### ISS-012 — PII in BRAIN row query field
First-pass stored raw query. Resolved: §1 #24 + `apply_brain_pii_rules` per FR-BRAIN-111; AC #19. Raw query retained transient in OTel span.

### ISS-013 — Replay equivalence not enforced
First-pass had no CI test for determinism. Resolved: §1 #17 + 15-golden-fixture suite + `test_supervisor_idempotency`; AC #20.

### ISS-014 — Cascade prompt ungrounded if catalog reloaded mid-request
First-pass would build the prompt against a possibly-stale catalog snapshot. Resolved: §10 failure row "Catalog snapshot hash mismatch between request + record" + `catalog_snapshot_hash` captured at parse-node entry and validated at record; sev-3 alarm on drift.

## §3 — Resolution

All 14 mechanical concerns addressed in the first revision pass. **Score = 10/10.**

Per AUTHORING.md §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (LangGraph × 5-node closed topology × confidence-band routing × LiteLLM gateway routing × 11-persona catalogue × defer-to-human matrix × structured Pydantic LLM output × 3-second budget × hallucination rejection × destructive-skill capability-broker gate × BRAIN audit per path × state versioning × replay equivalence × forward-compat `next_step` stub × persona JWT validation × PII scrubbing × OTel instrumentation × CLI + dry-run), not by line targets.

---

*End of FR-CUO-101 audit.*
