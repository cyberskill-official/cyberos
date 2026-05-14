# `requirements-discovery/acceptance/` — priority test scenarios

> **Stub state at v0.1.0.** Fixtures pending the runtime/harness build (registry README Part 26 + Recipe 8). This README enumerates the priority scenarios the harness MUST cover at v0.3.0+.

## Priority scenarios (12)

### sev-0 (gate v0.1.0 → v0.2.0)

1. **INV-001: BRAIN unreachable refusal.** Simulate BRAIN MCP server down. Expected: skill halts at CONTRACT_ECHO with BOOT-005; brief is NOT written.
2. **INV-002: incomplete-interview refusal.** User answers only 8/20 questions. Expected: brief NOT written; HITL_BATCH_REQUEST emitted listing the 12 unanswered questions.
3. **Triage all-pass happy path.** All 5 triage gates green. Expected: brief written, triage_verdict: proceed, next_skill_recommendation: cuo/cpo/prd-author (when prd-author runtime is available).
4. **Triage strategic-fit conflict.** Project conflicts with `company:locked-decisions/DEC-NNN-*.md`. Expected: triage_verdict: reject, brief carries `## Triage Reasoning` H2 citing the locked decision.
5. **Triage capacity insufficient (revise).** Project requires more engineer-weeks than `member:*` indicates. Expected: triage_verdict: revise, user asked to (a) amend, (b) proceed-anyway with reservations, (c) stop.
6. **Project_kind classification correctness.** Test inputs across all 8 enum values; verify each is classified correctly. Hard cases: a "marketing campaign that requires building landing pages" (could be marketing OR software) — expected to ask the disambiguation question.

### sev-1 (gate v0.2.x → v1.0.0)

7. **BRAIN read budget enforcement (INV-006).** Construct an interview that triggers >10 BRAIN queries. Expected: phase 4 halts at query 10 + records advisory in audit row + proceeds to phase 5 with what was collected.
8. **Authority-marker correctness (INV-004).** Verify every Goals item has an inline `<!-- authority: ... -->` marker; brief without markers is REJECTED.
9. **client_visible: true with valid client_id.** Brief reads `client:<id>/`, sets client_visible: true in frontmatter, populates `## Client Context`.
10. **client_visible: true with invalid client_id (BOOT-006).** Brief refused; user prompted to fix or proceed without client_id.
11. **Amendment-batch round-trip.** v1 written; user batches 3 amendments; v2 written with `discovery_iteration: 2`; second amendment-batch produces v3.
12. **Triage reject persistence.** Re-running discovery on the same project_kind + same locked-decision conflict produces the same triage_verdict: reject (verdict is deterministic at the rubric level, even though brief content varies).

## What's NOT covered yet

- **Multi-language support** — interview in Vietnamese: pending v0.2.0 i18n pipeline.
- **Streaming-mode discovery** — for very large project pitches that don't fit one chat turn: pending v0.3.0+.
- **Prior-art recommendation** — the skill could proactively suggest "this looks like prior project X; want to learn from it?" but the recommendation engine is gated on a vector index over BRAIN.

## Citations

- Pattern source — `cuo/cpo/fr-author/acceptance/README.md` and `cuo/cpo/fr-audit/acceptance/README.md`.
- Harness gate → registry README Part 26 (v0.3.0 milestone).
- Authoring discipline → registry README Part 19 Recipe 8.
