# TRACE-006 acute-verb sweep - results (2026-07-19)

Read-only audit. No task status was changed. Findings are fix-task CANDIDATES, not re-flips: a `done` task is never silently re-opened - a follow-up task is authored (draft -> audited -> shipped) and only its own testing->done gate moves it.

## Scope and method

- Corpus: the 166 in-scope `done` specs from the 2026-07-18 sizing note.
- Unit swept this pass: the 234 ACUTE high-risk-verb clauses (verbs refuse, reject, render, halt, preserve - the sharp end). The 335 emit-only clauses are NOT in this pass (see "Next").
- Rule under test: TRACE-006 (audit_rubric@1.0 RUBRIC.md #9) - for a #1 clause that cites a test, the cited test's ASSERTION must be at least as strong as the clause's VERB. A green test that asserts less than the verb is a defect.
- Calibration anchor: TASK-IMP-108 #1.7 (render a staleness report). Its pre-fix grep-in-payload test FAILS TRACE-006; its post-fix visible-markup test PASSES. Every batch was judged against that bar. The anchor was in the imp batch and self-checked PASS.
- Execution: 8 independent read-only audit delegations (one per module / group), each opening the cited test bodies. The severe absence claims were then independently corroborated on the host repo (source of truth), not just from the delegations' own greps.

## Headline

The dominant defect is NOT the weak-assertion pattern TRACE-006 was written for. It is that cited tests are ABSENT: 112 of 234 acute clauses (48%) have no on-disk test that asserts the verb, and most of those cite a test file/function that was never written. That is a TRACE-004 integrity failure (a `done` task whose status is not backed by the test it cites) - the coverage gate at the testing->done transition is supposed to prevent exactly this. The genuine TRACE-006 pattern (a green test that asserts less than the verb) is 31 clauses.

Total across 234 acute clauses: PASS = 64, WEAK = 31 (true TRACE-006), INSUFFICIENT = 112 (TRACE-004), N/A = 27.

## Per-module tally

| module | acute | PASS | WEAK | INSUFFICIENT | N/A |
|---|---|---|---|---|---|
| ai     | 46 | 11 | 10 | 16 | 9 |
| auth   | 35 | 4  | 1  | 29 | 1 |
| memory | 32 | 15 | 10 | 5  | 2 |
| proj   | 30 | 0  | 1  | 28 | 1 |
| imp    | 26 | 17 | 2  | 0  | 7 |
| skill  | 21 | 4  | 1  | 13 | 3 |
| cuo    | 16 | 2  | 3  | 10 | 1 |
| docs   | 11 | 8  | 1  | 1  | 1 |
| chat   | 8  | 3  | 1  | 4  | 0 |
| obs    | 3  | 0  | 0  | 2  | 1 |
| mcp    | 2  | 0  | 0  | 2  | 0 |
| email  | 2  | 0  | 0  | 2  | 0 |
| tpl    | 2  | 0  | 1  | 0  | 1 |
| TOTAL  | 234 | 64 | 31 | 112 | 27 |

## What the split means

- WEAK (31): a real, green, on-disk test exists but its assertion is weaker than the verb - e.g. a `refuse` clause whose test only checks the caller got an error, never that the guarded effect did not happen; a `render` clause whose test finds the value in a data payload no view reads; a `halt` clause whose test only greps doctrine prose for the MUST-HALT string. These are the sharp TRACE-006 fixes: strengthen the assertion.
- INSUFFICIENT (112): no on-disk test asserts the verb at all. Almost all of these cite a test file/function that is not on disk - the specified work was never built as specified, or shipped as a thinner helper.

## Root-cause clustering

The failures are not uniform. They track HOW the task shipped, not the verb:

- Clean (rubric-era, discipline holds): imp 0 INSUFFICIENT, docs 1. These are recent tasks whose cited tests exist and hit the rendered output / raised error / byte-equal bar. imp is the module that owns the audit rubric and it passes its own bar (17/26 PASS, the 2 WEAK are agent-workflow "structural by necessity" doctrine-grep tests, IMP-108 #1.5 / #1.6).
- Gutted (specified a service/client never built as specified):
- proj: `services/proj-sync/` does not exist; `apps/web` has zero test files; the cited `.tsx` client tests and proj-sync Rust tests are all absent. Only one real slice test on disk. 28/30 INSUFFICIENT.
- auth: the cited security tests (jti-replay, revoke, SAML assertion-sig, alg-confusion, HIBP, Lumi residency, cutover immutability) return zero matches in any .rs file - never written. 29/35 INSUFFICIENT.
- cuo: the entire CUO-101 supervisor suite and `services/cuo/` Rust tree are absent (real cuo tests live at modules/cuo/tests/). 10/16 INSUFFICIENT.
- skill: the Vietnam skill crates (108/109/110), SKILL-103 schema-v1, the SKILL-114 broker variant, and the SKILL-201 OCI registry were never built; 3 clauses PASS via consolidated-but-present assertions. 13/21 INSUFFICIENT.
- obs / mcp / email: the service sink-tests and MCP-001 protocol suite named in each spec are absent; only adjacent tests ship.
- Mixed: ai (11 PASS / 10 WEAK / 16 absent) and memory (15 PASS / 10 WEAK / 5 absent - the healthiest service module; real Python tests on disk).

## WEAK (31) - true TRACE-006 fix candidates (a green test asserts less than the verb)

- ai (10): AI-007 #6 (preserve: only is_some, no before/after; reload metric unread); AI-011 #7 (refuse: only redact->Err, no "call did not proceed unredacted"); AI-014 #7 (refuse+emit: in-memory Err only, no 503, no persona-tampered emit); AI-015 #4 / #10 / #11 (reject x3: assert a Display substring against an in-test re-implementation of the parser, not the shipped init_zdr_table); AI-016 #6 / #12 (refuse x2: disjunctive match accepts a cost-miss arm, so it can go green without the residency refusal firing); AI-020 #2 (preserve: hand-built literals, no real rerank, no index map-back); AI-022 #2 (preserve: inject-side only, extracted trace_id dropped).
- auth (1): AUTH-002 #11 (refuse: asserts the 400 but never that the subject row was not created).
- memory (10): MEMORY-103 #14 (preserve: .any() existence, not chain-order); 106 #2 / #3 (refuse: pure decision ==Refuse, row-not-ingested never asserted); 112 #11 (preserve: tests the new filter, not old-semantics preservation); 113 #7 (preserve: re-derivation, no before/after, no signature check); 115 #10 / #11 (refuse: only the raised exception, not chain/HEAD unchanged); 116 #1 (preserve: confirms dedup off by default, not 4-phase equality); 118 #6 (preserve: asserts op label "put", no value-equality vs a real put); 120 #9 (render: asserts projection extra.mode, not consumer-visible output).
- proj (1): PROJ-014 #1 (render: value inside a Vec<KanbanColumn> view-model, not a rendered view - the exact IMP-108 pattern; also 5 columns not the 6 named).
- imp (2): IMP-108 #1.5 (reject: greps STATUS-REFERENCE prose for the route-back rule, never runs a route-back); IMP-108 #1.6 (halt: greps ship-tasks prose for the MUST-HALT string, never drives a stop-before-re-entry).
- skill (1): SKILL-112 #15 (reject: bare predicate returns False, the FM-113 enforcement outcome never exercised at a sink).
- cuo (3): CUO-201 #3 (halt+emit: cited AC is the empty/null case, exercises neither verb); CUO-201 #7 (halt+emit: asserts the return object + file count, never the memory sink nor the HITL_HALT/drain outcome); CUO-203 #1 (halt: asserts a StripeRepeatHalt object, not that the workflow stopped).
- docs/chat/tpl (3): DOCS-005 #5 (render: greps the builder SOURCE, never renders the catalog); CHAT-268 #5 (render: asserts the WIRE payload, not the client placeholder - no apps/web test); TPL-001 #6 (render: greps the contract DOC for slot grammar, not a real substitution).

## INSUFFICIENT (112) - no on-disk test asserts the verb (TRACE-004)

Almost all cite a test file/function absent from disk. Grouped by module:

- ai (16): AI-001 #9, #14; AI-003 #7, #14; AI-005 #5, #6; AI-011 #15; AI-014 #4; AI-015 #6; AI-016 #7; AI-019 #8, #11; AI-020 #9; AI-021 #5; AI-022 #1, #3.
- auth (29): AUTH-004 #8; 005 #3; 006 #11; 101 #13, #18; 103 #8; 104 #5, #21, #22; 107 #4, #5, #11, #13, #15, #19; 108 #2, #6, #7, #21, #23, #25; 109 #8, #9, #15, #19, #22, #23; 110 #23, #24.
- memory (5): MEMORY-106 #6; 113 #3; 114 #1; 115 #14; 116 #10.
- proj (28): PROJ-005 #17, #19, #20; 006 #14; 007 #21; 008 #20; 009 #11; 010 #13; 012 #8, #18; 014 #6, #14, #17, #18; 015 #3, #7, #10, #16, #19; 016 #2, #3, #9, #13; 017 #2, #4, #5, #6, #13.
- skill (13): SKILL-103 #2, #5; 108 #3; 109 #3, #7; 110 #5, #16; 113 #7; 114 #11; 115 #8, #9; 201 #3, #4.
- cuo (10): CUO-101 #7, #9, #11, #12, #13, #18, #19; 104 #3; 106 #8, #12.
- docs/chat/obs/mcp/email (11): DOCS-007 #3; CHAT-267 #12, #14; CHAT-269 #18, #20; OBS-002 #4, #9; MCP-001 #4, #27; EMAIL-001 #24, #26.

## Corroboration (independent of the delegations)

Verified on the host repo (bash), not just from the delegations' own greps:
- proj: `services/proj-sync/` absent; `apps/web` test/spec/stories files = 0.
- auth: jti_replay / assertion_signature_required / verify_lumi_token / cutover_completed / hs256_token_rejected = 0 matches in any .rs; the one PASS (create_tenant_rejects_reserved_root_slug) is real on disk.
- ai: memory_writer_test.rs absent; precheck_refuses_over_budget present.
- skill: vietnam crate tests = 0; skill-broker integration.rs PRESENT.
- cuo: test_supervisor_* = 0; services/cuo absent; modules/cuo/tests/test_smoke.py present.
- obs: proxy_test.rs = 0, inject_logql_test.rs = 1.
- mcp: mcp-gateway/tests holds only sep986_* (TASK-MCP-003).
- imp (clean is real): test_render_stamp.sh and test_task_lint.sh both PRESENT.

## N/A (27)

Not defects - the tagged verb was an incidental stem (a label gloss, rationale prose, a MAY-gated parenthetical, a doc-content descriptor, or a table row name), not the clause's operative requirement. Listed per module in the delegation records; excluded from fix-candidates.

## Next

- The 335 emit-only clauses are unswept. Given the acute picture (48% unbacked), emit is expected to show the same shape at lower blast radius. Operator to decide: sweep emit too (the "3" choice), or stop at acute and triage.
- Remediation is a program, not one task: 31 WEAK (strengthen an existing test) are quick; 112 INSUFFICIENT split into "write the missing test against shipped behaviour" vs "the behaviour was never built" (the harder, spec-vs-as-built gap). Each becomes its own draft->audited->shipped fix-task. Prioritisation is the operator's call.
