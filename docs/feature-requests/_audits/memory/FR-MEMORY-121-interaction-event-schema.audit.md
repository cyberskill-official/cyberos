---
fr_id: FR-MEMORY-121
audited: 2026-06-29
verdict: PASS
score: 10/10
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
eu_ai_act_review: limited — dedicated `## AI Risk Assessment` present (COND-003 closed)
---

## §1 — Verdict summary

FR-MEMORY-121 defines the single BRAIN capture primitive: one frozen interaction-event shape (`schema_version` 1, `event_id` UUIDv7, `tenant_id`, `subject_id`, `occurred_at_ns`, `module`, `event_type`, `event_class`, `target_ref`, `content_ref`, `session_id`, `trace_id`, `source_channel`, `attributes`) that every module emits as a `memory.interaction_event` aux row on the existing hash-chained `l1_audit_log`, reusing `cyberos-audit-chain` byte-for-byte so memory's reconcile + the FR-MEMORY-101 layer-2 ingest accept it unchanged. Scope: 18 §1 normative clauses (row kind + schema, frozen field set, no-raw-content rule, content_ref union, chain integration, op-from-class, emit API, consent gate, validation, size bounds, canonical determinism, consent cache, RLS, metrics, versioned contract artifact, typed builder, replay idempotency, bounded vocabulary). 11 §2 rationale paragraphs. §3 carries the migration (generated columns + partial indexes + replay-dedup unique index, no new table), the event/enum types, the emit + consent-gate Rust, the `emit_genesis_with_op` shared-crate addition, and the JSON-Schema contract excerpt. 20 ACs; §5 has 8 named test fns across 3 files; §10 lists 24 failure rows; §11 has 11 implementation notes plus a dedicated `## AI Risk Assessment`.

The four Stephen-2026-06-29 decisions are encoded: DEC-2700 (one shape, every module) → §1 #1/#2/#15/#18; DEC-2701 (platform-only, pointers/hashes not raw content) → §1 #3/#4 + content_ref union; DEC-2702 (consent-gated on the FR-EVAL-001 notice) → §1 #8 + consent_gate; DEC-2703 (aux rows on l1_audit_log, not a second store) → §1 #5 + migration; DEC-2704 (versioned, frozen contract) → §1 #2/#15. The day-1-wide-capture intent is correctly scoped to "schema + contract"; the emitters that make it real are deferred to FR-MEMORY-122, which this FR `blocks`.

## §2 — Findings (all resolved)

### ISS-001 — Second-store temptation
A unified interaction-event could have justified a fresh `interaction_event` table with its own RLS. Resolved: §1 #5 + DEC-2703 + the migration deliberately adds NO table (generated columns + partial indexes on `l1_audit_log`), so there is one system of record, one RLS surface, and the FR-MEMORY-101 reconcile/ingest accept the rows unchanged. `disallowed_tools` forbids a parallel store.

### ISS-002 — Raw content leakage into a years-retained chain
The naive shape inlines message/document/email bodies. Resolved: §1 #3/#4 mandate `content_ref` (pointer | hash | none); raw bodies stay in the owning store under its own RLS; the 2 KiB `attributes` cap + `memory_interaction_event_body_bytes` histogram (§1 #10/#13) are the anti-leak pair. AC #3/#12 enforce.

### ISS-003 — Capture before consent (covert-monitoring risk)
Without a gate in the primitive, an emitter could capture a person before the FR-EVAL-001 notice is acknowledged. Resolved: §1 #8 puts `has_acknowledged` in the one emit path; `emit` returns `Skipped{ConsentNotAcknowledged}` and writes zero rows; system actors (`subject_id=null`) are exempt. AC #8/#9/#10 cover block/pass/exempt. This is the governance-first property made structural.

### ISS-004 — Silent schema break across six dependent modules
A mutable shape breaks the first time it changes. Resolved: §1 #2/#15 freeze the field set at `schema_version: 1`, publish `interaction-event.schema.json` as the dependency surface, and state the additive-only evolution rule (new optional fields + new verbs allowed; removals/retypes require v2 + migration note). AC #17 asserts the const + that every emitted body validates.

### ISS-005 — Reads indistinguishable from mutations
Recording "opened a document" as a `put` pollutes mutation analyses. Resolved: §1 #6 derives `op` from `event_class` (`read→view`, else `put`) and adds `emit_genesis_with_op` (old `emit_genesis` becomes a `'put'` shim, so AUTH/OBS callers are untouched). AC #6 verifies a read-class event writes `op='view'`.

### ISS-006 — Best-effort vs critical-path
If emit could fail a sign-in or a message send, capture would degrade the product. Resolved: §1 #7 makes `emit` best-effort (returns `EmitError`, caller logs+swallows, exactly like AUTH `emit_token_issued` and chat `audit::emit`); AC #7 asserts no panic with the pool down.

### ISS-007 — Consent query on every interaction (perf)
A consent-ledger round-trip per event would put DB load on every keystroke-adjacent action. Resolved: §1 #12 caches the verdict per `(tenant, subject)` for ≤ 60 s with a documented revocation bound; AC #14 asserts ≤ 1 query per burst. §10 lists both stale-true and stale-false windows as by-design with the bound stated.

### ISS-008 — Replay double-counting
CDC/replay re-delivering an event would inflate a person's activity. Resolved: §1 #17 + the `l1_iev_event_id_uq` unique index make re-emit a no-op; retries reuse the same UUIDv7 `event_id`. AC #19 asserts one row after a double emit.

### ISS-009 — FR-EVAL-001 not yet shipped (forward dependency)
The consent gate reads a ledger FR-EVAL-001 owns, which does not exist yet. Resolved: §7 + §11 document the stub (operator-seeded, default capture-OFF — the safe default) that the gate resolves against until FR-EVAL-001 lands; only the stub is replaced, not the call site. The dependency is declared in `depends_on`.

## §3 — Resolution

All nine concerns addressed. **Score = 10/10.** Depth is bounded by the genuine surface — one frozen shape × chain-as-aux-row × content_ref privacy × consent gate × versioned contract × op-from-class × replay idempotency — not by line targets. Every §1 clause is cited by ≥ 1 AC; every AC maps to a §5 test fn across the three named test files; the protocol-touching pieces (new row kind, `emit_genesis_with_op`, the contract artifact) each have a §11 anchor. The `## AI Risk Assessment` closes COND-003 (limited class, capture feeds a downstream AI-assisted evaluation; disclosed-monitoring + minimisation + no-autonomous-decision documented).

The ID renumber (the prior `FR-MEMORY-121` awh-gate-result FR moved to FR-MEMORY-124, with FR-APP-005's dependency + prose and CHANGELOG.md updated) is recorded in the FR's `renumbered_from`/`renumbered_note` frontmatter and is consistent across the repo's non-generated sources.

---

*End of FR-MEMORY-121 audit.*
