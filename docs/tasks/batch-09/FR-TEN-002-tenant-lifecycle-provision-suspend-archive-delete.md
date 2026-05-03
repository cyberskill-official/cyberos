---
title: "TEN — tenant lifecycle: provision, suspend, archive, delete, signed-zip export"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p3
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P3 / 2027-Q4"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Implement the **full tenant lifecycle**: **provision** (residency + plan tier + initial admin user + DPIA acceptance), **suspend** (read-only state for billing or compliance issues), **archive** (no-write state with read-only Member access for 90 days), **delete** (extending FR-CP-002's RTBE pattern from synthetic-tenant drill to production multi-tenant), **export** (signed-zip portability matching FR-BRAIN-001's pattern but tenant-wide). Every transition is **audit-heavy**: founder + DPO sign on suspend / archive / delete; founder + Engineering Lead + tenant admin three-party-sign on provision. The lifecycle is the single most-load-bearing operational pathway in P3 because it determines whether external customers can trust the platform with their data + whether they can leave with their data intact. Crypto-shred at delete (per FR-CP-002's pattern) is the irreversibility floor.

## Customer Quotes

<untrusted_content source="founder_anticipation">
"Before we trust a SaaS with our company's operating data, we ask three questions: how do we suspend it if billing breaks? how do we get our data out? how do we know the data is actually deleted when we leave?" — anticipated by Stephen from prior B2B SaaS evaluation conversations
</untrusted_content>

## Problem

PRD §14.4.1 P3 scope explicitly: "Tenant lifecycle — provision, suspend, archive, delete, export. The .zip export pattern (Part 5.3.4) is the canonical export format." Three failure modes the platform must structurally avoid:

- **Provisioning drift.** A tenant signs up; their residency is wrong; their initial DPIA isn't accepted; their first persona-version isn't signed. Without structured provisioning, every onboarding becomes a manual support ticket.
- **Suspension as ops escape valve.** A tenant whose payment fails or who triggers a compliance investigation needs a suspended-but-recoverable state. Without it, the only options are "keep running" or "delete" — neither is right for short-term issues.
- **Delete as marketing claim only.** "We delete your data on request" is meaningless without crypto-shred + audit trail + a Certificate of Erasure (FR-CP-002's pattern).

## Proposed Solution

The shape of the answer is the lifecycle state machine + per-state operational behaviour + the export pipeline + the multi-party sign chains.

**Lifecycle state machine.**

States + allowed transitions (extending FR-TEN-001's `cyberos_meta.tenant.status` enum):

```
provisioning → active                     (after DPIA accepted + first admin signed in + first persona pinned)
provisioning → cancelled                  (provisioning fails or aborted)
active       → suspended                   (billing failure, compliance issue, customer-requested pause)
suspended    → active                      (resolved; founder + DPO sign)
suspended    → archive_pending              (extended suspension > 90 days)
active       → archive_pending              (customer-requested wind-down with extended read-access)
archive_pending → archived                  (90-day archive timer elapses; no read access; tenant snapshot zip retained)
archived     → deletion_pending             (customer-requested or 7-year retention floor reached for last data class)
deletion_pending → deleted                  (FR-CP-002 RTBE flow runs; crypto-shred completed; certificate issued)
any (except deleted) → deletion_pending     (with founder + DPO + tenant-admin three-party sign + 14-day revocation window)
```

The states have specific operational behaviours:

| State | Read | Write | Billing | AI | MCP |
|---|---|---|---|---|---|
| `provisioning` | only admin user | only admin (limited surfaces) | none | disabled | disabled |
| `active` | all members | all per RBAC | metered | enabled | enabled |
| `suspended` | all members read-only | rejected | continues (until cancelled) | disabled | disabled |
| `archive_pending` | members read-only; admin can export | rejected | suspended | disabled | disabled |
| `archived` | no read | rejected | none | disabled | disabled |
| `deletion_pending` | none | none | none | none | none |
| `deleted` | nothing exists | nothing exists | nothing | nothing | nothing |
| `cancelled` | none | none | none | none | none |

Transitions are validated at the platform-level GraphQL surface; no state can be skipped (e.g. cannot go `active → deleted` directly; must pass through `archive_pending` + `archived` + `deletion_pending` for the safety windows).

**Provisioning flow.**

1. **Sign-up.** A prospective tenant administrator visits `https://cyberos.world/sign-up`; provides: legal entity name, residency choice (vn/sg/eu/us — IMMUTABLE), plan tier (T1/T2/T3), primary admin email, billing details.
2. **Identity verification.** Email confirmation + (for T2/T3) corporate-domain verification via DNS TXT record + (for T3) signed corporate paperwork uploaded to FR-DOC-001.
3. **DPIA acceptance.** The tenant admin reviews the residency-specific DPIA template (FR-CP-003's library) + signs acceptance.
4. **Initial persona-version pin.** The tenant admin reviews the platform-default CUO persona-versions for each skill + accepts (or pins specific versions).
5. **Tenant provisioned.** The platform allocates: per-tenant schema in the residency shard's Postgres; per-tenant `hr_secure` + `inv_secure` KMS keys (FR-HR-001 + FR-INV-001 patterns); per-tenant blob-store paths; per-tenant audit-chain head; per-tenant NATS subject-prefix; per-tenant CUO persona-active rows.
6. **First admin user.** Receives passkey-enrolment magic-link (FR-AUTH-001).
7. **Status transitions to `active`.** Compliance Cockpit's "tenant onboarded" event fires.

A new-tenant onboarding takes ≤ 30 minutes end-to-end for T1; longer for T2/T3 due to verification.

**Suspension flow.**

Triggers:
- **Billing failure.** 3 consecutive failed payment attempts → automatic suspension (with 7-day grace period of warning Notify cards to admin first).
- **Compliance issue.** A regulator inquiry or an internal sev-0 issue (e.g. apparent fraud or sanctioned-entity match) → manual suspension by founder + DPO.
- **Customer request.** Tenant admin explicitly requests pause via `/tenant/admin/suspend`.

On suspension:
- Status `active → suspended`.
- All write paths return `code: TENANT_SUSPENDED`.
- AI Gateway calls return `code: TENANT_SUSPENDED`.
- MCP Gateway returns `code: TENANT_SUSPENDED`.
- Members can read existing data (within 30 days; suspension > 30 days converts to archive_pending).
- Tenant admin sees a banner explaining the suspension reason + the resolution path.
- Audit row in `platform.tenant_lifecycle.{tenant}` scope.

Resolution: founder + DPO sign the resume action; status `suspended → active`.

**Archive flow.**

A tenant in `archive_pending`:
- 90-day timer; tenant admin + Members retain read access during this window for export + reference.
- After 90 days → `archived`.
- In `archived`: blob storage is retained for the residency-floor retention period (PDPL 7y for general, 10y for compensation; GDPR Article 17 limits — discussed in FR-CP-004).
- During archived: a tenant's snapshot signed-zip is retained in the residency shard's cold archive (S3 Glacier with Object Lock).

Restoration from archived (rare): the tenant admin requests via support; founder + DPO + tenant-admin three-party sign re-instantiates the tenant from the archive snapshot. Status `archived → active`. The tenant is re-billable from re-instantiation.

**Deletion flow.**

The full RTBE pattern from FR-CP-002 extended to multi-tenant production:

1. Tenant admin (or DPO acting on regulator request) submits deletion request.
2. 14-day revocation window starts.
3. Founder + DPO + tenant admin three-party sign (the tenant admin's countersign confirms intent + the founder's identity check).
4. Final export bundle generated + offered to tenant admin for download (signed zip; 30-day retention).
5. Crypto-shred: per-tenant KMS keys destroyed across all per-tenant key paths (`hr_secure/{tenant}`, `inv_secure/{tenant}`, `tenant/{tenant}`, blob-store-key/{tenant}). The keys are scheduled for destruction in HashiCorp Vault with a 7-day pending window.
6. Soft-delete: all per-tenant rows marked `deleted_at = now()` with the tenant's `tenant_id` rotated to a synthetic post-deletion UUID.
7. Audit-row pseudonymisation: per-row PII payloads are one-way-hashed (FR-AUTH-002 + FR-CP-002 pattern); the chain hashes preserve.
8. Hard-delete: after 90 days post-soft-delete, physical row removal + S3 deletion of cold archive partitions.
9. Certificate of Erasure: PDF signed by founder + DPO + (when applicable) tenant admin's auditor.
10. Status `deletion_pending → deleted`.

The crypto-shred is the architectural irreversibility. Even with full DB access, post-shred ciphertext rows are unrecoverable.

**Export flow.**

`/tenant/admin/export` (admin + DPO only; step-up):

1. Generate a tenant-wide signed-zip:
   - Per-module exports (BRAIN .cyberos-memory pattern from FR-BRAIN-001 extended to all data).
   - Per-Member identity + comp + equity + personal data (when admin is exporting, encrypted parts are sealed with the admin's public key for unsealing offline).
   - Audit chain head + verification dump.
   - Compliance evidence bundle (FR-CP-003 regulator-artefact).
2. Sign with the tenant's export key (Ed25519).
3. Upload to a one-time download URL with 7-day expiry + step-up-gated.
4. The tenant admin can verify against the platform's published export public key.

Exports can be requested at any time during `active` or `suspended` or `archive_pending`; in `archived` only the most-recent automated snapshot is available; in `deletion_pending` the final export is the irrevocable last copy.

**Schema additions (extending FR-TEN-001).**

```sql
-- Tenant lifecycle event log (separate from the canonical audit log; queryable for the lifecycle dashboard).
CREATE TABLE cyberos_meta.tenant_lifecycle_event (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  event_kind TEXT NOT NULL,                                  -- "provisioned" | "suspended" | "resumed"
                                                            -- | "archive_pending_started" | "archived"
                                                            -- | "deletion_requested" | "deletion_revoked"
                                                            -- | "deleted" | "exported" | "restored"
  reason_md TEXT NOT NULL,
  signed_by_founder_at TIMESTAMPTZ,
  signed_by_dpo_at TIMESTAMPTZ,
  signed_by_tenant_admin_at TIMESTAMPTZ,
  signed_by_engineering_lead_at TIMESTAMPTZ,
  related_export_blob_id UUID,
  related_certificate_blob_id UUID,
  occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX tenant_lifecycle_event_tenant_idx ON cyberos_meta.tenant_lifecycle_event (tenant_id, occurred_at DESC);
```

**Tenant admin UI.**

`/tenant/admin/lifecycle` shows:
- Current state + days-in-state.
- Action buttons per allowed transition.
- Lifecycle event history.
- Export action.
- Deletion request flow with 14-day revocation reminder.
- Plan tier + billing status.

**MCP tool surface.**

- `cyberos.tenant.my_lifecycle_status` — read; tenant admin.
- `cyberos.tenant.list_my_lifecycle_events` — read; tenant admin.
- `cyberos.tenant.request_export` — `destructive: false; idempotent: true`; tenant admin.
- `cyberos.tenant.request_suspension(reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`; tenant admin or platform-founder.
- `cyberos.tenant.request_deletion(reason)` — `destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true; irreversible: true`. The 14-day revocation window kicks in even after this call; the tenant admin's countersign is the final commit.

This is the second `irreversible: true` MCP tool in the platform (alongside FR-CP-002's `cyberos.cp.rtbe_request`); registration is permitted because the human-in-the-loop floors are robust + the 14-day revocation window is the ultimate fail-safe.

CUO scope contracts: read all tenant-lifecycle data allowed; mutation tools forbidden.

## Alternatives Considered

- **Skip suspension state.** Rejected: billing-failure recovery + compliance investigations need a non-destructive escape valve.
- **Allow `active → deleted` direct.** Rejected: the safety windows are the structural protection.
- **Force tenant admin to manually export before deletion.** Rejected: the platform offers a one-click final export; admin can take it or skip it.
- **No 14-day revocation window on deletion.** Rejected: the window catches accidental deletion (admin mis-clicks; admin compromised); social-engineering attack mitigation.
- **Hosted lifecycle management (Stripe billing portal etc).** Rejected: residency + compliance + the cross-module reset on lifecycle events cannot be enforced through a hosted provider.

## Sales/CS Summary

CyberOS treats your data as your data. You can suspend the platform if billing breaks, archive it if you wind down, delete it permanently with cryptographic erasure, or export everything at any time as a signed zip. Every state change is audit-grade: the founder, our Data Protection Officer, and your administrator each sign. Deletion is irreversible — we destroy the encryption keys protecting your data, so even our own engineers can't recover it. You're never locked in.

## Success Metrics

- **Primary metric.** P3 sprint demo passes: (1) a synthetic external tenant is provisioned end-to-end through the 5-step flow; (2) the tenant transitions through suspended → active → archive_pending → archived → deletion_pending → deleted with the synthetic test scripts; (3) the export bundle is generated + verified against the public key; (4) the Certificate of Erasure is produced; (5) post-shred read of the synthetic tenant's ciphertext returns "key not found".
- **Compliance metric.** Zero data recovered from a deleted-state tenant via any internal access path.
- **Latency metric.** Provisioning end-to-end ≤ 30 minutes for T1; suspension takes effect within 60 seconds of mutation.

## Scope

**In-scope.**
- The 8-state lifecycle state machine.
- Provisioning flow with DPIA-acceptance + persona-pinning.
- Suspension with grace period + auto-trigger from billing.
- Archive lifecycle with 90-day read-only window.
- Deletion flow extending FR-CP-002 to production multi-tenant.
- Export pipeline (tenant-wide signed-zip).
- Per-state operational gates (read/write/AI/MCP).
- Tenant admin UI at `/tenant/admin/lifecycle`.
- The 5 MCP tools.
- Audit integration in scope `platform.tenant_lifecycle.{tenant}`.
- Lifecycle event log with multi-party sign chain.

**Out-of-scope (deferred).**
- Restore from cold archive (P4 — operational tooling).
- Cross-shard tenant migration (P4+).
- Account merge between two related tenants (P4+ — corporate-action support).
- Self-service plan tier change (P4+ Billing).

## Dependencies

- FR-TEN-001 (multi-tenancy substrate).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-002 (synthetic-tenant RTBE pattern; this FR extends to production).
- FR-CP-003 (DPIA library + Member-self DSAR).
- FR-BRAIN-001 / FR-BRAIN-002 / FR-BRAIN-003 (tenant data scope).
- FR-OBS-001 / FR-OBS-002 / FR-OBS-003.
- FR-DOC-001 (corporate paperwork upload at provisioning for T3).
- FR-BILL-001 (billing-failure → suspension; co-shipped in batch-09).
- HashiCorp Vault per-shard for crypto-shred orchestration.
- Compliance: PDPL Decree 13 (deletion + retention); GDPR Article 17 (P3 — eu-shard); SOC 2 CC6 + CC8; ISO 27001 Annex A.16 (incident management) + A.18 (compliance with legal requirements); per-shard data-sovereignty laws.
- Locked decisions referenced: DEC-261 (8-state lifecycle), DEC-262 (14-day revocation window on deletion), DEC-263 (90-day archive window), DEC-264 (3-party sign on suspend/delete), DEC-265 (`tenant.request_deletion` is the 2nd irreversible MCP tool).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Lifecycle is deterministic operations. The provisioning flow's DPIA-acceptance step pre-supposes the tenant has reviewed AI-related risks (FR-CP-003); the persona-pinning step pre-supposes the tenant accepts the active CUO persona-versions.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
