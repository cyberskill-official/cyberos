---
task_id: TASK-SKILL-202
audited: 2026-07-23
verdict: PASS (after revision)
score_pre_revision: 6/10
score_post_revision: 10/10
issues_resolved: 7
template: task@1
audit_rubric_version: audit_rubric@2.0
machine_floor: task-lint clean - `node tools/install/docs-tools/task-lint.mjs docs/tasks/skill/TASK-SKILL-202-skill-quality-floor/spec.md` exits 0 with zero findings, run against the audited revision
---

## §1 — Verdict summary

Seven §1 clauses, seven ACs, six edge cases including a security-class row. The audit pressure fell on measurement honesty (plan's 21/24 vs measured 24/25 - corrected to measured values with the discrepancy recorded), degradation loudness for the delisted workflow surface, and anti-cargo-cult mechanics in the backport (byte-copy detection). Two count-drift traps were converted from counts to properties so the checkers stay true as the corpus moves.

## §2 — Findings (all resolved)

### ISS-001 — plan figures (21 skills, 24 pairs) did not match the measured corpus
Fresh measurement against the built payload found 24 skills missing both discipline halves and 25 author/audit pairs. Writing the plan's numbers into normative clauses would ship assertions that fail against reality on day one. Resolved: clauses and metrics use measured values; `source_decisions` records the plan-vs-measured discrepancy; AC 4 and t04 key on "SCOPE equals the payload's measured pair set" (a property), not a literal 25.

### ISS-002 — delisting silently broke certify-nfrs.md in the first draft
Removing the four skills from the payload without touching the workflow that routes to them converts silent improvisation into a missing-skill error at best - or an agent improvising anyway at worst. Resolved: clause 1.2 requires the loud not-yet-shipped notice at the routing step; AC 2 asserts it; the delist-not-implement default is a recorded decision for the review gate.

### ISS-003 — chain-allowlist entries would have become rot
`chain-allowlist.txt`'s own header says an entry no payload dir matches triggers a rot warning; delisting without cleaning the four UNPAIRED lines would trade one checker's noise for another's. Resolved: clause 1.1 couples the two removals; AC 1 asserts the chain-coverage checker stays green.

### ISS-004 — the backport could be satisfied by byte-copying one reference doc 20 times
The discipline's value is per-skill input-surface specificity; a generic copy passes a naive existence check while adding no protection. Resolved: clause 1.3 requires each doc to name the skill's own input surface; AC 3's test compares docs pairwise for full-content identity (identical files fail, shared sections pass).

### ISS-005 — SCOPE expansion without the file classes would turn the parity checker red repo-wide
Expanding SCOPE to 25 pairs while 14 lack the class files makes the checker fail on main - a self-inflicted red that would get the expansion reverted. Resolved: clause 1.4 couples SCOPE expansion to authoring the missing classes "at parity with the existing deepened pairs, not as empty placeholder files"; AC 4 asserts pass on the payload AND the single-file-deletion negative.

### ISS-006 — the stub-floor threshold needed a rationale and an exemption path
An arbitrary "60 lines" invites both gaming (padding) and legitimate-small-skill friction. Resolved: the floor pairs size with required-section structure (Alternatives rejects line-count-only), and the edge case defines the explicit, reviewed exemption path starting empty - the escape is visible, not improvised.

### ISS-007 — placeholder detection overlap with existing tooling was unstated
TASK-SKILL-115's sweep-placeholders tooling already owns placeholder-syntax detection; a floor checker that duplicated it would create two authorities. Resolved: clause 1.5 scopes the floor to size + structure and names the existing sweep as the placeholder authority.

## §3 — TRACE-006 semantic sufficiency (per clause)

| Clause | Verb demand | Cited test asserts | Verdict |
|---|---|---|---|
| 1.1 MUST NOT vendor the four + MUST drop allowlist lines | payload absence + allowlist absence + no rot warning | AC 1: asserts all three | sufficient |
| 1.2 MUST state not-yet-shipped at routing step | notice substring at the right step | AC 2: asserts presence | sufficient |
| 1.3 MUST carry both discipline halves, per-skill | frontmatter keys + non-empty per-skill doc + non-byte-copy | AC 3: asserts all three per the 20 enumerated skills | sufficient after revision (ISS-004) |
| 1.4 MUST enumerate every pair + classes present | exit 0 on payload + SCOPE completeness + deletion negative | AC 4: asserts all three | sufficient after revision (ISS-005) |
| 1.5 MUST fail under-floor skills and fail the build | pass real payload, fail two fixture classes, fail injected build | AC 5: asserts all four | sufficient |
| 1.6 MUST cover the six behaviors, glob-registered | suite green under run_all discovery | AC 6: asserts registration; t01-t05 are the behaviors | sufficient |
| 1.7 MUST record delisting/backport/expansion | three substrings in top entry | AC 7: asserts all three | sufficient |

## §4 — Resolution

Seven findings - one measurement-truth, six material - all resolved in the audited revision. **Score = 10/10.**

Status transition `draft -> ready_to_implement` is authorised by this verdict per `STATUS-REFERENCE.md` §1.1. The two human-acceptance gates in `/ship-tasks` are unchanged - this audit clears the spec-correctness gate only.

---

*End of TASK-SKILL-202 audit.*
