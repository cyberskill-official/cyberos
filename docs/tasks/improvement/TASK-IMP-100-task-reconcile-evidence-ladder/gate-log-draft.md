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

## PR-review addendum (2026-07-17, Devin Review)

**F-out (defect, fixed).** `--out` resolved against the repo root but never CONFINED to it -
an absolute path or a `../` value would write, and mkdir, anywhere the process could reach.
The sibling `coverage-scope.mjs` has always refused out-of-root paths via `relUnderRoot`; a
tool whose whole contract is "read-only instrument" cannot be the looser of the two. Same
predicate, same refusal (exit 2), same message shape. t04 gained two arms: `--out
../escaped.md` and an absolute `--out /tmp/...` - both refused, neither file created.
Spec §3's security line ("refuses paths escaping the store root") was written and then not
implemented for --out; the review caught the gap between the sentence and the code.

**F-home (info, fixed anyway).** rung2 derived the `.workflow` artefact home by slicing three
dash-segments off the folder name - true for the whole current corpus, wrong the moment a task
id has a different shape, and it would silently miss a real artefact bundle (a false
adopt_candidate). The home is now keyed by the task ID the caller already resolved. No
behavior change for the corpus; verified TASK-IMP-092 still reads resume_at_phase(confirm-done).

**F-chain / F-lease / F-symlink / F-nongit (info, affirmed).** The bot independently verified
the chain-blanking's sorted-key/quote-escape reasoning, the lease reboot+orphan guards (incl.
that the fresh-lease case never trips the horizon check), the relative symlink target math, and
the rev-parse-based non-git probe. Recorded; no change.

Reruns: test_task_reconcile 6/6, test_workflow_helpers 14/14, 25/25 repo-wide, build ok,
sync OK 1.0.0.

## PR-review addendum 2 (2026-07-17, Devin Review)

**F-eisdir (defect, fixed).** rung2 built its bundle text by `readFileSync`-ing every entry
whose NAME matched the bundle pattern. A directory matching that pattern (plausible under a
`.workflow/<id>/` home) threw EISDIR, which escaped rung2, hit main's top-level catch, and took
the process down with exit 1. A measuring instrument must degrade to a rung verdict; dying on a
directory listing is the one thing it may never do. rung2 now stats first: a directory (or any
unreadable entry) contributes its NAME to the bundle text and nothing else. Arm added to t03
(a `artefacts-bundle.d/` directory committed into the home; the run must still emit
reconcile-report@1 at exit 0). Suite 6/6.

**F-precision (info, affirmed).** `Number(process.hrtime.bigint())` loses sub-microsecond
precision past ~104 days of uptime; the error is many orders of magnitude below a 10 s TTL, so
the second-scale comparisons are unaffected. The bot traced acquire/reap/throw ordering and the
release-in-finally (which never fires when acquireLease throws, so a foreign lease is never
clobbered) and found the logic self-consistent. Recorded, no change.

**F-parity (info, affirmed).** The bot verified byte-for-byte equivalence between
`normativeHalf()` in task-reconcile.mjs and the independent Python `body_sha` in the suite -
the cross-implementation agreement that makes TASK-IMP-102's binding load-bearing. That the two
were written independently and agree is the point of the arm; confirmation welcome.
