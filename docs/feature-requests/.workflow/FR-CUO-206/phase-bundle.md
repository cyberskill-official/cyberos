# FR-CUO-206 phase bundle (condensed per-FR ship artefacts)

## repo-context-map (step 1)
Patterns followed: contract docs live at modules/skill/contracts/feature-request/ (STATUS-REFERENCE,
MANIFEST_SCHEMA precedent); python helpers as pure functions in modules/cuo/cuo/ (package with tests
in modules/cuo/tests/, plain unittest, no deps); two-phase atomic writes per AGENTS.md §4.1; init.sh
scaffolding block at the `mkdir -p "$CY"` cluster. Blast radius: 4 new files, 4 modified; modules cuo
+ skill + tools/cyberos-init; no cross-module edges beyond documented contract references.
Placement check: cuo is correct (workflow run-state is CTO-workflow machinery, not skill contracts
alone - the contract doc rightly lands in skill/contracts like its siblings).

## edge-case matrix (step 5) - rows -> covering test
NULL/EMPTY: manifest missing on resume -> fresh run (contract Lifecycle; trivially exercised by every
  first run); steps[] empty -> resume_plan starts at 1 (test_resume via done=set()).
BOUNDS: step index 0/32 rejected (test_schema... bad2); current_step 1..31 enum (validate).
MALFORMED: missing root field -> validate error (test_schema); truncated JSON on disk -> resume treats
  as no manifest per contract (cache semantics, §10 #4).
RACE: crash between artefact + manifest write -> step re-runs, idempotent (FR §10 #1; write_atomic
  leaves no .tmp - test_atomic asserts empty tmp listing).
SECURITY: manifest hand-edited to skip a gate -> gates re-ask (test_hitl_reask_on_resume + doc clause
  "NOT be treated as approval"); fr_sha256 mismatch forces full redo (test_workflow_version_mismatch).
DEGRADATION: workflow doc version bumped mid-flight -> needs_human (test_workflow_version_mismatch);
  artefact corrupted -> stale-from-earliest (test_resume_plan_intact_and_stale).

## implementation summary (steps 6-14)
New: modules/skill/contracts/feature-request/SHIP-MANIFEST.md (ship-manifest@1 field table, lifecycle,
queue selection); modules/cuo/cuo/ship_manifest.py (validate/write_atomic/resume_plan/select_next/
finalize, ~130 lines, stdlib only); modules/cuo/tests/test_ship_manifest.py (8 tests = AC 1..8);
docs/feature-requests/.workflow/.gitignore (*.ship.json).
Modified: ship-feature-requests.md (workflow_version 2.3.1 -> 2.4.0; Resume semantics section: write
points, 4-rule resume, gate re-ask, terminal handling, queue algorithm; cross-ref entry);
EXECUTION-DISCIPLINE.md (Run-state manifests section); plugin wrapper SKILL.md (resume-on-restart
paragraph); init.sh (scaffolds .workflow/.gitignore).
Field finding folded back: FR-CUO-209's t08 was a point-in-time scope guard as permanent invariant -
amended post-ship (FR §1 #8 + AC 8 + audit §11), t08 repurposed to workflows_vendored_intact.

## observability (step 15)
Operator-facing state transitions echo as greppable lines (contract-pinned): resume line
`resume <FR-ID>: ...` and queue line `queue: picked <id> ...`. Error branches return structured
reasons (needs_human reason string carries both versions). No PII surface.

## code review vs §1 clauses (steps 16-18)
#1 contract fields all present incl. fr_sha256 (test AC1 greps 14 tokens) PASS; #2 write points +
tmp/rename in doc (AC2) PASS; #3 resume/staleness/version rules (AC3+AC4) PASS; #4 deterministic
queue + reasoning echo (AC5) PASS; #5 gitignore + init scaffold (AC6, git check-ignore live) PASS;
#6 done-deletes/route-back-keeps (AC7) PASS; #7 Resume section + ED pointer + wrapper mention
(greps in AC2/AC8 + manual) PASS; #8 gate re-ask normative (AC8) PASS. Secret scan: none. Injection:
no shell interpolation from manifest content. Backwards compat: no existing manifests exist; version
bump to 2.4.0 makes any stray one needs_human by design.

## coverage gate (testing phase, steps 21-29)
Suite: modules/cuo/tests/test_ship_manifest.py - 8/8 pass (one per AC). Statement coverage on the
touched python file (ast-statement basis, import traced): 100.0% (75/75), gate >= 90% PASS.
tests_failed=0; files_below_90pct=[]; ecm_rows_uncovered=[] (each ECM row names its covering test in
the matrix above). Full regression: all 5 tools/cyberos-init suites PASS; pre-commit payload rebuild +
sync OK 1.9.1 across 6 artifacts; git check-ignore confirms *.ship.json untracked.
Strengthened during gate: validate() error branches (5) + write_atomic failure-cleanup path added to
the AC1/AC2 tests after the first coverage read showed them unhit (77.3% -> 100%).

## HITL record
Gate 1 (reviewing -> ready_to_test): APPROVED by Stephen Cheng (CTO), 2026-07-12, in-chat -
"Approve + pre-authorize done".
Gate 2 (testing -> done): pre-authorized at gate 1; testing phase stayed green (8/8, 100% coverage,
5/5 suites) - done recorded per that standing verdict.
