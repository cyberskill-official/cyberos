---
id: TASK-KB-009
title: "KB dual-language `translation_of` link — vi/en pairing with locale-aware reader display and translation parity audit"
eu_ai_act_risk_class: not_ai  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-05-17T00:00:00+07:00
department: engineering
author: @stephencheng
template: task@1
module: KB
priority: p1
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-KB-001, TASK-KB-007, TASK-AI-003, TASK-MEMORY-111]
depends_on: [TASK-KB-001]
blocks: []

source_pages:
  - website/docs/modules/kb.html#translation

source_decisions:
  - DEC-1960 2026-05-17 — Bidirectional `translation_of` link: each doc has optional pointer to its translation counterpart (vi ↔ en)
  - DEC-1961 2026-05-17 — Closed enum `kb_locale` = {vi, en}; cardinality 2 (extensible later); reader display per user locale
  - DEC-1962 2026-05-17 — Parity check: AI-suggested diff-summary when source updated; CDO reviews + propagates to translation
  - DEC-1963 2026-05-17 — Both sides indexed independently (TASK-KB-004 + TASK-KB-005); search returns whichever locale matches; reader auto-switches if pair exists
  - DEC-1964 2026-05-17 — memory audit kinds: kb.translation_linked, kb.translation_parity_alert, kb.translation_display_switched

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0009_translation_link.sql
    - services/kb/src/translation/mod.rs
    - services/kb/src/translation/parity_checker.rs
    - services/kb/src/translation/locale_router.rs
    - services/kb/src/handlers/translation_routes.rs
    - services/kb/src/audit/translation_events.rs
    - services/kb/tests/translation_link_bidirectional_test.rs
    - services/kb/tests/translation_locale_enum_cardinality_test.rs
    - services/kb/tests/translation_parity_alert_test.rs
    - services/kb/tests/translation_locale_aware_display_test.rs
    - services/kb/tests/translation_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/{kb,ai}/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test translation

  disallowed_tools:
    - asymmetric link (per DEC-1960 — must be bi-dir)

effort_hours: 4
subtasks:
  - "0.3h: 0009_translation_link.sql"
  - "0.3h: translation/mod.rs"
  - "0.5h: parity_checker.rs"
  - "0.4h: locale_router.rs"
  - "0.3h: handlers/translation_routes.rs"
  - "0.3h: audit/translation_events.rs"
  - "1.5h: tests — 5 test files"
  - "0.4h: docs"

risk_if_skipped: "Without translation linking, vi/en doc pairs drift independently. Without DEC-1962 parity check, source updates leave translation stale. Without DEC-1963 reader auto-switch, users land on wrong-locale doc."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship translation linking at `services/kb/src/translation/` with bi-directional links + parity check + locale-aware display, 3 memory audit kinds.

1. **MUST** validate `kb_locale` against closed enum per DEC-1961.

2. **MUST** define table extension at migration `0009`:
   ```sql
   ALTER TABLE kb_documents ADD COLUMN locale TEXT CHECK (locale IS NULL OR locale IN ('vi','en'));
   ALTER TABLE kb_documents ADD COLUMN translation_of UUID REFERENCES kb_documents(doc_id);
   CREATE INDEX docs_translation_idx ON kb_documents(tenant_id, translation_of) WHERE translation_of IS NOT NULL;
   GRANT UPDATE (locale, translation_of) ON kb_documents TO cyberos_app;
   ```

3. **MUST** enforce bidirectional link per DEC-1960 — if A.translation_of=B, then B.translation_of MUST equal A. Validated at write.

4. **MUST** check parity at `parity_checker.rs::check(doc, translation)` per DEC-1962:
   - Triggered on source doc version update
   - TASK-AI-003 generates diff-summary of changes
   - Alert emitted (memory audit + CDO notification)
   - CDO reviews + propagates to translation

5. **MUST** route reader display per DEC-1963 at `locale_router.rs::route(doc, user_locale)`:
   - If doc.locale matches user_locale, return doc
   - Else if doc has translation_of in user_locale, redirect to translation
   - Else show as-is with banner "Translation not available"

6. **MUST** expose endpoints:
   ```text
   PUT    /v1/kb/docs/{id}/translation        body: {translation_of_doc_id}
   GET    /v1/kb/docs/{id}/translation-parity (CDO check)
   ```

7. **MUST** emit 3 memory audit kinds per DEC-1964. PII per TASK-MEMORY-111: diff-summary SHA256.

8. **MUST** thread trace_id from link/parity/display → audit.

9. **MUST NOT** allow asymmetric link per DEC-1960.

10. **MUST** un-link by setting translation_of = NULL (still requires bidirectional update).

---

## §2 — Why this design

**Why bi-directional (DEC-1960)?** Users navigating from either side need access to the other.

**Why locale enum cardinality 2 (DEC-1961)?** Current scope (VN agency); extensible if Indonesian/Thai expansion needed.

**Why parity check (DEC-1962)?** Source updates outpace manual translation; without alerts, translations stale silently.

**Why auto-switch reader (DEC-1963)?** Search may surface either locale; user sees their preferred without manual click.

---

## §3 — API contract

Sample link creation:
```json
PUT /v1/kb/docs/{vi_doc}/translation
{ "translation_of_doc_id": "uuid-en-doc" }
```

Bi-directional auto-update: en doc's translation_of also set to vi doc.

Parity check response:
```json
{
  "vi_doc_id": "uuid",
  "en_doc_id": "uuid",
  "vi_last_updated": "2026-05-15T10:00:00Z",
  "en_last_updated": "2026-04-20T10:00:00Z",
  "drift_days": 25,
  "diff_summary": "VN version added Section 3 about new VAT regulation; EN missing this section.",
  "ai_suggested_translation": "..."
}
```

---

## §4 — Acceptance criteria
1. **locale enum cardinality 2**. 2. **Bi-directional link enforced**. 3. **CHECK constraint on locale values**. 4. **Index on translation_of for lookups**. 5. **Parity check via TASK-AI-003**. 6. **Source update triggers parity audit**. 7. **Locale router auto-switches**. 8. **No-translation banner shown when missing**. 9. **3 memory audit kinds emitted**. 10. **PII scrubbed (diff-summary SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **CDO-only link/parity write**. 14. **Un-link respects bidirectional invariant**. 15. **Independent search indexes (TASK-KB-004+005 per locale)**. 16. **Append-only via REVOKE except 2 cols**. 17. **Both sides remain queryable**. 18. **Self-reference rejected**. 19. **Cross-tenant link rejected (FK + RLS)**. 20. **Multiple-translation chain prevented (one-to-one)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn bidirectional_link_enforced() {
    let ctx = TestContext::with_vi_and_en_docs().await;
    ctx.link_translation(ctx.vi_doc, ctx.en_doc).await;
    let en = ctx.fetch_doc(ctx.en_doc).await;
    assert_eq!(en.translation_of, Some(ctx.vi_doc));
}

#[tokio::test]
async fn locale_router_auto_switches() {
    let ctx = TestContext::with_translation_pair().await;
    let r = ctx.fetch_doc_as_user(ctx.vi_doc, "en").await;
    assert_eq!(r.served_doc_id, ctx.en_doc);
}

#[tokio::test]
async fn parity_alert_on_source_update() {
    let ctx = TestContext::with_translation_pair().await;
    ctx.update_doc(ctx.vi_doc, "new content").await;
    tokio::time::sleep(Duration::from_secs(1)).await;
    let audits = ctx.fetch_memory_audits("kb.translation_parity_alert").await;
    assert!(!audits.is_empty());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-KB-001.
**Cross-module:** TASK-AI-003 (diff summary), TASK-KB-007 (Q&A respects locale), TASK-AUTH-101 (CDO), TASK-MEMORY-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Self-reference link | validate | 400 | use different doc |
| Cross-tenant link | RLS + FK | 404 | inherent |
| Asymmetric link write | validator | reject; rollback | inherent |
| User locale unknown | default to en | inherent | tenant config |
| Translation missing (one-side) | banner shown | inherent | create translation |
| AI parity check timeout | retry; degrade | sev-2; manual review | inherent |
| Locale = NULL on doc | warn at link | reject 400 | set locale |
| Triangulation (A→B, B→C) | one-to-one constraint | reject | use direct |
| Locale change mid-pair | update both | manual CDO | inherent |
| Independent index drift | per-locale TASK-KB-004/005 | inherent | inherent |

## §11 — Implementation notes
- §11.1 Bi-dir enforcement: trigger on UPDATE sets the partner's translation_of too.
- §11.2 Parity check cron: nightly compares last_updated of both sides; alerts on > 7-day drift.
- §11.3 Locale router accepts Accept-Language header or user.preferred_locale.
- §11.4 memory audit body: doc_id pair, drift_days; diff_summary SHA256.
- §11.5 Future locales: add to enum + migration; reader router handles automatically.

---

*End of TASK-KB-009 spec.*
