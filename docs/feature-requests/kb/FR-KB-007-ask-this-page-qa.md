---
id: FR-KB-007
title: "KB Ask-this-page Q&A — CUO-grounded answer over current + linked docs with span-level citations and answer-or-decline gate"
module: KB
priority: MUST
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CDO)
created: 2026-05-17
shipped: null
brain_chain_hash: null
related_frs: [FR-KB-006, FR-CUO-101, FR-AI-003, FR-BRAIN-108, FR-BRAIN-111]
depends_on: [FR-KB-001, FR-KB-006, FR-CUO-101, FR-BRAIN-108]
blocks: []

source_pages:
  - website/docs/modules/kb.html#ask-this-page

source_decisions:
  - DEC-1940 2026-05-17 — Q&A grounded ONLY in current page + linked docs (1-hop) — no open-world search to prevent hallucination
  - DEC-1941 2026-05-17 — Closed enum `qa_answer_kind` = {confident, partial, decline_no_evidence, decline_low_confidence}; cardinality 4
  - DEC-1942 2026-05-17 — Every claim cited with span-level reference (doc_id + chunk_id + char_range); UI highlights source on hover
  - DEC-1943 2026-05-17 — Confidence threshold 0.7 → answer; below → decline; user sees "Not enough evidence in this page + linked docs"
  - DEC-1944 2026-05-17 — Rate limit: 50 questions/user/day per FR-CUO-101 standard
  - DEC-1945 2026-05-17 — BRAIN audit kinds: kb.qa_asked, kb.qa_answered, kb.qa_declined, kb.qa_failed

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0007_qa_questions.sql
    - services/kb/src/qa/mod.rs
    - services/kb/src/qa/context_assembler.rs
    - services/kb/src/qa/answer_generator.rs
    - services/kb/src/qa/citation_extractor.rs
    - services/kb/src/handlers/qa_routes.rs
    - services/kb/src/audit/qa_events.rs
    - services/kb/tests/qa_confident_with_citations_test.rs
    - services/kb/tests/qa_decline_no_evidence_test.rs
    - services/kb/tests/qa_decline_low_confidence_test.rs
    - services/kb/tests/qa_answer_kind_enum_cardinality_test.rs
    - services/kb/tests/qa_rate_limit_test.rs
    - services/kb/tests/qa_citation_spans_test.rs
    - services/kb/tests/qa_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/{kb,cuo,ai}/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test qa

  disallowed_tools:
    - answer without citation (per DEC-1942)
    - search open world (per DEC-1940)
    - answer below confidence threshold (per DEC-1943)

effort_hours: 8
sub_tasks:
  - "0.3h: 0007_qa_questions.sql"
  - "0.4h: qa/mod.rs"
  - "0.7h: context_assembler.rs (current + 1-hop linked)"
  - "0.9h: answer_generator.rs (FR-AI-003 with structured prompt)"
  - "0.6h: citation_extractor.rs"
  - "0.4h: handlers/qa_routes.rs"
  - "0.3h: audit/qa_events.rs"
  - "3.0h: tests — 7 test files"
  - "1.4h: UI Ask box + span highlight"

risk_if_skipped: "Without Q&A, users scroll long docs hoping for answer (UX friction). Without DEC-1940 grounding, AI hallucinates from training data. Without DEC-1942 citations, answers untrustable. Without DEC-1943 decline gate, low-confidence guesses mislead users."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship Q&A at `services/kb/src/qa/` grounded in current page + 1-hop links, span-cited, decline-on-low-confidence, 4 BRAIN audit kinds.

1. **MUST** validate `qa_answer_kind` against closed enum per DEC-1941.

2. **MUST** assemble context at `context_assembler.rs::assemble(doc_id, question)` per DEC-1940:
   - Current doc chunks (all)
   - 1-hop linked docs (via FR-BRAIN-108 inbound/outbound links) → their top-3 most relevant chunks via FR-KB-006 rerank
   - Capped at 50k tokens total

3. **MUST** generate answer at `answer_generator.rs::generate(context, question)` per DEC-1940 with FR-AI-003 prompt:
   - System prompt restricts to provided context only
   - Output JSON: `{answer, confidence_score, citations: [{doc_id, chunk_id, char_start, char_end}]}`

4. **MUST** extract citations at `citation_extractor.rs::extract(answer)` per DEC-1942:
   - Every factual claim must reference a chunk + char range
   - Reject if any claim un-cited → mark as decline_low_confidence

5. **MUST** apply confidence gate per DEC-1943:
   - confidence ≥ 0.7 → return confident or partial
   - confidence < 0.7 → return decline_low_confidence
   - 0 citations → return decline_no_evidence

6. **MUST** enforce rate limit per DEC-1944 — 50/user/day; return 429.

7. **MUST** define table at migration `0007`:
   ```sql
   CREATE TABLE kb_qa_questions (
     question_id UUID PRIMARY KEY,
     tenant_id UUID NOT NULL,
     doc_id UUID NOT NULL,
     question_text TEXT NOT NULL,
     answer_kind TEXT NOT NULL
       CHECK (answer_kind IN ('confident','partial','decline_no_evidence','decline_low_confidence')),
     answer_text TEXT,
     confidence_score NUMERIC(3,2),
     citations JSONB NOT NULL DEFAULT '[]',
     asked_by UUID NOT NULL,
     trace_id CHAR(32),
     created_at TIMESTAMPTZ NOT NULL DEFAULT now()
   );
   CREATE INDEX qa_user_day_idx ON kb_qa_questions(tenant_id, asked_by, created_at DESC);
   ALTER TABLE kb_qa_questions ENABLE ROW LEVEL SECURITY;
   CREATE POLICY qa_rls ON kb_qa_questions
     USING (tenant_id = current_setting('auth.tenant_id')::uuid)
     WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
   REVOKE UPDATE, DELETE ON kb_qa_questions FROM cyberos_app;
   -- Append-only — Q&A history immutable
   ```

8. **MUST** expose endpoint:
   ```text
   POST /v1/kb/docs/{id}/ask    body: {question}
   ```

9. **MUST** emit 4 BRAIN audit kinds per DEC-1945. PII per FR-BRAIN-111: question + answer text SHA-256 hashed; citation ids ok.

10. **MUST** thread trace_id from ask → assemble → generate → cite → audit.

11. **MUST NOT** search open world per DEC-1940 (only doc + 1-hop links).

12. **MUST NOT** return answer without ≥1 citation per DEC-1942.

13. **MUST NOT** return below confidence threshold per DEC-1943.

---

## §2 — Why this design

**Why 1-hop linked (DEC-1940)?** Docs cross-reference; questions often need linked context (e.g. "what's the policy?" → linked Decree 145).

**Why span citations (DEC-1942)?** UI highlights source on hover; users verify claims; eliminates trust-but-can't-verify problem.

**Why confidence gate (DEC-1943)?** Better to say "I don't know" than confidently mislead; trust over coverage.

**Why rate limit (DEC-1944)?** AI cost per call non-trivial; per-user cap prevents runaway.

---

## §3 — API contract

```text
POST /v1/kb/docs/{id}/ask
```

Sample request:
```json
{
  "question": "What's the maximum OT per week?"
}
```

Sample response (confident):
```json
{
  "answer_kind": "confident",
  "answer_text": "Maximum OT per week is 12 hours per Decree 145 Art. 107.",
  "confidence_score": 0.92,
  "citations": [
    {
      "doc_id": "uuid-decree-145",
      "chunk_id": "uuid-art-107-chunk",
      "char_start": 1200,
      "char_end": 1280,
      "snippet": "...overtime shall not exceed 12 hours per week..."
    }
  ]
}
```

Sample response (decline):
```json
{
  "answer_kind": "decline_no_evidence",
  "answer_text": "Not enough evidence in this page + linked docs to answer.",
  "confidence_score": 0.3,
  "citations": []
}
```

---

## §4 — Acceptance criteria
1. **answer_kind enum cardinality 4**. 2. **Context = current doc + 1-hop linked**. 3. **No open-world search**. 4. **Every claim cited with chunk + char range**. 5. **Confidence ≥0.7 → confident or partial**. 6. **<0.7 → decline_low_confidence**. 7. **0 citations → decline_no_evidence**. 8. **Rate limit 50/user/day**. 9. **4 BRAIN audit kinds emitted**. 10. **PII scrubbed (question + answer SHA256)**. 11. **RLS denies cross-tenant**. 12. **Trace_id preserved**. 13. **Context capped 50k tokens**. 14. **Append-only via REVOKE**. 15. **UI span highlight from citations**. 16. **Decline messages user-friendly**. 17. **History queryable per user**. 18. **Failure → decline_low_confidence + sev-2**. 19. **Linked docs respect FR-KB-003 visibility**. 20. **Question length capped 1000 chars**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn confident_answer_has_citations() {
    let ctx = TestContext::with_doc_about_ot_caps().await;
    let r = ctx.ask(ctx.doc_id, "What's the max OT per week?").await;
    assert_eq!(r.answer_kind, "confident");
    assert!(!r.citations.is_empty());
    for c in &r.citations {
        assert!(c.char_end > c.char_start);
    }
}

#[tokio::test]
async fn decline_when_no_evidence() {
    let ctx = TestContext::with_doc_about_finance().await;
    let r = ctx.ask(ctx.doc_id, "What's the capital of France?").await;
    assert_eq!(r.answer_kind, "decline_no_evidence");
    assert_eq!(r.citations.len(), 0);
}

#[tokio::test]
async fn rate_limit_50_per_day() {
    let ctx = TestContext::with_user().await;
    for _ in 0..50 { ctx.ask(ctx.doc_id, "test").await; }
    let r = ctx.ask(ctx.doc_id, "test").await;
    assert_eq!(r.status_code, 429);
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** FR-KB-006, FR-CUO-101.
**Cross-module:** FR-AI-003 (LLM), FR-BRAIN-108 (link graph for 1-hop), FR-KB-003 (linked doc visibility), FR-BRAIN-111 (PII).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| AI timeout | retry 1x | decline_low_confidence + sev-2 | inherent |
| Context exceeds 50k tokens | truncate | sev-3 audit | inherent |
| 1-hop links empty | use doc-only context | inherent | inherent |
| Citation extraction fail | mark decline | sev-2 | inherent |
| Rate limit window edge | sliding 24h | inherent | inherent |
| Cross-tenant ask | RLS | 403 | inherent |
| Linked doc forbidden | filter | exclude from context | inherent |
| Question malformed | validate | 400 | rephrase |
| Citation char range invalid | reject answer | sev-2 | bug fix |
| AI quota | downstream | sev-2 | inherent |

## §11 — Implementation notes
- §11.1 AI prompt: "Answer ONLY from provided context. Cite every claim with chunk_id + char range. If insufficient evidence, return decline."
- §11.2 1-hop link query: `FR-BRAIN-108 link_graph WHERE source=$doc_id OR target=$doc_id`.
- §11.3 Rate limit via Redis sliding window: 50 ops per 24h per (tenant, user).
- §11.4 BRAIN audit body: doc_id, asked_by, answer_kind, confidence; question+answer SHA256.
- §11.5 UI span highlight: citation char_start/end maps to rendered HTML offset.

---

*End of FR-KB-007 spec.*
