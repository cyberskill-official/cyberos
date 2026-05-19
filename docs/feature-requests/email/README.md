# EMAIL module — feature request index

_Generated 2026-05-17 — 11 FRs, 85 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-EMAIL-001](FR-EMAIL-001-stalwart-deployment.md) | MUST | 1 | 12 | EMAIL Stalwart Rust mail server deployment — JMAP + IMAP + SMTP + ManageSieve + MTA-STS + DANE + per |
| [FR-EMAIL-002](FR-EMAIL-002-authbridge-plugin.md) | MUST | 1 | 6 | EMAIL Stalwart authbridge plugin — JMAP/IMAP/SMTP auth delegates to AUTH JWT validation + per-tenant |
| [FR-EMAIL-003](FR-EMAIL-003-missive-ux.md) | MUST | 2 | 16 | EMAIL Missive-style team UX — shared inbox, thread assignment, internal comments, Genie actions pane |
| [FR-EMAIL-004](FR-EMAIL-004-dkim-arc-bimi.md) | MUST | 1 | 6 | EMAIL DKIM signing + ARC chain forward + BIMI brand indicator — RFC 6376 + RFC 8617 + BIMI 1.0 per-t |
| [FR-EMAIL-005](FR-EMAIL-005-camel-dual-llm.md) | MUST | 2 | 12 | EMAIL CaMeL dual-LLM security layer — Privileged-LLM plans, Quarantined-LLM parses untrusted email c |
| [FR-EMAIL-006](FR-EMAIL-006-tracked-domain-crm-link.md) | SHOULD | 1 | 5 | EMAIL tracked-domain → CRM auto-link — inbound message from tenant-tracked domain auto-creates/links |
| [FR-EMAIL-007](FR-EMAIL-007-convert-to-issue.md) | SHOULD | 1 | 6 | EMAIL convert-to-issue — one-click create FR-PROJ issue from message with thread backlink + attachme |
| [FR-EMAIL-008](FR-EMAIL-008-genie-prefix.md) | SHOULD | 2 | 8 | EMAIL Genie prefix — inbound subject prefix routes message to Genie (Branded AI) for automated draft |
| [FR-EMAIL-009](FR-EMAIL-009-outbound-1to1-send.md) | MUST | 1 | 4 | EMAIL outbound 1:1 send — DKIM-signed via FR-EMAIL-004 + AM confirm-before-send + queue + bounce han |
| [FR-EMAIL-010](FR-EMAIL-010-bulk-send-approval.md) | MUST | 1 | 5 | EMAIL bulk send (≥ 10 recipients) — AM + CFO/marketing dual-approval token + suppression-list filter |
| [FR-EMAIL-011](FR-EMAIL-011-dsar-message-export.md) | MUST | 2 | 5 | EMAIL DSAR message export — every message a subject authored or received + chained memory audit hashe |

## Cross-module dependencies

**This module depends on:**

- **AI**: FR-EMAIL-005→FR-AI-003
- **AUTH**: FR-EMAIL-002→FR-AUTH-004
- **CRM**: FR-EMAIL-006→FR-CRM-001
- **CUO**: FR-EMAIL-008→FR-CUO-101
- **PORTAL**: FR-EMAIL-008→FR-PORTAL-005
- **PROJ**: FR-EMAIL-007→FR-PROJ-001

**This module is depended on by:**

- **CRM**: FR-CRM-002→FR-EMAIL-006
- **INV**: FR-INV-010→FR-EMAIL-009

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._