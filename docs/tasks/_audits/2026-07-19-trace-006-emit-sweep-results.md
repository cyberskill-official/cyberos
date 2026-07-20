# TRACE-006 emit-verb sweep - results (2026-07-19)

Companion to `2026-07-19-trace-006-acute-sweep-results.md`. Together the two files cover the COMPLETE danger zone. Read-only audit; no task status changed. Findings are fix-task CANDIDATES, never silent re-flips of a `done` task.

## Scope note (disclosure)

The acute pass ran off an extraction that was lost when the sandbox restarted. I regenerated it with a corrected parser (v2) before this pass. Validation: v2's ACUTE total matched the original EXACTLY (234), a strong faithfulness signal, though per-module boundaries shift slightly (v2 handles continuation lines differently and picked up an APP-* task the first pass missed). v2 yields 355 emit-only clauses vs the 335 originally sized - slightly WIDER, so nothing in the original scope was skipped. All 355 were reached; none guessed.

## The bar applied

emit: WEAK = the test only shows the emit function was CALLED, an intent/flag was recorded, a log line was produced, or it asserts on a RETURNED OBJECT. STRONG = the row/metric/span is observable AT ITS SINK - read back out of the audit chain / memory store / metric registry / span exporter / DB and asserted.

## Per-module tally (355 emit-only clauses)

| module | emit | PASS | WEAK | INSUFFICIENT | N/A |
|---|---|---|---|---|---|
| auth   | 76 | 2 | 6  | 66 | 2 |
| ai     | 69 | 2 | 15 | 44 | 8 |
| proj   | 60 | 0 | 2  | 54 | 4 |
| memory | 49 | 6 | 10 | 25 | 8 |
| skill  | 44 | 0 | 9  | 30 | 5 |
| cuo    | 16 | 0 | 1  | 14 | 1 |
| email  | 14 | 0 | 2  | 12 | 0 |
| imp    | 11 | 6 | 4  | 0  | 1 |
| mcp    | 7  | 0 | 0  | 7  | 0 |
| chat   | 5  | 0 | 4  | 1  | 0 |
| obs    | 4  | 0 | 1  | 3  | 0 |
| TOTAL  | 355 | 16 | 54 | 256 | 29 |

## Full danger zone - combined totals (589 clauses)

| pass | clauses | PASS | WEAK | INSUFFICIENT | N/A |
|---|---|---|---|---|---|
| acute (refuse/reject/render/halt/preserve) | 234 | 64 | 31 | 112 | 27 |
| emit | 355 | 16 | 54 | 256 | 29 |
| TOTAL | 589 | 80 | 85 | 368 | 56 |

Emit is materially worse than acute: 72% INSUFFICIENT vs 48%, and only 4.5% PASS. Across the whole danger zone, 62% of high-risk-verb clauses in `done` tasks have no on-disk test asserting the verb.

## Structural findings (these matter more than the individual clause ids)

1. THE OBSERVABILITY LAYER IS SPECIFIED BUT UNASSERTED, CORPUS-WIDE. Not one OTel span or metric assertion exists anywhere in `services/auth` (30 of its 66 INSUFFICIENT). Same shape in memory (18 of 25), skill (16 of 30), and most of ai. `circuit_breaker_test.rs` is the ONLY test in the entire ai-gateway suite that reads a metric back from its registry. This is one systemic gap, not ~90 unrelated ones: the specs mandate spans/metrics that no test anywhere observes.

2. THE CANONICAL emit WEAK SHAPE: a payload-struct test standing in for a sink observation. The test proves the row can be CONSTRUCTED, never that it is EMITTED. Found in auth (memory_bridge / op::audit payload tests), proj (audit_row_test.rs pure builders), email (audit_row_test.rs builders), and skill (bundles.rs returned structs). Most of the 54 WEAK are this shape.

3. THE GOOD PATTERN EXISTS AND IS ACHIEVABLE. memory has 6 genuine PASS using `iter_audit_rows(store)` -> filter by `op` -> assert payload fields, plus a NEGATIVE-emission assertion (`SELECT COUNT(*) == 0` before consent) - the strongest form in the corpus. auth has 2 (read back from `l1_audit_log`). imp has 6. So the bar is reachable; most modules simply never reached it.

4. imp IS CLEAN AGAIN: 0 INSUFFICIENT in both passes. But its 4 emit WEAK are sharp and worth fixing precisely because this is the rubric-era module: IMP-112 #1.1 and #1.6 are SELF-FULFILLING - the test writes the artefact (`printf '[]' > review-findings.json`, or builds the JSON with node) and then asserts it exists; the producer (code-review-author) never runs. IMP-111 #1.2 and #1.4 are doctrine-greps against SKILL.md prose.

5. chat's 4 WEAK COLLAPSE TO ONE FIX: the chat integration harness runs with `audit_pool: None` (audit chain disabled), so no chat clause citing an audit row can be proven at its sink today. Fix the harness, fix all four.

## Notable individual findings

- MEMORY-112 #12 is a LIVE BUG, not just a test gap: `episode.logged` is consumed by `dream/detectors.py:202` but never produced anywhere. The cited test asserts `seq == 1`, which actually PROVES the aux row is not emitted (AC #4 requires HEAD +2). A dangling consumer in shipped code.
- AI-010 #3 is the sharpest calibration failure in the corpus: the test sends Token/Usage/Done into an mpsc channel and asserts they come back in that order. It is a tautology - no pipeline, no SSE bytes, nothing rendered. AI-010 #4 binds `let _sse = ev.to_sse_event();` and asserts nothing at all.
- AUTH-104's §1 has TWO clauses numbered 7, which makes any traces_to mapping ambiguous. Spec-hygiene defect worth a lint rule.
- SKILL-101 spec-vs-code drift: the spec mandates `skill.invoked_started` / `invoked_completed`; the code emits `skill.invocation_started` / `invocation_completed`.
- AUTH-005 #6's arm queries the sink but asserts `n <= 1`, which is satisfied at n=0 - it cannot prove emission. A test that passes when nothing happened.

## WEAK (54) - real fix candidates: the test runs green and asserts less than emit

- auth (6): AUTH-004 #6; AUTH-005 #6; AUTH-006 #4; AUTH-110 #8, #15, #22.
- ai (15): AI-001 #6, #11; AI-002 #3, #4, #5; AI-004 #3; AI-009 #16; AI-010 #3, #4, #11; AI-011 #9; AI-012 #8; AI-016 #7; AI-018 #10; AI-022 #5.
- proj (2): PROJ-001 #6; PROJ-004 #5.
- memory (10): MEMORY-107 #10; 108 #10; 109 #2; 112 #12; 114 #8, #9; 115 #5, #7, #15; 116 #6.
- skill (9): SKILL-101 #1, #2; 104 #3, #7; 106 #4; 107 #4; 108 #10; 109 #9; 110 #9.
- cuo/email/imp/chat/obs (12): CUO-208 #3; EMAIL-001 #13, #23; IMP-111 #1.2, #1.4; IMP-112 #1.1, #1.6; CHAT-267 #8; CHAT-268 #14; CHAT-269 #12, #15; OBS-006 #7.

## INSUFFICIENT (256) - no on-disk test asserts the verb

Per-module counts are authoritative in the tally table above. Ids below are transcribed from the per-module audit records; reconcile against the table when authoring fix-tasks.

- auth (66): 001 #13, #15; 002 #13, #14; 003 #5, #12; 004 #16; 005 #15; 006 #14; 101 #6, #14, #22, #23; 102 #13, #14, #16, #20, #21; 103 #10, #15, #19, #21, #22; 104 #7, #8, #12, #14, #17, #18, #25; 105 #6, #7, #8, #10, #11, #12, #14, #22, #23, #26; 106 #6, #8, #13, #14, #16, #21, #26; 107 #22, #25; 108 #4, #6, #9, #12, #14, #17, #18, #25; 109 #7, #10, #11, #16; 110 #9, #11, #20, #26.
- ai (44): 001 #4; 002 #8; 003 #5, #14, #16; 004 #6, #8, #9, #12, #14, #15; 005 #9, #14; 006 #14; 007 #13; 008 #14; 010 #13, #14, #15; 011 #14; 013 #6; 014 #6, #15; 015 #6, #8, #14; 016 #13; 017 #9, #10, #15; 018 #14; 019 #15, #16; 020 #5, #11, #16; 021 #1, #4, #7, #16; 022 #4, #8, #10, #16.
- proj (54): 001 #14; 002 #1, #6, #12; 003 #8, #11; 004 #9, #10, #11, #12; 005 #8, #11; 006 #4, #7, #10; 007 #7, #11, #14, #15, #18, #20; 008 #2, #9, #10, #13, #19; 009 #6, #9; 010 #4, #9; 011 #6, #10, #12, #14, #17, #21; 012 #5, #6, #11, #17; 013 #6, #10, #17; 014 #8, #13, #15, #19; 015 #11, #12; 016 #7, #12, #16; 017 #9, #12.
- memory (25): 101 #7; 102 #14; 103 #8; 105 #7, #11, #13; 106 #6, #11; 107 #7, #11, #14; 108 #14; 109 #5, #9, #13; 110 #8, #9, #10; 111 #7, #11, #12; 113 #14; 115 #12; 116 #8; 121 #13.
- skill (30): 101 #3, #5, #7, #8, #11; 102 #3, #15; 103 #8, #9; 104 #6, #10, #11; 105 #7, #11, #12, #13; 106 #7; 107 #7; 108 #13, #14; 109 #11, #12; 110 #8, #10, #11, #13, #14; 113 #6; 115 #4; 201 #9.
- cuo (14): 101 #6, #8, #14, #15, #16, #20, #26; 102 #6; 103 #6; 104 #7; 105 #8; 106 #3, #4, #7.
- email (12): 001 #16, #20, #22; 004 #4, #9, #10; 005 #8; 009 #3, #4, #5, #8; 011 #8.
- mcp (7): MCP-001 #13, #22, #23, #24; MCP-002 #7; MCP-004 #15, #25.
- chat (1): CHAT-101 #8.  obs (3): OBS-002 #10, #11; OBS-006 #12.

## Remediation shape

Three distinct kinds of work, in increasing cost:
1. Strengthen an existing green test (the 85 WEAK across both passes). Cheapest. Several collapse to one fix (chat's 4 -> the `audit_pool: None` harness).
2. Write a missing test against behaviour that DOES ship. Medium.
3. The behaviour was never built as specified (proj-sync, the cuo supervisor, the skill Vietnam crates, most span/metric wiring). This is a spec-vs-as-built reckoning, not a test-writing task, and is the bulk of the 368.

Each becomes its own draft -> audited -> shipped fix-task. Prioritisation is the operator's call. Highest value first: the security-relevant auth clauses, the memory data-guards, and MEMORY-112 #12 (a live dangling-consumer bug).
