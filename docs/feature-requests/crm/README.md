# CRM module — feature request index

_Generated 2026-05-17 — 10 FRs, 52 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-CRM-001](FR-CRM-001-account-contact-deal-schema.md) | MUST | 1 | 6 | CRM Account/Contact/Deal Postgres schema — closed entity primitives + custom pipelines + closed stag |
| [FR-CRM-002](FR-CRM-002-activity-feed.md) | MUST | 5 | 8 | CRM activity feed — auto-log inbound email + outbound send + chat mention + calendar event to per-co |
| [FR-CRM-003](FR-CRM-003-vn-account-type-mst.md) | MUST | 5 | 4 | CRM VN account types + MST — legal entity classification (Sole/LLC/JSC/FDI) + tax ID field with form |
| [FR-CRM-004](FR-CRM-004-convert-to-engagement.md) | MUST | 5 | 6 | CRM convert-to-engagement — deal.won → PROJ Engagement creation with rate card + billing_currency +  |
| [FR-CRM-005](FR-CRM-005-next-action-skill.md) | MUST | 6 | 6 | CRM CUO crm.next-action@1 skill — AI-ranked top-3 next moves per open deal with rationale and deep-l |
| [FR-CRM-006](FR-CRM-006-ai-lead-scoring.md) | SHOULD | 6 | 5 | CRM AI lead scoring — contact-creation-time score + nightly refresh based on activity signals, accou |
| [FR-CRM-007](FR-CRM-007-win-loss-analysis.md) | SHOULD | 6 | 5 | CRM win/loss analysis CUO draft — auto-generate analysis at deal close + BRAIN memory persistence fo |
| [FR-CRM-008](FR-CRM-008-mst-validate-skill.md) | MUST | 7 | 3 | CRM vietnam-mst-validate skill — synchronous GDT lookup on Account write to confirm MST format + entity n |
| [FR-CRM-009](FR-CRM-009-vietqr-skill.md) | MUST | 7 | 4 | CRM vietnam-bank-transfer skill — VietQR payment image generation for deal collection with embedded amoun |
| [FR-CRM-010](FR-CRM-010-hoadon-skill.md) | MUST | 7 | 5 | CRM vietnam-vat-invoice skill — Decree 123 hóa đơn auto-emit on deal.stage=won + invoice issuance + verif |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-CRM-001→FR-AUTH-003, FR-CRM-001→FR-AUTH-101
- **CUO**: FR-CRM-005→FR-CUO-101, FR-CRM-006→FR-CUO-101, FR-CRM-007→FR-CUO-101
- **EMAIL**: FR-CRM-002→FR-EMAIL-006
- **INV**: FR-CRM-010→FR-INV-007
- **PROJ**: FR-CRM-004→FR-PROJ-005

**This module is depended on by:**

- **EMAIL**: FR-EMAIL-006→FR-CRM-001
- **RES**: FR-RES-004→FR-CRM-001
- **TIME**: FR-TIME-008→FR-CRM-010

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._