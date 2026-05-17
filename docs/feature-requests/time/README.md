# TIME module — feature request index

_Generated 2026-05-17 — 9 FRs, 51 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-TIME-001](FR-TIME-001-time-entry-schema.md) | MUST | 1 | 5 | TIME TimeEntry append-only schema — correction_to link semantics + tenant-scoped RLS + invoice-grade |
| [FR-TIME-002](FR-TIME-002-timer-start-stop.md) | MUST | 1 | 5 | TIME timer start/stop — single-active-timer per Member + auto-stop on logout + ≤15-min resolution sn |
| [FR-TIME-003](FR-TIME-003-manual-entry-form.md) | MUST | 1 | 6 | TIME manual entry form — retroactive time logging with date validation + per-day total cap + FR-TIME |
| [FR-TIME-004](FR-TIME-004-auto-detect-proposals.md) | SHOULD | 2 | 6 | TIME auto-detect proposals — Member-confirm suggestions from PROJ activity (status changes + comment |
| [FR-TIME-005](FR-TIME-005-billable-flag-cascade.md) | MUST | 1 | 5 | TIME billable flag cascade — 4-step resolver (entry override → project default → engagement policy → |
| [FR-TIME-006](FR-TIME-006-weekly-approval-flow.md) | MUST | 1 | 6 | TIME weekly approval flow — Member submit → AM (engagement_admin) review → CFO visibility with auto- |
| [FR-TIME-007](FR-TIME-007-vn-labour-code-ot-cap.md) | MUST | 1 | 4 | TIME VN Labour Code Art. 107 OT cap — hard-block at entry write when monthly OT > 40h or yearly OT > |
| [FR-TIME-008](FR-TIME-008-expense-capture-ocr.md) | MUST | 2 | 8 | TIME expense capture — photo → AWS Textract OCR → hóa đơn parser → Member confirm + categorisation + |
| [FR-TIME-009](FR-TIME-009-per-cycle-rollup.md) | MUST | 1 | 6 | TIME per-cycle billable rollup → INV — per-Member × role × Engagement aggregation with rate-card app |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-TIME-001→FR-AUTH-003, FR-TIME-001→FR-AUTH-101
- **CRM**: FR-TIME-008→FR-CRM-010
- **PROJ**: FR-TIME-004→FR-PROJ-002, FR-TIME-005→FR-PROJ-006

**This module is depended on by:**

- **HR**: FR-HR-008→FR-TIME-001
- **INV**: FR-INV-001→FR-TIME-009
- **LEARN**: FR-LEARN-003→FR-TIME-001
- **RES**: FR-RES-001→FR-TIME-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._