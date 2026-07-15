# CRM module — task index

_Generated 2026-05-17 — 10 tasks, 52 engineering-hours total._

## tasks

| Task | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-CRM-001](TASK-CRM-001-account-contact-deal-schema/spec.md) | MUST | 1 | 6 | CRM Account/Contact/Deal Postgres schema — closed entity primitives + custom pipelines + closed stag |
| [TASK-CRM-002](TASK-CRM-002-activity-feed/spec.md) | MUST | 5 | 8 | CRM activity feed — auto-log inbound email + outbound send + chat mention + calendar event to per-co |
| [TASK-CRM-003](TASK-CRM-003-vn-account-type-mst/spec.md) | MUST | 5 | 4 | CRM VN account types + MST — legal entity classification (Sole/LLC/JSC/FDI) + tax ID field with form |
| [TASK-CRM-004](TASK-CRM-004-convert-to-engagement/spec.md) | MUST | 5 | 6 | CRM convert-to-engagement — deal.won → PROJ Engagement creation with rate card + billing_currency +  |
| [TASK-CRM-005](TASK-CRM-005-next-action-skill/spec.md) | MUST | 6 | 6 | CRM CUO crm.next-action@1 skill — AI-ranked top-3 next moves per open deal with rationale and deep-l |
| [TASK-CRM-006](TASK-CRM-006-ai-lead-scoring/spec.md) | SHOULD | 6 | 5 | CRM AI lead scoring — contact-creation-time score + nightly refresh based on activity signals, accou |
| [TASK-CRM-007](TASK-CRM-007-win-loss-analysis/spec.md) | SHOULD | 6 | 5 | CRM win/loss analysis CUO draft — auto-generate analysis at deal close + memory memory persistence fo |
| [TASK-CRM-008](TASK-CRM-008-mst-validate-skill/spec.md) | MUST | 7 | 3 | CRM vietnam-mst-validate skill — synchronous GDT lookup on Account write to confirm MST format + entity n |
| [TASK-CRM-009](TASK-CRM-009-vietqr-skill/spec.md) | MUST | 7 | 4 | CRM vietnam-bank-transfer skill — VietQR payment image generation for deal collection with embedded amoun |
| [TASK-CRM-010](TASK-CRM-010-hoadon-skill/spec.md) | MUST | 7 | 5 | CRM vietnam-vat-invoice skill — Decree 123 hóa đơn auto-emit on deal.stage=won + invoice issuance + verif |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-CRM-001→TASK-AUTH-003, TASK-CRM-001→TASK-AUTH-101
- **CUO**: TASK-CRM-005→TASK-CUO-101, TASK-CRM-006→TASK-CUO-101, TASK-CRM-007→TASK-CUO-101
- **EMAIL**: TASK-CRM-002→TASK-EMAIL-006
- **INV**: TASK-CRM-010→TASK-INV-007
- **PROJ**: TASK-CRM-004→TASK-PROJ-005

**This module is depended on by:**

- **EMAIL**: TASK-EMAIL-006→TASK-CRM-001
- **RES**: TASK-RES-004→TASK-CRM-001
- **TIME**: TASK-TIME-008→TASK-CRM-010

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._