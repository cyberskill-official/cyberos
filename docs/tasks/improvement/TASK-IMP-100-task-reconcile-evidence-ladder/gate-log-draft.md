# TASK-IMP-100 gate-log evidence (implementing -> ready_to_review)

E1 - suite (AC 1-5), full run:
  task-reconcile suite (TASK-IMP-100):
    ok   t01_clean_resume
    ok   t02_route_back
    ok   t03_adopt_candidate
    ok   t04_read_only_and_spec_drift
    ok   t05_payload_vendored
  test_task_reconcile: pass=5 fail=0

E2 - AC 6, SKILL.md prose contract:
  $ grep -c "Machine floor first" modules/skill/task-reconcile/SKILL.md          -> 1
  $ grep -c "NEVER executes a recommendation" modules/skill/task-reconcile/SKILL.md -> 1
  (plus the fork table and the "does NOT run when a valid ship-manifest exists" rule)

E3 - payload: build.sh -> "skills=53"; check-chain-coverage.sh -> "chain OK: 25 referenced,
  53 vendored, 6 allowlisted"; check-version-sync.sh -> "sync OK 1.0.0 across 7 artifacts".

E4 - DOGFOOD FINDING (the reason this task exists, proving itself):
  Run against the live corpus, the first draft returned route_back for TASK-IMP-092 - a task
  shipped correctly through both human gates. Cause: audits record a whole-FILE sha, and
  ship-tasks rewrites status/shipped IN THAT FILE at every phase, so any re-hash mismatches.
  Correction 1: verify the binding at the audit COMMIT, judge drift on the normative half
  (body + frontmatter minus status/shipped/routed_back_count/memory_chain_hash).
  That surfaced the sharper fact: the recorded prefix matches NO committed version -
    audit commit 53ef658f: spec hashed 4232bace8dca346c
    audit records:                     98efe9f21fd3a5c1
  because authoring computes the sha, then the same run flips status before committing. The
  audited bytes never existed in git. Correction 2: report the binding gap as a note, keep the
  substantive normative-half check as the verdict. Post-fix, TASK-IMP-092 reads:
    rungs {r1 pass, r2 pass, r3 absent, r4 pass, r5 skipped} -> resume_at_phase(confirm-done)
  Filed as follow-up IMP-19: audits should bind the normative half (or hash post-flip), so the
  binding is verifiable rather than merely well-intentioned.

E5 - read-only proof (AC 4): fixture tree sha256 fingerprint identical before/after both a
  --run-tests run and a --json run; the tool writes only under --out.
