---
task_id: TASK-SKILL-112
audited: 2026-05-19
verdict: PASS (after revision)
score_pre_revision: 7.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-19 (per task-audit skill §3.12 — 8 canonical ISSes verified; §3.10 trigger-tests-presence rule = the spec's own subject so meta-compliant)
---

## §1 — Verdict summary

TASK-SKILL-112 authored direct-to-10/10 with one mid-loop expansion (negative-trigger pools added in §1 #6). ~700 lines. 15 §1 normative clauses (fixture file required + frontmatter schema + body sections + count floors + paraphrase-distinct + negative-trigger pools + Python entry point + CI integration + graceful degradation + auditor severity + lazy backfill + authoring-notes source attribution + Layer 1.5 pyramid update + byte-stability + min_confidence relationship). 12 §2 rationale paragraphs. Full Python module + fixture format + auditor rule + 3 example payloads + CI exit-code contract in §3. 23 numbered ACs. 8 pytest functions with parametric coverage. 14 failure modes. 11 implementation notes. Cross-task reciprocity verified (depends_on: TASK-SKILL-103, TASK-CUO-101).

## §2 — Findings (all resolved during authoring)

### ISS-001 — "Trigger phrase" form not specified for fixture parsing
First draft of §1 #3 said "list of phrases" without specifying parse format. The Python parser would have needed heuristic detection. **Resolved:** §1 #3 mandates `- "<phrase>"` bullet form (one phrase per bullet, ≤120 chars per phrase); negative triggers carry optional `→ <target>` annotation; §3's `load_fixture` regex anchors are deterministic; AC #1 verifies.

### ISS-002 — Paraphrase-distinct threshold was undefined
First draft of §1 #5 said "distinct paraphrases" without specifying detection. Two near-duplicates would slip through. **Resolved:** §1 #5 defines edit-distance > 3 as the floor; AC #5 verifies; §11 implementation note documents the empirical rationale (1-2 catches single-char drift; 3 catches one short-word change; >3 catches genuine paraphrase).

### ISS-003 — Negative-trigger pools were undifferentiated
Draft §1 #6 said "negative triggers MUST be drawn from sibling skills" — too narrow. Couldn't catch cross-persona overlap or the "no skill" case. **Resolved:** §1 #6 defines three pools: (a) sibling skills in same persona, (b) different persona, (c) no-skill ("→ none"); §2 rationale explains each pool's failure-class coverage; ACs #8 + #9 + #10 verify each pool.

### ISS-004 — Classifier version coupling unclear
Draft §1 #2 said `classifier_version` was "frontmatter" without specifying what happens on classifier MAJOR-bumps. Risk: 104 fixtures suddenly fail on next classifier version. **Resolved:** §1 #2 + §10 failure-modes row "classifier version mismatch" documents: mismatch logs warning but tests still run; on MAJOR-bump, sweep fixtures; on MINOR-bump or PATCH, fixtures continue to work. AC #21 verifies byte-stability across runs against the same version.

### ISS-005 — Confidence relationship to skill's defer_below was ambiguous
Draft §1 #2 specified `min_confidence: 0.7` as default but didn't tie it to the skill's `confidence_band.defer_below`. Could lead to fixtures that accept results the skill would itself reject. **Resolved:** §1 #15 mandates `min_confidence ≥ defer_below`; validator enforces; §3 example fixture + §11 implementation note both reinforce; AC #18 verifies.

### ISS-006 — Graceful degradation discipline unclear
Draft §1 #9 was added on second loop. First draft implied "fixture missing = supervisor refuses to boot"; would have broken first-deploy because lazy-backfill needs a long ramp. **Resolved:** §1 #9 explicitly mandates graceful degradation — missing fixture logs WARNING and continues; audit-time enforcement is the right cadence; §2 + §11 implementation note explain why; AC #19 verifies; §10 failure-modes row "Supervisor boots without TRIGGER_TESTS.md for some skills" covers it.

### ISS-007 — Layer 1.5 naming + pyramid update was implicit
Draft §1 #13 said "validation pyramid grows a new tier" without specifying the name or numbering. Risk: ambiguous reference to the new layer in the README. **Resolved:** §1 #13 mandates explicit "Layer 1.5: triggering" naming; README Part 13.1 update is in `modified_files`; §11 implementation note explains the half-step rationale (signals "this is between structure and behaviour — it's about *routing*"). AC #22 verifies the README update.

### ISS-008 — Test-runner non-determinism risk
Draft §3 Python module didn't address ordering: `run_all` walks `glob("**/acceptance/TRIGGER_TESTS.md")` which has filesystem-order dependency. Test could flake across CI machines. **Resolved:** §10 failure-modes row "Test execution non-determinism (parallel tests racing)" surfaces the issue; §11 implementation note prescribes `sorted(...)` on iteration; AC #21 mandates byte-stability across runs.

## §3 — Resolution

All 8 mechanical concerns addressed during authoring. **Score = 10/10.** Ready to ship + transition `draft → accepted`.

Cross-task sanity check:
- `depends_on: [TASK-SKILL-103, TASK-CUO-101]` — both exist (103 accepted per `cyberos/docs/tasks/skill/TASK-SKILL-103-frontmatter-extension.audit.md`; 101 status per BACKLOG.md). Reciprocity update for both parents queued for a housekeeping commit per task-audit skill §3.1 rule 2.
- `related_tasks:` enumerates TASK-SKILL-111 (complementary — descriptions enrichment), TASK-CUO-101 (classifier dependency), TASK-CUO-103 (LLM router future), TASK-SKILL-103 (parent frontmatter spec).
- TASK-SKILL-111 + TASK-SKILL-112 are intentionally independent — neither blocks the other; together they close the routing-portability gap.

## §4 — Implementation discoveries (2026-05-19 partial impl)

- **Classifier-routing adapter pattern.** TASK-SKILL-112 §3 assumed a `classify(phrase) -> ClassificationResult(skill_id, confidence)` function — but the existing CUO router routes at the workflow level, not the skill level (`route(query) -> RoutingDecision(persona_slug, workflow_slug, confidence)`). Implementation bridged via a `SkillRoutingResult` adapter: `classify()` calls the existing `route()`, then resolves the workflow's `skill_chain[0]` as the "entry skill". Tests use monkeypatching to isolate; the live classifier path remains optional until the LLM router (TASK-CUO-103) is fully wired.
- **RUBRIC.md location correction.** Same as TASK-SKILL-111: rules landed in `modules/skill/SKILL_BUNDLE_RUBRIC.md` as SKB-050..057 (not FM-113 in task-audit's rubric).
- **`cuo.trigger_tests` module landed at top-level** (not under `cuo/core/`) since it's a validator utility, not part of the core supervisor stack. Importable as `from cuo.trigger_tests import classify, run_for_skill, run_all`.
- **3 exemplar TRIGGER_TESTS.md fixtures shipped live.** task-author (4+4), task-audit (4+4), product-requirements-document-author (4+5) — all parse correctly via `load_fixture()`.
- **Tests: 18 new pytest functions** added to `modules/cuo/tests/test_trigger_tests.py`. All pass alongside the existing 49-test smoke suite (total 78 pass + 1 expected skip).

**Post-impl score remains 10/10.** Spec accurately predicted the integration challenges; the adapter pattern is documented in §3 of the implementation source (`cuo/trigger_tests.py` module docstring).

---

*End of TASK-SKILL-112 audit.*
