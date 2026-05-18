---
id: NFR-CRM-008
title: "CRM conversion-to-engagement integrity — closed deal MUST link to a PROJ engagement"
module: CRM
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of won deals create or link to a PROJ engagement within 24h"
owner: COO
created: 2026-05-18
related_frs: [FR-CRM-004]
---

## §1 — Statement (BCP-14 normative)

1. When a deal transitions to `won`, a PROJ engagement **MUST** be created or linked within 24h.
2. The engagement record references the source deal via `source_deal_id`; the deal references the engagement via `engagement_id`.
3. The engagement carries inherited fields from the deal: scope summary, account, value, billing terms.
4. Manual creation override is allowed but requires explicit reason logged.
5. Orphan won deals (no linked engagement past 24h) trigger sev-3 visibility for COO.

## §2 — Why this constraint

The deal→engagement transition is the platform's revenue-to-delivery handoff. Without automatic linkage, won deals can go un-delivered (or unbilled). The bidirectional ref makes the relationship discoverable from either side. Manual override is the escape hatch for unusual cases.

## §3 — Measurement

- Counter `crm_won_deal_engagement_linked_total`.
- Gauge `crm_orphan_won_deal_count_past_24h`.
- Histogram `crm_deal_to_engagement_latency_hours`.

## §4 — Verification

- Integration test (T) — close deal; assert engagement created/linked.
- Snapshot test (T) — inherited fields match.
- Orphan-detection test (T).

## §5 — Failure handling

- Orphan > 24h → sev-3; COO notified.
- Linkage mismatch (deal→engagement→different deal) → sev-2.
- Manual override missing reason → reject.

---

*End of NFR-CRM-008.*
