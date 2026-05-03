---
title: "CRM — HubSpot migration: accounts/contacts/deals/activities + custom-field mapping + per-Member ownership preserved"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: internal_tooling
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Migrate the team's existing HubSpot CRM data into CyberOS CRM (FR-CRM-001..003) so the team can fully replace HubSpot at P1 → P2 exit. Migration covers: HubSpot Companies → CyberOS Accounts; Contacts → Contacts (with email-based dedup); Deals → Deals with stage mapping; Engagements (HubSpot's term for activities — meetings, calls, notes, emails) → CyberOS Activities; HubSpot custom fields → `metadata.imported_from.hubspot.{field_name}`; Owner mapping → CyberOS `primary_owner_member_id`. Plus: legacy-link redirect for HubSpot URLs at `link.cyberos.world/legacy/hubspot/...`; a 7-day rollback window; per-Member migration UI showing progress and errors. Outcome: the Account Manager opens CyberOS CRM on day one with the team's existing client relationships intact, including HubSpot's custom fields surfaced under metadata for legacy-query purposes.

## Problem

HubSpot today is described in PRD §1.1 as "mostly empty" — but the records that exist (the two long-term clients, prior prospects from the team's network, contact details for partners) are the substrate the Account Manager + founder rely on. Three failure modes the migration must avoid:

- **Lost relationship history.** Activities (meeting notes, prior call summaries, emails) carry the institutional memory of the relationship; migrating without them strands the relationship.
- **Custom-field decay.** HubSpot's free-form custom fields ("Acme deal — special discount approved by Stephen") are frequent; the importer must preserve them.
- **Owner reassignment confusion.** A HubSpot deal owned by the founder must surface as a CyberOS deal owned by the founder; otherwise the `/crm/my` view (FR-CRM-002) starts empty.

PRD §14.2.3 P1 → P2 exit gate ("CRM has at least 5 active client records and 10 deal records, with Genie-drafted next-actions accepted by sales rep at ≥40%") cannot be hit if the team rebuilds from scratch.

## Proposed Solution

The shape of the answer is a `cyberos-crm-import` CLI + service with a HubSpot-specific extractor implementing the same pattern as FR-PROJ-009 / FR-EMAIL-009.

**HubSpot extractor.**

Uses HubSpot CRM API v3 (`https://api.hubapi.com/crm/v3/`) via the team's existing API token (private-app token registered for migration). Supports:
- **GET `/objects/companies`** with paging → CyberOS `crm.account` rows.
- **GET `/objects/contacts`** → `crm.contact` rows; HubSpot's `firstname` + `lastname` + `email` + `phone` + `jobtitle` map directly; the `vid` (HubSpot ID) preserved in `metadata.imported_from.hubspot.vid`.
- **GET `/objects/deals`** → `crm.deal` rows; HubSpot's pipelines + stages map to CyberOS stages via a configurable mapping table.
- **GET `/objects/notes`, `/objects/calls`, `/objects/meetings`, `/objects/emails`** → `crm.activity` rows.
- **GET `/owners`** → resolved against `auth.member.email` for owner mapping.
- **GET `/properties/{type}`** → discovers custom fields per object type; their values are pulled per record and stored in `metadata.imported_from.hubspot.custom_fields.{field_name}`.

**Stage-mapping table.**

```yaml
# Default seeded; HR/Ops Lead can override per-tenant before running the import.
hubspot_to_cyberos_stage_mapping:
  appointmentscheduled: lead
  qualifiedtobuy: discovery
  presentationscheduled: discovery
  decisionmakerboughtin: proposal
  contractsent: proposal
  closedwon: closed_won
  closedlost: closed_lost
```

The mapping is applied at import; HubSpot custom stages are mapped to the closest CyberOS stage with the original name preserved in `metadata.imported_from.hubspot.original_stage`.

**Two-pass import.**

1. **First pass:** create accounts + contacts + deals; preserve HubSpot IDs in metadata.
2. **Second pass:** create activities (which reference accounts + contacts + deals); resolve cross-references via the preserved IDs.

**Owner mapping.**

The `cyberos-crm-import-matcher` service (reused from FR-PROJ-009's pattern):
1. Fetches HubSpot owners (typically email-based).
2. Matches against `auth.member.email`. Exact match → auto-assigned. Same-domain + similar-name → suggested match.
3. Unmatched owners (e.g. former employees) become "imported user" records that the HR/Ops Lead resolves before the import proceeds.

**Activity timeline preservation.**

HubSpot's activities (notes/calls/meetings/emails) carry timestamps + author + body. Migration:
- The `kind` is mapped: HubSpot `note` → `internal_note`; `meeting` → `meeting`; `call` → `call`; `email` → `email_out` or `email_in` based on direction.
- `body_md` is converted from HubSpot HTML to Markdown via `pandoc`.
- `occurred_at` preserved.
- `created_by_member_id` resolved via the owner-matcher; falls back to a synthetic "imported_user" Member if unresolved.
- Email activities also create `external_refs` pointing back to HubSpot (`{kind: "hubspot_email", id: ...}`).

**Custom-field surfacing.**

CyberOS does not natively render HubSpot custom fields, but the metadata is searchable + queryable:
- `crm.account.metadata.imported_from.hubspot.custom_fields.{field}` is indexed via `gin (metadata jsonb_path_ops)`.
- The Account 360 view (FR-CRM-002) shows a "Imported HubSpot fields" expandable panel when populated.
- A future P2 FR (FR-CRM-CUSTOM-FIELDS-001) ships first-class custom-field rendering; for P1 the metadata surface is sufficient.

**Legacy-link redirect.**

HubSpot URLs (`https://app.hubspot.com/contacts/{portal-id}/{object-type}/{vid}`) redirect via `link.cyberos.world/legacy/hubspot/{type}/{vid}` → CyberOS CRM URL. The redirect table is populated from `metadata.imported_from.hubspot.vid` lookups. Reuses the same Cloudflare Workers pattern as FR-PROJ-009 + FR-EMAIL-010.

**Migration UI (`/crm/import`).**

- Per-Member view: progress bar + counts (accounts/contacts/deals/activities) + error list.
- HR/Ops Lead aggregate view: team-wide migration status.
- Errors: parser failures (HTML-to-Markdown conversion edge cases), missing-owner mappings, custom-field type mismatches. Each error has a "retry" or "skip" action with explanation.

**Validation phase.**

- Sample 10 random Accounts; render side-by-side with HubSpot; the founder + Account Manager confirm.
- Run a synthetic query ("show all deals at proposal stage with amount > $50K") in both systems; compare.
- Verify the legacy-link redirect resolves correctly for ≥ 50 sampled URLs.

**Rollback.**

7-day rollback window: every record imported with `metadata.imported_from.hubspot.import_id == <this-import>` can be removed; legacy redirects removed; audit rows preserved. After 7 days, rollback is per-record.

**Migration → ongoing-sync (deferred).**

P1 ships migration only — a one-shot import + cutover. Ongoing two-way sync with HubSpot (live mirror) is an explicit non-goal; the cutover is clean. If a customer engagement requires HubSpot to remain authoritative for a transitional period, a separate P3 FR (FR-CRM-HUBSPOT-SYNC-001) would address it; the founder has indicated a clean cut is preferred.

**Audit + observability.**

- `crm.import.{tenant}` audit scope.
- Prometheus counters: `cyberos_crm_import_total{source, status}`, `cyberos_crm_import_duration_seconds`.
- OBS dashboard "Migration progress" panel.
- Alerts: import-failure spike, owner-match-rejection spike.

**MCP tool surface.**

- `cyberos.crm.import_status(import_id?)` (read).
- `cyberos.crm.import_start(source, credentials_ref, target_owner_overrides?)` (`destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`).
- `cyberos.crm.import_retry_failures(import_id)` (`destructive: false; idempotent: true`).
- `cyberos.crm.import_rollback(import_id)` (`destructive: true; requires_confirmation: true; sensitivity: high; step_up_required: true`).

## Alternatives Considered

- **Manual recreation in CyberOS.** Rejected: rebuilds the empty-CRM problem the founder explicitly named.
- **Live two-way sync with HubSpot.** Rejected for P1: complex to implement; the cutover-then-cancel pattern is the floor.
- **Use Salesforce Migration Tool / HubSpot's own export.** Rejected: their exports are CSV-only and lose activity-thread structure; the API extraction preserves more.
- **Skip custom fields.** Rejected: free-form custom fields carry institutional context the team relies on.
- **Skip the legacy-link redirect.** Rejected: existing references in emails / Notion docs / Slack messages bit-rot.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate: every active HubSpot account + contact + deal + activity present in CyberOS CRM with ≥ 99% completeness; legacy-link redirect resolves correctly for ≥ 95% of sampled URLs.
- **Latency / reliability.** A 1K-account / 5K-contact HubSpot tenant migrates in ≤ 4 hours; redirect lookups ≤ 100 ms p99.
- **Owner matching.** ≥ 95% auto-matched; remainder resolved by HR/Ops Lead within 2 business days.
- **Validation.** Founder + Account Manager confirm sample of 10 accounts side-by-side without discrepancies.

## Scope

**In-scope.**
- HubSpot extractor (Companies + Contacts + Deals + Notes/Calls/Meetings/Emails + Owners + Custom Properties).
- Stage-mapping table + per-tenant override.
- Two-pass import.
- Owner matcher reused from FR-PROJ-009.
- Activity-body HTML → Markdown via `pandoc`.
- Custom-field metadata preservation + GIN-indexed search.
- Legacy-link redirect at `link.cyberos.world/legacy/hubspot/...`.
- Migration UI at `/crm/import` (per-Member + HR/Ops aggregate).
- Validation playbook + 10-account sample-render comparison.
- 7-day rollback window.
- Audit + dashboards + alerts.
- The four MCP tools.

**Out-of-scope (deferred).**
- Salesforce / Pipedrive / Close / Zoho migration (P3 if customer demand).
- Live two-way sync (P3 — FR-CRM-HUBSPOT-SYNC-001).
- First-class custom-field rendering (P2 — FR-CRM-CUSTOM-FIELDS-001).
- Asana-style Asana CRM data (irrelevant; PROJ-009 handles Asana).
- Migration of HubSpot marketing data (forms, workflows, marketing emails) — out of scope; CyberOS does not have a marketing-automation surface in P1.

## Dependencies

- FR-CRM-001 / FR-CRM-002 / FR-CRM-003.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-OBS-001 / FR-OBS-002.
- FR-PROJ-009's owner-matcher infrastructure reused.
- HubSpot private-app API token registered for migration.
- Cloudflare Workers (legacy-link redirect — already deployed for FR-PROJ-009 + FR-EMAIL-010).
- `pandoc` binary for HTML→Markdown conversion.
- Compliance: PDPL Decree 13 (HubSpot data is heavily personal; the import is processing under the existing tenant ToS); SOC 2 CC8 (change management — the migration is a change of record).
- Locked decisions referenced: DEC-154 (HubSpot migration is one-shot cut, not live-sync), DEC-155 (custom fields preserved as metadata; first-class rendering deferred to P2).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. The migration is deterministic ETL; CaMeL re-ingests imported activity bodies through BRAIN where the AI risk classification already lives.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
