# TASK-IMP-097 code review

Reviewer: batch-4 ship-tasks sub-agent (serial owner of both ship-tasks.md tasks this round). Diff: tools/install/docs/index.md (+47, the runbook section), modules/cuo/chief-technology-officer/workflows/ship-tasks.md (+1, the §11a cross-reference line), tools/install/tests/test_full_sdp_payload.sh (+23/-1, t09 gate + header note).

## Clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | GUIDE source gains the "Running CyberOS under sandboxed agents" section covering time caps + background death (hook chains, package installs), the local-clone pattern with local-ref push-back explicitly not a remote push, manual hook-obligation replay with `--no-verify` plus recorded evidence, and mount unlink/permission quirks | tools/install/docs/index.md:129 heading; four symptom/cause/pattern entries carry every mandated fact; wording pins "local ref move, not a remote push ... no-push policy (a human pushes to remotes) stays intact" and "record the replayed obligations and their outputs in the commit message or the task's gate log" |
| 1.2 | ship-tasks.md gains ONE cross-reference line, no duplicated content, no workflow_version bump | one §11a sub-bullet after the one-writer-one-view bullet; `grep -c 'Running CyberOS under sandboxed agents'` = 1 on the source file (gate-log-draft.md E2); zero rule sentences restated; version untouched by this task (2.6.3 at this task's boundary - the bump belongs to TASK-IMP-099 and is disclosed there) |
| 1.3 | built payload's GUIDE.md carries the section, gated by a grep in test_full_sdp_payload.sh against a scratch build | t09_sandbox_runbook_guide ok - five greps (heading, local-clone line, local-ref-move clause, hook-replay line, `--no-verify`) against the suite's own scratch payload; suite tail 9/9 in gate-log-draft.md E1 |
| AC 1 | scratch payload GUIDE carries the section incl. local-clone and hook-replay lines | t09 ok (E1) + direct payload greps (E3): GUIDE.md:129 heading, all three gated phrases present |
| AC 2 | exactly one cross-reference line | recorded grep -c = 1 (E2), per the spec's verify-not-test rationale |

## Judgment

- **Consumer framing**: the section names no session paths, no vendor brands, no tool-specific flags beyond git itself; placeholders are `/mnt/<repo>` and `/tmp/work`. Generic "sandboxed agent" framing as the spec's edge case demands.
- **Policy surface**: the one sentence that could weaken policy is pinned the other way - the push-back is stated twice to be a local ref move with no remote touched, and t09's third grep keeps that clause in every future payload. The `--no-verify` guidance is conditional on recorded evidence, which strengthens (not weakens) the gate discipline it touches.
- **Placement**: the cross-reference sits inside §11a's swarm sub-bullets, directly under the one-writer-one-view rule it complements, so a reader meets rule and runbook together; §9's committed-object rule is named in the section's closing line from the GUIDE side.
- **Test economy**: t09 reuses the suite's existing scratch payload (zero extra builds); the five greps are content-anchored so GUIDE growth cannot break them, and `^##`-anchoring stops an inline mention from satisfying the heading check.
- **Disclosure (version)**: this task deliberately ships NO workflow_version bump - a prose pointer is not a normative change. The round's single bump (2.6.3 -> 2.6.4) and both suite pin moves land in TASK-IMP-099, reviewed and disclosed in docs/tasks/improvement/TASK-IMP-099-queue-selection-p0-p3/code-review.md.
- **Security**: none - documentation plus a read-only grep gate; no execution surface, no secrets, no network.

Verdict: no open findings.

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
