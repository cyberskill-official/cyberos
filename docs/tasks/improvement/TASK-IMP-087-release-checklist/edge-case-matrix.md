---
artefact: edge-case-matrix@1
task_id: TASK-IMP-087
total_rows: 8
created: 2026-07-16
verdict: pass (edge-case-matrix-audit: every category >=1 row; Covered-by names the RECORDED greps G1-G9 in gate-log-draft.md - the spec's ACs are ops-verified by design (single operator markdown document, suite out of scope per Non-Goals), so recorded re-runnable greps stand where test functions would; SECURITY row points at scan+prohibition, DEGRADATION rows carry detection+recovery)
---
# Edge-case matrix - TASK-IMP-087

Covered-by references G1-G9 = the recorded greps in `docs/tasks/improvement/TASK-IMP-087-release-checklist/gate-log-draft.md` (each listed with its verbatim command so any reviewer can re-run it against the living document).

| # | category | trigger | expected behavior | covered by |
|---|---|---|---|---|
| 1 | null/empty | waived-without-reason: a future edit flips a row to `waived` leaving the Evidence cell empty | structure violation per the document's Row contract ("A `waived` state REQUIRES its reason in the Evidence cell") - AC 1 catches it | G3 (waived-row evidence-length grep; today 0 waived rows - vacuously green, disclosed as such, and the grep is the re-runnable detector) |
| 2 | bounds | a row edit drops or adds a cell (4 or 6 cells instead of 5) | every checklist row splits into exactly 5 cells; any other count is an AC 1 structure violation | G1 (awk field count = 7 on all 18 rows matching `^\| [A-E][0-9]+ \|`) |
| 3 | malformed | a state outside the closed set lands (`pending`, `done`, `n/a`, blank) | State is drawn only from {open, checked, waived}; anything else is undefined state = AC 1 violation (spec Success Metrics: "zero lines in an undefined state") | G2 (state-cell extraction + dedup: output exactly `3 checked / 15 open`, nothing else) |
| 4 | order / drift | drift-after-batch-3: batch 3 lands IMP-04/06/07/11 and rows A5/E1-E4 flip to `checked` with commit ids | the document is living by design - ACs verify shape, not final states (spec §3 edge case 2); flips keep the row contract and cite commits per the footer's flip rule | G1+G2+G3 re-runnable unchanged on every future edit (they assert shape and closed set, never specific states); footer names the flip rule |
| 5 | DEGRADATION | conventions-moving: agent-surface conventions shift again between research date and tag day | detection: C1 carries the research date 2026-07-16 and the explicit re-verify-before-tag instruction rather than pretending permanence (spec §3 edge case 3); recovery: fresh research pass recorded in C1's Evidence cell with its date + changed matrix rows edited | G6 (re-verify-line presence = 1, research-date presence, candidate-row markers `.devin/rules/` `.agents/skills/` `.windsurfrules` each >= 1) |
| 6 | SECURITY | secret-temptation: B3 (live-session evidence) and B4 (dispatch-run links) invite pasting credentials into Evidence cells when the lines get worked | the header prohibits credentials in the file; AC 4's recorded scan is the standing detector any reviewer re-runs before accepting an edit | G8 (credential-pattern grep: ghp_/github_pat_/xox?-/AKIA/-----BEGIN/assignment-shaped values/Bearer -> zero hits, exit 1) |
| 7 | DEGRADATION | cross-link off-layout: `../IMPROVEMENT_HANDOFF.md` is a sibling of the checkout, deliberately untracked - a clone elsewhere breaks the link | detection: the doc names the canonical location (`~/Projects/CyberSkill/IMPROVEMENT_HANDOFF.md`) and marks it "deliberately not tracked in-repo"; G9 records the layout dependency explicitly rather than hiding it; recovery: place the handoff beside the checkout or read the seven lines quoted into this checklist (they are self-contained) | G9 (existence check OK on the stated layout + the six evidence commits resolving via `git cat-file -t`) |
| 8 | order | matrix-vs-checklist confusion: the 3-column channel matrix could be misread as malformed checklist rows by a naive whole-file structure check | checklist rows are keyed by the row-id pattern `^\| [A-E][0-9]+ \|`; the matrix is §1.4 reference content with its own 3-column shape by design - the gate log states the distinction so the structure check cannot false-positive | G1/G2 (pattern-scoped) + gate-log preamble sentence naming the split |

Documented-by-design: executing the checklist lines is not an edge case of this task - it is the release run itself, operator-owned (spec Non-Goals); the 15 `open` states are the correct current truth, not gaps. Security class beyond row 6: none - the document executes nothing (spec §3 last bullet).
