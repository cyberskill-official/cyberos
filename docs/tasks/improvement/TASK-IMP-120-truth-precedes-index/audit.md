---
audited_file: docs/tasks/improvement/TASK-IMP-120-truth-precedes-index/spec.md
audited_file_sha256: af4def614d483fff
audited_body_sha256_prefix: 8aacd63f9e6fc0bb
rubric: audit_rubric@2.0
audited_at: 2026-07-18T22:35:00+07:00
auditor: claude-opus-4.8
verdict: pass
score: 10/10
machine_floor: task-lint 0 errors, 1 info (TRACE-001)
---

# Audit - TASK-IMP-120

Machine floor first per TASK-IMP-084: re-ran `task-lint` on the AMENDED spec - 0 errors, 1 info
(TRACE-001, the `## 1. Clauses` heading shape, as with 117/118/119). VERIFIED by command
(`node tools/install/docs-tools/task-lint.mjs docs/tasks/improvement/TASK-IMP-120-truth-precedes-index/spec.md`;
exit 0).

## Amendment (2026-07-18 re-audit) - operator-approved cone growth

The guard shipped and committed at 38199b27 (clauses 1.1-1.5). Its behaviour change broke one
out-of-cone seam - `tools/install/tests/test_e2e_skeleton.sh::t01_spine_green` (and the mini-spine in
`t04_scratch_isolation`), which deliberately flipped the INDEX first and asserted the frontmatter
still read the OLD status: the exact index-first, truth-lagging shape 1.1 forbids. The prior
ship-manifest's `integration_finding` had already SIZED this and left it for a follow-up per cone
discipline (IMP-119). The operator approved GROWING this task's cone to correct the seam rather than
opening a separate task. This re-audit blesses the amendment:

- `modified_files` gains `tools/install/tests/test_e2e_skeleton.sh` (the cone entry).
- Clause 1.6 + AC7 + edge row 12: the seam MUST drive its flips truth-first and MUST positively
  assert an index-first flip REFUSES (exit 6) without moving the row. Cited tests:
  `t05_index_first_flip_refuses` and `t01_spine_green` (truth-first spine).
- Dependencies corrected - see ISS-008.

The amendment WEAKENS no existing clause or AC. 1.1-1.5 and AC1-AC6 are byte-unchanged; the edit
touched only the frontmatter cone list, the Dependencies paragraph, and appended 1.6 / edge-12 / AC7
(VERIFIED: `git diff` of the spec shows only those four regions).

## Findings

ISS-001 (info, accepted): TRACE-001 heading shape. 1.1-1.6 cite tests; AC1-AC7 cite back.

ISS-002 (accepted, and the reason this is p1): two instances in one day, opposite directions, two
different actors - the agent on TASK-IMP-116 and a swarm sub-agent on TASK-IMP-028. Both were caught
by a manual reconcile someone chose to run. The fix was already named in 116's goal file BEFORE the
second instance happened, which is why this is a task and not a discovery.

ISS-003 (accepted): the Alternatives section rejects having flip WRITE the frontmatter. One tool
owning both the truth and its index means a bug in it corrupts the record with no second opinion. The
tool's value is that it is narrow and refuses; widening it to write specs makes it the thing it
guards against.

ISS-004 (accepted): edge case 5 - `status: done  # comment` must be stripped before comparing. Live
in 501 specs today, so a naive compare would refuse every flip on an FM-001-carrying spec and wedge
the workflow on contact. Caught in the matrix rather than in production.

ISS-005 (accepted): edge case 6 - two `status:` lines refuse. Ambiguous truth is not truth, and
picking the first would be the machine choosing which record is authoritative.

ISS-006 (accepted): AC6 replays 116's exact sequence and requires a refusal at the FIRST flip. A fix
that cannot stop the case that motivated it is decoration.

ISS-007 (NEW, accepted - the amendment): cone grown to `test_e2e_skeleton.sh` with clause 1.6 / AC7 /
edge row 12. The seam encoded "flip is index-only, truth may lag" as CORRECT (its header comment and
t01's inline assertion) - exactly the behaviour 1.1 forbids - so the guard's landing made it 3/3 RED.
Growing the cone to fix the seam to the new contract (truth-first drive + a positive
index-first-refuses assertion) is the operator's approved disposition. 1.6's verbs and the tests that
discharge them are recorded under TRACE-006 below. No existing clause/AC weakened; the seam's other
coverage (spine green, reconcile recommendation, corpus survival, scratch isolation) is preserved,
not deleted.

ISS-008 (NEW, accepted - stale originated claim corrected): the Dependencies section claimed
"`backlog-mutate` already parses frontmatter for the insert path." FALSE. `cmdInsert` reads only
BACKLOG rows and never opens a spec; before this task NO command parsed a spec's frontmatter - the
`resolveSpecPaths` + `frontmatterStatus` reader is what THIS task ADDED, inside `cmdFlip`.
- BEFORE: "None. `backlog-mutate` already parses frontmatter for the insert path and already has
  exit 6 for pre-image refusals. This adds a precondition, not a mechanism."
- AFTER: "None. `backlog-mutate` already has exit 6 for pre-image refusals, and `flip` already
  resolves and rewrites a single BACKLOG row. It did NOT previously parse spec frontmatter anywhere:
  `insert` reads only BACKLOG rows and never opens a spec ... THIS task adds that frontmatter read
  to `flip` (`resolveSpecPaths` + `frontmatterStatus` ...) and gates the existing index write on it,
  reusing exit 6. A precondition on an existing refusal path, plus the small reader it needs - not a
  new writer, and not a new refusal code."
This is the same class of defect as the session's governing finding (authors do not check what they
originate) and the AC5-on-IMP-117 correction the operator called out. VERIFIED against source:
`cmdInsert` and its helpers (parseRow/parseCountsHeader/retallyTotals/retallyHeader/sectionBlocks/
blockOf) contain no spec read; `resolveSpecPaths` and `frontmatterStatus` are referenced only from
`cmdFlip`.

## TRACE-006 (semantic sufficiency - per clause, judgment family)

For each §1 clause: [verb it demands] vs [what its cited test asserts]. 1.1-1.5 carry over from the
prior audit (their clauses are byte-unchanged); 1.6 is added here.

- 1.1 refuse (exit 6, naming both) -> t14 asserts exit 6 + names the frontmatter value and the
  target, and the index row is unmoved. Discharges "refuse" (absent side effect + signalled
  refusal). PASS.
- 1.2 proceed unchanged -> t15 asserts the flip succeeds and the row/header/Totals retally exactly as
  before. PASS.
- 1.3 refuse on unreadable/ambiguous -> t16 asserts refusal on missing / no-status / two-status.
  PASS.
- 1.4 insert unaffected -> t17 asserts insert runs identically with the guard present. PASS.
- 1.5 SKILL states the order (both copies) -> t18 asserts the ordering contract in source + vendored
  payload. PASS.
- 1.6 (two verbs) - "exercise flips truth-first" AND "assert index-first REFUSES (exit 6), row
  unmoved."
  - Verb "refuse": demands the guarded index write did NOT happen AND a refusal was signalled.
    `t05_index_first_flip_refuses` asserts BOTH: the flip exits 6, the refusal names the
    truth-precedes-index contract, AND the BACKLOG row is still `[draft]` (unmoved); then the
    truth-first flip proceeds and moves the row. Discharges "refuse".
  - Verb "exercise truth-first": `t01_spine_green` writes the frontmatter to `<next>` BEFORE each
    flip and asserts the index catches up (row == `<next>` after the flip) through the whole
    lifecycle to `done`. Discharges the truth-first drive.
  - PASS (verb-demand vs test-assertion recorded above). SCOPE NOTE: this is the spec-correctness
    gate (draft -> ready_to_implement), so it blesses the clause and its declared tests; the tests
    themselves are this cone's Phase B seam-fix and are run GREEN in the Phase B commit (the coverage
    concern, a different gate - RUBRIC.md §9). This audit does NOT claim the tests are green; it
    verifies the spec's contract for them.

## Rubric families

- FM: clean (task-lint re-run, 0 errors). SEC: seven required sections present and non-empty. COND:
  three-bullet AI-authorship disclosure. Dependencies now truthful (ISS-008).
- QA: 12 edge rows across all six categories (row 12 added: the index-first seam case, proven
  end-to-end by t05). 1 SECURITY, 2 DEGRADATION. QA-008 clean (Dependencies leads with "None.").
- SAFE: adds a precondition to an existing refusal path; reuses exit 6 and relUnderRoot. The cone
  growth adds a test file to the cone (coverage), no new writer, no widened runtime surface.
- TRACE: 1.1-1.6 -> tests; AC1-AC7 -> clauses. AC6 (the motivating 116 replay) and AC7 (the seam the
  guard broke) are load-bearing.

## Verdict

pass - 10/10. The amendment closes the loop the guard opened: the behaviour change that broke the
seam is now covered BY that seam (truth-first spine + a positive index-first-refuses assertion),
inside this task's cone, and the one false originated claim in Dependencies is corrected to the
truth. The binding is rebound to the amended normative half
(`audited_body_sha256_prefix: 8aacd63f9e6fc0bb`).

Verified vs reconstructed:
- VERIFIED by command: task-lint on the amended spec (0 errors, 1 info); the sha bindings (recomputed
  via task-reconcile's `normativeHalf` over the amended file); `cmdInsert` has no spec read (source
  inspection of backlog-mutate.mjs).
- SPEC-GATE SCOPE (not yet green at this commit): clause 1.6's cited tests `t05_index_first_flip_refuses`
  and the truth-first `t01_spine_green` are blessed here as the clause's contract; they are
  implemented and verified GREEN in this cone's Phase B commit (coverage gate), which follows this
  Phase A spec+audit commit.
