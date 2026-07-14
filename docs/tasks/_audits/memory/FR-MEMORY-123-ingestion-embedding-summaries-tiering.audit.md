---
task_id: TASK-MEMORY-123
audited: 2026-06-29
verdict: PASS
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 14
template: engineering-spec@1
authoring_md_compliance: 2026-06-29 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
strict_redo_pass: 2026-06-29 (no-line-cap expansion per task-audit skill §0; ISS-009..014 added)
eu_ai_act_risk_class: limited
---

## §1 — Verdict summary

TASK-MEMORY-123 specifies the BRAIN ingestion/index pipeline — Phase 2 of `docs/strategy/cyberos-brain-evaluation-plan.md`. Current scope: 20 §1 clauses (per-tenant event cursor over the TASK-MEMORY-121 stream with WIDE day-1 capture; gateway-routed embedding into pgvector; hot-tier HNSW for sub-second recall; rolling per-subject/per-channel/per-window summaries with `covered_seq_range` + version supersede; summaries-first recall with drill; hot/warm/cold tiering by age; `POST /v1/memory/recall`; tenant RLS + TASK-EVAL-001 per-subject access exclude with deny-by-default; per-hit provenance pointers to l1_audit_log; read-time chain_anchor verify; audit-chain-is-system-of-record / Layer-1-wins; idempotent UPSERT; residency + spend-cap discipline with pending-not-bypass; backfill + re-embed + index-rebuild; OTel metrics incl. recall p50/p99 + spend + access-denied; tenant RLS on all brain tables; TASK-MEMORY-117 store-ACL on summary writes; graceful query-embed degrade; graceful shutdown; SHOULD importance/recency weighting). 11 §2 rationale paragraphs. §3 contains: 3 migrations (brain_event_embedding with partial hot HNSW + RLS, brain_summary with current-version HNSW + RLS, ingest cursor + tier watermark) + the BrainEvent/RecallQuery/RecallHit/Provenance types + ingest_worker + summaries-first access-scoped recall + the embed_client gateway-only contract + the Python ai-gateway contract note. 22 ACs. §5 has 9 named tests. §10 lists 30 failure rows. §11 lists 12 implementation notes. A dedicated `## AI Risk Assessment` (eu_ai_act_risk_class: limited) closes the spec. The FR honours all four of Stephen's 2026-06-29 decisions (WIDE capture, access-restricted recall, ai-gateway residency/spend, Phase-2-is-the-brain) and the canonical FR map (`depends_on [TASK-MEMORY-121, TASK-MEMORY-122]`, `blocks [TASK-EVAL-003]`).

## §2 — Findings (all resolved)

### ISS-001 — Derived index masquerading as source of truth
A vector index + LLM summaries are lossy and model-dependent; if treated as authoritative they corrupt a record that may inform pay or a legal dispute. Resolved: §1 #11 + DEC-2721 make `l1_audit_log` the system of record and the brain a read-only derived lens; §1 #14 + AC #16 prove derivability via `--rebuild`; `disallowed_tools` forbids treating the index as truth.

### ISS-002 — Cross-subject leakage within a tenant
Tenant RLS alone stops cross-company leakage but the closest vector neighbour to a query is returned regardless of subject, so one employee's record could surface in another's evaluation context. Resolved: §1 #8 + DEC-2722 apply the TASK-EVAL-001 per-subject predicate AFTER RLS as an EXCLUDE (not a derank) with deny-by-default; AC #10 (closest neighbour excluded) + AC #11 (unknown subject denied) + the `subject_scope`/`unknown_subject` counters.

### ISS-003 — Evidence with no provenance is indefensible
"The brain said so" cannot back a performance or IP decision. Resolved: §1 #9 + DEC-2726 require a provenance pointer per hit (event → single audit_row_id; summary → covered_seq_range + top contributors); AC #12; the `Provenance` type in §3; TASK-EVAL-003 cites these exact rows.

### ISS-004 — Unbounded cost + latency as the log grows
Embedding and searching every raw event forever makes recall latency and AI spend climb linearly. Resolved: §1 #4/#5 (rolling summaries, summaries-first) + §1 #6 (hot/warm/cold tiering with a PARTIAL hot HNSW index) + DEC-2724/2515; AC #6 (summaries-first) + AC #7 (tiering) + the tier-distribution metric.

### ISS-005 — Embeddings bypassing residency + spend policy
A worker calling a model provider directly would re-implement and drift from the gateway policy, could ship employee text out of region, and could blow the spend cap silently. Resolved: §1 #2/#13 + DEC-2723 route ALL embedding/summary generation through the ai-gateway (TASK-AI-022); over-cap → `pending_*` + backoff, never a direct call; `disallowed_tools` forbids the bypass; AC #15; the embed_client + Python contract note in §3.

### ISS-006 — Tamper between Layer 1 and the derived index
The index can lag a Layer-1 tamper, feeding stale/tampered evidence into an evaluation. Resolved: §1 #10 reuses the TASK-MEMORY-101/108 read-time chain_anchor verify (recompute SHA-256 of the current Layer-1 row; mismatch → drop hit + sev-1); AC #13.

### ISS-007 — Restart-mid-batch duplication
A crash after the cursor advances but before commit could double-write rows. Resolved: §1 #12 idempotent UPSERT on `(tenant_id, source_seq)` for events; summaries supersede by version rather than duplicate; AC #3 + AC #5.

### ISS-008 — Recall down when the embedder is down
If the query cannot be embedded, naive recall returns nothing. Resolved: §1 #18 graceful degrade to full-text over summaries (reuse TASK-MEMORY-108 PGroonga) with the degradation surfaced in `explain`; empty results are 200 not 404; AC #18.

### ISS-009 — Summary write could escape the ACL boundary (strict-redo pass)
The worker materialises subject summaries into the memory tree; without ACL it could write into a human-only subtree. Resolved: §1 #17 applies TASK-MEMORY-117 store-ACL under a reserved `brain-ingest` actor; rejection emits a `memory.acl_denied` aux row; AC #19; failure-mode row + §11 note (recall still works off `brain_summary` even when the subtree write is denied).

### ISS-010 — Stale summaries miss the freshest events (strict-redo pass)
A summary's `covered_seq_range` lags new events in its window, so recall could miss the latest interactions. Resolved: §1 #4 re-summarises and supersedes on new events in a window; §1 #5 drill closes the residual gap; AC #5 + the "summary embedding stale vs current events" failure row (drill / re-summarise recovery).

### ISS-011 — Model migration with no version tracking (strict-redo pass)
Swapping the embedding model with a live index leaves mixed/incomparable vectors and no migration path. Resolved: §1 #14 `--reembed --model <alias>` + `embed_model_version` per row in both tables; AC #17; failure-mode row (mixed versions during migration; recall still answers).

### ISS-012 — Tiering pass not idempotent (strict-redo pass)
Re-running the age-based tiering could duplicate or lose rows. Resolved: §1 #6 makes transitions idempotent via the per-tenant `brain_tier_watermark`; AC #7 asserts a second pass is a no-op; failure-mode row.

### ISS-013 — Cold raw unreachable (strict-redo pass)
If cold archiving dropped the raw event, evidence behind an old summary could not be produced on demand. Resolved: §1 #6 keeps the raw row in Layer 1 (the truth) + cold storage, retrievable by `audit_row_id`, with only the summary indexed; AC #8.

### ISS-014 — Hard dependency on ranking signals not yet wired (strict-redo pass)
Requiring TASK-MEMORY-114 importance + TASK-MEMORY-113 recency would block the brain in tenants where those are absent. Resolved: §1 #20 makes them a SHOULD-weight with graceful fallback to raw cosine + RRF; AC #22; the dependency list marks the edge soft; failure-mode row.

## §3 — Resolution

All 14 mechanical concerns addressed. **Score = 10/10.**

Per task-audit skill §0 master rule: depth is bounded by the genuine surface (event-log ingest × gateway-policed embedding × summaries-first × hot/warm/cold tiering × tenant-RLS-plus-subject-access exclude × provenance × chain-verify × derivability/rebuild × residency/spend × store-ACL), not by line targets. The spec reuses the Phase-1 substrate (Layer-1 audit chain, pgvector/HNSW, ai-gateway) exactly as the strategy note and the canonical FR map require; it does not re-architect. House style (engineering-spec@1: BCP-14 MUST/SHOULD/MAY, §-numbered sections, em dashes / arrows / × / §, SQL + Rust + Python sketches, failure-mode table, AC↔§1 traceability) is consistent with TASK-PROJ-008 and TASK-MEMORY-101/108. The added `## AI Risk Assessment` (limited) places the consequential-decision risk downstream in TASK-EVAL-003 with a human in the loop and binds the brain's own residual risk to normative clauses.

Note for the wave reviewer: an unrelated pre-existing file `TASK-MEMORY-121-awh-gate-result-audit-row.md` already occupies the 121 id for the awh-gate-result aux row (P1). This FR cites 121/122 with the BRAIN-wave meaning per the assignment's canonical FR map (121 = interaction-event schema, 122 = capture emitters), which the parallel BRAIN/EVAL agents author. The id collision is a backlog-grooming item (task: "Groom BACKLOG.md: add BRAIN/EVAL workstream + reprioritize"), not a defect in this spec.

---

*End of TASK-MEMORY-123 audit.*
