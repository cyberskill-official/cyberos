---
id: NFR-TEN-002
title: "TEN four-residency pinning — tenant data MUST stay in pinned region; 0 cross-region writes"
module: TEN
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of tenant writes land in pinned region; cross-region writes audited + blocked"
owner: CTO
created: 2026-05-18
related_frs: [FR-TEN-103]
---

## §1 — Statement (BCP-14 normative)

1. Each tenant **MUST** be pinned to exactly one of the four declared residencies (VN, SG, EU, US) at provision time; the pin is immutable post-provision.
2. All persistent writes (DB rows, blob storage, audit logs) for a tenant **MUST** land in the pinned region.
3. Cross-region writes **MUST** be blocked at the storage layer with `E_REGION_VIOLATION`.
4. Reads from non-pinned regions are permitted only for tenant-administrator-initiated migrations + are explicitly logged.
5. Region migration (rare) is a planned operation with its own saga; informal "move" is not permitted.

## §2 — Why this constraint

Data residency is a hard regulatory + commercial constraint — tenants choose a region based on their legal need; mistakenly writing tenant data to a different region violates that contract. Storage-layer enforcement is the only place where the rule can't be bypassed by application code. The migration saga ensures region moves go through a documented, auditable process.

## §3 — Measurement

- Counter `ten_cross_region_write_total{tenant, source_region, target_region}` — must be 0.
- Per-tenant audit row of every cross-region read.
- Gauge `ten_tenant_count_by_region{region}` for capacity planning.

## §4 — Verification

- Integration test (T) — write to wrong region → block.
- Chaos test (T) — coerce app to wrong region; assert storage-layer block.
- Static analysis (T) — every write goes through the residency-aware connection pool.

## §5 — Failure handling

- Cross-region write attempt → block + audit + sev-3 (app bug).
- Successful cross-region write detected (shouldn't be possible) → sev-1; halt; data-residency contract broken.
- Region migration started → operator-driven; auto-paused if any issue.

---

*End of NFR-TEN-002.*
