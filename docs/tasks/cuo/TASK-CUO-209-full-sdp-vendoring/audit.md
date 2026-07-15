---
task_id: TASK-CUO-209
audited: 2026-07-12
verdict: PASS (after revision)
score_pre_revision: 7/10
score_post_expansion: 9/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
---

# TASK-CUO-209 audit

## §1 - Verdict summary

Audited for expansion safety: 20 -> ~52 vendored skills with the two workflows explicitly frozen. The decisive controls are the reviewable set-as-data block, the computed counts, the size budget, and the sibling checkers running over the expanded set. The NFR-four pairing question was resolved upstream in TASK-SKILL-116's allowlist semantics. Traceability closes over t01-t08 in tools/install/tests/test_full_sdp_payload.sh.

## §2 - Findings (all resolved)

### ISS-001 the root cause was about to be preserved
Expanding a hardcoded string keeps the exact failure mode that lost debugging-cycle. Resolved: §1 #2 one-name-per-line block with stage comments - the set becomes reviewable data; AC 2.

### ISS-002 the NFR four would trip the pair rule
Four intentionally single skills fail UNPAIRED the day they vendor. Resolved: allowlist-with-reason mechanism (TASK-SKILL-116 §1 #3, cross-cited in §1 #1), AC 5 green over the expanded set.

### ISS-003 hardcoded counts in the manifest
author_audit_skills: 20 was already a lie-in-waiting. Resolved: §1 #3 computed counts driving both the manifest and the profile determination, AC 3 two-build fixture.

### ISS-004 plugin bloat could degrade skill selection
More vendored skills = more trigger surface for agents. Resolved: 2 MB budget with build-time assert (§1 #6), GUIDE lifecycle map setting expectations (§1 #4), and a documented trim fallback (payload keeps all, plugin trims) as the named escape hatch (§10 #1).

### ISS-005 workflow scope creep
Vendoring upstream/downstream pairs invites wiring them into the chain in the same change. Resolved: §1 #8 freeze + AC 8 diff-clean on both workflow docs; wiring is future work by separate task.

### ISS-006 map completeness unverifiable
A lifecycle map with TBD rows would defeat its purpose. Resolved: AC 4 requires exactly 14 rows, each naming pair + invoker, no TBD (t04).

## §3 - Resolution

All six findings addressed as cited. Dependencies on TASK-SKILL-116/117 are declared on all three tasks; TASK-SKILL-118 interplay (per-pair parity scope) is stated without creating a hard dep. **Score = 10/10.**

*End of TASK-CUO-209 audit.*

## §10 - Post-implementation gates (2026-07-12, ship run)

- §10.4 coverage: PASS - t01-t08 green on rerun; all five cyberos-install suites green (42 cases).
  Live: profile=full skills=52 payload=8499200 plugin_zip=1029894 (49% of budget); chain OK.
- awh/caf: N/A (declared); floor = bash -n + suites + live build with both hook checks.
- HITL gate 1: APPROVED by Stephen Cheng 2026-07-12. HITL gate 2: ACCEPTED same date via explicit
  operator pre-authorization; gates stayed green.
- Field finding folded upstream: reduced-profile chain-check exemption recorded on TASK-SKILL-116
  (§1 #5 + audit §11 + t07).

*TASK-CUO-209 shipped 2026-07-12. The payload now covers the full 14-stage SDP.*

## §11 - Post-ship amendment log

- 2026-07-12 (TASK-CUO-206 in flight): t08 `workflows_diff_clean` tripped on TASK-CUO-206's legitimate
  ship-workflow edit - the AC 8 guard was a point-in-time scope clause implemented as a permanent
  invariant (same class as the TASK-SKILL-116 reduced-profile finding). Amended: §1 #8 + AC 8 note the
  temporal scope; t08 repurposed to `workflows_vendored_intact` (both docs present in payload,
  skill_chain + Resume semantics structure intact). Verdict unchanged: PASS, Score = 10/10.
