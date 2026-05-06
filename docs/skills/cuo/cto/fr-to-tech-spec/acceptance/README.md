# `fr-to-tech-spec/acceptance/` — priority test scenarios

> Acceptance fixtures for `cuo/cto/fr-to-tech-spec`. **Stub state at v0.1.0** — fixtures are pending the runtime/harness build per registry README Part 26 + Recipe 8. This README enumerates the priority scenarios the harness MUST cover when v0.3.0 ships.

## Priority scenarios (10)

Severity ordering: sev-0 = "must pass before v0.2.0 promotion"; sev-1 = "must pass before v1.0.0 promotion"; sev-2 = "nice to have, regression coverage."

### sev-0 (gate v0.1.0 → v0.2.0)

1. **INV-001: pass-only ingestion.** Input: 3 FRs, two with `pass` audit verdicts and one with `needs_human`. Expected: 2 specs written, 1 REFUSED + reason in output envelope. The needs_human FR MUST NOT have a spec written.
2. **INV-001 strict: stale verdict.** Input: 1 FR whose audit report has `overall_status: stale`. Expected: REFUSED with `refused_verdict: stale`.
3. **Happy path: single-FR spec.** Input: 1 FR with pass verdict, well-formed acceptance criteria, target_release `2026-Q3`. Expected: 1 spec PASS, all sections populated, sizing distribution non-zero, open_questions_count ≤ 2.
4. **Standalone-mode interview round-trip.** Invoke from chat with text `write a tech spec for FR-007`. Expected: STANDALONE_INTERVIEW.md script runs; user's answers synthesise into a valid input envelope; PLAN phase emits human-readable summary; user approves; WORKER writes the spec.
5. **Output envelope schema validation.** Every batch outcome (`BATCH_COMPLETE`, `HALTED_HITL`, `EXHAUSTED`, `REFUSED_NON_PASS_INPUTS`) produces an envelope that validates against `envelopes/fr-to-tech-spec.output.json`.

### sev-1 (gate v0.2.x → v1.0.0)

6. **INV-005: XL sizing escalation.** Input: 1 FR whose acceptance criteria decompose to 1 XL work-package. Expected: spec is HITL_PAUSE with `hitl_category: sizing_uncertainty` UNLESS the open-questions section explicitly justifies XL with a paragraph (caught at INV-005 check).
7. **INV-002: citation completeness.** Input: 1 FR. Synthesise a buggy spec authoring path that omits FR section refs. Expected: INV-002 fires; refinement_proposal emitted; pipeline pauses.
8. **Cross-skill chained run.** Invoke `cuo/cpo/fr-author` → `cuo/cpo/fr-audit` → `cuo/cto/fr-to-tech-spec` against a real PRD. Expected: end-to-end pipeline produces ≥1 PASS spec; trace_id is consistent across all three skills' audit rows; `genie.action_log` has the full sequence.
9. **Determinism is NOT required (judgement skill).** Invoke twice against the same input. Expected: outputs differ in word choice but converge on the same architecture summary, same work-packages, same sizing distribution. Cosine similarity of the two specs ≥ 0.85.

### sev-2 (regression coverage)

10. **Empty FR-list rejection.** Input envelope with `fr_paths: []`. Expected: schema validation fails (minItems: 1), `BOOT-003`.

## What's NOT covered yet

- **Multi-FR specs** — when a single tech spec covers multiple FRs (because they share a component). Pattern is plausible but not yet specified; defer to v0.2.x.
- **Spec-to-spec cross-references** — when one tech spec references another (e.g. "see TS-003 for the auth design"). Pattern requires the (future) `tech_spec@1` contract to define a cross-reference syntax.
- **Tech-spec audit** — the (future) `cuo/cto/tech-spec-audit` skill will validate specs against a rubric. Acceptance fixtures for THAT skill will live at `cuo/cto/tech-spec-audit/acceptance/`.

## Authoring discipline

When the harness ships and these fixtures get authored:

- Each fixture is a self-contained directory: `acceptance/<NN>-<slug>/` with `input/`, `expected-output/`, and `README.md` explaining the scenario.
- Inputs are real FR markdowns (or carefully-synthesised ones) — NOT mocks. The skill must work against shapes that real PRDs produce.
- Expected outputs are WHOLE specs (not snippets). Diff against actual output is full-spec, line-by-line.
- Determinism check: for sev-0 fixtures, run twice; expected outputs must be byte-identical (where determinism is contracted) or cosine-similar (where judgement is contracted, per scenario 9).

## Citations

- Pattern source — `cuo/cpo/fr-author/acceptance/README.md` and `cuo/cpo/fr-audit/acceptance/README.md`. Both follow the same priority-scenario style.
- Harness gate → registry README Part 26 (v0.3.0 milestone).
- Authoring discipline → registry README Part 19 Recipe 8.
