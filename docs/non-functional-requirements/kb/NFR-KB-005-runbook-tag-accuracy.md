---
id: NFR-KB-005
title: "KB runbook-tag accuracy — runbook docs MUST be findable by incident-keyword + tag combo"
module: KB
category: usability
priority: SHOULD
verification: T
phase: P1
slo: "100% of tagged runbooks surface in top-3 for canonical incident-keyword queries"
owner: CTO
created: 2026-05-18
related_frs: [FR-KB-008]
---

## §1 — Statement (BCP-14 normative)

1. Documents tagged `kind:runbook` **MUST** be retrievable via the API `?tag=runbook&query=<incident-keyword>` and rank in the top-3 results for the keyword.
2. Runbook tag set **MUST** be a closed enum maintained at `modules/kb/runbook-tags.yaml`; ad-hoc strings are not accepted.
3. Each runbook **MUST** carry `{severity, system_affected, last_executed_at, last_owner}` metadata.
4. Stale runbooks (no execution recorded in 90+ days) **MUST** be flagged in the UI for review.
5. On incident creation in OBS, the platform **MUST** auto-link suggested runbooks based on tag + keyword match.

## §2 — Why this constraint

Runbooks live or die by findability under incident pressure. The top-3 floor is the cognitive ceiling — operators don't scroll past it during a sev-1. The closed-enum tag set prevents tag-sprawl that fragments the index. The stale flag turns runbook-rot into an actionable signal. Auto-link on incident is the time-saver during the highest-stress moments.

## §3 — Measurement

- Per-quarter benchmark: top-3 accuracy on canonical incident queries.
- Counter `kb_runbook_stale_count` — must trend toward 0.
- Counter `kb_runbook_auto_link_clicked_total` — surfaces usefulness.

## §4 — Verification

- Integration test (T) — fixture runbooks + queries; assert top-3.
- CI gate (T) — tag enum compliance for all runbooks.
- Quarterly review — sample 10 runbooks; assess freshness.

## §5 — Failure handling

- Top-3 rate < 100% → sev-3; investigate ranking or metadata.
- Stale > 20% of corpus → sev-3; runbook-hygiene push.
- Auto-link rate low → product feedback on incident-keyword matching.

---

*End of NFR-KB-005.*
