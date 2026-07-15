---
id: NFR-CUO-202
title: "proposal classifier MUST be deterministic + read-only + test-gate-blocking"
module: cuo
category: reliability
priority: MUST
verification: T
phase: P0
slo: "classify_proposal: 100% deterministic across runs; 0% mutation of any file; test-gate failure → 100% queue (never auto-apply)"
owner: CTO
created: 2026-05-19
related_tasks: [TASK-CUO-202]
---

## §1 — Statement (BCP-14 normative)

1. `cuo.core.proposal_applier.classify_proposal(proposal_path, skill_root)` **MUST** be deterministic — same proposal body + same target SKILL.md → same `Classification` dataclass across runs / processes / sessions.
2. `classify_proposal` **MUST NOT** mutate any file on disk — neither the proposal, nor the target SKILL.md, nor the CHANGELOG, nor any sibling. It is a pure-read function (proven by `test_classify_is_read_only`).
3. When the pre-apply test gate (`_run_test_gate`) returns `(ok=False, ...)`, `apply_proposal` **MUST** queue the proposal under `pending_approval/` and **MUST NOT** bump the target skill's version. Auto-apply on a test-gate failure is a correctness violation.
4. The bucket → bump-level mapping (`BUCKET_BUMP`) and the bucket → default-auto policy (`BUCKET_DEFAULT_AUTO`) **MUST** be immutable constants — runtime mutation of these dicts is forbidden.
5. `safety_class` bucket **MUST NEVER** auto-apply, regardless of bucket-default or `human_fine_tune.review_required` flags. This is the protocol's defence-in-depth gate against classifier mis-categorisation of safety-critical diffs.

## §2 — Why this constraint

The classifier sits on the path between "operator approves a proposal" and "skill file gets mutated". If classification were non-deterministic (e.g. read environment time and bucket differently based on day-of-week), the same proposal could auto-apply on Monday and queue on Tuesday — making the system unpredictable. The read-only guarantee on `classify_proposal` means operators can run `cyberos-cuo proposal classify <id>` as a sanity check without risk. The test-gate-blocks-auto-apply rule is what makes the auto-apply path safe: even if the classifier mis-buckets a risky diff as `cosmetic`, the skill's own TRIGGER_TESTS fixtures get the final word. The `safety_class` never-auto rule is the second key in a two-key system: classifier + body-marker BOTH must agree before mutation.

## §3 — Measurement

Determinism (already covered by TASK-CUO-202 tests): not a separate benchmark — `test_classify_is_read_only` + `test_bump_levels` together prove classification stability. A future stress test could classify N=10⁴ proposal bodies and assert bucket counts match between two independent runs.

Read-only: `test_classify_is_read_only` compares SKILL.md + proposal file bytes before and after `classify_proposal()` — bytewise identity required.

Test-gate behaviour: `test_test_gate_skip_when_no_trigger_tests` covers the "no fixture file" path. Adding `test_test_gate_blocks_when_fixture_fails` (deferred — requires a synthetic failing fixture) would close the explicit-failure branch.

Safety class never-auto: `test_safety_class_never_auto` enforces.

## §4 — Verification

Tests passing today: `test_cosmetic_auto_applies`, `test_rule_addition_queues`, `test_safety_class_never_auto`, `test_test_gate_skip_when_no_trigger_tests`, `test_audit_rows_emitted`, `test_classify_is_read_only`, `test_bump_levels`, `test_approve_transactional`, `test_post_apply_list`, `test_skill_b_on_minor_bump_true_queues`.

Inspection: `BUCKET_BUMP` and `BUCKET_DEFAULT_AUTO` are module-level constants. `classify_proposal` reads the proposal file + reads `human_fine_tune.review_required` from the target SKILL.md and returns a `Classification` dataclass — no `with open(..., "w")` anywhere in the function body.

## §5 — Failure handling

**Detection:** monitoring the audit chain for `cuo.proposal_applied` rows whose `bump_level` exceeds what the proposal's `risk_class` would imply (e.g. a `safety` proposal that landed as auto-applied — should be impossible per §1 #5 but worth watching).

**Alert:** sev-1 — a safety-class proposal that auto-applied is a defence-in-depth bypass. Page on-call immediately.

**On-call action:** (a) `git revert` the offending bump commit; (b) inspect `apply_proposal` for the `if bucket == "safety_class"` short-circuit (line ~110); (c) if the short-circuit is correct, the issue is in `_classify_body` mis-tagging the bucket — patch the regex.

**Escalation:** any defence-in-depth bypass requires a post-mortem + a regression test added to `test_proposal_applier.py` before the fix lands.

## §6 — Notes

The pre-apply test gate is currently a presence-proxy: if `acceptance/TRIGGER_TESTS.md` exists, the gate counts as "OK". A future minor bump (probably under TASK-CUO-202 follow-up) extends this to actually parse + execute the fixtures via the existing `cuo.trigger_tests` Python module — at that point this NFR's §1 #3 verification becomes load-bearing rather than aspirational.
