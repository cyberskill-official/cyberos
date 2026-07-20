---
artefact: repo-context-map@1
task_id: TASK-IMP-097
created: 2026-07-17
verdict: pass (repo-context-map-audit: patterns pinned to file:line, outside-domain count stated, ADR trigger evaluated)
---
# Repo context map - TASK-IMP-097

## Baseline patterns the new code must follow
- GUIDE source of truth: tools/install/docs/index.md ships verbatim as the payload's GUIDE.md via a straight copy (tools/install/build.sh:198, `cp "$here/docs/index.md" "$out/GUIDE.md"`) - a section added to the source IS the consumer document; there is no second render step to keep in sync
- GUIDE voice: H2 sections, consumer-facing, imperative steps; the one brand mention in the file is the agent list under Prerequisites (tools/install/docs/index.md:41) - the new section keeps the generic "sandboxed agent" framing and placeholder paths (`/mnt/<repo>`, `/tmp/work`) per spec edge case 1
- suite harness shape: test_full_sdp_payload.sh builds ONE scratch payload up front (line 13) and every scenario greps it; ok/fail counters and a `pass=N fail=N` tail - t09 reuses that payload and builds nothing
- GUIDE gates already standing: t04 fails on any "TBD" and on numbered table rows without a valid invoker - the new section is prose-only (no `| <digit>` rows, no TBD)
- doctrine placement: ship-tasks.md §11a Swarm-execution sub-bullets carry environment learnings; single-line cross-references with backticked repo paths are the house style, and scripts/check_doc_anchors.sh resolves every such path (modules/cuo is a scanned root)
- version discipline: workflow_version moves only on normative rule changes; a prose pointer is not normative, so 2.6.3 stays in this task (TASK-IMP-099 carries the round's single bump, per both specs' Dependencies sections)

## Schemas / interfaces in scope
- new GUIDE section "Running CyberOS under sandboxed agents": four symptom -> cause -> working-pattern entries (hook chains and package installs killed by per-command caps + background death, with manual obligation replay, `--no-verify`, and recorded evidence; slow builds over the synced mount -> local clone with a local ref move back; package-manager churn kept off the mount; mount unlink/permission lag -> wait and re-check) + one closing pointer to the §11a/§9 rules it complements
- modules/cuo/chief-technology-officer/workflows/ship-tasks.md: ONE sub-bullet inserted in §11a Swarm execution directly after the one-writer-one-view bullet, pointing constrained environments at the GUIDE section - no rule text duplicated
- tools/install/tests/test_full_sdp_payload.sh: t09_sandbox_runbook_guide - five greps against the scratch GUIDE.md (section heading, local-clone line, local-ref-move/no-push clause, hook-replay line, `--no-verify`), each with its own failure message

## Files outside the immediate domain (tools/install/)
1. modules/cuo/chief-technology-officer/workflows/ship-tasks.md (modified, +1 cross-reference line - spec-declared in `modified_files`)

files_outside_immediate_domain: 1 (<= 3 -> no ADR trigger; the one file is spec-declared doctrine prose).

## Blast radius
file_count: 3 modified, 0 new (tools/install/docs/index.md +47 lines, ship-tasks.md +1 line, test_full_sdp_payload.sh +23/-1) | module_count: 2 (tools/install docs+tests, modules/cuo workflow) | cross_module_edges: suite -> tools/install/build.sh scratch payload (read-only use); GUIDE <-> ship-tasks.md (prose pointers both ways, no duplicated rule text to drift) module_placement_warning: null (spec declares `service: tools/install/docs`; every touched file is spec-declared) Behavioral radius: consumer repos receive the runbook inside GUIDE.md on their next install or re-vendor; a ship-tasks reader in a constrained environment gets a pointer instead of re-learning the facts by incident; every future payload build is gated on carrying the section. No executable behavior changes anywhere - documentation plus one grep gate.
