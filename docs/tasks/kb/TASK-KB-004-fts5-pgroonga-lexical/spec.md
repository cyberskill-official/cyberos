---
id: TASK-KB-004
title: "KB FTS5 + PGroonga lexical search — VN bigram tokenisation + English stemming + per-tenant index with tier filter"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: kb
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-KB-001, TASK-KB-002, TASK-KB-003, TASK-KB-006, TASK-MEMORY-111]
depends_on: [TASK-KB-001, TASK-KB-003]
blocks: []

source_pages:
  - website/docs/modules/kb.html#lexical-search

source_decisions:
  - DEC-1910 2026-05-17 — PGroonga primary (better VN bigram support); FTS5 fallback for read-replica/dev environments
  - DEC-1911 2026-05-17 — Closed enum `lexical_engine` = {pgroonga, fts5_fallback}; cardinality 2
  - DEC-1912 2026-05-17 — VN bigram tokenisation enabled by default for VN tenants; English stemming via Snowball for global
  - DEC-1913 2026-05-17 — Index updates synchronously on doc version commit; eventual consistency window ≤ 1s
  - DEC-1914 2026-05-17 — Search results filtered by TASK-KB-003 visibility tier at query time (RLS handles tenant + tier)
  - DEC-1915 2026-05-17 — memory audit kinds: kb.lexical_query_executed, kb.index_updated, kb.search_failed

language: rust 1.81
service: cyberos/services/kb/
new_files:
  - services/kb/migrations/0004_pgroonga_fts5_index.sql
  - services/kb/src/search/lexical.rs
  - services/kb/src/search/pgroonga_client.rs
  - services/kb/src/search/fts5_fallback.rs
  - services/kb/src/search/bigram_tokeniser.rs
  - services/kb/src/handlers/search_routes.rs
  - services/kb/src/audit/lexical_search_events.rs
  - services/kb/tests/lexical_pgroonga_test.rs
  - services/kb/tests/lexical_vn_bigram_test.rs
  - services/kb/tests/lexical_english_stem_test.rs
  - services/kb/tests/lexical_tier_filter_test.rs
  - services/kb/tests/lexical_engine_enum_cardinality_test.rs
  - services/kb/tests/lexical_audit_emission_test.rs

modified_files:
  - services/kb/src/lib.rs

allowed_tools:
  - file_read: services/kb/**
  - file_write: services/kb/{src,tests,migrations}/**
  - bash: cd services/kb && cargo test lexical

disallowed_tools:
  - bypass tier filter (per DEC-1914)
  - skip VN bigram for VN tenants (per DEC-1912)

effort_hours: 6
subtasks:
  - "0.4h: 0004_pgroonga_fts5_index.sql"
  - "0.4h: search/lexical.rs"
  - "0.7h: pgroonga_client.rs"
  - "0.6h: fts5_fallback.rs"
  - "0.5h: bigram_tokeniser.rs"
  - "0.4h: handlers/search_routes.rs"
  - "0.3h: audit/lexical_search_events.rs"
  - "2.3h: tests — 6 test files"
  - "0.4h: docs"

risk_if_skipped: "Without lexical search, KB un-searchable beyond title scan. Without DEC-1912 VN bigram, VN content searches return 0 results. Without DEC-1914 tier filter, search results leak restricted docs."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship lexical search at `services/kb/src/search/lexical.rs` with PGroonga primary + FTS5 fallback + VN bigram + tier filter, 3 memory audit kinds.

1. **MUST** validate `lexical_engine` against closed enum per DEC-1911.

2. **MUST** index at migration `0004`:
   ```sql
   CREATE EXTENSION IF NOT EXISTS pgroonga;
   ALTER TABLE kb_documents ADD COLUMN search_doc_tsv TSVECTOR;
   CREATE INDEX kb_docs_pgroonga_idx
     ON kb_documents
     USING pgroonga ((slug || ' ' || title || ' ' || rendered_plaintext))
     WITH (tokenizer='TokenBigramSplitSymbolAlphaDigit');
   CREATE INDEX kb_docs_fts5_idx
     ON kb_documents
     USING gin(to_tsvector('english', slug || ' ' || title || ' ' || rendered_plaintext));
   ```

3. **MUST** tokenise per DEC-1912 — bigram for VN tenants, Snowball English for others. Detection per tenant.locale.

4. **MUST** query at `lexical.rs::search(tenant, query, engine?, limit)`:
- Default engine = pgroonga
- Fallback to fts5 on PGroonga error
- Apply RLS tier filter (TASK-KB-003)
- Return ranked snippets

5. **MUST** update index synchronously on doc version commit per DEC-1913 — TRIGGER ON INSERT/UPDATE kb_documents.

6. **MUST** expose endpoint:
   ```text
   POST /v1/kb/search/lexical    body: {query, engine?, limit?, filters?}
   ```

7. **MUST** emit 3 memory audit kinds per DEC-1915. PII per TASK-MEMORY-111: query text SHA-256 hashed.

8. **MUST** thread trace_id from query → engine → audit.

9. **MUST NOT** return docs the user lacks permission for per DEC-1914 (RLS + tier filter).

10. **MUST NOT** skip VN bigram for VN tenants per DEC-1912.

---

## §2 — Why this design

**Why PGroonga primary (DEC-1910)?** Better tokenisation for VN/CJK languages; FTS5 weak on multi-byte UTF-8 word boundaries.

**Why FTS5 fallback (DEC-1910)?** PGroonga extension may not be available in all envs (e.g. dev SQLite); fallback maintains feature.

**Why bigram for VN (DEC-1912)?** Vietnamese has no spaces in compound words (e.g. "côngtyTNHH"); bigram catches partial matches.

**Why sync index (DEC-1913)?** Search-as-you-type UX needs fresh results; async lag = stale UX.

---

## §3 — API contract

```text
POST /v1/kb/search/lexical
```

Sample request:
```json
{
  "query": "công ty TNHH thanh toán",
  "engine": "pgroonga",
  "limit": 10,
  "filters": {"category": "finance"}
}
```

Sample response:
```json
{
  "results": [
    {
      "doc_id": "uuid",
      "title": "Quy trình thanh toán cho công ty TNHH",
      "snippet": "... <b>công ty TNHH</b> cần xuất hóa đơn cho mỗi <b>thanh toán</b>...",
      "rank": 0.92,
      "engine": "pgroonga"
    }
  ],
  "total": 1
}
```

---

## §4 — Acceptance criteria
1. **lexical_engine enum cardinality 2**. 2. **PGroonga primary**. 3. **FTS5 fallback on PGroonga error**. 4. **VN bigram for VN tenant**. 5. **English stemming for global tenant**. 6. **Tier filter applied (RLS + visibility_tier)**. 7. **Synchronous index update via trigger**. 8. **3 memory audit kinds emitted**. 9. **PII scrubbed (query text SHA256)**. 10. **RLS denies cross-tenant**. 11. **Trace_id preserved**. 12. **Snippet highlighting**. 13. **Result rank score returned**. 14. **Pagination support**. 15. **filters parameter supported**. 16. **Empty result returns empty array (not 404)**. 17. **Eventual consistency ≤ 1s post-write**. 18. **Search performance < 100ms p95**. 19. **Query length capped 500 chars**. 20. **Cross-language search per locale**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn vn_bigram_matches() {
    let ctx = TestContext::vn_tenant_with_doc("Quy trình thanh toán cho công ty TNHH").await;
    let r = ctx.lexical_search("công ty TNHH").await;
    assert!(!r.results.is_empty());
}

#[tokio::test]
async fn english_stem_matches_plural() {
    let ctx = TestContext::en_tenant_with_doc("invoicing best practices").await;
    let r = ctx.lexical_search("invoice").await;
    assert!(!r.results.is_empty());
}

#[tokio::test]
async fn tier_filter_excludes_role_restricted() {
    let ctx = TestContext::with_public_and_role_restricted_doc().await;
    let r = ctx.lexical_search_as(ctx.am_user, "topic").await;
    let restricted_ids = ctx.role_restricted_doc_ids();
    assert!(r.results.iter().none(|d| restricted_ids.contains(&d.doc_id)));
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-KB-001, TASK-KB-003. **Downstream:** TASK-KB-006 (rerank consumes lexical results). **Cross-module:** TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| PGroonga extension missing | catch | fallback FTS5 | install extension |
| Query syntax invalid | parse | 400 | fix query |
| Index update lag | sync trigger | inherent | inherent |
| Cross-tenant query | RLS | 0 rows | inherent |
| Large query (>500 chars) | validate | 400 | shorten |
| Empty result | inherent | [] | inherent |
| Index corruption | sev-1 | manual REINDEX | inherent |
| Locale detection wrong | fallback default | sev-3 | tenant config fix |
| Snippet highlighting fail | fallback raw text | inherent | inherent |
| Rank computation error | fallback 0.5 | sev-3 | bug fix |

## §11 — Implementation notes
- §11.1 PGroonga TokenBigramSplitSymbolAlphaDigit handles VN + English mixed.
- §11.2 Trigger on kb_documents INSERT/UPDATE refreshes tsvector + reindex row.
- §11.3 Result snippet via PGroonga's snippet_html() function with <b> highlight.
- §11.4 memory audit body: tenant_id, engine, result_count; query SHA256.
- §11.5 Pagination via OFFSET/LIMIT; cursor-based for >1k results.

---

*End of TASK-KB-004 spec.*
