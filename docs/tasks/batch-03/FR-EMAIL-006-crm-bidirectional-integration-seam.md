---
title: "EMAIL — bidirectional CRM integration seam (auto-log outgoing; surface incoming as CRM signals; CRM context in composer)"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Wire the EMAIL ↔ CRM seam so **outgoing emails to a CRM contact auto-log as a CRM activity** with thread reference + summary + sentiment, **incoming emails surface as CRM signals** in the relevant deal / account view, **the EMAIL composer surfaces CRM context** (contact role, last activity, deal stage, recent BRAIN facts) inline, and **CUO drafts deal-aware reply suggestions** that incorporate the engagement history. CRM ships in batch-05 (FR-CRM-001+); this FR ships the EMAIL side of the seam with a stub on the CRM side that batch-05 fills in. The seam respects per-Member RBAC (a Member without CRM access sees no CRM signals; a Member with CRM read-only access sees signals but cannot trigger writes); CaMeL-sanitised content only flows into CRM facts.

## Problem

Account Managers today copy-paste between Gmail and HubSpot and lose 20 minutes per day on activity logging that the platform should automate. The PRD §9.4.3 names this specifically: "outgoing emails to a CRM contact auto-log as a CRM activity; incoming emails surface as CRM signals". Without the seam, the CRM data quality issue (mostly empty CRM, current state of CyberSkill's HubSpot per the founder's note in PRD §1.1's Origin) re-emerges in CyberOS.

The PRD's Bet 1 (agent parity) requires that the seam be invocable by both humans through the UI and agents through MCP — a CUO/CRO skill (P2) drafting a deal follow-up needs the same activity-logging path that the human Account Manager uses.

## Proposed Solution

The shape of the answer is a small EMAIL-side integration with CRM-stub interfaces, a contact-resolver service that maps email addresses to CRM records, and three concrete UX surfaces.

**Contact resolver.** A new service `cyberos-email-contact-resolver` listens on `cyberos.{tenant}.email.message.{received,sent}` and resolves recipient + sender email addresses to:

- A `Member` (internal — `*@cyberskill.world`).
- A CRM `Contact` (linked to a CRM `Account` and possibly a `Deal`).
- An external unknown contact (no CRM match).

Resolution uses the CRM module's `cyberos.crm.contact_by_email` lookup (stubbed in this FR; real in batch-05 FR-CRM-002). Cache: in-memory LRU of 10K entries per replica, invalidated on CRM update events.

**Auto-log outgoing as CRM activity.** When a Member sends an email to a CRM contact:

1. Stalwart's `email.message.sent` event fires.
2. The contact-resolver matches the recipient to a CRM contact.
3. A CRM activity is created via `cyberos.crm.create_activity(contact_id, kind: "email-out", thread_id, subject_hash, body_summary, sent_by, occurred_at)`.
4. The activity is linked to the deal (if the contact has an active deal) and to the account.
5. The CRM module's contact + account + deal records show the activity in their timelines (the CRM-side rendering is in batch-05).
6. Audit row in `email.{tenant}` and `crm.{tenant}` scopes records the auto-log.

A Member can disable auto-log per-thread (one-off; the next email to the same contact resumes auto-log) or per-contact (sticky; useful for personal correspondence with a CRM-linked friend).

**Incoming emails as CRM signals.** When an inbound email arrives from a CRM contact:

1. Resolver matches sender → contact.
2. CRM signal created: `cyberos.crm.create_signal(contact_id, kind: "email-in", thread_id, classification, sentiment, extracted_facts)`.
3. The signal renders on the contact's timeline + the deal's "recent activity" widget.
4. If the email's CaMeL classification is `sales` or `support` and a deal exists, the deal's `last_inbound_at` is updated; if no active deal exists and `classification: sales`, CUO/COO surfaces a Notify "create new deal?" suggestion.
5. Sentiment trends (rolling per-contact) are surfaced on the contact's profile.

**EMAIL composer CRM-context panel.** When the composer is open with a recipient resolvable to a CRM contact, a side-panel renders:

- Contact name + role + photo.
- Linked account + deal (if any).
- Active engagement (PROJ side, surfaced via the engagement → contact link).
- Last 3 activities (email + meetings + calls + tasks).
- Top 5 BRAIN facts about the contact (e.g. "prefers vi-VN", "last raised pricing concern Q2").
- Suggested next-action chips (route through CUO/COO).

The panel is read-only inside the composer; clicking "View full profile" opens the CRM contact page.

**Deal-aware reply suggestions.** The suggested-reply path (FR-EMAIL-004) is extended: when the recipient is in an active deal, CUO/COO's suggestion tool also pulls the deal's stage + last-action + open commitments and adapts the suggestion accordingly. Examples:

- Deal stage `proposal-sent`: suggestion drafts a follow-up that asks for proposal feedback.
- Deal stage `negotiation`: suggestion is shorter, references prior pricing terms, defers to the human.
- Deal stage `closed-won`: suggestion shifts to relationship-maintenance tone.

The deal-aware suggestion is a *third* CUO suggestion alongside the two generic ones; the Member chooses.

**Schema (EMAIL side).**

```sql
CREATE TABLE email.crm_activity_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  thread_id TEXT NOT NULL,
  message_id TEXT NOT NULL,
  contact_id UUID,                         -- references crm.contact (FK enforced when CRM ships)
  deal_id UUID,
  activity_id UUID,                        -- the CRM activity row created
  direction TEXT NOT NULL,                 -- "in" | "out"
  classification TEXT,
  sentiment TEXT,
  auto_log_enabled BOOLEAN NOT NULL DEFAULT true,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE email.thread_disable_auto_log (
  thread_id TEXT NOT NULL,
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  disabled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (thread_id, tenant_id, member_id)
);

CREATE TABLE email.contact_disable_auto_log (
  recipient_email TEXT NOT NULL,
  tenant_id UUID NOT NULL,
  member_id UUID NOT NULL,
  disabled_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  PRIMARY KEY (recipient_email, tenant_id, member_id)
);
```

**RBAC.** The contact-resolver caches CRM data scoped per-Member; a Member without CRM read access sees no CRM signals or context (the panel is empty; auto-log still happens server-side but is not exposed). The auto-log itself runs under a service-context with `crm.activity.write` permission scoped to the tenant; this is the only service-level CRM write in the platform and is audit-logged with `actor_kind: 'system'`, `actor_subject: <originating member>`.

**Backfill.** When CRM is first enabled (batch-05), a backfill job walks the last 90 days of `email.message_index` and creates CRM activities + signals retroactively for resolved contacts. The backfill is run once; audit rows record the bulk creation.

**MCP tool surface (extends FR-EMAIL-001/002/004).**

- `cyberos.email.disable_auto_log_for_thread(thread_id)` — `destructive: false`.
- `cyberos.email.disable_auto_log_for_contact(recipient_email)` — `destructive: false`.
- `cyberos.email.list_unresolved_contacts(since)` — read; surfaces email addresses that frequently correspond but are not in CRM, for the Account Manager to add.

A small Notify card appears for unresolved frequent contacts ("you've emailed 10 times — add to CRM?"), routing through CUO/COO.

## Alternatives Considered

- **Skip auto-log; require manual logging.** Rejected: this is the status quo we're replacing; manual logging is the single-largest source of CRM data-quality decay.
- **Log every inbound + outbound regardless of CRM match.** Rejected: noise floods the CRM with irrelevant `external@gmail.com` activities; resolved-only is the floor.
- **Log to CRM via webhook from EMAIL only (no resolver service).** Rejected: webhook chains are hard to debug at scale; a dedicated resolver service is the architectural seam.
- **Defer to batch-05 entirely.** Rejected: the EMAIL-side hooks must be in place when CRM ships; otherwise CRM has nothing to consume.

## Success Metrics

- **Primary metric.** When CRM ships in batch-05 + this seam: ≥ 90% of outbound emails to known CRM contacts are auto-logged; ≥ 90% of inbound emails surface as CRM signals; ≥ 0 manual-log entries created by the Account Manager in a 14-day window after activation (proves the auto-log is sufficient).
- **Quality metric.** Deal-aware reply-suggestion acceptance rate ≥ 35% on the Account Manager's threads (vs. ≥ 30% baseline for generic suggestions).
- **Latency metric.** Activity-log creation occurs within 2 s of `email.message.sent` p95.

## Scope

**In-scope.**
- `cyberos-email-contact-resolver` service.
- Auto-log outgoing pipeline.
- Inbound CRM-signal pipeline.
- EMAIL composer CRM-context panel.
- Deal-aware reply-suggestion extension to CUO/COO.
- `email.crm_activity_log` + the two disable tables.
- Per-thread + per-contact disable controls.
- 90-day backfill job triggered by CRM activation.
- The four new MCP tools.
- Audit integration cross-scope (`email.{tenant}` + `crm.{tenant}`).
- CRM stub interfaces (real in batch-05 FR-CRM-002).

**Out-of-scope (deferred).**
- CRM-side rendering of activities + signals (batch-05 FR-CRM-001/002).
- Cross-account auto-link (one CRM contact corresponding from multiple email addresses) — P2.
- Automatic deal creation from inbound classification (P2; today CUO suggests, human creates).
- Attachment auto-attach to CRM (P2; today the email thread reference is the link, attachments are reachable via the thread).

## Dependencies

- FR-EMAIL-001 / FR-EMAIL-002 / FR-EMAIL-003 / FR-EMAIL-004.
- FR-INFRA-001 / FR-AUTH-001 / FR-MCP-001 / FR-AI-001.
- FR-BRAIN-001 / FR-BRAIN-002 (BRAIN facts surface in the composer panel).
- FR-CRM-001 + FR-CRM-002 (batch-05) for the real CRM side; this FR ships against stub interfaces.
- Compliance: PDPL Decree 13 (CRM data is personal data; cross-module flow needs the DPIA that FR-CP-001 covers — the EMAIL → CRM activity path is added to the CRM DPIA at batch-05).
- Locked decisions referenced: DEC-088 (auto-log enabled by default; per-thread + per-contact disable), DEC-089 (only resolver-matched contacts get logged).

## AI Risk Assessment

The deal-aware reply suggestion + the unresolved-contact suggestion are AI surfaces emitting content to humans. EU AI Act risk class: `limited`.

### Data Sources

CRM data + EMAIL data + BRAIN facts, all per-tenant. No third-party data. Suggestions go through the same persona-stamped AI Gateway as FR-EMAIL-004.

### Human Oversight

- Auto-log is configurable per Member per thread / contact.
- Suggestions are inserted into the composer; the Member edits and sends; send is destructive + step-up.
- Backfill is one-time + audit-logged.

### Failure Modes

- **Resolver mis-matches contact.** Mitigation: confidence threshold; below 0.85 the auto-log is suppressed and a Notify asks the Member to confirm the link.
- **Auto-log on a thread the Member wants private.** Mitigation: per-thread disable + per-contact disable; the disable propagates retroactively (existing activities can be unlinked by the Member with a single click).
- **Backfill creates duplicate activities.** Mitigation: dedup by `message_id` before insertion; idempotent at the EMAIL message level.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted seam architecture + CRM-context panel + failure modes.
- **Human review:** `@stephen-cheng` reviewed; CRM schema details to be re-aligned with batch-05's FR-CRM-001 at PR-review.
