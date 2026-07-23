---
id: TASK-IMP-138
title: Platform entry-point identity - thin spine vs explicit dual identity
template: task@1
type: improvement
module: improvement
status: ready_to_implement
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks: [TASK-MEMORY-303]
routed_back_count: 0
awh: N/A
verify: I
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 6
service: repo root + tools/install
new_files:
  - scripts/tests/test_entrypoint_identity.sh
modified_files:
  - AGENTS.md
  - CLAUDE.md
  - .cursorrules
  - .cursor/rules/cyberos.mdc
  - GEMINI.md
  - .github/copilot-instructions.md
  - .windsurfrules
  - tools/install/install.sh
  - CHANGELOG.md
source_pages:
  - "AGENTS.md:1 (root file on THIS platform repo is '# CyberOS Layer-1 Memory Protocol - AGENTS.md' - the full §0-§18 memory spec, not the task/HITL workflow spine)"
  - "CLAUDE.md (byte-duplicates the same full memory protocol - a second normative-looking copy with no divergence guard between the two)"
  - "pointer files measured 2026-07-23: .cursorrules, .cursor/rules/cyberos.mdc, GEMINI.md, .github/copilot-instructions.md, .windsurfrules all say 'Canonical instructions: AGENTS.md (root) and .cyberos/AGENT-ENTRY.md' - on this repo that sends an agent to memory law under the label 'canonical instructions', with task/HITL law only in the second file"
  - "tools/install/install.sh:512-529 (consumer installs write a THIN root AGENTS.md spine - workflow pointer + gates + HITL + memory pointer - with the marked-append discipline) and :515 ('Platform monorepo exception: root AGENTS.md remains the normative protocol source') + :336 (is_platform_repo() keys on modules/memory/memory.schema.json presence) - the dual identity is deliberate, implemented, and undocumented at the surfaces agents actually read"
  - "tools/install/install.sh:444-447 (consumer-side: 'The memory PROTOCOL lives at .cyberos/memory/AGENTS.md. The repo-root AGENTS.md is the [spine]... (that would bury the workflow every agent needs)') - the installer's own rationale for why a root protocol file is the wrong default"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T8 'Platform entry-point identity' authored as an improvement task and explicitly flagged as an operator-decision fork (plan file cyberos_hardening_plan_49404998; audit finding H6; plan approval boundary: 'T8 (entry-point restructure)... explicitly operator-decision forks')."
  - "2026-07-23 authoring: the fork is presented, not resolved - both branches are specified to implementable depth and the spec marks implementation BLOCKED until the operator picks one. The invariant ACs below hold under EITHER branch so the task's acceptance surface is stable across the decision."
---

# TASK-IMP-138: Platform entry-point identity - thin spine vs explicit dual identity

## Summary

On every consumer repo, `install.sh` writes root `AGENTS.md` as a thin workflow spine (task law, gates, HITL, memory pointer). On this platform repo, root `AGENTS.md` is the full Layer-1 *memory protocol*, `CLAUDE.md` duplicates it wholesale, and every pointer file (`.cursorrules`, `GEMINI.md`, `.windsurfrules`, copilot instructions, cursor rules) tells agents that root `AGENTS.md` is the first half of "canonical instructions". An agent on the platform repo that loads only the most-native file gets memory law and can miss task/HITL law entirely - the inverse of the installer's own documented rationale ("a root protocol file... would bury the workflow every agent needs"). This task resolves the identity - but WHICH resolution is a structural operator decision, presented here as a fork and deliberately not resolved by the author.

## Implementation status: BLOCKED - operator decision fork

**Implementation MUST NOT begin until the operator picks Branch A or Branch B below (recorded as a source_decisions entry on this spec plus the standard HITL record).** Everything else in this spec - the invariant clauses, the ACs, the test suite - is written to hold under either branch, so the audit gate passes now and the build queue simply cannot pick the task up until the fork is closed. This block is the task's own §9.1-rule-5 marker: BLOCKED on decision "platform entry-point identity, Branch A vs Branch B".

**Branch A - thin spine everywhere.** The platform repo's root `AGENTS.md` becomes the same thin workflow spine consumers get; the memory protocol's normative source moves to `modules/memory/cyberos/data/AGENTS.md` (already the vendored source of truth for installs), with `.cyberos/memory/AGENTS.md` as its installed copy; `install.sh`'s `is_platform_repo()` exception (:515, :336) is deleted; `CLAUDE.md` becomes a pointer. Consequence: one identity everywhere, but a structural move of the protocol's normative home - links, tooling, and §0.4-adjacent wording that assume "root AGENTS.md is the protocol" must be swept (the walker cites `AGENTS.md §3`; memory docs reference sections by bare `AGENTS.md`).

**Branch B - explicit dual identity.** Root `AGENTS.md` stays the protocol on the platform repo, but the identity is declared everywhere it matters: root `AGENTS.md` gains a first-screen preamble ("on this platform repo this file is the Layer-1 memory protocol; task/HITL law lives at `.cyberos/AGENT-ENTRY.md` - read that first for work"), every pointer file's wording changes from "Canonical instructions: AGENTS.md (root) and ..." to name the workflow spine FIRST and label root AGENTS.md as the memory protocol, and `CLAUDE.md` either becomes a pointer or carries a marked-copy banner + a drift check against AGENTS.md. Consequence: no structural move, smaller diff, but the platform keeps a root file whose name means something different than it does on every consumer repo - permanently carried complexity.

## Problem

Audit finding H6, verified first-hand 2026-07-23: the five pointer files all present root `AGENTS.md` as canonical instructions; on this repo that file is the memory protocol (`AGENTS.md:1`), and `CLAUDE.md` is a full second copy of it with no divergence guard. The installer itself implements the opposite default for consumers and documents why (`install.sh:444-447`), keeping the platform as a special case (`:515`) that nothing at the reading surfaces explains. Concrete failure modes: an agent reading only root `AGENTS.md` (the file the most tools read natively) learns memory law but not the two HITL gates or the never-push rule from the workflow side; and the AGENTS.md/CLAUDE.md byte-duplication forks silently the first time someone edits one (the protocol has already been amended repeatedly - P19..P22 - so edits are routine).

## Proposed Solution

Present the fork (above), and once the operator picks a branch, implement it plus the branch-independent invariants: (1) an agent that reads only root `AGENTS.md` on this repo encounters, within the first screen of text, where task/HITL law lives; (2) every pointer file names the workflow spine unambiguously and describes root `AGENTS.md`'s actual identity truthfully; (3) the memory protocol has exactly one normative source and every other copy declares itself a copy (pointer or marked duplicate with a drift check); (4) a grep-based test suite (`scripts/tests/test_entrypoint_identity.sh`) pins all three so the identity cannot silently regress. The suite is written to assert the invariants, not the branch, so it survives the decision.

## Alternatives Considered

- **Author resolves the fork (pick Branch A now).** Rejected: the plan's approval boundary explicitly reserves this as an operator decision; Branch A moves the normative home of a protocol document that §0.2 protects with an amendment gate - exactly the kind of structural change an author must not smuggle through a task spec.
- **Split into two tasks (one per branch), park both, close the loser.** Rejected: two mutually-exclusive ready tasks in one backlog invite an agent to pick one by queue order - the opposite of an operator gate. One task, one recorded decision, one implementation.
- **Do nothing; agents should read both files.** Rejected: the audit's finding is precisely that "should" has no mechanism. The pointer files say "canonical", agents act on the first file loaded, and the failure mode (missing HITL law) is the highest-consequence class this hardening wave exists for.
- **Fix only CLAUDE.md duplication and leave the identity question.** Rejected: the duplication is a symptom; the identity mismatch between platform and consumer repos is the cause, and patching around it leaves H6 open.

## Success Metrics

- Primary: within one release of the operator decision - an agent-eyes-first read of root `AGENTS.md` reaches task/HITL law (pointer or preamble) within the first 30 lines; all five pointer files describe the spine truthfully; exactly one normative protocol source exists with every copy self-declaring; `test_entrypoint_identity.sh` green in `run_all.sh`. Baseline today: 0 of the three invariants hold and no test exists.
- Guardrail: no change to CONSUMER install behavior in Branch B (byte-identical spine); in Branch A, consumer installs are also unchanged (they already get the spine) - only the platform repo's own files move. The memory test suite stays green under either branch.

## Scope

In scope: the operator decision record; root `AGENTS.md`, `CLAUDE.md`, and the five pointer files on this repo; `install.sh`'s platform exception (Branch A deletes it; Branch B keeps + documents it); the invariant test suite; CHANGELOG.

### Out of scope / Non-Goals

- Any change to the memory protocol's CONTENT (§0-§18 text) - this task moves/labels the container, never edits the law; protocol changes stay behind §0.2's approval grammar.
- Consumer-repo pointer file wording shipped by the installer (already truthful for consumers; if the operator wants consumer wording updates they ride the normal payload release, not this task).
- The BRAIN store layout and doctor wiring - TASK-MEMORY-303 (related: its INTEROP.md + this task's identity work both reduce the "which document governs me" ambiguity, from different ends).

## Dependencies

None blocking on other tasks; blocked on the OPERATOR FORK above (this is a decision dependency, not a task dependency - deliberately not encoded in `depends_on`, which the queue reads as task-graph edges). Related: TASK-MEMORY-303 (memory-side contract hardening in the same wave).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** the root/CLAUDE duplication, all five pointer files' wording, and the installer's spine + platform-exception code were verified first-hand at HEAD; the two branches are constructed from the installer's own documented rationale and the audit's H6 finding.
- **Human review:** the hardening plan was operator-approved 2026-07-23 with T8 explicitly reserved as an operator fork; this spec implements that reservation by specifying both branches and blocking on the pick.

## 1. Description (normative)

- 1.1 Implementation MUST NOT begin until an operator decision selecting Branch A or Branch B is recorded on this spec (a dated `source_decisions` entry) - the fork is the operator's, and an implementer arriving via the queue MUST halt at this clause if the record is absent.
- 1.2 After implementation, root `AGENTS.md` on this repo MUST surface where task/HITL law lives within its first 30 lines - as the whole file (Branch A: it IS the spine) or as a preamble pointer (Branch B).
- 1.3 Every pointer file (`.cursorrules`, `.cursor/rules/cyberos.mdc`, `GEMINI.md`, `.github/copilot-instructions.md`, `.windsurfrules`, `CLAUDE.md`) MUST name `.cyberos/AGENT-ENTRY.md` as the workflow entry and MUST describe root `AGENTS.md`'s identity truthfully for whichever branch was chosen. The "Canonical instructions: AGENTS.md (root) and ..." wording MUST NOT survive unqualified under Branch B.
- 1.4 The memory protocol MUST have exactly one normative source file after this task (Branch A: `modules/memory/cyberos/data/AGENTS.md`; Branch B: root `AGENTS.md`), and every other copy MUST declare itself a copy - a pointer, or a marked duplicate protected by a drift check that fails CI when the copies diverge.
- 1.5 Under Branch A only: `install.sh`'s `is_platform_repo()` AGENTS.md exception MUST be removed, and the references that assume a root-file protocol home (walker citation strings, memory docs) MUST be swept to the new home. Under Branch B only: the exception MUST gain a comment pointing at this task's decision record.
- 1.6 A new suite `scripts/tests/test_entrypoint_identity.sh` MUST assert the branch-independent invariants mechanically: first-30-lines task-law reachability in root AGENTS.md; spine naming in all six files of 1.3; single-normative-source (exactly one file without a copy/pointer marker); and (when a marked duplicate exists) the drift check runs and passes. It registers via the `run_all.sh` glob.
- 1.7 `CHANGELOG.md` MUST record the chosen branch and the decision date.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - the spec carries a dated operator decision entry naming the chosen branch BEFORE any implementation commit touches the files in 1.3-1.5 (verified by inspection of the spec's git history at review) - test: `scripts/tests/test_entrypoint_identity.sh::t01_decision_recorded`
- [ ] AC 2 (traces_to: #1.2) - `head -30 AGENTS.md` contains a reference to `.cyberos/AGENT-ENTRY.md` (or the spine content itself) - test: `scripts/tests/test_entrypoint_identity.sh::t02_first_screen_reaches_task_law`
- [ ] AC 3 (traces_to: #1.3) - each of the six files names `.cyberos/AGENT-ENTRY.md`, and under Branch B none carries the unqualified "Canonical instructions: AGENTS.md (root)" phrasing - test: `scripts/tests/test_entrypoint_identity.sh::t03_pointers_truthful`
- [ ] AC 4 (traces_to: #1.4) - exactly one protocol file lacks a copy/pointer marker; every other file containing the protocol's H1 carries one; when a marked duplicate exists, the drift check passes and a mutated scratch copy makes it fail - test: `scripts/tests/test_entrypoint_identity.sh::t04_single_normative_source`
- [ ] AC 5 (traces_to: #1.5) - Branch A: `is_platform_repo` no longer special-cases AGENTS.md and a repo-wide grep finds no stale root-protocol assumption in the swept references; Branch B: the exception carries the decision-record comment - test: `scripts/tests/test_entrypoint_identity.sh::t05_branch_consistency`
- [ ] AC 6 (traces_to: #1.6, #1.7) - the suite is discovered green by `bash scripts/tests/run_all.sh`, and CHANGELOG's top entry names the chosen branch - test: `scripts/tests/test_entrypoint_identity.sh::t06_registered_and_recorded`

## 3. Edge cases

- **The queue picks this task before the fork is decided:** clause 1.1 is the halt - the implementer's first action is checking the decision record and halting to ask when absent. This is the designed outcome, not a failure; the task exists to force exactly one recorded decision.
- **Branch A and §0.4's resolution language:** the protocol text itself describes store resolution, not its own file location, so moving the normative home does not amend the law - but 1.5's sweep must verify no §-citation ("AGENTS.md §3" in walker/invariants strings) becomes ambiguous; where it would, the citation gains the explicit path.
- **Tools that hard-read CLAUDE.md expecting full protocol text (Branch A):** any such reader gets a pointer instead; the memory module's own data copy remains complete, so programmatic consumers (installer, walker) are unaffected - only prompt-layer readers change, which is the point.
- **Divergence between AGENTS.md and CLAUDE.md discovered DURING implementation:** the implementer must surface the diff to the operator before unifying - one of the copies contains edits the other never got, and choosing silently which survives is a protocol-content decision (§0.2 territory), not a container decision.
- **Cursor's always-applied rules load both files today:** after either branch the loaded set shrinks or gains pointers; the test suite's first-30-lines check keeps the workflow law reachable regardless of which file a tool loads first.
- **Security-class:** documentation/identity restructuring only; no execution surface, no data movement beyond text files; the §11 injection posture is unchanged (pointer files remain trusted repo content, not untrusted input).
