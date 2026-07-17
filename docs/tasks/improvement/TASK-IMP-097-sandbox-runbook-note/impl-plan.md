---
artefact: implementation-plan@1
task_id: TASK-IMP-097
created: 2026-07-17
estimate_pts: 1
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 2)
---
# Implementation plan - TASK-IMP-097

Slices (each maps to §1 clauses and edge-case-matrix rows):
1. The GUIDE section - insert "## Running CyberOS under sandboxed agents" into
   tools/install/docs/index.md between "Product vs platform version" and "Where
   to go next". Runbook voice, four entries, each symptom -> cause -> working
   pattern: (a) hook chains and package installs killed by the per-command cap,
   background processes dying with the call -> replay each hook obligation
   manually, commit with `--no-verify`, record the replayed obligations and
   their outputs in the commit message or gate log; (b) builds/suites over the
   synced mount -> clone the mounted repo to a local working copy, build/test
   there, land the result back via `git fetch` + `git merge --ff-only` on the
   mounted side - a local ref move, not a remote push, no-push policy intact;
   (c) package-manager churn kept in the local copy, never synced back; (d)
   mount unlink/permission lag -> wait and re-check. Generic framing, no vendor
   names, placeholder paths only. Prose-only so the standing t04 GUIDE gates
   stay clean. (§1 #1.1; rows 3, 4, 9.)
2. The cross-reference - ONE sub-bullet in
   modules/cuo/chief-technology-officer/workflows/ship-tasks.md §11a Swarm
   execution, directly after the one-writer-one-view bullet: constrained
   environments follow the GUIDE runbook (source
   `tools/install/docs/index.md`); the rules above stay normative. No rule text
   duplicated, NO workflow_version bump (TASK-IMP-099 carries the round's
   bump). Verify count=1 recorded in gate-log-draft.md. (§1 #1.2; rows 5, 6.)
3. The payload gate - t09_sandbox_runbook_guide in
   tools/install/tests/test_full_sdp_payload.sh, reusing the suite's one
   scratch payload ($TMP/payload built at line 13): five greps against
   GUIDE.md - the `^##`-anchored heading, "clone the mounted repo to a local
   working copy", "local ref move, not a remote push", "replay each hook
   obligation manually", and `--no-verify` - each with its own failure
   message; wired into the run line and the file-header comment. (§1 #1.3;
   rows 1, 2, 7, 8.)

Pattern conformance (context-map): index.md edits are the payload (build.sh
copies it verbatim to GUIDE.md - no other plumbing); the suite grows in its own
idiom (function + run-line, shared scratch payload, ok/fail counters); the
cross-reference follows §11a's sub-bullet house style with a backticked path
that scripts/check_doc_anchors.sh resolves.

Estimate: 1 pt (~2 h) - matches spec effort_hours: 2. Actual landed surface: 3
modified files, 0 new (tools/install/docs/index.md +47 lines,
modules/cuo/chief-technology-officer/workflows/ship-tasks.md +1 line,
tools/install/tests/test_full_sdp_payload.sh +23/-1); suite 9/9 in ~4 s
including the scratch build.
