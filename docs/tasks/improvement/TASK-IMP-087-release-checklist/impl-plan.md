---
artefact: implementation-plan@1
task_id: TASK-IMP-087
created: 2026-07-16
estimate_pts: 1
verdict: pass (implementation-plan-audit: every matrix row addressed by a slice, context-map patterns respected, estimate sane vs spec effort_hours 2)
---
# Implementation plan - TASK-IMP-087

Slices (each maps to §1 clauses and edge-case-matrix rows):

1. Verify the repo facts the evidence cells will assert, before writing a word (§1.6 honesty;
   rows 7, 8) - `dist/cyberos/package.json` name+bin, `dist/cyberos/.claude-plugin/marketplace.json`
   stamp, `.github/workflows/release.yml` triggers (`v*` tags + workflow_dispatch) and job list,
   `CHANGELOG.md` head (`[1.0.0] - 2026-07-14`, predates batches 1-2), `dist/cyberos/cyberos.plugin`
   presence, the six evidence commits via read-only `git cat-file`/`git log`
   (`feff8cef a882e705 81ac11a3` + `27292774..ca9ae490` = 6 commits + `e9cfb97a`), the batch-2
   PLAN-gate manifest (Q2/Q3/Q4 = IMP-06/07/11), and TASK-IMP-085 as the live IMP-04 vehicle.
2. Author `docs/release/RELEASE-CHECKLIST.md` header: purpose, row contract (5 cells, closed
   state set, waived-requires-reason, owner semantics operator/agent), no-credentials rule,
   operator-held-gate invariant, and the §1.6 cross-links (handoff sibling path, batch-1/2
   commits, decision record) (§1.1, §1.6; rows 1, 2, 3, 7).
3. Groups (a)-(e) as five 5-column tables with stable row ids A1-E4 and the IMP-15.1-15.7 tags:
   (a) A1 rollup + A2-A4 `checked` with commit+suite evidence + A5 IMP-04 `open` pointing at
   TASK-IMP-085 + A6 build/sync/suite trio verbatim; (b) B1 `npm pack --dry-run` + B2 pack+npx
   smoke verbatim + B3 plugin-zip live-session line (human evidence description) + B4 release.yml
   dispatch dry-run; (d) D1 CHANGELOG section + D2 GUIDE pass + D3 clone-and-coverage verbatim;
   (e) E1/E2/E4 the three decision lines `open` with the PLAN-gate pointer + E3 IMP-08
   implementation line (§1.2, §1.3, §1.5; rows 2, 3, 4, 6).
4. Group (c): C1 re-verify-before-tag line with research date 2026-07-16 + the 9-row channel
   matrix (7 shipped surfaces + `.agents/skills/` and `.devin/rules/`+`.windsurf/rules/`
   candidates, `.windsurfrules` kept) + the MCP-registration note (§1.4; rows 5, 8).
5. Record the AC greps into `gate-log-draft.md` (G1-G9, verbatim commands + actual outputs) as
   the seed for audit.md §gate-log, then the workflow artefacts referencing them (ACs 1-4;
   every matrix row's Covered-by).

Pattern conformance (context-map): governance-as-tracked-markdown under docs/; evidence-cell
idiom = commit ids + suite scores; closed-enum discipline; decision-pointer idiom; HITL
invariant stated in the header; zero payload/build coupling (docs/** is not vendored).
Out of scope honored: no line execution, no IMP-06/07/11 implementation, no CI automation.

Estimate: 1 pt (~2 h) - matches spec effort_hours: 2. Actual landed surface: 1 new production
file (~110 lines, 18 checklist rows + 9-row matrix), 0 modified files, gate log + 5 artefacts
in the task folder.
