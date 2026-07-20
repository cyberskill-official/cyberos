# TASK-IMP-092 gate-log evidence (implementing -> ready_to_review)

E1 - workflow helpers suite (AC 1-4), full run: test_workflow_helpers.sh ok (t01-t12)
    ok   t10_retally_corrects_lying_header
    ok   t11_footprint_holds_with_retally
    ok   t12_doctrine_view_rules_vendored

E2 - doctrine in source: ship-tasks.md:257  "Shared files are owned by ONE writer through ONE filesystem view per run
- cone-independence includes view-independence (v2.6.3, TASK-IMP-092)" ship-tasks.md:211  "MUST be measured on the committed object (`git show <commit>:<path>`), never a working view (v2.6.3, TASK-IMP-092)" workflow_version: 2.6.3

E3 - doctrine in the rebuilt payload (both vendored copies): dist/cyberos/cuo/ship-tasks.md                       -> 2.6.3, both passages present dist/cyberos/plugin/skills/ship-tasks/cuo/ship-tasks.md -> 2.6.3, both passages present

E4 - retally dogfooded on this batch's own rows (the tool under review maintained them): after 5 inserts:  ## improvement  (67 draft, 5 ready_to_implement, 20 done) after 5 flips:    ## improvement  (67 draft, 5 implementing, 20 done) counts tracked the rows through every mutation; no inherited baseline.

E5 - payload rebuild + sync: build.sh ok (skills=52, parity OK 25 author dirs); check-version-sync.sh -> "sync OK 1.0.0 across 7 artifacts"

## PR-review addendum (2026-07-17, CI payload gate)

`scripts/check_doc_anchors.sh` (TASK-SKILL-119) flagged DEAD: code-review.md:4 wrote the ship-tasks.md workflow path with a three-dot ellipsis in place of its middle directories. Fixed to the full path there and in coverage-gate.md's table (this addendum deliberately does not quote the bad token - the checker reads prose paths as references, as it should). Checker rerun: `anchors OK: 444 references resolved`, exit 0 (remaining WARNs are tolerated historical refs). Lesson folded into practice: artefact docs write real paths - the anchor checker treats prose paths as references, which is the point.
