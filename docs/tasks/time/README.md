# TIME module — task index

_Generated 2026-05-17 — 9 FRs, 51 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-TIME-001](TASK-TIME-001-time-entry-schema/spec.md) | MUST | 1 | 5 | TIME TimeEntry append-only schema — correction_to link semantics + tenant-scoped RLS + invoice-grade |
| [TASK-TIME-002](TASK-TIME-002-timer-start-stop/spec.md) | MUST | 1 | 5 | TIME timer start/stop — single-active-timer per Member + auto-stop on logout + ≤15-min resolution sn |
| [TASK-TIME-003](TASK-TIME-003-manual-entry-form/spec.md) | MUST | 1 | 6 | TIME manual entry form — retroactive time logging with date validation + per-day total cap + FR-TIME |
| [TASK-TIME-004](TASK-TIME-004-auto-detect-proposals/spec.md) | SHOULD | 2 | 6 | TIME auto-detect proposals — Member-confirm suggestions from PROJ activity (status changes + comment |
| [TASK-TIME-005](TASK-TIME-005-billable-flag-cascade/spec.md) | MUST | 1 | 5 | TIME billable flag cascade — 4-step resolver (entry override → project default → engagement policy → |
| [TASK-TIME-006](TASK-TIME-006-weekly-approval-flow/spec.md) | MUST | 1 | 6 | TIME weekly approval flow — Member submit → AM (engagement_admin) review → CFO visibility with auto- |
| [TASK-TIME-007](TASK-TIME-007-vn-labour-code-ot-cap/spec.md) | MUST | 1 | 4 | TIME VN Labour Code Art. 107 OT cap — hard-block at entry write when monthly OT > 40h or yearly OT > |
| [TASK-TIME-008](TASK-TIME-008-expense-capture-ocr/spec.md) | MUST | 2 | 8 | TIME expense capture — photo → AWS Textract OCR → hóa đơn parser → Member confirm + categorisation + |
| [TASK-TIME-009](TASK-TIME-009-per-cycle-rollup/spec.md) | MUST | 1 | 6 | TIME per-cycle billable rollup → INV — per-Member × role × Engagement aggregation with rate-card app |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-TIME-001→TASK-AUTH-003, TASK-TIME-001→TASK-AUTH-101
- **CRM**: TASK-TIME-008→TASK-CRM-010
- **PROJ**: TASK-TIME-004→TASK-PROJ-002, TASK-TIME-005→TASK-PROJ-006

**This module is depended on by:**

- **HR**: TASK-HR-008→TASK-TIME-001
- **INV**: TASK-INV-001→TASK-TIME-009
- **LEARN**: TASK-LEARN-003→TASK-TIME-001
- **RES**: TASK-RES-001→TASK-TIME-001

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._