# TASK-IMP-087 - code review packet

Files under review: new `docs/release/RELEASE-CHECKLIST.md` (the deliverable, ~110 lines,
18 checklist rows across groups (a)-(e) + a 9-row channel matrix; new directory `docs/release/`
introduced by this task exactly as the spec's `new_files` fixes it). Supporting records in the
task folder: `gate-log-draft.md` (G1-G9, the seed for audit.md §gate-log), context-map,
edge-case-matrix, impl-plan, obs-injection. `modified_files` is empty and stayed truthful -
zero production files touched outside docs/release/**. Cone-disjoint from batch siblings
TASK-IMP-085 (tools/install/docs-tools/**) and TASK-IMP-086 (docs/tasks/BACKLOG.md); their
working-tree dirt is covered by their own packets.

## §1 clause -> proof

| Clause | Requirement | Proof |
|---|---|---|
| 1.1 | file at `docs/release/RELEASE-CHECKLIST.md`; every line carries line text, owner, state from {open, checked, waived}, evidence cell (reason mandatory when waived) | file exists at the spec path; G1 - all 18 rows matching `^\| [A-E][0-9]+ \|` split into exactly 5 cells; G2 - state cells dedup to `3 checked / 15 open`, nothing outside the closed set; G3 - 0 waived rows today (vacuous) with the waived-requires-reason rule normative in the Row contract and G3 as the standing detector; owner column restricted to `operator`/`agent` per the brief's semantics (visible in G1's row dump) |
| 1.2 | the seven IMP-15 lines present: S1 landed-or-waived; npm pack + npx smoke; plugin-zip live-session test; channel-matrix re-verify; release.yml dry-run; CHANGELOG 1.0.0 section; fresh-clone consumer test | G4 - tags IMP-15.1 (A1 rollup over A2-A5+E1), IMP-15.2a (B1) + IMP-15.2b (B2), IMP-15.3 (B3), IMP-15.4 (C1), IMP-15.5 (B4), IMP-15.6 (D1), IMP-15.7 (D3) all present; current truth encoded: IMP-01/02/03 rows A2-A4 `checked` citing `27292774..ca9ae490` + their member suites (6/6, 13/13, 8/8), IMP-04 row A5 `open` pointing at TASK-IMP-085 on `batch/2-workflow-helpers` |
| 1.3 | one line per 2026-07-16 operator decision (IMP-06 config.yaml task@1 scaffold, IMP-07 section-4 drop, IMP-11 untracked manifests), state open, pointer to the decision record | G5 - E1 (IMP-06), E2 (IMP-07), E4 (IMP-11), each `open`, each citing the batch-2 PLAN gate: `docs/tasks/.workflow/task-author.improvement-batch-2.manifest.json` Q2/Q3/Q4 + spec `source_decisions`; E3 (IMP-08 channel implementation) added per the ship brief as a fourth scheduled line - additive over the spec's minimum three, disclosed here |
| 1.4 | channel-freshness matrix (surface file per tool incl. `.devin/rules/` and `.agents/skills/` candidates) with re-verify-before-tag instruction and research date | G6 - 9 tool rows (AGENTS.md spine, CLAUDE.md, GEMINI.md, .cursorrules + .cursor/rules/cyberos.mdc, .grok/GROK.md, .github/copilot-instructions.md, .agents/rules/cyberos.md, candidate `.agents/skills/`, candidate `.devin/rules/` + `.windsurf/rules/` with legacy `.windsurfrules` kept); C1 carries the literal re-verify-before-tag instruction, the 2026-07-16 research date, and the handoff §5 source list |
| 1.5 | machine lines name their command verbatim (npm pack, npx invocation, clone-and-coverage, build/sync/suite trio); human lines say what evidence satisfies them | G7 - six verbatim backticked commands extracted: the A6 trio, B1 `npm pack --dry-run`, B2 `npm pack` + the `npx --yes <path-to-tgz> help && ... install` smoke, B4 `gh workflow run release.yml` (operator-owned, command still named), D3 `git clone sachviet && npm ci && npm run coverage`; human lines A1/B3/C1/D1/D2/E1-E4 each state their satisfying evidence in prose (session record with date, research pass, section content naming the batch tasks, decision pointer or landing commit) |
| 1.6 | English; no secrets; cross-links to IMPROVEMENT_HANDOFF.md and the batch-1/2 evidence commits | document is English throughout; G8 - credential-pattern scan zero hits (and the text avoids credential vocabulary except the single prohibition); G9 - cross-links resolve: `../IMPROVEMENT_HANDOFF.md` on the stated sibling layout (marked deliberately untracked, canonical path named), commits `feff8cef`/`a882e705`/`81ac11a3` (§1 applied table) and `27292774..ca9ae490` (6-commit governed batch-1 run) and `e9cfb97a` (batch-2) all resolve via read-only git; referenced repo paths (release.yml, cyberos.plugin, GUIDE.md, CHANGELOG.md, the three A-row suites, TASK-IMP-085 spec, the PLAN-gate manifest) all exist |

## Acceptance criteria

AC 1 (§1 #1.1) recorded structure greps G1/G2/G3 - pass. AC 2 (§1 #1.2-1.4) recorded presence
greps G4/G5/G6 - pass. AC 3 (§1 #1.5) recorded command-cell extraction G7 - pass. AC 4
(§1 #1.6) recorded credential scan G8 + link check G9 - pass. All four are ops-verified by
design (spec: "a test suite for one markdown file is out of scope by design, see Non-Goals");
gate-log-draft.md holds the verbatim commands and outputs for audit.md §gate-log.

## Diff size

One new production file: `docs/release/RELEASE-CHECKLIST.md` (~110 lines; 18 checklist rows -
a:6 b:4 c:1+matrix d:3 e:4; 3 checked / 15 open / 0 waived). Zero modified files, zero
deletions, zero dependencies. `dist/` untouched (docs/** is not vendored - no payload
obligation arises; the A6 trio line belongs to the release run, not to this docs-only change).

## Deviations (disclosure)

1. E3 (IMP-08) is a fourth group-(e) line beyond §1.3's three decisions - required by the ship
   brief, consistent with spec §1.2's channel-matrix clause (the candidate rows need their
   implementation pointer), additive and disclosed.
2. IMP-15.2 is split into tagged lines 2a/2b so the pack dry-run and the npx smoke each carry
   their own verbatim command cell (§1.5); the presence grep counts both toward the seven.
3. The handoff cross-link points outside the repo (sibling checkout) because that is where the
   file lives, untracked by upstream decision; the document and G9 state the layout dependency
   plainly instead of pretending an in-repo path.

## Verdict

| Check | State |
|---|---|
| §1 clauses 1.1-1.6 | each proven above by a recorded grep or verified repo fact |
| Guardrail metric (recorded grep set proves seven IMP-15 lines + three decision lines + matrix) | pass (G4/G5/G6 on the record) |
| Primary metric (every line owner + closed state + evidence; zero undefined states) | pass (G1/G2/G3: 18/18 rows, 3 checked / 15 open / 0 waived) |
| Invariants (§5: gate operator-held, no credentials, HITL - agent never sets done) | intact (header states the gate; G8 clean; states written here are open/checked only, no line self-accepted) |

HALT: review acceptance (reviewing -> ready_to_test) is a human gate.
