---
id: TASK-IMP-061
title: "BRAIN Phase 0 consent completion"
template: task@1
type: improvement
module: improvement
status: done
priority: p0
author: "@stephencheng"
department: engineering
created_at: 2026-07-08T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: []
related_tasks:
  - TASK-EVAL-001
  - TASK-IMP-066
  - TASK-MEMORY-121
  - TASK-IMP-138
routed_back_count: 0
awh: N/A
verify: T
phase: "Wave 5 - platform and process"
owner: Stephen Cheng (CTO)
created: 2026-07-08
memory_chain_hash: null
effort_hours: 6
service: modules/memory
new_files:
  - modules/memory/runtime/starter/templates/CONSENT.md
  - modules/memory/runtime/starter/cyberos-starter/.cyberos/memory/store/meta/consent/README.md
  - modules/memory/tests/test_personnel_consent.py
  - tools/install/tests/test_memory_agents_protocol.sh
  - docs/batches/batch-12f-imp-061.md
  - docs/batches/batch-12f-gate1-acceptance.md
  - docs/batches/batch-12f-gate2-acceptance.md
modified_files:
  - modules/memory/cyberos/data/AGENTS.md
  - modules/memory/cyberos/data/memory.invariants.yaml
  - modules/memory/memory.invariants.yaml
  - modules/memory/cyberos/core/invariants.py
  - modules/memory/tools/tests/generate_vectors.py
  - modules/memory/tools/tests/vectors/21-personnel-no-consent/expected.json
  - modules/memory/runtime/starter/templates/PERSON.md
  - tools/install/build.sh
  - tools/install/install.sh
  - docs/tasks/BACKLOG.md
  - docs/tasks/improvement/TASK-IMP-061-brain-phase-0-consent-completion/spec.md
source_pages:
  - "docs/strategy/cyberos-deep-audit-and-auto-evolution-plan-2026-07-06.md:154 — Complete BRAIN Phase 0 before capture widens; monitoring notice drafted but not cleared."
  - "docs/strategy/cyberos-brain-evaluation-plan.md — Phase 0 governance first: notice + acknowledgment before capture."
  - "docs/deploy/brain-capture-activation.md — ordered activation; step 1 is counsel clearance of docs/legal/data-monitoring-and-evaluation-notice.md."
  - "modules/memory/tools/tests/generate_vectors.py:494-507 — fixture 21 personnel-no-consent documents that the validator does not yet enforce consent."
  - "modules/memory/runtime/hooks/gateguard.py:123-127 — PERSON writes already require consent.has_consent + consent_event referencing an audit/consent row."
  - "modules/memory/runtime/starter/templates/PERSON.md — consent block with ${CONSENT_EVENT_ID}; no consent-record template existed."
  - "tools/install/build.sh:220 — payload memory/AGENTS.md materialised from root thin spine, not modules/memory/cyberos/data/AGENTS.md (IMP-138 Branch A normative home)."
  - "docs/tasks/improvement/TASK-IMP-066-track-c-brain-activation-rollout-deploy-notice-a/spec.md — Track C closed won't-do for 1.x; depends_on IMP-061."
source_decisions:
  - "DEC-2520 / DEC-2525 (EVAL governance) — capture gated on acknowledgment; covert collection out of scope. This task does NOT clear the employment notice; it completes Layer-1 BRAIN consent scaffolding only."
  - "2026-07-25 operator (batch/12f): ship IMP-061 as Layer-1 protocol + consent-record scaffolding + enforceable invariant; NOT product EVAL activation or counsel clearance. Session HITL override applies."
---

# TASK-IMP-061: BRAIN Phase 0 consent completion

## Summary

Layer-1 BRAIN already *declares* consent on personnel memories (PERSON template, gateguard checklist, audit `consent_event_id`, fixture 21), but the protocol has no normative consent section, no consent-record store path, no walker invariant, and the install payload can vendor the thin root `AGENTS.md` into `.cyberos/memory/AGENTS.md` instead of the dense Layer-1 protocol. This task completes Phase 0 for the **agent BRAIN** (markdown store): protocol §19, `meta/consent/` scaffolding, `personnel-requires-consent` invariant + tests, and a build fix so the protocol (including §19) actually ships.

## Problem

Deep-audit Stage 5 and the brain-evaluation plan both order **governance before capture**. For the product BRAIN/EVAL path that means TASK-EVAL-001 notice + acknowledgment (and counsel clearance of `docs/legal/data-monitoring-and-evaluation-notice.md`). That product path is **not** this task: Track C (IMP-066) is closed won't-do for 1.x, and EVAL-001 remains `on_hold`.

For the **Layer-1 agent BRAIN** the gap is smaller and already named in-tree: fixture 21 says "validator currently doesn't enforce consent". PERSON writes require a consent event id, but there is no `CONSENT` template, no `meta/consent/` home, no `cyberos doctor` check, and no AGENTS.md clause. Agents can invent `has_consent: true` with a null event and stay doctor-green. Separately, `build.sh` materialises root thin `AGENTS.md` into the payload memory tree, so even a correct protocol update may not reach consumer installs.

## Proposed Solution

1. Add **AGENTS.md §19 (Phase 0 consent)** — personnel-classified / person-kind memories MUST carry `consent.has_consent: true` and a non-null `consent.consent_event` that resolves to `meta/consent/<id>.md` (or a matching audit row id).
2. Scaffold **consent records** under `meta/consent/` (starter README + `CONSENT.md` template); install creates the directory on fresh BRAIN scaffold.
3. Add walker invariant **`personnel-requires-consent`** (error level) + pytest coverage for pass/fail/resolve paths.
4. Flip fixture 21's expected critical code to `personnel-requires-consent` (forward-compat corpus + generator comment).
5. Fix **`build.sh`** to materialise `modules/memory/cyberos/data/AGENTS.md` into `$out/memory/AGENTS.md` (IMP-138 Branch A normative home).
6. Document explicit **non-goals**: counsel clearance of the employment monitoring notice; EVAL-001 / capture activation; Track C rollout.

## Alternatives Considered

- **Close as won't-do / blocked on counsel.** Rejected for the Layer-1 half: fixture 21 and gateguard already commit to enforceable consent; leaving it forever soft is the opposite of Phase 0. Counsel clearance remains a separate operator/legal gate documented in `docs/deploy/brain-capture-activation.md`.
- **Implement full EVAL-001 activation in this PR.** Rejected: IMP-066 closed won't-do for 1.x; EVAL-001 is `on_hold` pending AUTH; scope would be a product rewrite, not payload scaffolding.
- **Only docs, no invariant.** Rejected: without a walker check, Phase 0 is aspirational and fixture 21 stays a lie.

## Success Metrics

- Primary: a store with `classification: personnel` and no resolvable `consent.consent_event` fails `personnel-requires-consent`; a store with a matching `meta/consent/<id>.md` passes. Suite-asserted.
- Guardrail: payload `memory/AGENTS.md` after `build.sh` matches the dense protocol (contains `§19` / `Phase 0 consent`), not only the thin spine.

## Scope

In scope: Layer-1 protocol AGENTS.md §19; consent templates + starter/install scaffolding; invariant + tests; fixture 21 expectation; build.sh AGENTS vendor path; task lifecycle + batch evidence.

Out of scope / Non-Goals:

- Publishing or counsel-clearing `docs/legal/data-monitoring-and-evaluation-notice.md`.
- Wiring `SqlConsentGate` / TASK-EVAL-001 acknowledgment ledger / `CAPTURE_ENABLED`.
- Track C brain activation rollout (IMP-066 closed).
- Restoring a separate workbench `cyberos_validate.py` binary (living check is `cyberos.core.invariants`).

## Dependencies

None blocking. Related: EVAL-001 (product gate, separate), IMP-138 (Branch A already decided; this task finishes the build vendor path for the protocol file), IMP-066 (closed; depended on this task for Track C which will not ship in 1.x).

## AI Authorship Disclosure

- **Tools used:** Composer (Cursor agent) under CyberOS ship-tasks / session HITL for batch/12f.
- **Scope:** Layer-1 BRAIN Phase 0 consent scaffolding only. Re-derived and CONFIRMED: fixture 21 previously expected no critical codes and documented the missing rule (`generate_vectors.py`); gateguard PERSON path already required consent_event; PERSON template used `${CONSENT_EVENT_ID}` with no CONSENT template; build.sh materialised root thin AGENTS.md into memory/. Re-derived and CORRECTED: build vendor path root `AGENTS.md` -> `modules/memory/cyberos/data/AGENTS.md` (IMP-138 Branch A home). Measured and ADDED: AGENTS.md §19, `personnel-requires-consent` invariant, `meta/consent/` scaffold, `CONSENT.md` template, `test_personnel_consent.py` (8 cases). Product EVAL notice clearance and capture activation excluded.
- **Human review:** session HITL override for batch/12f (operator Stephen Cheng); Gate-1 and Gate-2 evidence under docs/batches/batch-12f-*.

## 1. Description (BCP-14 normative)

- 1.1 The protocol MUST add AGENTS.md §19 Phase 0 consent stating that any memory whose frontmatter has `classification: personnel` OR `kind: person` OR a `scope` containing `people` MUST include a `consent` object with `has_consent: true` and a non-null `consent_event` string.
- 1.2 `consent.consent_event` MUST resolve to either (a) a file `meta/consent/<consent_event>.md` under the store, or (b) an `audit_id` present in a legacy `audit/*.jsonl` row. Unresolved ids MUST fail the doctor walk.
- 1.3 Consent records under `meta/consent/` MUST be documented as append-only disclosures and MUST NOT be treated as a substitute for the product EVAL acknowledgment ledger.
- 1.4 The repo MUST ship starter template `CONSENT.md` and a `meta/consent/README.md` in the cyberos-starter tree.
- 1.5 `install.sh` MUST create `meta/consent/` when scaffolding a fresh BRAIN store.
- 1.6 Invariant `personnel-requires-consent` (level: error) MUST exist with check `cyberos.core.invariants.check_personnel_requires_consent`, registered identically in both invariants.yaml copies.
- 1.7 Tests MUST prove: clean store passes; personnel without consent fails; unresolved event fails; resolvable `meta/consent/<id>.md` passes.
- 1.8 Vector fixture `21-personnel-no-consent` MUST expect critical code `personnel-requires-consent`.
- 1.9 `build.sh` MUST materialise `modules/memory/cyberos/data/AGENTS.md` (not root thin `AGENTS.md`) into the install payload at `memory/AGENTS.md`.
- 1.10 This task MUST NOT claim counsel clearance of the employment monitoring notice, flip `CAPTURE_ENABLED`, or implement EVAL notice/ack APIs.
- 1.11 Session HITL evidence MUST be recorded under `docs/batches/batch-12f-*`; `done` requires recorded human verdicts (session override allowed).

## Acceptance criteria

- [x] AC 1 (traces_to: #1.1) - AGENTS.md §19 present with BCP-14 MUST clauses for personnel consent + resolution rules - verify: `grep -n '§19' modules/memory/cyberos/data/AGENTS.md`
- [x] AC 2 (traces_to: #1.2,#1.4,#1.5) - `meta/consent/` scaffolded in starter + fresh install; consent_event resolution documented; `CONSENT.md` template exists - verify: paths exist on disk
- [x] AC 3 (traces_to: #1.6,#1.7) - `personnel-requires-consent` fails open violations and passes resolved consent - test: `modules/memory/tests/test_personnel_consent.py`
- [x] AC 4 (traces_to: #1.7) - personnel consent suite green (8 cases) - test: `modules/memory/tests/test_personnel_consent.py`
- [x] AC 5 (traces_to: #1.8) - fixture 21 expects `personnel-requires-consent` - verify: `modules/memory/tools/tests/vectors/21-personnel-no-consent/expected.json`
- [x] AC 6 (traces_to: #1.9) - `build.sh` vendors dense protocol; payload `memory/AGENTS.md` contains `§19` - verify: `grep data/AGENTS.md tools/install/build.sh`
- [x] AC 7 (traces_to: #1.3,#1.10) - Spec non-goals exclude notice clearance / EVAL activation; no capture env flips - verify: recorded diff review
- [x] AC 8 (traces_to: #1.11) - Task reaches `done` with Gate-1 + Gate-2 session HITL evidence under `docs/batches/` - verify: `docs/batches/batch-12f-gate1-acceptance.md`

## Verification

1. `cd modules/memory && PYTHONPATH=cyberos:../.. pytest tests/test_personnel_consent.py -q`
2. `bash tools/install/build.sh` (or repo equivalent) and `grep -n '§19' dist/cyberos/memory/AGENTS.md`
3. Manual: construct temp store with personnel md lacking consent → `python -c 'from cyberos.core.invariants import check_personnel_requires_consent; ...'` returns failed.

## Edge cases

| Case | Expected |
|---|---|
| No memories/ yet | Pass (nothing to gate) |
| Operational classification, no consent block | Pass |
| `has_consent: true`, `consent_event: null` | Fail |
| `has_consent: false` on personnel | Fail |
| Event id file missing | Fail |
| Event id matches `meta/consent/<id>.md` | Pass |
| README.md under meta/consent/ | Ignored as a consent event |
| Product notice still DRAFT | Out of scope — documented, not blocked |

## Failure modes

| Failure | Detection | Mitigation |
|---|---|---|
| Protocol edited but thin AGENTS still vendored | AC6 grep | build.sh path fix |
| Invariant yaml drift between two copies | byte-identical check in review | edit both / cp |
| Agents set has_consent without event | AC3 | invariant |

## Implementation notes

- Prefer extending existing frontmatter `consent:` shape; do not invent a second schema.
- Keep resolution file-based first; jsonl audit_id match is back-compat only.
- Cite TASK-IMP-061 as the §0.2 approval channel for §19 (operator ship session 2026-07-25), same pattern as P19/P20/P22 section headers.
