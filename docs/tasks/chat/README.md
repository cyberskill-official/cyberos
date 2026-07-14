# CHAT module — task index

_Generated 2026-05-17 - 13 FRs, 101 engineering-hours total (TASK-CHAT-013 added 2026-06-29; TASK-CHAT-002 closed as superseded)._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [TASK-CHAT-001](TASK-CHAT-001-mattermost-fork/spec.md) | MUST | 1 | 8 | Mattermost v9.x fork at pinned MIT-Apache commit + automated license-drift watcher + CI gate |
| [TASK-CHAT-002](TASK-CHAT-002-authbridge-plugin/spec.md) | MUST | 1 | 10 | [CLOSED - superseded by TASK-CHAT-013] cyberos-chat-authbridge plugin - non-working (a Mattermost plugin cannot replace the login route) |
| [TASK-CHAT-003](TASK-CHAT-003-fargate-deployment/spec.md) | MUST | 1 | 6 | Per-tenant CHAT deployment — AWS Fargate + RDS Multi-AZ + Redis ElastiCache with Terraform module an |
| [TASK-CHAT-004](TASK-CHAT-004-vn-search/spec.md) | MUST | 1 | 12 | PGroonga + custom Vietnamese bigram tokeniser — VN message search with ≥ 80% recall CI gate and dual |
| [TASK-CHAT-005](TASK-CHAT-005-memory-bridge/spec.md) | MUST | 1 | 10 | memory bridge — Postgres logical replication from chat to memory Layer-3 ingest with p95 ≤ 5s latency |
| [TASK-CHAT-006](TASK-CHAT-006-slack-import/spec.md) | MUST | 2 | 12 | Slack import — `cyberos-chat import slack` with 8-step idempotent checkpoint-driven workflow |
| [TASK-CHAT-007](TASK-CHAT-007-zalo-import/spec.md) | SHOULD | 2 | 8 | Zalo manual export importer — `cyberos-chat import zalo --bundle.zip` with VN-Unicode normalisation  |
| [TASK-CHAT-008](TASK-CHAT-008-lumi-mention/spec.md) | MUST | 2 | 6 | @lumi mention parser — message mentions trigger CUO routing + memory capture row + reply |
| [TASK-CHAT-009](TASK-CHAT-009-retro-capture/spec.md) | SHOULD | 2 | 6 | Retro-capture flow — `@lumi remember the last N messages` with per-message opt-in checkboxes and agg |
| [TASK-CHAT-010](TASK-CHAT-010-decommission-signal/spec.md) | MUST | 2 | 5 | Decommission signal — (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95 over 14-day rolling window wit |
| [TASK-CHAT-011](TASK-CHAT-011-mobile-push/spec.md) | MUST | 2 | 6 | Mobile push delivery — APNS + FCM with privacy-preserving payload (title + sender only; no body) |
| [TASK-CHAT-012](TASK-CHAT-012-dsar-export/spec.md) | MUST | 2 | 6 | DSAR export — Data Subject Access Request: every message a subject authored + chained memory audit ha |
| [TASK-CHAT-013](TASK-CHAT-013-native-oidc-sso/spec.md) | MUST | 1 | 6 | CHAT native OIDC SSO - Mattermost federates to the TASK-AUTH-110 CyberOS OIDC provider via its native connector (replaces TASK-CHAT-002) |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: TASK-CHAT-013→TASK-AUTH-110 (the unified-path provider; TASK-CHAT-002→TASK-AUTH-004 closed as superseded)

**This module is depended on by:**

- **PORTAL**: TASK-PORTAL-006→TASK-CHAT-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._