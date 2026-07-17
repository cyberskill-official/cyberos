# TASK-IMP-097 gate-log evidence (implementing -> ready_to_review)

E1 - payload suite (AC 1), full run at this task's boundary (version still 2.6.3 - the bump
belongs to TASK-IMP-099), verbatim tail of
`bash tools/install/tests/test_full_sdp_payload.sh`:

    building scratch payload...
      ok   t01
      ok   t02
      ok   t03
      ok   t04
      ok   t05
      ok   t06
      ok   t07
      ok   t08
      ok   t09
    ----
    pass=9 fail=0

E2 - exactly one cross-reference line (AC 2, verify-not-test per spec):
  `grep -c 'Running CyberOS under sandboxed agents' modules/cuo/chief-technology-officer/workflows/ship-tasks.md`
  -> 1
  The line sits in §11a Swarm execution, directly after the one-writer-one-view sub-bullet.

E3 - section in the built payload (scratch build of the current source):
  GUIDE.md line 129: "## Running CyberOS under sandboxed agents"
  `grep -c 'clone the mounted repo to a local working copy\|local ref move, not a remote push\|replay each hook obligation manually'`
  over the scratch GUIDE.md -> 3 (all three gated phrases present)
  payload cuo/ship-tasks.md carries the cross-reference line (count 1).

E4 - sibling-suite guardrail at the same boundary, verbatim tail of
`bash tools/install/tests/test_workflow_helpers.sh`:

      ok   t10
      ok   t11
      ok   t12
    test_workflow_helpers: pass=12 fail=0

E5 - doc-anchor check over the new backticked path in ship-tasks.md
(`tools/install/docs/index.md` resolves): `bash scripts/check_doc_anchors.sh`
  -> "anchors OK: 445 references resolved across modules/skill + modules/cuo", exit 0.

Environment note (dogfooding the section's own subject): this run executed inside a
sandboxed agent with a 45 s per-command cap over a synced mount. The payload build measured
1.2 s on the mount, so every suite fit its own bash call and the local-clone pattern was not
needed; each suite ran as one capped command, one suite per call. No hooks were bypassed and
no commits were made by this sub-agent - branch mutations belong to the batch parent.
