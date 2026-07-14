---
id: TASK-CUO-209
title: "Vendor the full 14-stage SDP skill set by default - payload and plugin cover SOW through decommissioning, with a lifecycle map in GUIDE.md"
module: cuo
priority: MUST
status: done
class: product
verify: T
phase: Wave C - strengthen the workflows
owner: Stephen Cheng (CTO)
created: 2026-07-12
shipped: 2026-07-12
memory_chain_hash: null
related_tasks: [TASK-SKILL-116, TASK-SKILL-117, TASK-SKILL-118, TASK-IMP-068]
depends_on: [TASK-SKILL-116, TASK-SKILL-117]
blocks: []
source_pages:
  - modules/cuo/docs/appendices.md
  - tools/cyberos-init/build.sh
  - tools/cyberos-init/init.sh
source_decisions:
  - "2026-07-12 operator decision (plan approval): vendor the full SDP set BY DEFAULT - not an opt-in profile. The plugin today covers stages 5-10 of the 14-stage SDP; the upstream (SOW, PRD, SRS, NFR, SDD, threat-model) and downstream (deploy, release, runbook, retro/postmortem/decommission) pairs exist in modules/skill but never ship."
  - "Reduced profile semantics unchanged: doc-driven floor remains for payloads built without skill bodies."
language: bash + markdown
service: tools/cyberos-init/
new_files:
  - tools/cyberos-init/tests/test_full_sdp_payload.sh
modified_files:
  - tools/cyberos-init/build.sh
  - tools/cyberos-init/README.md
---

# TASK-CUO-209: Full-SDP vendoring by default

## §1 - Description

A repo that installs CyberOS should be able to run the whole software development process from the same payload - author an SOW or PRD upstream of /create-tasks, and produce deployment checklists, release notes, runbooks, and retrospectives downstream of /ship-tasks - without anyone hand-copying skills out of the monorepo.

Normative clauses:

1. The vendored skill set in `build.sh` MUST expand from the current 20 to the full SDP catalog: the existing 10 pairs, plus `debugging-cycle` (TASK-SKILL-116) and `architectural-spike` (TASK-SKILL-117) pairs, plus these pairs from `modules/skill/`: `statement-of-work`, `product-requirements-document`, `software-requirements-specification`, `software-design-document`, `threat-model`, `test-strategy`, `deployment-checklist`, `release-notes`, `runbook`, `retrospective`, `postmortem`, `decommissioning` - and the four NFR skills (`nfr-certification-author`, `nfr-evaluator`, `nfr-test-runner`, `nfr-regression-handler`). Author/audit twins ship together per the TASK-SKILL-116 pair rule; the NFR four ride the chain-allowlist mechanism as intentionally unpaired entries.
2. The vendored-set definition in `build.sh` MUST become a readable list (one name per line in a heredoc or adjacent manifest block with a per-line stage comment), replacing the single hardcoded string - the list is the reviewable contract for what ships.
3. `manifest.yaml`'s `author_audit_skills` count MUST be computed from the built payload at build time, never hardcoded; the same computed number MUST drive the `profile: full` determination.
4. The payload GUIDE (`dist` GUIDE.md source) MUST gain a lifecycle map: one row per SDP stage 1..14 -> skill pair -> invoked by (`/create-tasks`, `/ship-tasks`, or standalone-on-request), so operators see which stages the two commands automate and which they invoke ad hoc.
5. Both checks from the sibling FRs MUST pass over the expanded set: chain coverage (TASK-SKILL-116) and pair parity where the pair is at full contract (TASK-SKILL-118) - thin-but-shipped upstream pairs are permitted at their current completeness (parity applies per-pair as they are deepened; the checker's scope list says which pairs are held to full parity).
6. Build output MUST report payload size (bytes of the payload dir and of cyberos.plugin) on every build; the plugin zip MUST stay under 2 MB with the expanded set (current: ~322 KB), asserted in the build.
7. Reduced-profile behavior MUST be unchanged: a payload built without skill bodies still degrades to the doc-driven floor; `/init` behavior in target repos is unchanged apart from the larger `cuo/skills/` tree.
8. The two workflow docs MUST NOT change in this FR - upstream/downstream skills vendor as standalone-invocable; wiring them into new workflow steps is future work by separate FR. (Amended post-ship 2026-07-12: the guard is a point-in-time scope clause proven by this FR's commits in git history; the suite's t08 now asserts the durable form - both workflow docs vendored intact in the payload - so later FRs may legitimately evolve the docs. Surfaced by TASK-CUO-206.)

## §2 - Why this design

Vendor-by-default was the operator's explicit call: the plugin's value across many repos is the whole process, and an opt-in flag would leave most installs at stages 5-10 forever. Expanding the SET while freezing the WORKFLOWS (#8) keeps risk low - nothing changes for existing runs; new capability arrives as invocable skills. The readable list + computed counts turn build.sh's weakest spot (a drifting hardcoded string, root cause of the debugging-cycle gap) into reviewed data.

## §3 - Contract

`build.sh` vendored-set block (shape):

```bash
VENDORED_SKILLS="
statement-of-work-author            # SDP 1
statement-of-work-audit             # SDP 1
product-requirements-document-author # SDP 2
...
task-author              # SDP 5
...
decommissioning-audit               # SDP 14
nfr-certification-author            # SDP 4 (allowlisted unpaired)
"
```

Build report line: `cyberos-init: done. profile=full skills=<computed> payload=<bytes> plugin_zip=<bytes>`.

## §4 - Acceptance criteria

1. **All stages ship** (§1 #1) - after `build.sh`, every pair named in #1 exists (SKILL.md present) under both `cuo/skills/` and `plugin/skills/`; a stage->dir spot matrix in the test enumerates all 14 stages.
2. **Set is data, not a string** (§1 #2) - the vendored set reads as the one-per-line block with stage comments; the debugging-cycle regression (delete a line) is caught by the TASK-SKILL-116 check at build time.
3. **Counts computed** (§1 #3) - `manifest.yaml` skill count equals `ls`-derived reality for two differently-sized builds (full and a fixture-trimmed one); no literal count remains in the heredoc.
4. **Lifecycle map present and total** (§1 #4) - GUIDE.md's map has exactly 14 stage rows, each naming its pair and its invoker; no stage row says TBD.
5. **Sibling checks green over the expanded set** (§1 #5) - chain-coverage exits 0 (with the NFR/gate allowlist entries) and pair-parity exits 0 over its scoped list.
6. **Size budget** (§1 #6) - the build prints both sizes and fails if the plugin zip exceeds 2 MB; current expanded build passes.
7. **Reduced floor intact** (§1 #7) - a skill-less fixture build still yields `profile: reduced` and a working doc-driven payload.
8. **Workflows untouched** (§1 #8) - ship-tasks.md and create-tasks.md are diff-clean in this FR's commits (git history); the regression suite asserts the durable form (docs vendored intact in the payload) post-amendment.

## §5 - Verification

```bash
# tools/cyberos-init/tests/test_full_sdp_payload.sh
t01_stage_matrix_ships()         # AC 1
t02_set_is_reviewable_data()     # AC 2
t03_counts_computed()            # AC 3
t04_lifecycle_map_total()        # AC 4
t05_sibling_checks_green()       # AC 5
t06_size_budget()                # AC 6
t07_reduced_floor_intact()       # AC 7
t08_workflows_diff_clean()       # AC 8
```

## §6 - Implementation skeleton

build.sh: replace the set string with the commented block; compute counts via `find ... -name SKILL.md | wc -l`; size report + budget check; GUIDE source gains the 14-row table (stage, pair, invoker, artefact name).

## §7 - Dependencies

Depends on TASK-SKILL-116 (chain/pair checker + allowlist mechanics; also the single writer earlier on build.sh's set - land 116 first, then this expands the same block) and TASK-SKILL-117 (spike pair must exist to vendor). TASK-SKILL-118 deepens pairs independently; its parity scope list grows as pairs reach full contract.

## §8 - Example payloads

```
cyberos-init: done. profile=full skills=52 payload=2731008 plugin_zip=897412
```

## §9 - Open questions

None blocking. Which upstream artefacts /create-tasks should CONSUME automatically (e.g. detect an SRS and chain from it - it already accepts one as input) versus leave standalone stays future workflow work per #8.

## §10 - Failure modes inventory

1. Plugin bloat degrades agent skill-selection (too many similar descriptions) - descriptions already pass TASK-SKILL-111 trigger-enrichment conventions; the 2 MB budget plus the plugin's namespaced skill names keep the surface navigable. If selection quality regresses, the fallback is documented: trim the PLUGIN copy while keeping the payload copy (one-line build.sh split), by follow-up FR.
2. A vendored upstream pair is thin and an operator expects full-contract behavior - the GUIDE map marks contract level per pair (full/thin) so expectations are set at install time.
3. modules/skill renames a vendored dir - chain-coverage/pair checks fail the build at the renaming commit (TASK-IMP-068 path filter covers modules/skill/**).
4. Size creep over releases - the printed sizes land in every build log and the 2 MB assert stops the slow boil.
5. NFR four mistaken for author/audit pairs - the allowlist entry carries the reason string; pair check reads it and skips them by name, not by pattern.

## §11 - Implementation notes

Keep the vendored block sorted by SDP stage, not alphabetically - the review diff should read as the lifecycle. GUIDE map's "invoker" column values are exactly three strings (the two commands or `standalone`) so docs stay greppable.

*End of TASK-CUO-209.*
