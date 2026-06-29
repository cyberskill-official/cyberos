# CHAT module — feature request index

_Generated 2026-05-17 - 13 FRs, 101 engineering-hours total (FR-CHAT-013 added 2026-06-29; FR-CHAT-002 closed as superseded)._

## FRs

| FR | Priority | Slice | Hours | Title |
|---|---|---|---:|---|
| [FR-CHAT-001](FR-CHAT-001-mattermost-fork.md) | MUST | 1 | 8 | Mattermost v9.x fork at pinned MIT-Apache commit + automated license-drift watcher + CI gate |
| [FR-CHAT-002](FR-CHAT-002-authbridge-plugin.md) | MUST | 1 | 10 | [CLOSED - superseded by FR-CHAT-013] cyberos-chat-authbridge plugin - non-working (a Mattermost plugin cannot replace the login route) |
| [FR-CHAT-003](FR-CHAT-003-fargate-deployment.md) | MUST | 1 | 6 | Per-tenant CHAT deployment — AWS Fargate + RDS Multi-AZ + Redis ElastiCache with Terraform module an |
| [FR-CHAT-004](FR-CHAT-004-vn-search.md) | MUST | 1 | 12 | PGroonga + custom Vietnamese bigram tokeniser — VN message search with ≥ 80% recall CI gate and dual |
| [FR-CHAT-005](FR-CHAT-005-memory-bridge.md) | MUST | 1 | 10 | memory bridge — Postgres logical replication from chat to memory Layer-3 ingest with p95 ≤ 5s latency |
| [FR-CHAT-006](FR-CHAT-006-slack-import.md) | MUST | 2 | 12 | Slack import — `cyberos-chat import slack` with 8-step idempotent checkpoint-driven workflow |
| [FR-CHAT-007](FR-CHAT-007-zalo-import.md) | SHOULD | 2 | 8 | Zalo manual export importer — `cyberos-chat import zalo --bundle.zip` with VN-Unicode normalisation  |
| [FR-CHAT-008](FR-CHAT-008-lumi-mention.md) | MUST | 2 | 6 | @lumi mention parser — message mentions trigger CUO routing + memory capture row + reply |
| [FR-CHAT-009](FR-CHAT-009-retro-capture.md) | SHOULD | 2 | 6 | Retro-capture flow — `@lumi remember the last N messages` with per-message opt-in checkboxes and agg |
| [FR-CHAT-010](FR-CHAT-010-decommission-signal.md) | MUST | 2 | 5 | Decommission signal — (chat msgs) / (chat + slack + zalo msgs) ≥ 0.95 over 14-day rolling window wit |
| [FR-CHAT-011](FR-CHAT-011-mobile-push.md) | MUST | 2 | 6 | Mobile push delivery — APNS + FCM with privacy-preserving payload (title + sender only; no body) |
| [FR-CHAT-012](FR-CHAT-012-dsar-export.md) | MUST | 2 | 6 | DSAR export — Data Subject Access Request: every message a subject authored + chained memory audit ha |
| [FR-CHAT-013](FR-CHAT-013-native-oidc-sso.md) | MUST | 1 | 6 | CHAT native OIDC SSO - Mattermost federates to the FR-AUTH-110 CyberOS OIDC provider via its native connector (replaces FR-CHAT-002) |

## Cross-module dependencies

**This module depends on:**

- **AUTH**: FR-CHAT-013→FR-AUTH-110 (the unified-path provider; FR-CHAT-002→FR-AUTH-004 closed as superseded)

**This module is depended on by:**

- **PORTAL**: FR-PORTAL-006→FR-CHAT-005

---

_See `IMPLEMENTATION_ORDER.md` for the full topological build sequence._