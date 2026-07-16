---
artefact: repo-context-map@1
task_id: TASK-IMP-087
created: 2026-07-16
verdict: pass (repo-context-map-audit: patterns pinned to file:line or section, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-087

## Baseline patterns the new document must follow

- Governance-as-tracked-markdown: the repo's release/task governance lives in tables under
  docs/ (docs/tasks/BACKLOG.md index rows, docs/tasks/improvement/README.md) - pinned_in: spec
  Alternatives ("the repo's governance runs on tracked markdown under docs/; the release gate
  belongs in the same corpus"). The checklist is a new member of that corpus, not a GUIDE
  extension (GUIDE ships to consumers inside the payload; the release gate is platform-operator
  audience - spec Alternatives, first bullet).
- Placement: `docs/release/` is a new directory fixed by spec frontmatter (`service:
  docs/release`, `new_files: [docs/release/RELEASE-CHECKLIST.md]`). Nothing else lives there;
  nothing vendors it - build.sh's docs-tools/GUIDE copies (build.sh:165-178, :195) never touch
  docs/**, so the payload is untouched by this task (zero build coupling, per spec Dependencies).
- Evidence-cell idiom: commit ids + suite scores as proof, mirroring the handoff §1 applied
  table (`27292774..ca9ae490`, "member suites 6/6, 13/13, 8/8") - pinned_in:
  ../IMPROVEMENT_HANDOFF.md §1 row 4 and §5 evidence index.
- Owner/state discipline: closed enum sets with mandatory reasons, mirroring task@1 frontmatter
  enums (FM-104 status set) and the BACKLOG state grammar - the checklist's {open, checked,
  waived} with waived-requires-reason follows the same "closed set + structure violation on
  breach" doctrine (spec §1.1, §3 edge case 1).
- Decision-record pointers: operator decisions cite their gate - pinned_in: spec
  `source_decisions` ("2026-07-16 Stephen: PLAN batch 2 approved...; IMP-06/07/11 decisions
  recorded at the same gate") and `docs/tasks/.workflow/task-author.improvement-batch-2.manifest.json`
  (plan.open_questions Q2/Q3/Q4 name exactly those three decisions).
- HITL invariant: the document must not authorize tagging/publishing/pushing (spec §5); its
  header states the gate is operator-held - matching ship-tasks doctrine ("never push, merge,
  deploy, or self-accept", handoff §0.2).

## Schemas / interfaces in scope

- Row contract (normative, spec §1.1): `| # | Line | Owner | State | Evidence / command |`;
  row ids `[A-E][0-9]+` make checklist rows machine-distinguishable from the §1.4 channel
  matrix (a 3-column reference table by design). State from {open, checked, waived}; waived
  requires a reason in Evidence.
- IMP-15 tag grammar: the seven seed lines carry `IMP-15.1`..`IMP-15.7` verbatim (15.2 split
  into 2a/2b so each command sits in its own cell) - the AC 2 presence greps key on the tags.
- Verified repo facts consumed by evidence cells: `dist/cyberos/package.json` (name
  `@cyberskill/cyberos`, bin `cyberos -> cli/bin/cli.mjs`), `dist/cyberos/.claude-plugin/marketplace.json`
  (1.0.0), `dist/cyberos/cyberos.plugin` (the session-test asset), `.github/workflows/release.yml`
  (on: push tags `v*` + workflow_dispatch; jobs payload/channels/npm/desktop/.../docs),
  `CHANGELOG.md` head (`## [1.0.0] - 2026-07-14`, predating batches 1-2 - hence D1 stays open).

## Files outside the immediate domain (docs/release/**)

0 production files. The task's other writes (gate-log-draft.md and the five workflow artefacts,
this file among them) live in the task's own folder per the ship-tasks artefact convention -
session/governance artefacts, not production surface; the spec's `modified_files` is empty and
stays truthful.

files_outside_immediate_domain: 0 (<= 3 -> no ADR trigger).

## Blast radius

file_count: 1 new production file (docs/release/RELEASE-CHECKLIST.md), 0 modified |
module_count: 1 (docs/release) | cross_module_edges: none executable - the document points at
tools/install (build/sync/suite trio, tests), dist/cyberos (pack/npx/plugin assets),
.github/workflows/release.yml, CHANGELOG.md and the sibling handoff, but reads/mutates none of
them; siblings TASK-IMP-085 (tools/install/docs-tools/**) and TASK-IMP-086 (docs/tasks/BACKLOG.md)
are cone-disjoint by batch design.
Behavioral radius: zero on every executable path - no code, no build, no payload change.
The only behavioral commitment is procedural: the operator works the lines before tagging.
module_placement_warning: null (spec fixes the path; the directory is introduced by this task).
