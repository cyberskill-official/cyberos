---
id: NFR-KB-001
title: "KB document versioning history — every save MUST create an immutable version row"
module: KB
category: maintainability
priority: MUST
verification: T
phase: P0
slo: "100% of document saves produce a version row; history immutable; retention ≥ 1 year"
owner: CTO
created: 2026-05-18
related_frs: [FR-KB-001, FR-KB-002]
---

## §1 — Statement (BCP-14 normative)

1. Every document save **MUST** create an immutable version row in `kb_document_version` carrying `{doc_id, version_no, author_id, saved_at, body_hash, body_blob_ref}`.
2. Version rows are append-only; no UPDATE or DELETE in the application layer.
3. Body blobs **MUST** be content-addressable (hash-keyed); identical content shares storage.
4. Document version history **MUST** be retrievable via API and visible in the doc UI as a side panel.
5. Retention: version rows **MUST** be retained for ≥ 1 year; tenant-configurable longer.

## §2 — Why this constraint

KB docs accumulate authority — runbooks, policies, decisions. Without versioning, a wrong edit destroys context. Immutable history converts "what did this say last week?" from "no idea" into a deterministic answer. Content-addressable storage is the cost-control mechanism — small edits don't double storage.

## §3 — Measurement

- Counter `kb_document_save_total{result}`.
- Gauge `kb_document_version_count_per_doc` — surfaces edit-storm patterns.
- Storage gauge `kb_version_blob_dedup_ratio` — measures content-addressing savings.

## §4 — Verification

- Unit test (T) — save doc; assert version row created with correct hash.
- Integration test (T) — multi-save; assert history accessible + immutable.
- Property test (T) — random saves; assert no mutation post-save.

## §5 — Failure handling

- Version row missing post-save → sev-2; investigate; possibly replay save.
- Mutation detected → sev-1; immutability broken; halt KB writes.
- Retention < 1y → sev-3; investigate cleanup cron.

---

*End of NFR-KB-001.*
