---
id: NFR-PROJ-003
title: "PROJ memory_link integrity — every memory_link MUST resolve to a real memory row"
module: PROJ
category: reliability
priority: MUST
verification: T
phase: P0
slo: "100% of memory_link references resolve; 0 dangling links in production"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-PROJ-009, TASK-PROJ-010]
---

## §1 — Statement (BCP-14 normative)

1. Every `memory_link` field on a PROJ entity (issue, decision-anchor, cycle review) **MUST** reference a real, accessible memory row by its `(tenant_id, actor_id, seq)` triple.
2. memory_link creation **MUST** be validated at write time — the referenced row must exist before the memory_link is persisted.
3. Citation-drift detection (`TASK-PROJ-010`) **MUST** run daily, scanning all PROJ entities for memory_link references and asserting they still resolve.
4. Soft-deleted memory rows **MUST NOT** invisibly invalidate memory_links — the soft-delete preserves the seq; only hard-delete (regulatory) breaks links and the drift detector flags this.
5. Drift detector findings **MUST** notify the entity owner + log to the citation-drift queue for resolution.

## §2 — Why this constraint

memory_link is the platform's mechanism for attaching project entities to their memory context (decisions, threads, evidence). A dangling memory_link silently disconnects an entity from its rationale — operators read the issue, ask "why was this decided?", find a broken link. The drift detector catches the rare case (hard-delete) where the link was good at creation but became invalid later. The notify-owner rule turns drift into actionable maintenance.

## §3 — Measurement

- Counter `proj_memory_link_create_failed_total{reason=row_missing|tenant_mismatch}` — write-time gate.
- Daily drift report: `proj_memory_link_dangling_count` — must trend to 0.
- Counter `proj_memory_link_drift_resolved_total`.

## §4 — Verification

- Unit test (T) — create memory_link to missing seq → reject.
- Integration test (T) — hard-delete a memory row; assert drift detector flags + notifies.
- Daily production job (T) — full scan.

## §5 — Failure handling

- Write-time failure → caller fixes ref before retry.
- Drift detected → owner notified; SLA 7 days to resolve.
- Drift backlog > 100 → sev-3; module owner reviews.

---

*End of NFR-PROJ-003.*
