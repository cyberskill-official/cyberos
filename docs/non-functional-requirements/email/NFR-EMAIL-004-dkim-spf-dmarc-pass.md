---
id: NFR-EMAIL-004
title: "EMAIL DKIM/SPF/DMARC pass-through — outbound mail MUST be DMARC-aligned"
module: EMAIL
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of outbound emails pass DKIM + SPF + DMARC alignment for the tenant's sending domain"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-EMAIL-004, TASK-EMAIL-009]
---

## §1 — Statement (BCP-14 normative)

1. Every outbound email **MUST** carry a valid DKIM signature aligned to the tenant's sending domain; the platform refuses to send unsigned mail.
2. The tenant's SPF record **MUST** include the platform's sending IPs/relay; provisioning checks this and refuses send-domain activation otherwise.
3. The DMARC alignment **MUST** be strict — `From:` header domain matches DKIM `d=` and SPF Envelope-From.
4. Outbound validation (synthetic mail-to-self) **MUST** run hourly and report DMARC-pass rate; sustained < 100% triggers sev-2.
5. ARC seal (`TASK-EMAIL-004`) **MUST** be applied for forwarded mail; ARC failures are diagnosed via the OBS dashboard.

## §2 — Why this constraint

DMARC-misaligned mail lands in spam folders, gets bounced by strict receivers, and erodes the tenant's sender reputation. The platform's role is to make alignment automatic — the tenant configures DNS once, the platform signs/aligns every mail. The hourly synthetic check is the trip-wire on silent breakage (e.g., DKIM key rotation gone wrong).

## §3 — Measurement

- Hourly synthetic mail-to-self DMARC pass rate per tenant.
- Counter `email_outbound_dmarc_fail_total{tenant, reason=dkim|spf|alignment}` — should be 0.
- Gauge `email_dkim_key_age_days{tenant, selector}` — key rotation hygiene.

## §4 — Verification

- Integration test (T) — outbound mail + verifier-recipient; assert DMARC pass.
- Hourly synthetic (A) — production probe.
- Per-tenant DNS check at provisioning + monthly re-check.

## §5 — Failure handling

- Synthetic fail → sev-2; investigate DKIM keys + DNS.
- Per-tenant fail rate > 1% → sev-3; tenant DNS may have drifted; notify tenant admin.
- DKIM key age > 90 days → SHOULD rotate.

---

*End of NFR-EMAIL-004.*
