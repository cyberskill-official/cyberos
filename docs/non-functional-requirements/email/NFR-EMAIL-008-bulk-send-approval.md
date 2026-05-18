---
id: NFR-EMAIL-008
title: "EMAIL bulk-send approval — sends > 100 recipients MUST require explicit approval"
module: EMAIL
category: security
priority: MUST
verification: T
phase: P0
slo: "100% of bulk sends (>100 recipients) carry a recorded approval; 0 auto-sent bulks"
owner: CTO
created: 2026-05-18
related_frs: [FR-EMAIL-010]
---

## §1 — Statement (BCP-14 normative)

1. Outbound sends with > 100 unique recipients **MUST** be paused for explicit approval by a tenant role granted `email:bulk:approve`.
2. The approval row carries `{approver_id, approved_at, recipient_count, message_hash}`; the message_hash binds approval to exact content.
3. Mutation of message content post-approval **MUST** invalidate the approval; resubmission required.
4. The approval token is single-use, short-TTL (24h max); expired approvals require resubmission.
5. Bulk sends submitted by a service account (automation) **MUST** still require an attended human approval — there is no "API key bypass" for the bulk gate.

## §2 — Why this constraint

Bulk send is a high-blast-radius operation: a wrong recipient list or hostile prompt-injected body could damage brand or expose data. The approval gate is the human-in-the-loop check. The hash-binding prevents the "approve then swap content" attack. The no-automation-bypass rule is critical — automation that exempts itself from the gate defeats the purpose.

## §3 — Measurement

- Counter `email_bulk_send_attempt_total{result=approved|paused|rejected}`.
- Counter `email_bulk_send_no_approval_attempt_total` — must be 0.
- Histogram `email_bulk_send_approval_latency_hours`.

## §4 — Verification

- Integration test (T) — submit 101-recipient send; assert paused; approve; assert sends.
- Integration test (T) — service account submission; assert still requires approval.
- Property test (T) — mutate content post-approval; assert invalidated.

## §5 — Failure handling

- No-approval attempt → block + audit + sev-3 review.
- Service-account bypass attempt → sev-2 (the configuration may have a hole).
- Expired approval used → block + resubmission.

---

*End of NFR-EMAIL-008.*
