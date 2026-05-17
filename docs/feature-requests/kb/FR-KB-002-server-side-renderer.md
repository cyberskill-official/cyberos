---
id: FR-KB-002
title: "KB server-side renderer — markdown → sanitised HTML (ammonia) + sanitised plaintext for BRAIN ingest"
module: KB
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 4
slice: 4
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-KB-001, FR-KB-005, FR-AI-019, FR-BRAIN-111]
depends_on: [FR-KB-001]
blocks: []

source_pages:
  - website/docs/modules/kb.html#renderer

source_decisions:
  - DEC-1890 2026-05-17 — Render markdown → HTML server-side via ammonia (Rust XSS-safe whitelist HTML sanitiser); never trust client-rendered for security
  - DEC-1891 2026-05-17 — Plaintext extraction strips formatting + sanitises for FR-AI-019 BRAIN Layer 2 vector ingest
  - DEC-1892 2026-05-17 — Closed enum `render_target` = {html_full, html_excerpt, plaintext, json_ast}; cardinality 4
  - DEC-1893 2026-05-17 — Render cache keyed by (doc_id, version_id) — invalidated on new version per FR-KB-001 immutability
  - DEC-1894 2026-05-17 — BRAIN audit kinds: kb.doc_rendered, kb.render_failed, kb.render_cache_invalidated

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0002_render_cache.sql
    - services/kb/src/renderer/mod.rs
    - services/kb/src/renderer/markdown_to_html.rs
    - services/kb/src/renderer/plaintext_extract.rs
    - services/kb/src/renderer/ammonia_config.rs
    - services/kb/src/audit/renderer_events.rs
    - services/kb/tests/render_html_test.rs
    - services/kb/tests/render_plaintext_test.rs
    - services/kb/tests/render_xss_blocked_test.rs
    - services/kb/tests/render_target_enum_cardinality_test.rs
    - services/kb/tests/render_cache_invalidation_test.rs
    - services/kb/tests/render_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/kb/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test renderer

  disallowed_tools:
    - bypass ammonia sanitiser (per DEC-1890)
    - serve unsanitised HTML (per DEC-1890)

effort_hours: 5
sub_tasks:
  - "0.3h: 0002_render_cache.sql"
  - "0.3h: renderer/mod.rs"
  - "0.5h: markdown_to_html.rs"
  - "0.4h: plaintext_extract.rs"
  - "0.5h: ammonia_config.rs"
  - "0.3h: audit/renderer_events.rs"
  - "2.2h: tests — 6 test files"
  - "0.5h: docs"

risk_if_skipped: "Without server-side render, clients render markdown directly → XSS attack surface. Without DEC-1891 sanitised plaintext, BRAIN ingest carries HTML noise + breaks vector quality. Without DEC-1890 ammonia, raw user-input HTML reaches readers."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship server-side renderer at `services/kb/src/renderer/` producing sanitised HTML + plaintext + cached, 3 BRAIN audit kinds.

1. **MUST** validate `render_target` against closed enum per DEC-1892.

2. **MUST** render at `markdown_to_html.rs::render(doc, target)`:
   - Parse via `pulldown-cmark`.
   - Sanitise via ammonia per DEC-1890 with strict whitelist (no script, no inline event handlers, no javascript: URLs).
   - Output per target (full HTML, excerpt 200 words, plaintext, JSON AST).

3. **MUST** extract plaintext at `plaintext_extract.rs::extract(html)` per DEC-1891 — strips tags, decodes entities, normalises whitespace.

4. **MUST** cache per DEC-1893 at table:
   ```sql
   CREATE TABLE kb_render_cache (
     cache_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     doc_id UUID NOT NULL,
     version_id UUID NOT NULL,
     target TEXT NOT NULL CHECK (target IN ('html_full','html_excerpt','plaintext','json_ast')),
     rendered_content TEXT NOT NULL,
     rendered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
     UNIQUE (tenant_id, doc_id, version_id, target)
   );
   CREATE INDEX render_cache_doc_idx ON kb_render_cache(tenant_id, doc_id, version_id);
   ALTER TABLE kb_render_cache ENABLE ROW LEVEL SECURITY;
   CREATE POLICY render_cache_rls ON kb_render_cache
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   GRANT DELETE ON kb_render_cache TO cyberos_app;  -- invalidation
   ```

5. **MUST** invalidate cache on new version per DEC-1893 — DELETE rows where doc_id matches but version_id differs from current.

6. **MUST** expose endpoints:
   ```text
   GET    /v1/kb/docs/{id}/render?target=html_full   (cache-checked)
   POST   /v1/kb/docs/{id}/render                     (force re-render, CDO)
   ```

7. **MUST** emit 3 BRAIN audit kinds per DEC-1894. PII per FR-BRAIN-111: rendered_content text SHA-256 hashed.

8. **MUST** thread trace_id from render request → renderer → cache → audit.

9. **MUST NOT** bypass ammonia per DEC-1890.

10. **MUST NOT** serve uncached invalidated content (must re-render).

---

## §2 — Why this design

**Why ammonia (DEC-1890)?** Pure-Rust XSS-safe sanitiser; whitelist approach; widely audited.

**Why server-side (DEC-1890)?** Client trust = XSS risk; always sanitise before serving.

**Why plaintext for BRAIN (DEC-1891)?** Vector quality depends on clean text; HTML tags poison embeddings.

**Why cache (DEC-1893)?** Markdown render is non-trivial; cache 100x speedup on repeat reads.

---

## §3 — API contract

```text
GET    /v1/kb/docs/{id}/render?target=html_full
POST   /v1/kb/docs/{id}/render
```

Sample response:
```json
{
  "doc_id": "uuid",
  "version_id": "uuid",
  "target": "html_full",
  "rendered_content": "<h1>Onboarding Guide</h1>...",
  "rendered_at": "2026-05-17T10:00:00Z"
}
```

---

## §4 — Acceptance criteria
1. **render_target enum cardinality 4**. 2. **Markdown → HTML works**. 3. **HTML sanitised (no script/event handlers)**. 4. **Plaintext extraction removes tags**. 5. **XSS payload blocked**. 6. **Excerpt 200 words capped**. 7. **JSON AST returned correctly**. 8. **Cache hit on repeat**. 9. **Cache invalidated on new version**. 10. **3 BRAIN audit kinds emitted**. 11. **PII scrubbed (rendered_content SHA256)**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **Force re-render CDO-only**. 15. **UNIQUE(doc, version, target) constraint**. 16. **Append-only via REVOKE UPDATE (only DELETE on invalidation)**. 17. **Ammonia whitelist documented**. 18. **Render performance < 50ms for 10k-char doc**. 19. **Large doc (>1MB) supported with timeout**. 20. **AT-rules (style) sanitised**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn xss_payload_blocked() {
    let md = r#"# Hello\n<script>alert(1)</script>\n<img src=x onerror=alert(1)>"#;
    let html = render_html(md);
    assert!(!html.contains("<script>"));
    assert!(!html.contains("onerror"));
}

#[tokio::test]
async fn plaintext_strips_tags() {
    let md = r#"# Heading\n**bold** _italic_"#;
    let plain = render_plaintext(md);
    assert!(!plain.contains("<"));
    assert!(plain.contains("Heading"));
    assert!(plain.contains("bold"));
}

#[tokio::test]
async fn cache_invalidated_on_new_version() {
    let ctx = TestContext::with_doc().await;
    ctx.render(ctx.doc_id, "html_full").await;
    ctx.create_new_version(ctx.doc_id).await;
    let cache = ctx.fetch_cache_for_doc(ctx.doc_id).await;
    let old_version_rows = cache.iter().filter(|r| r.version_id == ctx.original_version).count();
    assert_eq!(old_version_rows, 0);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-KB-001.
**Downstream:** FR-KB-005 (semantic ingest uses plaintext).
**Cross-module:** FR-AI-019 (BRAIN Layer 2 ingest), FR-AUTH-101 (CDO role), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Ammonia rejects all (config too strict) | empty output sev-2 | inherent | tune config |
| Markdown parse fail | catch | sev-2; raw fallback | data fix |
| Large doc timeout | 5s limit | 504 + sev-2 | split doc |
| Cache table corruption | DELETE+re-render | inherent | inherent |
| XSS bypass attempt | tests catch | inherent | bug fix |
| Plaintext encoding issue | UTF-8 enforcement | inherent | inherent |
| Cross-tenant cache leak | RLS | 0 rows | inherent |
| Version mismatch (race) | UNIQUE | last-write-wins | inherent |
| AT-rule injection | ammonia blocks | inherent | inherent |
| Inline SVG with script | ammonia blocks | inherent | inherent |

## §11 — Implementation notes
- §11.1 Ammonia config: allow basic HTML tags + safe attributes (href, src, alt, title); no event handlers; HTTP/HTTPS only.
- §11.2 Plaintext: ammonia first → strip remaining tags → normalise whitespace.
- §11.3 Cache TTL: indefinite; invalidated only on new doc version.
- §11.4 BRAIN audit body: doc_id, version_id, target; rendered_content SHA256.
- §11.5 Force re-render: bypasses cache lookup; useful for ammonia config updates.

---

*End of FR-KB-002 spec.*
