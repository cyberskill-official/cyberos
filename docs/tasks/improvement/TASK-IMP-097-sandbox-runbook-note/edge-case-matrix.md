---
artefact: edge-case-matrix@1
task_id: TASK-IMP-097
total_rows: 9
created: 2026-07-17
verdict: pass (edge-case-matrix-audit: every category >=1 row, covered-by names real test functions or recorded evidence, SECURITY row points at pinned wording, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-097

Test functions live in tools/install/tests/test_full_sdp_payload.sh unless stated.

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | a future edit drops or renames the GUIDE section heading | t09 fails naming the missing heading before the payload can ship | t09_sandbox_runbook_guide (heading grep) |
| 2 | null/empty | the section survives but a gated sentence is reworded away (local-clone line or hook-replay line) | the specific grep fails with a message naming which line vanished - five independent greps, five distinct messages | t09_sandbox_runbook_guide (per-line greps) |
| 3 | bounds | GUIDE grows around the section (new sections before/after it) | greps are content-anchored, not line-anchored; heading grep is `^##`-anchored so a demotion to H3 or an inline mention cannot satisfy it | t09_sandbox_runbook_guide + inspection (grep '^## Running CyberOS under sandboxed agents') |
| 4 | malformed | the section accidentally introduces a "TBD" or a `| <digit>` table row | the standing t04 gate fails the suite - the new section is prose-only by design | t04_lifecycle_map_total (pre-existing, unmodified) |
| 5 | malformed | the cross-reference line duplicates in ship-tasks.md (the drift the spec's guardrail forbids) | recorded evidence: `grep -c 'Running CyberOS under sandboxed agents' modules/cuo/chief-technology-officer/workflows/ship-tasks.md` = 1 in gate-log-draft.md; AC 2 is verify-not-test by spec (a test for one prose line is out of proportion; t12 already pins the file's normative content) | gate-log-draft.md E2 |
| 6 | concurrency/order | this task and TASK-IMP-099 both edit ship-tasks.md in batch 4 | serialized by plan: one sub-agent (this one) drives 097 then 099 through ONE filesystem view - the §11a one-writer-one-view rule applied to its own document | batch-4 plan §0a + t12_doctrine_view_rules_vendored (the rule itself stays gated, tools/install/tests/test_workflow_helpers.sh) |
| 7 | SECURITY | the runbook read as license to push: "push back" misread as a remote push | wording pins the boundary in both deliverables: GUIDE says "local ref move, not a remote push ... no remote is touched, and the workflow's no-push policy (a human pushes to remotes) stays intact"; t09's third grep keeps the clause present forever | t09_sandbox_runbook_guide (local-ref-move clause grep) + code-review.md security row |
| 8 | DEGRADATION | payload built from a stale checkout (section in source, absent from a consumer's GUIDE) | detection: t09 greps a SCRATCH build of the current source on every suite run; recovery: the pre-commit payload rebuild + payload-gate.yml CI re-prove the build on every push (distribution-sync chain in ship-tasks.md) | t09_sandbox_runbook_guide + distribution-sync hooks (pre-existing) |
| 9 | DEGRADATION | consumer-facing leak: session paths or vendor names in the runbook (spec edge case 1) | section uses generic "sandboxed agent" framing and placeholder paths only (`/mnt/<repo>`, `/tmp/work`); detection at review - code-review.md checks the section names no vendor and no session path; recovery is a wording fix, gated thereafter by review | code-review.md judgment (AI-specific row) |

Documented-by-design: the GUIDE section states environment facts and working patterns, not new workflow rules - the two normative rules it complements (one-writer-one-view §11a, committed-object evidence §9) remain solely in modules/cuo/chief-technology-officer/workflows/ship-tasks.md at their v2.6.3 wording, which is why this task ships NO workflow_version bump (TASK-IMP-099 carries the round's bump). SECURITY-class beyond row 7: none - documentation and a read-only grep gate; no execution surface added.
