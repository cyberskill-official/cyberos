# `prd-author/acceptance/` — priority test scenarios

> **Stub state at v0.1.0.** Fixtures pending v0.3.0 harness.

## Priority scenarios (12)

### sev-0 (gate v0.1.0 → v0.2.0)

1. **INV-001: refuse rejected brief.** Input: `project_brief@1` with `triage_verdict: reject`. Expected: outcome `REFUSED_REJECTED_BRIEF`; no PRD written; refusal_reason cites the brief's triage_reason.
2. **INV-002: zero llm-implicit on Goals.** Construct a flow where the synthesis step would naturally produce a Goal with no clear citation. Expected: authority-elevation pass (Q5 of STANDALONE_INTERVIEW.md) prompts the user; final PRD has zero llm-implicit Goals.
3. **INV-003: refuse revise without override.** Input: `triage_verdict: revise`, envelope's `proceed_despite_revise` is false (default). Expected: outcome `REFUSED_REVISE_NEEDS_OVERRIDE`.
4. **INV-003 happy path: revise with override.** Input: same brief + `proceed_despite_revise: true`. Expected: PRD written; body carries `## Reservations Recorded From Discovery` H2 citing the brief's triage flags.
5. **Happy path: PRD from passing brief.** Input: passing brief + 3 follow-up answers. Expected: PRD written, all 11 required H2 sections populated, every Goals item carries an authority marker, schema validates.

### sev-1 (gate v0.2.x → v1.0.0)

6. **INV-007: confidentiality non-degradation.** Brief is `regulated`; PRD attempts `internal`. Expected: rejected; user prompted.
7. **EU AI Act limited classification flows through.** Brief has `eu_ai_act_risk_class: limited`; PRD inherits + populates `## EU AI Act Considerations` per the contract template.
8. **EU AI Act high classification triggers CLO sign-off field.** Brief has `eu_ai_act_risk_class: high`; PRD's `cl_sign_off` field is required before `prd_status` can flip from `draft` to `approved`.
9. **Amendment-batch round-trip.** v1 written; user batches 4 amendments; v2 written with `prd_iteration: 2`; second batch produces v3.
10. **Chained-mode from requirements-discovery.** End-to-end: discovery emits brief with `next_skill_recommendation: cuo/cpo/prd-author` → supervisor invokes prd-author → PRD written. Trace_id consistent across both skills' audit rows.
11. **non-software project_kind.** Brief has `project_kind: marketing_campaign`. Expected: PRD authored with marketing-appropriate `## Quality Bars` (not "p95 latency" — instead "campaign delivery rate", "open rate", etc.).

### sev-2 (regression coverage)

12. **Refused brief doesn't pollute output_dir.** When INV-001 fires, no partial PRD file is left in output_dir.

## Citations

- Pattern source — sibling skills' acceptance/README.md files.
- Harness gate → registry README Part 26 (v0.3.0 milestone).
