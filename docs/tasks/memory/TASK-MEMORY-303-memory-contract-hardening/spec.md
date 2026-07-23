---
id: TASK-MEMORY-303
title: Memory hardening - schema single-source, INTEROP.md, walker + doctor
template: task@1
type: improvement
module: memory
status: done
priority: p1
author: "@stephencheng"
department: engineering
created_at: 2026-07-23T00:00:00+07:00
ai_authorship: generated_then_reviewed
eu_ai_act_risk_class: not_ai
client_visible: false
depends_on: []
blocks: [TASK-IMP-140]
related_tasks: [TASK-MEMORY-261, TASK-MEMORY-302, TASK-MEMORY-117, TASK-MEMORY-119]
routed_back_count: 0
awh: N/A
verify: T
phase: "post-1.1.0"
owner: Stephen Cheng (CTO)
created: 2026-07-23
memory_chain_hash: null
effort_hours: 16
service: modules/memory
new_files:
  - modules/memory/INTEROP.md
  - modules/memory/tests/test_schema_single_source.py
  - modules/memory/tests/test_interop_doc.py
  - modules/memory/tests/test_walker_sessions_dreams.py
  - modules/memory/tests/test_session_id_stamping.py
  - tools/install/tests/test_doctor_gate.sh
modified_files:
  - modules/memory/memory.schema.json
  - modules/memory/tests/test_schema_drift.py
  - modules/memory/cyberos/core/invariants.py
  - modules/memory/memory.invariants.yaml
  - modules/memory/cyberos/core/writer.py
  - tools/install/gates/run-gates.sh
  - tools/install/build.sh
  - CHANGELOG.md
source_pages:
  - "measured 2026-07-23: the two TRACKED memory.schema.json copies differ - modules/memory/cyberos/data/memory.schema.json carries StoreAcl/StoreAclEntry/StoreAclMode (P20 §14.4.7); modules/memory/memory.schema.json does NOT; tools/install/build.sh:161 vendors the STALE root copy into every payload (the third copy in the distribution chain), so installed repos validate against a schema missing the ACL definitions the protocol mandates"
  - "modules/memory/tests/test_schema_drift.py:31-33 (_COMMITTED = _MEMORY / 'docs' / 'memory.schema.json' - a path that does not exist, so all three drift tests pytest.skip silently; the docstring's regen command cites the same phantom docs/ path)"
  - "AGENTS.md §14.1 ('A consumer that does not adopt the ledger MUST obey INTEROP.md (<= 6,000 chars)'); measured 2026-07-23: no INTEROP.md exists anywhere in the repo"
  - "modules/memory/cyberos/core/invariants.py:86-92 (_CANONICAL_TOP_LEVEL_DIRS lacks 'sessions' and 'dreams', so exercising §18.2 session bodies or §7.7.4 dream artefacts creates dirs the doctor rejects); AGENTS.md §7.7.2 names walker invariant dream-applied-row-has-provenance, §14.4.7 names store-yaml-acl-valid, §18.8 names four session lifecycle invariants - none of these ids exist in modules/memory/memory.invariants.yaml (13 declared ids, all implemented, none of these)"
  - "measured 2026-07-23: the live store .cyberos/memory/store/ carries stray top-level adrs/ and impl-plans/ (two dirs, not the five TASK-MEMORY-261's context lists - three were evidently cleaned since), so layout-root-canonical FAILs and a protocol-compliant agent must refuse writes (§12 FROZEN_RECOVERABLE)"
  - "modules/memory/cyberos/core/writer.py: zero occurrences of 'session' (grep) - §18.7's extra.session_id is unwired on put/move/delete; sessions/.active is the active-session marker per §18.7"
  - "tools/install/gates/run-gates.sh: zero occurrences of 'doctor' - BRAIN health is not part of the machine-gate floor even when memory is installed; the memory CLI surface for installed repos is `cs memory <args>` (tools/install/cli/bin/cli.mjs:83-94, dispatching python3 -m cyberos, present only when cyberos-memory is locally installed)"
source_decisions:
  - "2026-07-23 operator: CyberOS Hardening Plan approved; Phase 2 T6 'Memory hardening' authored as an improvement task (plan file cyberos_hardening_plan_49404998; audit finding H10 + medium memory items)."
  - "2026-07-23 authoring: schema unification direction is package-data-forward - the root copy is regenerated from the msgspec Structs (which already emit StoreAcl, per cyberos/core/store_acl.py) and every copy must be byte-identical to the generator output; the data copy is not rolled back, because §14.4.7 makes StoreAcl normative."
  - "2026-07-23 authoring: the live-store repair clause executes the operator-approved disposition that TASK-MEMORY-261 (draft) specifies the decision procedure for; if 261 is unshipped when this task starts, its ADR decision step runs first as a prerequisite inside this task's HITL flow. Expressed via related_tasks + body prose, not depends_on, to avoid editing an existing task's frontmatter for reciprocity."
  - "2026-07-23 authoring: plan says 'three memory.schema.json copies'; measured truth is two tracked copies + the vendored payload copy as the third in the distribution chain. Spec is written to the measured shape."
---

# TASK-MEMORY-303: Memory hardening - schema single-source, INTEROP.md, walker + doctor wiring

## Summary

Five memory-protocol promises are broken in ways that compound: the schema has forked (the root + vendored copies lack the StoreAcl definitions §14.4.7 makes normative, while the package-data copy has them) and the drift test that should have caught it points at a nonexistent path and silently skips; the §14.1-mandated `INTEROP.md` does not exist; using the protocol's own §7.7 dream or §18 session features would create `dreams/`/`sessions/` dirs the doctor's canonical-layout allowlist rejects, and the walker invariants AGENTS.md names for those features are undeclared; the live store is already FROZEN_RECOVERABLE from stray `adrs/` + `impl-plans/` dirs; and §18.7's `extra.session_id` stamping is unwired in the canonical writer. This task restores schema single-sourcing with a real drift test, authors INTEROP.md, teaches the walker the protocol's own features, repairs the live store under operator gate, wires `cyberos doctor` into the machine-gate floor where memory is installed, and stamps `extra.session_id`.

## Problem

Audit finding H10 plus the memory-row medium items, all verified first-hand 2026-07-23:

1. **Schema fork, guarded by a skipping test.** `modules/memory/cyberos/data/memory.schema.json` (the copy the Python package loads) carries `StoreAcl`/`StoreAclEntry`/`StoreAclMode`; `modules/memory/memory.schema.json` (the copy `build.sh:161` vendors into every payload) does not. Consumers therefore validate STORE.yaml ACLs against a schema that has never heard of them. `test_schema_drift.py` exists to catch exactly this and catches nothing: `_COMMITTED` points at `modules/memory/docs/memory.schema.json`, which does not exist, so every test in the module `pytest.skip`s - a green that means "did not look".
2. **INTEROP.md is a dangling MUST.** AGENTS.md §14.1 binds non-ledger consumers to a <= 6,000-char `INTEROP.md`; no such file exists anywhere. Every cross-agent consumer is currently bound to a document nobody can read.
3. **The walker rejects the protocol's own features.** `_CANONICAL_TOP_LEVEL_DIRS` omits `sessions` (§18.2 bodies) and `dreams` (§7.7.4 artefacts): a store that exercises dreaming or transcripts goes doctor-RED. The invariants AGENTS.md explicitly names for these features (`dream-applied-row-has-provenance` §7.7.2, `store-yaml-acl-valid` §14.4.7, the §18.8 session lifecycle set) are absent from `memory.invariants.yaml`.
4. **The live BRAIN is frozen.** `.cyberos/memory/store/` carries stray top-level `adrs/` and `impl-plans/` (measured: two dirs today; TASK-MEMORY-261's earlier context listed five, three since cleaned). `layout-root-canonical` fails, so §12 forces protocol-compliant agents to refuse writes - the audit trail is silently absent while the repo's own doctrine (AGENT-ENTRY.md #4) tells agents to record decisions into it.
5. **§18.7 unwired.** The canonical writer never stamps `extra.session_id`, so even with an active session, put/move/delete rows carry no session linkage and the §18.8 walker checks would have nothing to verify.

## Proposed Solution

**Schema:** regenerate `modules/memory/memory.schema.json` from the generator (`tools/cyberos_generate_schema.py`, whose Struct source already emits StoreAcl per `cyberos/core/store_acl.py`); fix `test_schema_drift.py`'s `_COMMITTED` to the real root-copy path and add `modules/memory/tests/test_schema_single_source.py` asserting (a) the generator's `--check` passes against the root copy, (b) root and package-data copies are byte-identical, (c) `build.sh` vendors from the root copy path (source-grep), and (d) the drift test can never silently skip - a missing committed schema is a FAIL, not a skip. **INTEROP.md:** author `modules/memory/INTEROP.md` (<= 6,000 chars) covering the §14.1 consumer subset - read paths, the no-write rule for `audit/`/`HEAD`/`.lock`, canonical-writer routing, `STORE.yaml` ACL honor-for-writes (§14.4.6), sync_class export semantics (§14.3) - and vendor it via `build.sh` next to the schema. **Walker:** add `sessions` and `dreams` to `_CANONICAL_TOP_LEVEL_DIRS`; declare and implement the missing invariants (`dream-applied-row-has-provenance`, `store-yaml-acl-valid`, `session-lifecycle` covering §18.8's four checks) in `memory.invariants.yaml` + `invariants.py`. **Store repair:** an operator-gated `move` of the stray dirs' contents into their canonical homes per the ADR disposition TASK-MEMORY-261 specifies (decision first if 261 is unshipped), leaving the audit chain intact, ending with `cyberos doctor` OK on the live store. **Doctor gate:** `run-gates.sh` gains a doctor gate that runs when `.cyberos/memory/store/` exists AND the memory CLI is importable (`python3 -m cyberos doctor`), SKIPs (with provenance line) when either is absent, and fails RED on doctor FAIL. **Session stamping:** `writer.py` stamps `extra.session_id` on every put/move/delete row while `sessions/.active` names an active session (§18.7), covered by `test_session_id_stamping.py`.

## Alternatives Considered

- **Roll the package-data copy back to match the root (drop StoreAcl).** Rejected: §14.4.7 makes StoreAcl normative and TASK-MEMORY-117 shipped its enforcement; the root copy is the stale side, so unification is package-data-forward.
- **Point build.sh at the package-data copy instead of regenerating the root.** Rejected: leaves two tracked copies whose equality nothing enforces - the current defect with the arrow flipped. Single-sourcing means one generator output, every copy byte-identical, a test that fails on divergence.
- **Write INTEROP.md into `.cyberos/memory/` directly.** Rejected: `.cyberos/` is the installed, machine-refreshed tree (gitignored); the source of truth belongs in `modules/memory/` beside the protocol docs and vendors outward like the schema does (§0.4's update rule: machine updates refresh docs without touching `store/`).
- **Auto-repair the live store in this task without an operator gate.** Rejected twice over: §0.3 memory-file immutability plus the standing instruction that BRAIN mutations here are operator-gated; and TASK-MEMORY-261 already specifies the decision procedure (add-to-canonical vs relocate) - executing before that ADR would guess the disposition. The gate is a HITL halt inside this task's implementation.
- **Wire doctor into gates unconditionally.** Rejected: most consumer installs have no memory store; an unconditional gate would RED every repo that never opted into memory. Presence-gated with a loud SKIP line preserves the fail-closed posture where memory exists without taxing repos where it does not.

## Success Metrics

- Primary: by the next CyberOS release - every tracked + vendored `memory.schema.json` is byte-identical and StoreAcl-bearing; `pytest modules/memory/tests/test_schema_drift.py` executes (not skips) and passes; `INTEROP.md` exists <= 6,000 chars and ships in the payload; `cyberos doctor` on the live store reports OK (0 layout errors) and on a store with `sessions/` + `dreams/` dirs reports OK; run-gates on this repo shows the doctor gate PASS. Baselines today: copies differ, drift test skips, no INTEROP.md, live store FAILs layout, no doctor gate.
- Guardrail: the full `modules/memory` pytest suite stays green; stores WITHOUT memory installed see exactly one new SKIP line in run-gates and no behavior change; the audit chain on the live store is append-only through the repair (verified by `cyberos verify` before/after).

## Scope

In scope: schema regeneration + drift-test fix + single-source test, INTEROP.md authoring + vendoring, walker allowlist + three new invariant families, the operator-gated live-store repair, the presence-gated doctor gate in run-gates.sh, §18.7 stamping in writer.py, CHANGELOG.

### Out of scope / Non-Goals

- The single-source-of-truth refactor for the canonical-dir set across scaffolders and the five-artifact-dir ADR - TASK-MEMORY-261's scope; this task executes the live-store move under that ADR and adds the two protocol dirs to the allowlist, nothing more.
- Fixing the applier that raw-writes artefacts to the store root - TASK-MEMORY-302 (bug, draft) owns the root cause; this task repairs the state it left behind.
- PII/denylist gating inside `put`, sidecar format (b) emission, tmp-file nonce compliance - real §5/§8.3 gaps, deliberately deferred to keep this task shippable; recorded here so the deferral is discoverable.
- The BRAIN recording of the audit itself - TASK-IMP-140's final step, which this task unblocks (`blocks: [TASK-IMP-140]`).

## Dependencies

Blocks TASK-IMP-140 (its §13 BRAIN-recording step needs the store un-frozen by this task's repair). Related: TASK-MEMORY-261 (draft - specifies the layout ADR + single-sourcing this task's repair executes under; runs first inside this task's HITL flow if still unshipped), TASK-MEMORY-302 (draft bug - the applier root cause), TASK-MEMORY-117 (done - shipped the StoreAcl enforcement the stale schema copies contradict), TASK-MEMORY-119 (done - shipped the transcript ledger whose §18.7 stamping this task completes).

## AI Authorship Disclosure

- **Tools used:** Claude (Fable 5) running the CyberOS `task-author` skill in Cursor, as the task-authoring wave of the 2026-07-23 hardening plan.
- **Scope:** the schema diff (definition-key comparison), the drift test's phantom path, the INTEROP.md absence, the allowlist contents, the live store's stray dirs (two, not the plan-era five), the writer's session silence, and the gates' doctor silence were all measured first-hand at HEAD; no BRAIN writes were performed during authoring.
- **Human review:** the hardening plan was operator-approved 2026-07-23; the package-data-forward unification direction and the repair's HITL gating are recorded decisions for the review acceptance gate.

## 1. Description (normative)

- 1.1 `modules/memory/memory.schema.json` MUST be regenerated so that the generator's `--check` passes against it and it carries the StoreAcl/StoreAclEntry/StoreAclMode definitions; the package-data copy and every vendored copy MUST be byte-identical to it. One generator, one content, N copies.
- 1.2 `test_schema_drift.py` MUST point `_COMMITTED` (and its docstring regen command) at the real committed path, and a missing committed schema MUST fail the test rather than skip - a conformance test that can skip on its trigger condition is not a conformance test.
- 1.3 A new `modules/memory/INTEROP.md` MUST exist at <= 6,000 characters covering the §14.1 consumer subset (read paths; MUST NOT write `audit/`, `HEAD`, `.lock`; canonical-writer routing for chain-touching ops; §14.4.6 STORE.yaml honor-for-writes; §14.3 sync_class semantics), and `build.sh` MUST vendor it into the payload beside the schema.
- 1.4 `_CANONICAL_TOP_LEVEL_DIRS` MUST include `sessions` and `dreams`, and `memory.invariants.yaml` + `invariants.py` MUST declare and implement `dream-applied-row-has-provenance` (§7.7.2: every dream-applied row carries extra.dream_id + extra.proposal_id), `store-yaml-acl-valid` (§14.4.7: every STORE.yaml validates against the schema's StoreAcl), and `session-lifecycle` (§18.8: start/end pairing, monotonic turn_seq, no orphan turns). Each new invariant MUST fail on a constructed violating fixture and pass on a clean store.
- 1.5 The live store's stray top-level dirs (`adrs/`, `impl-plans/` at authoring time) MUST be relocated to their canonical homes via ledger-recorded `move` operations under an explicit operator approval recorded at this task's HITL gate, following the disposition ADR per TASK-MEMORY-261 (executing 261's decision step first if it is unshipped). After the repair, `cyberos doctor` on the live store MUST report zero layout errors and `cyberos verify` MUST confirm the chain intact.
- 1.6 `run-gates.sh` MUST gain a `doctor` gate that runs `python3 -m cyberos doctor` when `.cyberos/memory/store/` exists and the module is importable, maps doctor FAIL to gate RED, and emits a provenance SKIP line when store or CLI is absent. The gate MUST NOT change behavior on repos without memory.
- 1.7 `writer.py` MUST stamp `extra.session_id` on every put/move/delete audit row while `sessions/.active` names an active session, and MUST NOT stamp when no session is active (§18.7).
- 1.8 `CHANGELOG.md` MUST record the schema unification, INTEROP.md, the walker/doctor additions, and the store repair.

## 2. Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - generator `--check` exits 0 against the root copy; root, package-data, and a scratch payload's vendored copy hash identically; the root copy contains the three StoreAcl definition keys - test: `modules/memory/tests/test_schema_single_source.py::test_all_copies_identical_and_acl_bearing`
- [ ] AC 2 (traces_to: #1.2) - `test_schema_drift.py` collects and runs (0 skips) on this repo, and monkeypatching `_COMMITTED` to a missing path makes it FAIL not skip - test: `modules/memory/tests/test_schema_single_source.py::test_drift_test_cannot_skip`
- [ ] AC 3 (traces_to: #1.3) - INTEROP.md exists, `len(read_text()) <= 6000`, contains the five mandated content anchors, and appears in a scratch payload build - test: `modules/memory/tests/test_interop_doc.py::test_interop_present_bounded_vendored`
- [ ] AC 4 (traces_to: #1.4) - a seeded store with `sessions/` + `dreams/` dirs passes layout; three constructed fixtures (dream row missing proposal_id; malformed STORE.yaml; session turn after session.end) each fail exactly their invariant; a clean store passes all three - test: `modules/memory/tests/test_walker_sessions_dreams.py::test_new_invariants_pass_and_fail_correctly`
- [ ] AC 5 (traces_to: #1.5) - on the live store post-repair: `cyberos doctor` reports zero layout errors, `cyberos verify` passes, the relocation rows are present on the chain, and the operator approval is recorded at the HITL gate (verified at review; the repair itself is demonstrated on a fixture store cloned from the live layout) - test: `modules/memory/tests/test_walker_sessions_dreams.py::test_repair_fixture_relocation_preserves_chain`
- [ ] AC 6 (traces_to: #1.6) - run-gates on a scratch repo WITH a seeded healthy store shows `PASS doctor`; with a store seeded to violate layout shows `FAIL` and RED exit; with no store shows the SKIP provenance line and unchanged exit - test: `tools/install/tests/test_doctor_gate.sh::t01_doctor_gate_three_states`
- [ ] AC 7 (traces_to: #1.7) - with an active session, a put/move/delete each carry `extra.session_id` equal to the active id; with no active session the key is absent - test: `modules/memory/tests/test_session_id_stamping.py::test_stamp_present_iff_active`
- [ ] AC 8 (traces_to: #1.8) - CHANGELOG's top entry names all four deliverable groups - test: `modules/memory/tests/test_interop_doc.py::test_changelog_records_hardening`

## 3. Edge cases

- **`_SANDBOX_FRAGMENTS` contains `/sessions/`:** the sandbox check tests the STORE'S OWN PATH, not entries inside it - adding a `sessions/` child dir does not trip it; a store legitimately installed under a path containing `/sessions/` remains rejected as before. The walker test includes this non-interference case.
- **Store with legacy v1 debris AND the new dirs:** `layout-root-canonical` keeps rejecting genuinely unknown dirs; only the two protocol-mandated names are added. The repair clause covers exactly the live store's measured strays; anything else found at repair time is surfaced to the operator, not auto-moved.
- **Doctor gate on a FROZEN store mid-repair:** until 1.5 completes, the new gate would RED this repo's own runs. Implementation order inside the task is therefore repair-before-gate-wiring, and the HITL review verifies the ordering was honored (the spec makes the order normative via this edge case).
- **`python3 -m cyberos` present but a different package:** the gate probes importability of the cyberos memory module specifically (e.g. `python3 -c "import cyberos.core"` exit 0), not merely a binary named cyberos - the name-collision lesson from TASK-IMP-130 applied to gating.
- **Session file exists but is stale (crashed session):** §18.7 stamping trusts `sessions/.active`; a stale marker means rows carry a dead session id - accepted for this task (transcript-ledger lifecycle hygiene is TASK-MEMORY-119's domain), and the stamping test documents it.
- **INTEROP.md growing past 6,000 chars later:** the bound is normative (§14.1); the test pins it so a future edit that exceeds it fails CI rather than silently violating the protocol it documents.
- **Security-class:** the doctor gate executes only the repo's own installed memory module; INTEROP.md is documentation; the repair uses ledger-recorded moves under operator approval - no new execution or exfiltration surface. Session ids in audit rows are opaque ULIDs, no PII.
