---
id: NFR-TEN-003
title: "TEN plan-tier enforcement — over-quota operations MUST be blocked or queued"
module: TEN
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of plan-quota violations either blocked OR queued with operator-visible state"
owner: CFO
created: 2026-05-18
related_tasks: [TASK-TEN-002, TASK-TEN-004]
---

## §1 — Statement (BCP-14 normative)

1. Plan tiers (Free, Pro, Business, Enterprise) declare numeric quotas (users, projects, storage GB, API calls/month).
2. Over-quota operations **MUST** be either blocked with `E_QUOTA_EXCEEDED` or queued for the next billing cycle, per quota type declared in the plan definition.
3. Quota counters **MUST** be visible to the tenant admin via the admin SPA with current usage vs limit.
4. Quota grace periods (default 10%, plan-configurable) **MUST** be observable in the audit log.
5. Plan downgrade **MUST** refuse if current usage exceeds the lower plan's quotas; the tenant admin sees the exact blocker.

## §2 — Why this constraint

Plan enforcement is the platform's commercial guardrail. Unbounded over-quota usage breaks the business model. Blocking-vs-queuing per quota type lets the platform offer "hard caps" on resources we cap technically (storage) and "soft" overflow on resources where overflow is just measurable (API calls). Visibility to tenant admin prevents surprise blocks.

## §3 — Measurement

- Counter `ten_quota_block_total{tenant, plan, quota_kind}`.
- Gauge `ten_quota_usage_ratio{tenant, quota_kind}` — visible per-tenant.
- Counter `ten_downgrade_blocked_by_usage_total`.

## §4 — Verification

- Integration test (T) — exceed quota; assert blocked or queued per declaration.
- Property test (T) — random usage; assert quota enforcement consistent.
- Snapshot test (T) — admin SPA shows current usage.

## §5 — Failure handling

- Quota counter drift > 5% → sev-3; counter reconciliation.
- Block bypass detected → sev-2; quota gate has a hole.
- Downgrade allowed despite usage → sev-2.

---

*End of NFR-TEN-003.*
