---
title: "EMAIL — deliverability operations: warm-up plan, FBL processing, reputation monitoring, suppression-list management, content-rate guard"
author: "@stephen-cheng"
department: operations
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P1 / 2026-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Operationalise outbound email deliverability beyond the protocol setup in FR-EMAIL-001: a **warm-up plan** for new sending domains and IPs (graduated send-volume curve over 30 days); **Feedback Loop (FBL) processing** for the major mailbox providers (Microsoft, Yahoo, AOL, mail.ru, Comcast); **reputation monitoring** via Google Postmaster Tools and Microsoft SNDS APIs with red/yellow/green status into the OBS Compliance Cockpit; **suppression-list management** with hard-bounce auto-add, soft-bounce escalation, and one-click manual override; **content-rate guards** that detect and throttle outbound spikes that look like spam; and a **deliverability runbook library** that lets the on-call diagnose and remediate reputation degradation in ≤ 4 hours. Without this FR, Stalwart's protocol coverage is theoretical; with it, the team can replace Gmail without watching their domain's reputation collapse.

## Problem

The PRD §4.1 G6 (commercial readiness) and the P1 → P2 exit gate (PRD §14.2.3 — "EMAIL has fully replaced Gmail for at least 21 consecutive days") both depend on outbound deliverability being correct in production, not in theory. Three real-world failure modes a small team must avoid:

- **Cold-start sending.** A brand-new sending domain (e.g. `cyberos.world` for the platform's mail) sending 200 messages on day one to varied recipients triggers spam classifiers; the domain's reputation lands in the spam folder for a quarter. The recovery cost is months.
- **Bounce blindness.** Hard bounces (the mailbox doesn't exist) keep retrying; ISPs rate-limit; eventually the IP reputation degrades. Without auto-suppression, every team member's sent folder accumulates these silently.
- **Reputation degradation without alerting.** The first sign of trouble is a customer asking why an email "didn't arrive" — by then, recovery is days. Postmaster Tools + SNDS surface the trouble earlier; without them being polled and alerted on, the signal is wasted.

## Proposed Solution

The shape of the answer is a `cyberos-email-ops` service that owns warm-up scheduling + FBL ingestion + reputation polling + suppression management, plus runbook content + dashboards.

**Warm-up plan (new domains / IPs).**

For a new sending domain (and per-IP if Stalwart's egress IPs change), the warm-up curve over 30 days:

| Day | Max msgs/day | Notes |
|---|---|---|
| 1 | 30 | only to known-good recipients (seed list) |
| 2 | 50 | |
| 3 | 75 | |
| 5 | 125 | |
| 7 | 200 | broaden to allowed recipient list |
| 10 | 400 | |
| 14 | 700 | |
| 21 | 1,200 | full warm |
| 28 | 2,000 | full production |

The warm-up scheduler enforces the daily cap; messages over the cap are queued and released the next UTC day. The schedule is configurable per-tenant and per-domain; the default is the table above. During warm-up, the recipient list is gated against a "warm-up allowlist" managed by HR/Ops Lead.

**FBL (Feedback Loop) processing.**

Major mailbox providers offer FBL programs that send abuse complaints back as ARF-formatted reports to a registered address. CyberOS:

- Registers `fbl@cyberos.world` (and `fbl@{tenant-domain}` per tenant from P3) with: Microsoft (SNDS), Yahoo (Complaint Feedback Loop), AOL, mail.ru, Comcast, plus any additional providers added quarterly.
- A dedicated Stalwart mailbox catches FBL reports.
- A `cyberos-email-fbl-processor` service parses ARF reports, extracts the original-message reference, marks the recipient as "complained" in the suppression list, surfaces a sev-2 alert if the complaint rate exceeds 0.1% over a 24-hour window (Microsoft's industry threshold), and writes audit rows in `email.fbl.{tenant}` scope.

**Reputation monitoring.**

- **Google Postmaster Tools.** Polled daily via the Postmaster Tools API: domain reputation (low/medium/high), IP reputation, spam rate, IPv6 vs. IPv4 split, encryption rate, DKIM/SPF/DMARC pass rate, delivery errors. Surfaced in OBS dashboard "Email reputation" with a 90-day trend.
- **Microsoft SNDS.** Polled daily for IP-level "filter result" classifications.
- **Mail-Tester.com / GlockApps** integrations for one-shot deliverability scoring (used during incident response or before a campaign-style send, not continuous).
- **Bounce-rate dashboard.** Per-domain per-7-days hard-bounce rate, soft-bounce rate, deferred rate.

Thresholds:
- Domain reputation `medium` → sev-2 alert (Genie panel).
- Domain reputation `low` → sev-1 alert (CHAT `#ops-alerts`); pause non-critical outbound; runbook engaged.
- Spam rate > 0.3% → sev-1 alert (Microsoft's red-flag threshold).
- Hard-bounce rate > 5% in any 24-hour window → sev-1 alert.

**Suppression-list management.**

Schema:

```sql
CREATE TABLE email.suppression (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  recipient_email TEXT NOT NULL,
  reason TEXT NOT NULL,                   -- "hard_bounce" | "soft_bounce_persistent" | "fbl_complaint"
                                          -- | "manual" | "list_unsubscribe" | "spamtrap"
  reason_detail TEXT,
  suppressed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  suppressed_until TIMESTAMPTZ,           -- null = permanent; soft-bounce-persistent = 30 days
  released_at TIMESTAMPTZ,
  released_by UUID,
  release_reason TEXT,
  UNIQUE (tenant_id, recipient_email, reason, suppressed_at)
);
```

Rules:
- Hard bounce → permanent suppression; manual override via `email.suppression.release` MCP tool.
- Soft bounce 5+ in 14 days → 30-day suppression; auto-release after.
- FBL complaint → permanent + flagged for HR/Ops review.
- `List-Unsubscribe` header (RFC 8058) processed automatically on inbound unsubscribe-click; permanent + per-list scope.
- Spamtrap detected (an email address known to be a trap) → permanent + sev-1 alert.

The send pipeline checks the suppression list pre-send; suppressed recipients return `code: SUPPRESSED` to the Member with an explanation chip.

**Content-rate guards.**

Outbound spike detection per Member per hour:
- Threshold: > 3× the Member's rolling 7-day average; > 100 messages in 1 hour.
- Above threshold → soft block; CUO Notify card asks the Member to confirm.
- Above 2× threshold → hard block until the founder unblocks; pages the on-call.

Spike detection prevents a compromised account or a misfired automation from sending 5,000 messages in an hour and torching reputation.

**Deliverability runbook library.**

Runbooks at `obs/runbooks/email-*.md`:
- `email-domain-reputation-low.md`
- `email-spam-rate-spike.md`
- `email-bounce-rate-spike.md`
- `email-fbl-complaint-spike.md`
- `email-blocklist-listed.md` (Spamhaus, Barracuda, etc.)
- `email-dkim-failure-cluster.md`
- `email-mta-sts-policy-error.md`

Each runbook has the standard FR-OBS-002 §"Runbook library" structure (TL;DR, diagnostics, common causes, mitigation, rollback, escalation).

**Blocklist monitoring.**

Daily check against major DNSBLs (Spamhaus SBL/CSS/PBL, Barracuda, Sorbs, UCEPROTECT, SpamCop). Listing on any major list → sev-1 alert + runbook + delisting workflow (most major lists offer self-service delisting once the underlying issue is fixed).

**MCP tool surface.**

- `cyberos.email.deliverability_status` (read).
- `cyberos.email.list_suppressed_recipients(reason?)` (read).
- `cyberos.email.release_suppression(recipient_email, reason)` (`destructive: true; requires_confirmation: true`).
- `cyberos.email.warmup_status(domain?)` (read).
- `cyberos.email.list_blocklist_listings` (read).

## Alternatives Considered

- **Skip warm-up; send full volume from day one.** Rejected: this is exactly how reputation collapses.
- **Outsource deliverability to a transactional email provider (Postmark, Mailgun) for outbound.** Considered for "transactional" outbound (password resets, system notifications) — accepted at P3+ as a hybrid pattern; in P1 we own the full stack to learn the operational shape and stay residency-clean. Tracked as OQ-EMAIL-TRANSACTIONAL-PROVIDER.
- **Manual suppression-list management via spreadsheet.** Rejected: the failure mode is exactly the silent-suppression-decay we're avoiding.
- **No content-rate guards; trust Members to not misuse.** Rejected: a compromised credential is the actual threat model, not Member misbehaviour.

## Success Metrics

- **Primary metric.** P1 → P2 exit-gate: domain reputation `high` at Google Postmaster Tools and Microsoft SNDS for 14 consecutive days; hard-bounce rate ≤ 2% on a 30-day window; spam-folder-rate ≤ 0.3%.
- **Operational metric.** Time-to-detection (TTD) on a reputation regression ≤ 24 hours (the polling cadence is daily; a degradation surfaces in the next poll cycle).
- **Suppression metric.** Hard-bounce auto-suppression catches ≥ 99% of permanent failures within one bounce.

## Scope

**In-scope.**
- `cyberos-email-ops` service.
- 30-day warm-up scheduler with the per-day caps.
- FBL registration with the 5+ major providers.
- FBL ARF parser + complaint handler.
- Postmaster Tools + SNDS daily polling + dashboards.
- Suppression-list schema + auto-add rules + manual release.
- Content-rate guards per Member per hour.
- Daily DNSBL check.
- 7+ runbooks in the library.
- 5 MCP tools.
- OBS dashboard "Email reputation" + alerts.

**Out-of-scope (deferred).**
- Hybrid transactional-provider integration (P3 — OQ-EMAIL-TRANSACTIONAL-PROVIDER).
- Automated delisting submission (mostly manual at the major lists; we monitor + alert + provide the runbook).
- Per-recipient reputation caching (P2 — we react to provider-level signals only in P1).
- Email security training for Members (P2; integrates with LEARN module).

## Dependencies

- FR-EMAIL-001 (Stalwart core).
- FR-INFRA-001 / FR-AUTH-001 / FR-MCP-001 / FR-OBS-001 / FR-OBS-002.
- Google Postmaster Tools account + verified domain.
- Microsoft SNDS account + IPs registered.
- DNSBL APIs (Spamhaus DQS, Barracuda).
- Compliance: PDPL Decree 13 (suppression list contains personal data; per-tenant scope + 7-year retention floor); SOC 2 CC7.
- Locked decisions referenced: DEC-092 (warm-up curve floors), DEC-093 (auto-suppression on hard bounce + FBL).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. Deliverability operations are deterministic; spike detection is rule-based.
