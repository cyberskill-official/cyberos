---
id: NFR-MEMORY-003
title: "memory per-tenant cursor isolation — cursor advance for tenant A never affects tenant B"
module: memory
category: security
priority: MUST
verification: T
phase: P0
slo: "Property test: 0 cross-tenant cursor influence under 1000 random advance sequences"
owner: CSO
created: 2026-05-18
related_frs: [FR-MEMORY-101, FR-MEMORY-103]
---

## §1 — Statement (BCP-14 normative)

1. The Layer-2 ingest cursor (`services/memory/src/layer2/cursor.rs`) **MUST** be scoped per (tenant_id, actor_id) — there is no global cursor that crosses tenants.
2. A cursor advance under tenant A **MUST** never cause tenant B's cursor to advance, rewind, or stall. The cursor table has a UNIQUE constraint on (tenant_id, actor_id).
3. A property-based test **MUST** drive 1000 random (tenant, advance_amount) sequences and verify no cross-tenant influence.
4. The cursor table **MUST** carry RLS policy `tenant_id = current_setting('app.tenant_id')::uuid` per NFR-AUTH-002 contract.
5. Cursor reset operations (admin tool) **MUST** be tenant-scoped — there is no "reset all tenants" command.

## §2 — Why this constraint

Per-tenant cursor isolation is the multi-tenancy guarantee for the ingest pipeline. Without it, an aggressive writer in tenant A could starve tenant B's ingest (shared cursor pointer race) or a buggy reset in tenant A could rewind tenant B's data flow. The RLS policy enforces the boundary at the DB layer; the property test verifies the application code respects the boundary. The no-global-reset rule prevents operator error from cascading across tenants.

## §3 — Measurement

- Per-tenant gauge `memory_layer2_cursor_pos{tenant_id, actor_id}`.
- Counter `memory_layer2_cursor_advance_total{tenant_id}` — should increment only for the advancing tenant.
- memory doctor invariant: no two cursors share (tenant_id, actor_id).

## §4 — Verification

- Property test `services/memory/tests/cursor_isolation_test.rs` (T) — drives 1000 random advance sequences across 50 tenants; asserts each tenant's cursor moves only on its own advances.
- RLS test (T) — attempts cross-tenant cursor read; asserts denied.

## §5 — Failure handling

- Cross-tenant influence detected in property test → sev-0 PR block.
- RLS bypass detected in prod → sev-0; halt ingest pipeline; emergency CSO + CTO call.
- UNIQUE constraint violation → sev-1; investigate data corruption.

---

*End of NFR-MEMORY-003.*
