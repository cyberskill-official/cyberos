---
id: TASK-IMP-143
title: "1.4.x — stuck-WIP hub sentinel + signed HITL verdict artifacts"
template: task@1
type: improvement
module: improvement
status: done
priority: p2
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T18:40:00+00:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: [TASK-IMP-140, TASK-CUO-303]
blocks: [TASK-IMP-144]
related_tasks: [TASK-IMP-139, TASK-IMP-082, TASK-IMP-108]
routed_back_count: 0
awh: N/A
verify: T
phase: "1.4.x"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 8
service: tools/docs-site + tools/install/docs-tools
new_files:
  - tools/install/docs-tools/verdict-artifact.mjs
  - tools/install/tests/test_verdict_artifact.sh
  - tools/docs-site/tests/test_stuck_wip_hub.sh
modified_files:
  - tools/docs-site/render-status-hub.mjs
  - tools/install/docs-tools/backlog-mutate.mjs
  - tools/install/tests/test_hitl_lock.sh
  - docs/verification/benchmark-gates.md
  - modules/cuo/chief-technology-officer/workflows/ship-tasks.md
  - CHANGELOG.md
  - docs/batches/batch-9-post-120-followups.md
  - docs/batches/batch-8-audit-hardening.md
source_pages:
  - "scripts/tests/test_benchmark_gates.sh::t_g13 (report-only stuck-WIP detector; threshold N=30)"
  - "docs/tasks/cuo/TASK-CUO-303-hitl-mechanical-lock/spec.md Non-Goal: signed/attributed verdict artifacts deferred to 1.4.x"
  - "tools/docs-site/render-status-hub.mjs draftStaleness / TASK-IMP-082 fp- byte-stability (no bare wall clock)"
source_decisions:
  - "2026-07-25 session operator: implement remaining IMP drafts IMP-143/144; HITL auto-approve with evidence; Wave 0 #7 closed (no ruleset to scrub)."
---

# TASK-IMP-143: 1.4.x stuck-WIP hub + signed HITL

## Summary

Promote G13's report-only stuck-WIP detector onto the status hub as a visible triage surface (threshold N=30 days, operator decides), and replace honor-system `--verdict-by` actor strings with content-addressed attributed HITL verdict artifacts that gate flips must mint and bind.

## Problem

1. G13 (`test_benchmark_gates.sh::t_g13`) finds in-flight tasks older than 30 days, but only as suite stdout — operators never see it on the status hub they already open.
2. TASK-CUO-303 records a human verdict path (`--verdict-by` + `--verdict-evidence`), but the actor string is honor-system: any string unlocks the gate. Named Non-Goal of CUO-303; 1.4.x owns attribution.

## Proposed Solution

1. **Stuck-WIP hub sentinel** — `render-status-hub.mjs` renders a "Stuck WIP (G13)" section listing in-flight tasks whose age exceeds N=30 days against a deterministic as-of date (CHANGELOG date for the current VERSION, overridable by `CYBEROS_NOW` / `CYBEROS_HUB_ASOF`). Each row links the task id for triage (resume / route_back / on_hold). Report-only: no status mutation. As-of is committed-input-derived so TASK-IMP-082's fp- stamp stays byte-stable without an env pin.
2. **Attributed verdict artifacts** — on every HITL gate flip, `backlog-mutate` mints (or validates) a content-addressed verdict JSON under `docs/tasks/_verdicts/` carrying `{schema, actor, timestamp, task_id, from, to, evidence_path, evidence_sha256, artifact_sha256}` where `artifact_sha256` hashes the canonical payload excluding itself. Gate flips refuse (exit 8) when the evidence file is missing/empty (unchanged) **or** when a pre-supplied artifact fails hash/actor/transition binding. The minted path is recorded on the flip JSON envelope and in the `status_overridden` reason field (evidence path remains the human note; artifact path is additional attribution).

## Alternatives Considered

- **Cryptographic signatures (ed25519) as the floor.** Deferred: content-addressed attribution closes the honor-system actor gap without key distribution; signatures remain a follow-on.
- **Wall-clock ages on the hub.** Rejected: breaks TASK-IMP-082 byte-stable stamps. As-of from VERSION's CHANGELOG date (or env pin in tests) keeps determinism.
- **Leave G13 suite-only.** Rejected: the draft and IMP-140 Non-Goal name the hub surface as the 1.4.x deliverable.

## Success Metrics

- Primary: status hub HTML contains a rendered "Stuck WIP (G13)" section (or an explicit empty-state line when none are stale) when as-of is pinned; gate flips mint a verifiable `_verdicts/*.json` artifact whose `evidence_sha256` matches the evidence file.
- Guardrail: `test_hitl_lock.sh` and `test_render_status_hub.sh` stay green; fp- stamp unchanged for an unchanged corpus when as-of is derived from the same CHANGELOG/VERSION inputs.

## Scope

### In scope

- Status-hub stuck-WIP render + tests.
- Verdict artifact mint/validate helper + backlog-mutate integration + tests.
- Docs: ship-tasks HITL steps, benchmark-gates G13 note, CHANGELOG, batch-9 / Wave 0 #7 close-out.

### Out of scope / Non-Goals

- Transition-locked state engine (TASK-IMP-144 / 1.5.0).
- Changing which transitions are the two HITL gates.
- Cryptographic signing keys / PKI.
- Making G13 fail the suite (stays report-only).

## Dependencies

`depends_on: [TASK-IMP-140, TASK-CUO-303]` — both `done`. Soft: TASK-IMP-082 (stamp), TASK-IMP-108 (staleness render pattern).

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) authoring from the batch-9 draft and CUO-303 Non-Goal.
- **Scope:** full AC authoring + implementation in this wave.
- **Human review:** session operator override — temporary HITL auto-approve with recorded evidence files (2026-07-25).

## 1. Description

- 1.1 The status hub MUST render a Stuck WIP (G13) surface for in-flight statuses (`implementing`, `ready_to_review`, `reviewing`, `ready_to_test`, `testing`) whose age in whole days against the hub as-of date exceeds `CYBEROS_G13_THRESHOLD_DAYS` (default 30). Age source: `created_at` then `created` frontmatter; unparseable ages are listed as unparseable, not silently dropped.
- 1.2 Hub as-of MUST be deterministic: `CYBEROS_NOW` or `CYBEROS_HUB_ASOF` when set; otherwise the dated CHANGELOG heading matching `VERSION`; otherwise refuse to classify by age and render an empty-state noting as-of unavailable (never read bare `Date.now()` into the page bytes).
- 1.3 Each stale row MUST name task id, status, age days, and the triage hint `resume / route_back / on_hold`, and MUST be markup outside the JSON payload (same render doctrine as TASK-IMP-108 §1.7).
- 1.4 On HITL gate flips (`reviewing→ready_to_test`, `testing→done`), after CUO-303 evidence checks pass, the flip MUST mint a verdict artifact under `docs/tasks/_verdicts/` (creating the directory as needed) whose payload binds actor, timestamp (`CYBEROS_NOW` when set, else UTC ISO), task_id, from, to, evidence_path (as given), evidence_sha256 (sha256 of evidence file bytes), and artifact_sha256 (sha256 of the canonical JSON without the artifact_sha256 field).
- 1.5 If `--verdict-artifact <path>` is supplied, the flip MUST validate that file's bindings (actor equals `--verdict-by`, task/from/to match, evidence_sha256 matches the evidence file, artifact_sha256 self-checks) and MUST NOT overwrite it; validation failure refuses exit 8 with no backlog write.
- 1.6 The flip JSON envelope MUST include `verdict_artifact` (path) and `evidence_sha256`; when a BRAIN store is present, the `status_overridden` payload `reason` MUST remain the evidence path (CUO-303 contract) and the artifact path MUST appear in flip stdout/JSON only (no BRAIN schema change required).
- 1.7 `ship-tasks.md` HITL steps and CHANGELOG Unreleased MUST document attributed verdict artifacts; Wave 0 decision #7 is closed in batch docs (no branch ruleset / nothing to scrub).

## Acceptance Criteria

- [ ] AC 1 (traces_to: #1.1–1.3) — `tools/docs-site/tests/test_stuck_wip_hub.sh` pins as-of, plants a stale in-flight fixture, asserts rendered "Stuck WIP (G13)" markup with triage hint; asserts no `Date.now` / bare wall-clock path in the age classifier.
- [ ] AC 2 (traces_to: #1.4–1.6) — `tools/install/tests/test_verdict_artifact.sh` (and extended `test_hitl_lock.sh`) prove mint on gated flip, hash bind, bad-artifact refusal exit 8, non-gate flips unchanged.
- [ ] AC 3 (traces_to: #1.7) — ship-tasks + CHANGELOG + batch-8/9 Wave 0 #7 close-out mention the artifact path and the closed ruleset residual.

## Test plan

1. `bash tools/docs-site/tests/test_stuck_wip_hub.sh`
2. `bash tools/install/tests/test_verdict_artifact.sh`
3. `bash tools/install/tests/test_hitl_lock.sh`
4. `bash .cyberos/cuo/gates/run-gates.sh`
