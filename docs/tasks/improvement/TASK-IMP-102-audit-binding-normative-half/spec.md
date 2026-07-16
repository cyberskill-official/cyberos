---
id: TASK-IMP-102
title: Audits bind the normative half, not bytes the workflow rewrites
template: task@1
type: improvement
module: improvement
status: reviewing
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-17T11:20:00Z
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-100]
blocks: []
related_tasks: [TASK-IMP-084, TASK-IMP-086]
routed_back_count: 0
awh: N/A
verify: T
phase: "pre-1.0.0 release"
owner: Stephen Cheng (CTO)
created: 2026-07-17
shipped: null
memory_chain_hash: null
effort_hours: 3
service: modules/skill
new_files: []
modified_files:
  - modules/skill/task-audit/SKILL.md
  - tools/install/docs-tools/task-reconcile.mjs
  - tools/install/tests/test_task_reconcile.sh
source_pages:
  - "TASK-IMP-100 gate-log E4 (the dogfood finding): audits record audited_file_sha256 over the WHOLE spec file, but ship-tasks rewrites status/shipped in that same file at every phase - and authoring hashes the spec BEFORE the status flip, so the audited bytes exist in no commit (audit commit 53ef658f carries 4232bace8dca346c; the audit records 98efe9f21fd3a5c1)"
  - "modules/skill/task-audit/SKILL.md (payload_hash_field: audited_file_sha256; re_entrancy: idempotent_on_audited_file_sha256; fixity_notes claiming byte-stability for a given audited_file_sha256)"
  - "tools/install/docs-tools/task-reconcile.mjs R1 (the binding-gap note and the audit-commit fallback that today's convention forces)"
source_decisions:
  - "2026-07-17 Stephen: fix IMP-19 now as a batch-5 third member (recorded HITL answer)."
---

# TASK-IMP-102: Audits bind the normative half, not bytes the workflow rewrites

## Summary

An audit's byte-binding is supposed to answer one question: does this audit describe the spec on disk? Today it cannot. `audited_file_sha256` covers the whole file including `status` and `shipped` - fields ship-tasks rewrites at every phase - and authoring hashes the spec before flipping status, so the audited bytes never reach a commit. Add `audited_body_sha256_prefix` over the NORMATIVE half (body + frontmatter minus the lifecycle-mutable fields), teach task-reconcile to prefer it, and keep the legacy field accepted so the existing corpus stays readable.

## Problem

TASK-IMP-100's first live run flagged TASK-IMP-092 - shipped correctly through both human gates - as drifted. The instrument was right about the evidence and wrong about the conclusion, because the evidence it was handed cannot mean what it claims. A hash that no commit carries is the 086 class: a claim about bytes nobody can check. Every audit in the corpus carries one.

## Proposed Solution

Define the normative half explicitly: the spec body plus its frontmatter minus `status`, `shipped`, `routed_back_count`, `memory_chain_hash` - everything the audit actually judged, and nothing the workflow rewrites afterwards. task-audit's contract gains `audited_body_sha256_prefix` (16 hex) alongside the retained `audited_file_sha256_prefix`, with `re_entrancy` and `fixity_notes` re-stated against the body hash - the property that is true. task-reconcile's R1 prefers the body field when present (a direct, commit-independent comparison), falls back to today's audit-commit reconstruction for legacy audits, and stops reporting a binding gap when the body field answers the question.

## Alternatives Considered

- Hash after the status flip. Rejected: it makes the whole-file hash true for exactly one moment, and the next phase flip breaks it again - the field would still be unverifiable for every task past its first transition.
- Move status out of the spec into a sidecar. Rejected: frontmatter status is the record of truth (STATUS-REFERENCE §1); relocating it to protect a hash inverts the priority.
- Rewrite the corpus's audits to carry the new field. Rejected: historical audits describe historical specs; the legacy path reads them correctly and marks them legacy - honest and cheap.

## Success Metrics

- Primary: an audit carrying the body field verifies its binding directly - a lifecycle flip leaves R1 green with no binding-gap note, and a clause edit reds it - suite-asserted every run. Baseline: 100 % of audits carry an unverifiable whole-file hash. Deadline: final acceptance.
- Guardrail: legacy audits (no body field) still resolve via the audit-commit path with the gap named - the existing corpus does not become unreadable.

## Scope

In scope: the task-audit contract fields, the normative-half definition, task-reconcile R1's preference order, suite arms.

### Out of scope / Non-Goals

- Backfilling the field into existing audits (legacy path handles them).
- task-lint enforcement of the new field (a rubric change; separate decision).
- Any change to what an audit judges - only what it records about what it judged.

## Dependencies

- depends_on TASK-IMP-100 (R1 is the consumer). Same batch, serial - and per the new depends_on evidence gate, 100's coverage-gate artefact is its evidence.

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS task-author skill inside Cowork.
- **Scope:** spec drafted from TASK-IMP-100's dogfood finding; implementation under ship-tasks supervision.
- **Human review:** the fix-now decision is the operator's recorded 2026-07-17 answer; both HITL gates are recorded human verdicts.

## 1. Description (normative)

- 1.1 task-audit's contract MUST define the normative half as the spec body plus frontmatter minus `status`, `shipped`, `routed_back_count`, `memory_chain_hash`, and MUST record `audited_body_sha256_prefix` (16 hex) over it.
- 1.2 `audited_file_sha256_prefix` MUST be retained (provenance of the exact bytes seen), and the contract MUST state that it is NOT a verifiable binding after any lifecycle flip - the body field is the binding.
- 1.3 `re_entrancy` and `fixity_notes` MUST be re-stated against the body hash, so the skill claims only what holds.
- 1.4 task-reconcile R1 MUST prefer `audited_body_sha256_prefix` when present: match -> pass with no binding-gap note; mismatch -> red naming normative drift. Absent -> today's audit-commit path, gap noted as legacy.
- 1.5 Suite arms MUST cover: body field present + lifecycle flip -> pass, no gap note; body field present + clause edit -> red; body field absent (legacy) -> the audit-commit path with the gap named.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.4, #1.5) - body field + lifecycle flip -> R1 pass, zero binding-gap note - test: `tools/install/tests/test_task_reconcile.sh::t06_body_binding_preferred`
- [ ] AC 2 (traces_to: #1.4, #1.5) - body field + normative edit -> R1 red, drift named - test: `tools/install/tests/test_task_reconcile.sh::t06_body_binding_preferred (drift arm)`
- [ ] AC 3 (traces_to: #1.4, #1.5) - legacy audit (no body field) -> audit-commit path, gap named as legacy - test: `tools/install/tests/test_task_reconcile.sh::t06_body_binding_preferred (legacy arm)`
- [ ] AC 4 (traces_to: #1.1, #1.2, #1.3) - the contract records the body field, retains the file field with its caveat, and re-states re_entrancy/fixity against the body hash - verify: recorded greps in the gate log (prose contract; same rationale as TASK-IMP-090 AC 1).

## 3. Edge cases

- Audit carrying BOTH fields where the file field matches too (audit written post-flip): body field decides; no gap note - the file field is provenance, not a gate (AC 1's shape).
- Spec whose frontmatter lacks a lifecycle field entirely: the normalizer drops what is present and hashes the rest - absence is not drift (covered by the normalizer's field-list semantics, exercised in t06).
- A future lifecycle field added to the template: the normalizer's list is the single place to extend; the contract names it so the next author knows where to look.
- Legacy audit whose commit is unreachable (shallow clone): the existing "binding unverifiable" note stands - a note, never a verdict (t04's existing arm).
- Security-class: none - hashing and prose; no execution surface.
