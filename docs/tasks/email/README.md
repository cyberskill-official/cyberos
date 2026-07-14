# EMAIL module — task index

_Generated 2026-05-17 — 11 FRs, 85 engineering-hours total._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-EMAIL-001](TASK-EMAIL-001-stalwart-deployment/spec.md) | MUST | 1 | 12 | EMAIL Stalwart Rust mail server deployment — JMAP + IMAP + SMTP + ManageSieve + MTA-STS + DANE + per |
| [TASK-EMAIL-002](TASK-EMAIL-002-authbridge-plugin/spec.md) | MUST | 1 | 6 | EMAIL Stalwart authbridge plugin — JMAP/IMAP/SMTP auth delegates to AUTH JWT validation + per-tenant |
| [TASK-EMAIL-003](TASK-EMAIL-003-missive-ux/spec.md) | MUST | 2 | 16 | EMAIL Missive-style team UX — shared inbox, thread assignment, internal comments, Genie actions pane |
| [TASK-EMAIL-004](TASK-EMAIL-004-dkim-arc-bimi/spec.md) | MUST | 1 | 6 | EMAIL DKIM signing + ARC chain forward + BIMI brand indicator — RFC 6376 + RFC 8617 + BIMI 1.0 per-t |
| [TASK-EMAIL-005](TASK-EMAIL-005-camel-dual-llm/spec.md) | MUST | 2 | 12 | EMAIL CaMeL dual-LLM security layer — Privileged-LLM plans, Quarantined-LLM parses untrusted email c |
| [TASK-EMAIL-006](TASK-EMAIL-006-tracked-domain-crm-link/spec.md) | SHOULD | 1 | 5 | EMAIL tracked-domain → CRM auto-link — inbound message from tenant-tracked domain auto-creates/links |
| [TASK-EMAIL-007](TASK-EMAIL-007-convert-to-issue/spec.md) | SHOULD | 1 | 6 | EMAIL convert-to-issue — one-click create FR-PROJ issue from message with thread backlink + attachme |
| [TASK-EMAIL-008](TASK-EMAIL-008-genie-prefix/spec.md) | SHOULD | 2 | 8 | EMAIL Genie prefix — inbound subject prefix routes message to Genie (Branded AI) for automated draft |
| [TASK-EMAIL-009](TASK-EMAIL-009-outbound-1to1-send/spec.md) | MUST | 1 | 4 | EMAIL outbound 1:1 send — DKIM-signed via TASK-EMAIL-004 + AM confirm-before-send + queue + bounce han |
| [TASK-EMAIL-010](TASK-EMAIL-010-bulk-send-approval/spec.md) | MUST | 1 | 5 | EMAIL bulk send (≥ 10 recipients) — AM + CFO/marketing dual-approval token + suppression-list filter |
| [TASK-EMAIL-011](TASK-EMAIL-011-dsar-message-export/spec.md) | MUST | 2 | 5 | EMAIL DSAR message export — every message a subject authored or received + chained memory audit hashe |

## Cross-module dependencies

**This module depends on:**

- **AI**: TASK-EMAIL-005→TASK-AI-003
- **AUTH**: TASK-EMAIL-002→TASK-AUTH-004
- **CRM**: TASK-EMAIL-006→TASK-CRM-001
- **CUO**: TASK-EMAIL-008→TASK-CUO-101
- **PORTAL**: TASK-EMAIL-008→TASK-PORTAL-005
- **PROJ**: TASK-EMAIL-007→TASK-PROJ-001

**This module is depended on by:**

- **CRM**: TASK-CRM-002→TASK-EMAIL-006
- **INV**: TASK-INV-010→TASK-EMAIL-009

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._