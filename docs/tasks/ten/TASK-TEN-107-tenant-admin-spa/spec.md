---
id: TASK-TEN-107
title: "TEN tenant-admin SPA — seats + billing + audit + residency + retention dashboard for ROOT-CFO tenant administration"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: ten
priority: p1
status: draft
verify: T
phase: P2
milestone: P2 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-TEN-101, TASK-TEN-003, TASK-TEN-103, TASK-TEN-106, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-TEN-101]
blocks: []

source_pages:
  - website/docs/modules/ten.html#admin-spa

source_decisions:
  - DEC-2390 2026-05-17 — Single-page admin app for ROOT-CFO showing seats + billing (TASK-TEN-003) + audit (read-only memory view) + residency (TASK-TEN-103) + retention (TASK-TEN-106) in unified UI
  - DEC-2391 2026-05-17 — Closed enum `admin_section` = {seats, billing, audit, residency, retention, danger_zone}; cardinality 6
  - DEC-2392 2026-05-17 — Read-only for most sections; write actions (cancel sub, change residency, delete tenant) require explicit confirm dialog + sev-1 audit
  - DEC-2393 2026-05-17 — All write actions thread through existing service endpoints (TASK-TEN-003/103/106); SPA is presentation only
  - DEC-2394 2026-05-17 — memory audit kinds: ten.admin_section_viewed, ten.admin_write_action_confirmed, ten.admin_write_action_executed, ten.admin_access_denied

language: typescript / react + rust 1.81
service: cyberos/services/{ten,portal-web}/
new_files:
  - services/portal-web/src/ten/admin/AdminLayout.tsx
  - services/portal-web/src/ten/admin/SeatsSection.tsx
  - services/portal-web/src/ten/admin/BillingSection.tsx
  - services/portal-web/src/ten/admin/AuditSection.tsx
  - services/portal-web/src/ten/admin/ResidencySection.tsx
  - services/portal-web/src/ten/admin/RetentionSection.tsx
  - services/portal-web/src/ten/admin/DangerZoneSection.tsx
  - services/portal-web/src/ten/admin/ConfirmDialog.tsx
  - services/ten/src/handlers/admin_audit_routes.rs
  - services/ten/src/audit/admin_events.rs
  - services/portal-web/tests/admin-spa-sections.spec.ts
  - services/ten/tests/admin_section_enum_cardinality_test.rs
  - services/ten/tests/admin_write_confirm_required_test.rs
  - services/ten/tests/admin_audit_emission_test.rs

modified_files:
  - services/portal-web/src/app/admin/page.tsx

allowed_tools:
  - file_read: services/{ten,portal-web,auth}/**
  - file_write: services/{ten,portal-web}/{src,tests}/**
  - bash: cd services/portal-web && pnpm test; cd services/ten && cargo test admin

disallowed_tools:
  - write actions without confirm (per DEC-2392)
  - new write endpoints in TEN-107 (per DEC-2393 — read-only audit + presentation)

effort_hours: 16
subtasks:
  - "1.0h: AdminLayout.tsx"
  - "1.5h: SeatsSection.tsx"
  - "2.0h: BillingSection.tsx"
  - "1.5h: AuditSection.tsx (memory read-only view)"
  - "1.5h: ResidencySection.tsx"
  - "1.5h: RetentionSection.tsx"
  - "2.0h: DangerZoneSection.tsx"
  - "1.0h: ConfirmDialog.tsx"
  - "0.5h: handlers/admin_audit_routes.rs"
  - "0.3h: audit/admin_events.rs"
  - "2.0h: Rust + TS tests"
  - "1.2h: docs + screen captures"

risk_if_skipped: "Without admin SPA, ROOT-CFO uses scattered tools across TEN-003/103/106 — friction. Without DEC-2392 confirm, destructive actions click-through. Without DEC-2393 presentation-only, SPA duplicates business logic."
---

## §1 — Description (BCP-14 normative)

The TEN service + portal-web frontend **MUST** ship admin SPA at `services/portal-web/src/ten/admin/` with 6 sections + confirm dialog + presentation-only writes, 4 memory audit kinds.

1. **MUST** validate `admin_section` against closed enum per DEC-2391.

2. **MUST** render 6 sections per DEC-2390:
- seats: list members, manage active seats (read from TASK-TEN-101)
- billing: subscription status, invoices, payment method (TASK-TEN-003)
- audit: memory audit log filter view (read-only)
- residency: current + change residency (TASK-TEN-103)
- retention: per-module retention policies (TASK-TEN-106)
- danger_zone: cancel subscription, delete tenant (TASK-TEN-106 attestation)

3. **MUST** show ConfirmDialog per DEC-2392 for any write action — explicit type-tenant-name confirmation.

4. **MUST** delegate writes to existing service endpoints per DEC-2393:
- SeatsSection → TASK-TEN-101 endpoints
- BillingSection → TASK-TEN-003
- ResidencySection → TASK-TEN-103
- RetentionSection → TASK-TEN-106

5. **MUST** restrict SPA access to ROOT-CFO role via TASK-AUTH-101.

6. **MUST** expose backend endpoint for audit-section read:
   ```text
   GET /v1/ten/admin/audit-events?since=...&kind=...   (paginated memory read view)
   ```

7. **MUST** emit 4 memory audit kinds per DEC-2394. PII per TASK-MEMORY-111: section enum (public) ok; action-specific data hashed.

8. **MUST** thread trace_id from UI → backend → audit.

9. **MUST NOT** allow write without confirm per DEC-2392.

10. **MUST NOT** introduce new business logic per DEC-2393 (presentation only).

---

## §2 — Why this design

**Why unified SPA (DEC-2390)?** Scattered admin tools = error-prone. Single panel = single source of truth for tenant state.

**Why confirm dialog (DEC-2392)?** Destructive actions (delete tenant) need friction; type-to-confirm is industry standard.

**Why presentation-only (DEC-2393)?** Business logic in one place (the service); SPA is view. Prevents drift.

---

## §3 — API contract

```text
GET /v1/ten/admin/audit-events?since=2026-05-01&limit=50&kind=ten.subscription_cancelled
```

---

## §4 — Acceptance criteria
1. **admin_section enum cardinality 6**. 2. **6 sections rendered**. 3. **ROOT-CFO-only access**. 4. **Confirm dialog on write**. 5. **Type-to-confirm for destructive**. 6. **4 memory audit kinds emitted**. 7. **PII scrubbed (action data SHA256)**. 8. **RLS denies cross-tenant**. 9. **Trace_id preserved**. 10. **Writes delegate to existing endpoints**. 11. **No new business logic in SPA**. 12. **Audit section paginated**. 13. **section view audit on navigation**. 14. **Non-CFO access blocked + audit sev-2**. 15. **Append-only audit-events log (read-only here)**. 16. **Danger zone separated visually**. 17. **Mobile-responsive (CFO on phone)**. 18. **Loading states for async**. 19. **Error states UI**. 20. **Keyboard navigation**.

---

## §5 — Verification

```ts
test('seats section shows active members', async ({page}) => {
  await page.goto('/admin');
  await page.click('text=Seats');
  await expect(page.locator('[data-testid=seat-list]')).toBeVisible();
});

test('danger zone requires confirm', async ({page}) => {
  await page.goto('/admin/danger-zone');
  await page.click('text=Delete Tenant');
  await expect(page.locator('[data-testid=confirm-dialog]')).toBeVisible();
  await page.click('text=Confirm');  // disabled
  await expect(page.locator('text=Confirm')).toBeDisabled();
  await page.fill('[data-testid=tenant-name-input]', 'my-tenant');
  await expect(page.locator('text=Confirm')).toBeEnabled();
});
```

```rust
#[tokio::test]
async fn non_cfo_blocked() {
    let ctx = TestContext::with_am_user().await;
    let r = ctx.try_fetch_admin_audit_events_as(ctx.am_user).await;
    assert_eq!(r.status_code, 403);
}
```

---

## §7 — Dependencies
**Upstream:** TASK-TEN-101. **Cross-module:** TASK-TEN-003, TASK-TEN-103, TASK-TEN-106, TASK-AUTH-101, TASK-MEMORY-111.

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Non-CFO access | role check | 403 + sev-2 audit | inherent |
| Confirm bypass attempt | guard | reject | inherent |
| Cross-tenant view | RLS | 0 rows | inherent |
| Audit pagination N/A page | inherent | empty | inherent |
| Destructive action mid-fail | rollback | retry | inherent |
| Concurrent admin sessions | inherent | each isolated | inherent |
| memory audit query slow | indexed | tune | inherent |
| SPA crash | error boundary | reload | inherent |
| Backend timeout | UI shows error | retry | inherent |
| Mobile UI break | responsive tests | inherent | inherent |

## §11 — Implementation notes
- §11.1 SPA uses React + Tailwind; shadcn/ui components.
- §11.2 Danger zone: red border + slow-typing confirm field (anti-muscle-memory).
- §11.3 Audit section reads memory via SQL through TASK-MEMORY-108 query API; not direct table access.
- §11.4 memory audit body: section enum, action_kind; specific data SHA256.
- §11.5 Mobile responsive via Tailwind breakpoints.

---

*End of TASK-TEN-107 spec.*
