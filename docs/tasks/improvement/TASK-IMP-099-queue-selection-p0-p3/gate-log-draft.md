# TASK-IMP-099 gate-log evidence (implementing -> ready_to_review)

E1 - workflow helpers suite (AC 1-3), full run, verbatim tail of
`bash tools/install/tests/test_workflow_helpers.sh`:

      ok   t01
      ok   t02
      ok   t03
      ok   t04
      ok   t05
      ok   t06
      ok   t07
      ok   t08
      ok   t09
      ok   t10
      ok   t11
      ok   t12
      ok   t13
    test_workflow_helpers: pass=13 fail=0

E2 - rule and version in the SOURCE
(modules/cuo/chief-technology-officer/workflows/ship-tasks.md):
  line 3:   workflow_version: 2.6.4
  line 312: order by priority: `p0` before `p1` before `p2` before `p3` (legacy MoSCoW values
            map per FM-105), then `created` ascending, then id ascending. Echo the
  `grep -ci moscow` over the file -> 1 (the legacy-mapping parenthetical is the only mention;
  the sentence tail and the echo-line format are byte-identical to the pre-change bytes)

E3 - t13 negative-pattern probes (the rule SHAPE is targeted, not the word):
  echo 'order by priority (MUST before SHOULD before COULD), then' | grep -Eiq
  '(MUST|SHOULD|COULD|WON.?T)[[:space:]]+before[[:space:]]+(MUST|SHOULD|COULD|WON.?T)'
    -> match: the retired wording is CAUGHT
  echo 'legacy MoSCoW values map per FM-105' | <same pattern>
    -> no match: the parenthetical is allowed
  single-sided prose (skill_chain step 27 "MUST be passed ... BEFORE this workflow", §11a
  "A batch SHOULD be shipped") cannot pair MoSCoW values around "before" -> t13 green over
  the full doc proves no false positive in practice.

E4 - rule and version in the rebuilt payload (scratch build of the current source):
  payload cuo/ship-tasks.md line 3   -> workflow_version: 2.6.4
  payload cuo/ship-tasks.md line 312 -> the p0-p3 rule with the FM-105 parenthetical
  payload plugin/skills/ship-tasks/cuo/ship-tasks.md line 3 -> workflow_version: 2.6.4

E5 - sibling-suite guardrail re-run AFTER this task's changes, verbatim tail of
`bash tools/install/tests/test_full_sdp_payload.sh`:

      ok   t08
      ok   t09
    ----
    pass=9 fail=0

Pin disclosure pointer: t12's exact pin moved 2.6.3 -> 2.6.4 per spec §1.2, and t09's
IDENTICAL undeclared pin moved with it (leaving it ships a red suite and violates AC 3) -
full disclosure in code-review.md §DISCLOSURE.

Environment note: run inside a sandboxed agent (45 s per-command cap, synced mount); each
suite fit one capped bash call; no commits by this sub-agent - branch mutations belong to
the batch parent.
