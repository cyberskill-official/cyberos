---
id: TASK-KB-008
title: "KB runbook category — applicability tags (provider / region / severity) for OBS triage with TASK-OBS-007 incident routing"
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
module: KB
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 5
slice: 5
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-KB-001, TASK-KB-007, TASK-OBS-007, TASK-MEMORY-111]
depends_on: [TASK-KB-001, TASK-OBS-007]
blocks: []

source_pages:
  - website/docs/modules/kb.html#runbook

source_decisions:
  - DEC-1950 2026-05-17 — Runbook is special category with applicability tags: provider, region, severity; OBS triage queries by tag match
  - DEC-1951 2026-05-17 — Closed enum `runbook_provider` = {aws, gcp, azure, vercel, supabase, custom}; cardinality 6
  - DEC-1952 2026-05-17 — Closed enum `runbook_region` = {global, sg-1, eu-1, us-1, vn-1}; cardinality 5
  - DEC-1953 2026-05-17 — Closed enum `runbook_severity` = {sev1_critical, sev2_high, sev3_medium, sev4_low}; cardinality 4
  - DEC-1954 2026-05-17 — Multi-tag match: OBS-007 surfaces runbooks matching incident's provider AND region AND severity (or ANY for global tag)
  - DEC-1955 2026-05-17 — memory audit kinds: kb.runbook_tagged, kb.runbook_matched_for_incident

build_envelope:
  language: rust 1.81
  service: cyberos/services/kb/
  new_files:
    - services/kb/migrations/0008_runbook_tags.sql
    - services/kb/src/runbook/mod.rs
    - services/kb/src/runbook/tag_matcher.rs
    - services/kb/src/handlers/runbook_routes.rs
    - services/kb/src/audit/runbook_events.rs
    - services/kb/tests/runbook_provider_enum_cardinality_test.rs
    - services/kb/tests/runbook_region_enum_cardinality_test.rs
    - services/kb/tests/runbook_severity_enum_cardinality_test.rs
    - services/kb/tests/runbook_multi_tag_match_test.rs
    - services/kb/tests/runbook_global_match_test.rs
    - services/kb/tests/runbook_audit_emission_test.rs

  modified_files:
    - services/kb/src/lib.rs

  allowed_tools:
    - file_read: services/{kb,obs}/**
    - file_write: services/kb/{src,tests,migrations}/**
    - bash: cd services/kb && cargo test runbook

  disallowed_tools:
    - tag without enum validation (per DEC-1951/1952/1953)

effort_hours: 5
subtasks:
  - "0.3h: 0008_runbook_tags.sql"
  - "0.3h: runbook/mod.rs"
  - "0.6h: tag_matcher.rs"
  - "0.4h: handlers/runbook_routes.rs"
  - "0.3h: audit/runbook_events.rs"
  - "2.0h: tests — 6 test files"
  - "1.1h: TASK-OBS-007 integration smoke + docs"

risk_if_skipped: "Without runbook tags, OBS incidents trigger but engineers manually search KB → slow response. Without DEC-1954 multi-tag, wrong runbooks surface (AWS guide for GCP incident). Without enum validation, tag sprawl breaks triage automation."
---

## §1 — Description (BCP-14 normative)

The KB service **MUST** ship runbook tagging at `services/kb/src/runbook/` with 3-dim tags (provider/region/severity), OBS triage match, 2 memory audit kinds.

1. **MUST** validate enums per DEC-1951/1952/1953.

2. **MUST** define table extension at migration `0008`:
   ```sql
   ALTER TABLE kb_documents ADD COLUMN is_runbook BOOLEAN NOT NULL DEFAULT false;
   ALTER TABLE kb_documents ADD COLUMN runbook_providers TEXT[];
   ALTER TABLE kb_documents ADD COLUMN runbook_regions TEXT[];
   ALTER TABLE kb_documents ADD COLUMN runbook_severities TEXT[];

   ALTER TABLE kb_documents ADD CONSTRAINT runbook_providers_valid
     CHECK (runbook_providers IS NULL OR runbook_providers <@ ARRAY['aws','gcp','azure','vercel','supabase','custom']::TEXT[]);
   ALTER TABLE kb_documents ADD CONSTRAINT runbook_regions_valid
     CHECK (runbook_regions IS NULL OR runbook_regions <@ ARRAY['global','sg-1','eu-1','us-1','vn-1']::TEXT[]);
   ALTER TABLE kb_documents ADD CONSTRAINT runbook_severities_valid
     CHECK (runbook_severities IS NULL OR runbook_severities <@ ARRAY['sev1_critical','sev2_high','sev3_medium','sev4_low']::TEXT[]);

   CREATE INDEX runbook_provider_idx ON kb_documents USING GIN (runbook_providers) WHERE is_runbook = true;
   CREATE INDEX runbook_region_idx ON kb_documents USING GIN (runbook_regions) WHERE is_runbook = true;
   CREATE INDEX runbook_severity_idx ON kb_documents USING GIN (runbook_severities) WHERE is_runbook = true;

   GRANT UPDATE (is_runbook, runbook_providers, runbook_regions, runbook_severities) ON kb_documents TO cyberos_app;
   ```

3. **MUST** match for incident at `tag_matcher.rs::match(incident_provider, incident_region, incident_severity)` per DEC-1954:
   - SELECT runbooks WHERE provider IN (incident_provider, 'custom') AND region IN (incident_region, 'global') AND severity IN (incident_severity, ...higher_severities)
   - Order by specificity (exact match > global)

4. **MUST** expose endpoints:
   ```text
   PUT    /v1/kb/docs/{id}/runbook-tags    body: {providers, regions, severities}
   GET    /v1/kb/runbooks/match?provider=aws&region=sg-1&severity=sev1_critical
   ```

5. **MUST** emit 2 memory audit kinds per DEC-1955. PII per TASK-MEMORY-111: tag enums + counts ok.

6. **MUST** thread trace_id from OBS-007 incident → matcher → audit.

7. **MUST NOT** accept invalid enum values per DEC-1951/1952/1953 (CHECK constraints).

---

## §2 — Why this design

**Why 3 dims (DEC-1950)?** Triage filtering needs all three — same-severity AWS runbook irrelevant if incident is GCP.

**Why arrays (DEC-1950)?** A runbook may apply to multiple providers (e.g. "S3 / GCS bucket misconfigured") — array models that naturally.

**Why specificity ranking (DEC-1954)?** Multiple matches likely — global runbook + region-specific; show specific first.

---

## §3 — API contract

Sample runbook tag set:
```json
PUT /v1/kb/docs/{id}/runbook-tags
{
  "providers": ["aws"],
  "regions": ["sg-1"],
  "severities": ["sev1_critical", "sev2_high"]
}
```

Sample match query:
```json
GET /v1/kb/runbooks/match?provider=aws&region=sg-1&severity=sev1_critical

Response:
{
  "matches": [
    {"doc_id": "uuid", "title": "AWS SG-1 critical outage runbook", "specificity_score": 3},
    {"doc_id": "uuid", "title": "Global AWS outage runbook", "specificity_score": 1}
  ]
}
```

---

## §4 — Acceptance criteria
1. **provider enum cardinality 6**. 2. **region enum cardinality 5**. 3. **severity enum cardinality 4**. 4. **CHECK constraints enforce enum values**. 5. **Multi-tag match (provider AND region AND severity)**. 6. **Global tag matches any incident region**. 7. **Custom provider catches non-cloud-vendor incidents**. 8. **GIN indexes on each tag array**. 9. **Specificity ranking**. 10. **2 memory audit kinds emitted**. 11. **PII: tag enums (public) ok**. 12. **RLS denies cross-tenant**. 13. **Trace_id preserved**. 14. **CTO-only tag write**. 15. **TASK-OBS-007 integration tested**. 16. **Non-runbook docs not indexed (WHERE is_runbook)**. 17. **Empty tag arrays = no match (not all match)**. 18. **Append-only via REVOKE except 4 tag cols**. 19. **Multiple incident tags supported**. 20. **Severity escalation (higher sev runbooks shown for sev1)**.

---

## §5 — Verification

```rust
#[tokio::test]
async fn multi_tag_match_returns_specific_first() {
    let ctx = TestContext::with_runbooks_aws_sg_and_global().await;
    let r = ctx.match_runbooks("aws", "sg-1", "sev1_critical").await;
    assert!(r.matches[0].title.contains("SG-1"));
}

#[tokio::test]
async fn global_tag_catches_any_region() {
    let ctx = TestContext::with_global_runbook().await;
    let r = ctx.match_runbooks("aws", "us-1", "sev1_critical").await;
    assert!(!r.matches.is_empty());
}

#[tokio::test]
async fn invalid_enum_rejected() {
    let r = ctx.set_runbook_tags(ctx.doc_id, vec!["invalid_provider"], vec!["sg-1"], vec!["sev1"]).await;
    assert!(r.is_err());
}

// 5.4..5.10
```

---

## §7 — Dependencies
**Upstream:** TASK-KB-001, TASK-OBS-007.
**Cross-module:** TASK-MEMORY-111 (audit).

## §10 — Failure modes
| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Invalid enum tag | CHECK | 400 | use valid |
| Empty tag arrays | inherent | no match | inherent |
| Incident region missing | use 'global' | inherent | inherent |
| Multiple matches same specificity | tied | inherent | inherent |
| GIN index slow | EXPLAIN | tune | inherent |
| Tag updated mid-incident | snapshot at match time | inherent | inherent |
| Cross-tenant runbook leak | RLS | 0 rows | inherent |
| Non-runbook doc tagged | is_runbook=true required | 400 | set flag |
| Severity hierarchy unclear | order: sev1>sev2>sev3>sev4 | inherent | inherent |
| Custom provider catch-all | inherent | inherent | inherent |

## §11 — Implementation notes
- §11.1 GIN array index: `WHERE is_runbook = true` reduces index size 100x for non-runbook-heavy KBs.
- §11.2 Specificity score: 3 for triple match, 2 for double, 1 for single (global).
- §11.3 memory audit body: doc_id, tags; incident match audit includes incident_id from OBS-007.
- §11.4 Tag updates surface in TASK-KB-007 Q&A (runbook context for incident questions).
- §11.5 Auto-suggest tags via TASK-AI-003 on doc save — CTO confirms.

---

*End of TASK-KB-008 spec.*
