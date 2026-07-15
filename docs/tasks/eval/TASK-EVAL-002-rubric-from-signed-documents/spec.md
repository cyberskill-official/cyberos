---
id: TASK-EVAL-002
title: "evaluation rubric from the three signed employment documents — versioned, effective-dated, bilingual VN/EN; each item cites its exact source clause (document + clause_ref) so an assessment can name the contract clause it measured against"
ai_authorship: generated_then_reviewed  # UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
client_visible: false
type: feature
created_at: 2026-06-29T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: EVAL
priority: p0
status: draft
verify: T
phase: P3
milestone: P3 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-06-29
shipped: null
memory_chain_hash: null
related_tasks: [TASK-EVAL-001, TASK-EVAL-003, TASK-EVAL-004, TASK-MEMORY-123, TASK-AUTH-003, TASK-PROJ-008]
depends_on: [TASK-EVAL-001]
blocks: [TASK-EVAL-003]

source_pages:
  - docs/strategy/cyberos-brain-evaluation-plan.md#phase-3-the-rubric
source_decisions:
  - DEC-2600 (the rubric is built ONLY from the three signed documents — Labor Contract, NDA/non-compete/IP, Total Rewards & Career Path Appendix — and every rubric item MUST cite its exact source: document id + clause_ref; an item with no citable clause is rejected at write time)
  - DEC-2601 (governance first — TASK-EVAL-002 is a hard dependent of TASK-EVAL-001; no rubric row may be authored before the TASK-EVAL-001 consent/access/retention layer exists, and rubric reads are access-gated by it)
  - DEC-2602 (GENIE/Lumi MAY draft rubric items from the documents but a human MUST approve before an item is effective — the draft→approved gate is HITL; an unapproved item never reaches TASK-EVAL-003)
  - DEC-2603 (the rubric is versioned + effective-dated; a published version is immutable, re-curation produces a new version, and an assessment records which rubric_version it ran against so a past assessment stays interpretable when the contract changes)
  - DEC-2604 (every rubric mutation — draft, edit, approve, publish, supersede — emits a hash-chained memory audit row; the rubric is a governance artifact, not loose config)

eu_ai_act_risk_class: high

language: rust 1.81
service: cyberos/services/eval/
new_files:
  - services/eval/migrations/0002_rubric.sql
  - services/eval/src/rubric/mod.rs
  - services/eval/src/rubric/model.rs
  - services/eval/src/rubric/authoring.rs
  - services/eval/src/rubric/versioning.rs
  - services/eval/src/rubric/draft_genie.rs
  - services/eval/tests/rubric_authoring_test.rs
  - services/eval/tests/rubric_versioning_test.rs
  - services/eval/tests/rubric_rls_test.rs
modified_files:
  - services/eval/src/lib.rs                 # mount the rubric module
  - services/eval/src/audit.rs               # add eval.rubric_* row kinds
allowed_tools:
  - file_read: services/eval/**
  - file_write: services/eval/{src,tests,migrations}/**
  - bash: cd services/eval && cargo test rubric
disallowed_tools:
  - write a rubric_item without a non-null source_doc + clause_ref (per DEC-2600 — every item is clause-cited)
  - mark a rubric_item or rubric_version effective without a human approver_subject_id (per DEC-2602 — the draft→approved gate is HITL)
  - mutate a published rubric_version in place (per DEC-2603 — published versions are immutable; re-curation makes a new version)
  - author any rubric row before the TASK-EVAL-001 governance layer is present (per DEC-2601 — governance first)

effort_hours: 8
subtasks:
  - "0.5h: 0002_rubric.sql migration (rubric + rubric_version + rubric_item tables, RLS, append-only audit triggers, REVOKE update/delete on published rows)"
  - "1.0h: model.rs — Rubric / RubricVersion / RubricItem structs + the closed enums (SourceDoc, ObligationKind, CheckType, VersionState)"
  - "1.5h: authoring.rs — human-curated create/edit/approve/publish flow; clause-citation validation; bilingual VN/EN field pairing"
  - "1.0h: versioning.rs — effective-dating, supersede-not-mutate, resolve_effective(at) → the version live on a date"
  - "1.0h: draft_genie.rs — GENIE DRAFT path: Lumi proposes items from the 3 documents into status=draft only; never effective without human approve"
  - "0.5h: wire eval.rubric_{drafted,edited,approved,published,superseded} audit rows through the memory chain"
  - "1.0h: GET/POST endpoints (author + read), access-gated by TASK-EVAL-001 grants, tenant-scoped by RLS"
  - "1.5h: rubric_authoring_test + rubric_versioning_test + rubric_rls_test (clause-citation enforced, HITL gate, immutability, effective-dating, tenant isolation)"
risk_if_skipped: "Without a structured rubric, TASK-EVAL-003 has nothing to score against — the three contracts stay as prose in three files and any 'assessment' is a gut feel with no defensible basis. Without per-item clause citation, an assessment can claim a person fell short of an obligation it cannot point to in a signed document — exactly the unfounded judgement Vietnam's Labor Code and PDPD make indefensible. Without versioning + effective-dating, amending a contract silently rewrites the standard every past assessment was measured against, so history becomes uninterpretable. Without the human-approval gate, GENIE's first-draft reading of a legal clause becomes the operative standard with no human ever having signed off — the model would be setting the bar it later scores people against."
---

## §1 — Description (BCP-14 normative)

The rubric layer **MUST** turn the three signed employment documents into a structured, versioned, clause-cited framework that TASK-EVAL-003 can evaluate recorded evidence against, with a human approving every item before it is effective. The contract:

1. **MUST** define three tables — `rubric`, `rubric_version`, `rubric_item` — in `services/eval/migrations/0002_rubric.sql`, all tenant-scoped (`tenant_id UUID NOT NULL`) and all under RLS (TASK-AUTH-003). A `rubric` is the named framework for a tenant (e.g. "CyberSkill employment rubric"); a `rubric_version` is one immutable, effective-dated published cut of it; a `rubric_item` is one checkable obligation, working term, KPI, or milestone within a version.

2. **MUST** make every `rubric_item` cite its exact source in the signed documents (DEC-2600):
    - `source_doc TEXT NOT NULL CHECK (source_doc IN ('labor_contract','nda_ip','total_rewards_appendix'))` — the closed set of the three documents and nothing else.
    - `clause_ref TEXT NOT NULL` — the exact clause/article identifier within that document (e.g. `art.3.1`, `§5(b)`, `appendix-B.KPI-2`).
    - `source_quote_vi TEXT` and `source_quote_en TEXT` — the verbatim clause text in each language, so a reviewer and the employee can read the standard at its source.
    - An INSERT or update that leaves `source_doc` or `clause_ref` null **MUST** be rejected (`422 rubric_item_uncited`); an item that cannot name its clause is not a rubric item.

3. **MUST** classify each item by what kind of thing it checks:
    - `item_kind TEXT NOT NULL CHECK (item_kind IN ('obligation','working_term','kpi','career_milestone'))`.
    - For `item_kind='obligation'`, a further `obligation_kind TEXT CHECK (obligation_kind IN ('confidentiality','non_compete','ip_assignment'))` — the three NDA obligation families called out in the brain-evaluation plan. Non-obligation items leave `obligation_kind` null.

4. **MUST** carry, per item, a machine-usable check descriptor that TASK-EVAL-003 consumes:
    - `check_type TEXT NOT NULL CHECK (check_type IN ('evidence_presence','threshold_numeric','attestation','periodic_review','milestone_reached'))` — how the item is evaluated (is there evidence of X; is a number at/above a threshold; is there a signed attestation; was a review done on cadence; was a milestone reached).
    - `check_params JSONB NOT NULL DEFAULT '{}'` — typed parameters for the check (e.g. `{"metric":"on_time_delivery","operator":">=","target":0.9}` for a `threshold_numeric` KPI). The schema of `check_params` is keyed by `check_type`; an unknown shape for a given `check_type` **MUST** be rejected at write time.
    - `weight NUMERIC(5,2) NOT NULL CHECK (weight >= 0)` — the item's contribution to its section's roll-up. Weights are relative within a version, not a promise of a global 0..100.

5. **MUST** be fully bilingual VN/EN (the documents are bilingual, dated 2026-01-01, under Labor Code 45/2019/QH14 + Decree 145/2020). Every human-facing item field **MUST** exist as a `_vi` / `_en` pair: `title_vi`/`title_en`, `description_vi`/`description_en`, plus the `source_quote_vi`/`source_quote_en` of clause 2. Vietnamese is the primary, legally-operative text; English is the working translation. An item missing the `_vi` side **MUST** be rejected (`422 rubric_item_missing_vi`).

6. **MUST** version the rubric and treat a published version as immutable (DEC-2603):
    - `rubric_version` carries `version_no INT`, `state TEXT CHECK (state IN ('draft','approved','published','superseded'))`, `effective_from DATE`, `effective_to DATE` (null = open-ended), `published_at_ns BIGINT`, `published_by_subject_id UUID`.
    - Once `state='published'`, the version's items and its `effective_from` **MUST NOT** be mutated. Re-curation (a contract amendment, a corrected reading) creates a NEW `rubric_version` with `version_no + 1`; the prior version is moved to `superseded` and its `effective_to` is set. A migration **MUST** `REVOKE UPDATE, DELETE` on published `rubric_version` / `rubric_item` rows from the `cyberos_app` role.

7. **MUST** resolve, for any date, the single version that was effective then: `resolve_effective(rubric_id, at_date) → rubric_version` returns the published version whose `[effective_from, effective_to)` contains `at_date`. TASK-EVAL-003 calls this so an assessment for a period uses the standard that was actually in force then, not whatever is current.

8. **MUST** enforce the human-approval (HITL) gate before any item or version becomes effective (DEC-2602):
    - GENIE/Lumi MAY create items in `state='draft'` only (clause 9), and MAY suggest edits, but **MUST NOT** approve or publish.
    - A `rubric_version` transition to `approved` or `published` **MUST** record a human `approver_subject_id UUID NOT NULL` and an `approved_at_ns`. A transition with a null or service-account approver **MUST** be rejected (`403 rubric_requires_human_approver`).
    - Only an item that belongs to a `published` version is visible to TASK-EVAL-003; a `draft`/`approved`-but-unpublished item **MUST NOT** be evaluable.

9. **MUST** support a GENIE DRAFT path that proposes items from the three documents without ever setting the standard (DEC-2602):
    - `draft_genie.rs` calls Lumi (the TASK-EVAL-001-governed analysis path) to read the three documents and propose `rubric_item` rows — each pre-filled with a candidate `source_doc`, `clause_ref`, `item_kind`, `check_type`, and bilingual title/description — into `state='draft'`.
    - Every GENIE-drafted item **MUST** carry `authored_by = 'genie'` and a `genie_confidence NUMERIC(4,3)`; a human-authored item carries `authored_by = 'human'`. The provenance is never erased on later human edit (an `edited_by_subject_id` is added; `authored_by` stays).
    - A GENIE draft that cannot ground an item in a specific clause **MUST** leave it flagged `needs_clause_ref` rather than inventing a citation; the human supplies the clause before approval. The model **MUST NOT** fabricate a `clause_ref` that is not in the document.

10. **MUST** be access-gated by the TASK-EVAL-001 governance layer, not by an access rule this task invents (DEC-2601). Authoring (create/edit/approve/publish) **MUST** require the TASK-EVAL-001 grant for rubric administration (founder + designated rubric admins); reading the rubric **MUST** require a valid grant. The rubric tables **MUST NOT** ship before TASK-EVAL-001 exists.

11. **MUST** emit one hash-chained memory audit row per rubric mutation through `services/eval/src/audit.rs` (DEC-2604): `eval.rubric_drafted`, `eval.rubric_edited`, `eval.rubric_approved`, `eval.rubric_published`, `eval.rubric_superseded`. Each payload carries `{rubric_id, rubric_version_id, version_no, item_id?, actor_subject_id, authored_by?, source_doc?, clause_ref?, trace_id}`. These chain into the same `l1_audit_log` the rest of CyberOS uses (TASK-PROJ-008 / TASK-MEMORY-123 pattern), so the rubric's full curation history is tamper-evident.

12. **MUST** expose a minimal authoring + read API, tenant-scoped and TASK-EVAL-001-gated:
    - `POST /v1/eval/rubrics/{id}/versions` — open a new draft version.
    - `POST /v1/eval/rubrics/{id}/versions/{vid}/items` — add/edit a draft item (clause-citation + bilingual + check-shape validated).
    - `POST /v1/eval/rubrics/{id}/versions/{vid}/draft-from-documents` — trigger the GENIE draft path (clause 9), returns the proposed items in `draft`.
    - `POST /v1/eval/rubrics/{id}/versions/{vid}/approve` and `.../publish` — the HITL transitions (human approver required).
    - `GET /v1/eval/rubrics/{id}?at=<date>` — read the version effective on a date (defaults to today), with all items and their citations.

13. **MUST** validate, at publish time, that the version is coherent before it can become the standard:
    - Every item is clause-cited (clause 2), bilingual (clause 5), and has a check shape valid for its `check_type` (clause 4).
    - `effective_from` does not overlap an existing published version's open interval for the same `rubric` (no two versions effective on the same day).
    - At least one item exists. Publishing an empty version **MUST** be rejected (`422 rubric_version_empty`).
    A publish that fails any check **MUST** return the specific failure and leave the version unpublished.

14. **MUST** keep the rubric independent of any one person — a rubric and its versions describe the *standard*, not an assessment of an individual. Per-person scoring lives in TASK-EVAL-003 and references `rubric_version_id`; this task **MUST NOT** store any per-employee score, evidence, or assessment. (Separation of the standard from the judgement is what lets the same version fairly measure everyone.)

15. **MUST** emit OTel metrics: `eval_rubric_items_total{item_kind,source_doc}` (gauge of items in the current published version), `eval_rubric_publishes_total{result}` (counter; result ∈ ok | rejected), and `eval_rubric_genie_drafts_total{outcome}` (counter; outcome ∈ proposed | needs_clause_ref).

---

## §2 — Why this design (rationale for humans)

**Why clause citation per item (DEC-2600, §1 #2)?** The whole point of building the rubric from the three signed documents is that an evaluation can be defended by pointing at the exact clause it measured against. A rubric item that says "maintains confidentiality" with no `source_doc`/`clause_ref` is an opinion; one that cites `nda_ip / §4.2` with the verbatim text is a measurable obligation the employee agreed to. Rejecting an uncited item at write time keeps the rubric honest — it can only ever assert what a signed document says.

**Why versioned + immutable + effective-dated (DEC-2603, §1 #6 #7)?** Contracts change. If the rubric were mutable config, amending the Total Rewards appendix in July would silently rewrite the standard that the January–June assessments were measured against, and last quarter's review would no longer mean what it said. Immutable published versions plus `resolve_effective(at)` mean every assessment is anchored to the standard that was actually in force then, and the history stays interpretable forever.

**Why the human-approval gate, with GENIE only drafting (DEC-2602, §1 #8 #9)?** Lumi reading a legal clause is a fast first draft, not a ruling. Letting the model's reading become the operative standard with no human sign-off would mean the system sets the bar it later scores people against — the exact thing the brain-evaluation plan's "human in the loop" principle forbids. GENIE proposes into `draft`; a human approves into `published`. The model accelerates the curation; it never owns it.

**Why GENIE must not fabricate a clause_ref (§1 #9)?** A model asked to cite a clause will, if unconstrained, produce a plausible-looking citation that may not exist in the document — the same failure mode seen in the obs-triage local-model proof (it invented a runbook URL by copying an example). The `needs_clause_ref` flag forces the human to supply the real citation rather than rubber-stamping an invented one. A rubric grounded in fabricated citations is worse than no rubric.

**Why bilingual VN/EN with Vietnamese primary (§1 #5)?** The employment documents are bilingual and governed by Vietnamese law; the Vietnamese text is the legally-operative one. Storing both, with `_vi` required, means the standard is readable to the Vietnamese-speaking team in the authoritative language and to others in the working translation, and an assessment can show the employee the clause in the language they signed.

**Why separate the standard from the judgement (§1 #14)?** A rubric describes what "good" and "compliant" mean for everyone; an assessment is one person measured against it. Keeping per-person scoring out of this task (it lives in TASK-EVAL-003, referencing `rubric_version_id`) is what makes the rubric fair — the same version measures everyone, and changing the standard for one person is structurally impossible.

**Why governance-first dependency on TASK-EVAL-001 (DEC-2601, §1 #10)?** The rubric is the spine of a system that affects pay and progression. Building it before the consent, access, and retention layer exists would mean authoring the most sensitive artifact in the platform with no access control and no disclosed basis. TASK-EVAL-001 is a hard `depends_on`, and rubric reads/writes are gated by its grants, so the rubric never exists outside the governance frame.

**Why audit every mutation (DEC-2604, §1 #11)?** The rubric's curation history is itself evidence — who proposed an item, who approved it, when it was published, when it was superseded. Hash-chaining each mutation into `l1_audit_log` (the TASK-PROJ-008 / TASK-MEMORY-123 pattern) means a reviewer can later reconstruct exactly how the standard came to be and prove it was not quietly altered.

**Why publish-time coherence checks (§1 #13)?** A version becomes the standard the moment it is published; a half-finished version (an uncited item, a missing translation, an overlapping effective date) would corrupt every assessment that runs against it. Validating coherence at the publish boundary — not at every keystroke — lets authors work freely in `draft` but guarantees that what becomes operative is complete.

**Why a closed `check_type` enum (§1 #4)?** TASK-EVAL-003 has to know how to evaluate each item mechanically. A free-form check would push interpretation into the evaluation engine per item. A small closed set (evidence presence, numeric threshold, attestation, periodic review, milestone reached) covers the obligations, working terms, KPIs, and milestones in the three documents and keeps the evaluation engine's logic bounded and testable.

---

## §3 — API contract

### Migration

```sql
-- services/eval/migrations/0002_rubric.sql

CREATE TABLE rubric (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    name            TEXT NOT NULL,
    created_at_ns   BIGINT NOT NULL,
    UNIQUE (tenant_id, name)
);

CREATE TABLE rubric_version (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rubric_id               UUID NOT NULL REFERENCES rubric(id),
    tenant_id               UUID NOT NULL,
    version_no              INT  NOT NULL,
    state                   TEXT NOT NULL CHECK (state IN ('draft','approved','published','superseded')),
    effective_from          DATE,
    effective_to            DATE,
    approver_subject_id     UUID,                 -- human; required for approved/published
    approved_at_ns          BIGINT,
    published_by_subject_id UUID,
    published_at_ns         BIGINT,
    UNIQUE (rubric_id, version_no)
);

CREATE TABLE rubric_item (
    id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rubric_version_id UUID NOT NULL REFERENCES rubric_version(id),
    tenant_id         UUID NOT NULL,
    -- citation (DEC-2600): every item names its clause
    source_doc        TEXT NOT NULL CHECK (source_doc IN ('labor_contract','nda_ip','total_rewards_appendix')),
    clause_ref        TEXT NOT NULL,
    source_quote_vi   TEXT,
    source_quote_en   TEXT,
    -- classification
    item_kind         TEXT NOT NULL CHECK (item_kind IN ('obligation','working_term','kpi','career_milestone')),
    obligation_kind   TEXT CHECK (obligation_kind IN ('confidentiality','non_compete','ip_assignment')),
    -- check descriptor consumed by TASK-EVAL-003
    check_type        TEXT NOT NULL CHECK (check_type IN
                      ('evidence_presence','threshold_numeric','attestation','periodic_review','milestone_reached')),
    check_params      JSONB NOT NULL DEFAULT '{}',
    weight            NUMERIC(5,2) NOT NULL CHECK (weight >= 0),
    -- bilingual (DEC, §1 #5): _vi is required, _en is the working translation
    title_vi          TEXT NOT NULL,
    title_en          TEXT,
    description_vi    TEXT,
    description_en    TEXT,
    -- provenance (HITL, §1 #9)
    authored_by       TEXT NOT NULL CHECK (authored_by IN ('human','genie')),
    genie_confidence  NUMERIC(4,3),
    needs_clause_ref  BOOLEAN NOT NULL DEFAULT false,
    edited_by_subject_id UUID
);

CREATE INDEX idx_rubric_version_effective ON rubric_version (rubric_id, effective_from, effective_to);
CREATE INDEX idx_rubric_item_version       ON rubric_item (rubric_version_id);

ALTER TABLE rubric          ENABLE ROW LEVEL SECURITY;
ALTER TABLE rubric_version  ENABLE ROW LEVEL SECURITY;
ALTER TABLE rubric_item     ENABLE ROW LEVEL SECURITY;
CREATE POLICY rubric_tenant_iso         ON rubric         USING (tenant_id = current_setting('app.tenant_id')::uuid);
CREATE POLICY rubric_version_tenant_iso ON rubric_version USING (tenant_id = current_setting('app.tenant_id')::uuid);
CREATE POLICY rubric_item_tenant_iso    ON rubric_item    USING (tenant_id = current_setting('app.tenant_id')::uuid);

-- §1 #6: published versions + their items are immutable to the app role.
-- (Re-curation makes a new version; the cyberos_ops admin role bypasses for corrections-of-record.)
REVOKE UPDATE, DELETE ON rubric_version FROM cyberos_app;
REVOKE UPDATE, DELETE ON rubric_item    FROM cyberos_app;
```

### Rust model

```rust
// services/eval/src/rubric/model.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum SourceDoc { LaborContract, NdaIp, TotalRewardsAppendix }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ItemKind { Obligation, WorkingTerm, Kpi, CareerMilestone }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ObligationKind { Confidentiality, NonCompete, IpAssignment }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum CheckType { EvidencePresence, ThresholdNumeric, Attestation, PeriodicReview, MilestoneReached }

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum VersionState { Draft, Approved, Published, Superseded }

#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct RubricItem {
    pub id:                uuid::Uuid,
    pub rubric_version_id: uuid::Uuid,
    pub source_doc:        SourceDoc,
    pub clause_ref:        String,
    pub source_quote_vi:   Option<String>,
    pub source_quote_en:   Option<String>,
    pub item_kind:         ItemKind,
    pub obligation_kind:   Option<ObligationKind>,
    pub check_type:        CheckType,
    pub check_params:      serde_json::Value,
    pub weight:            rust_decimal::Decimal,
    pub title_vi:          String,
    pub title_en:          Option<String>,
    pub description_vi:    Option<String>,
    pub description_en:    Option<String>,
    pub authored_by:       String,           // "human" | "genie"
    pub genie_confidence:  Option<rust_decimal::Decimal>,
    pub needs_clause_ref:  bool,
}

/// Validate an item before it may be written into a draft version (§1 #2 #4 #5).
pub fn validate_item(it: &RubricItemDraft) -> Result<(), RubricError> {
    if it.clause_ref.trim().is_empty() {
        return Err(RubricError::Uncited);                 // 422 rubric_item_uncited
    }
    if it.title_vi.trim().is_empty() {
        return Err(RubricError::MissingVi);               // 422 rubric_item_missing_vi
    }
    check_params_shape_for(it.check_type, &it.check_params)?; // 422 on unknown shape
    if let ItemKind::Obligation = it.item_kind {
        if it.obligation_kind.is_none() {
            return Err(RubricError::ObligationKindRequired);
        }
    }
    Ok(())
}
```

### Versioning + HITL transitions

```rust
// services/eval/src/rubric/versioning.rs

/// The published version effective on `at` (§1 #7). TASK-EVAL-003 anchors an
/// assessment to whatever standard was actually in force for the period.
pub async fn resolve_effective(
    pool: &sqlx::PgPool, rubric_id: uuid::Uuid, at: chrono::NaiveDate,
) -> Result<RubricVersion, RubricError> {
    sqlx::query_as::<_, RubricVersion>(
        "SELECT * FROM rubric_version
          WHERE rubric_id = $1 AND state = 'published'
            AND effective_from <= $2
            AND (effective_to IS NULL OR effective_to > $2)
          ORDER BY version_no DESC LIMIT 1"
    ).bind(rubric_id).bind(at).fetch_optional(pool).await?
     .ok_or(RubricError::NoEffectiveVersion)
}

/// HITL publish (§1 #8 #13). A human approver is mandatory; the version must be
/// coherent; the effective interval must not overlap a live published version.
pub async fn publish_version(
    tx: &mut sqlx::PgTransaction<'_>,
    version_id: uuid::Uuid,
    approver_subject_id: uuid::Uuid,        // MUST be a human subject, not a service account
    effective_from: chrono::NaiveDate,
) -> Result<(), RubricError> {
    if is_service_account(approver_subject_id).await? {
        return Err(RubricError::RequiresHumanApprover);   // 403 rubric_requires_human_approver
    }
    assert_version_coherent(tx, version_id).await?;       // every item cited, bilingual, check-shape ok, non-empty
    assert_no_effective_overlap(tx, version_id, effective_from).await?;

    // supersede whatever is currently open-ended for this rubric, then publish this one
    supersede_current_open(tx, version_id, effective_from).await?;
    sqlx::query(
        "UPDATE rubric_version
            SET state='published', effective_from=$2,
                approver_subject_id=$3, approved_at_ns=$4,
                published_by_subject_id=$3, published_at_ns=$4
          WHERE id=$1 AND state IN ('draft','approved')"
    ).bind(version_id).bind(effective_from).bind(approver_subject_id)
     .bind(now_ns()).execute(&mut **tx).await?;

    audit::emit_tx(tx, "eval.rubric_published", json!({
        "rubric_version_id": version_id, "actor_subject_id": approver_subject_id,
        "effective_from": effective_from, "trace_id": current_trace_id(),
    })).await?;
    Ok(())
}
```

### GENIE draft path (proposes only)

```rust
// services/eval/src/rubric/draft_genie.rs

/// Lumi reads the three signed documents and PROPOSES items into state='draft'
/// (§1 #9). It never approves, never publishes, and never fabricates a clause_ref:
/// an item it cannot ground is flagged needs_clause_ref for a human to fill.
pub async fn draft_from_documents(
    tx: &mut sqlx::PgTransaction<'_>,
    version_id: uuid::Uuid,
    docs: &SignedDocSet,                    // the 3 governed documents (TASK-EVAL-001 access)
) -> Result<Vec<RubricItem>, RubricError> {
    let proposed = lumi::propose_rubric_items(docs).await?;   // TASK-EVAL-001-governed analysis call
    let mut out = Vec::new();
    for p in proposed {
        let needs_ref = p.clause_ref.trim().is_empty();       // model could not ground it -> human supplies
        let item = insert_draft_item(tx, version_id, RubricItemDraft {
            source_doc:       p.source_doc,
            clause_ref:       p.clause_ref,                    // verbatim from the doc, or "" -> needs_clause_ref
            item_kind:        p.item_kind,
            check_type:       p.check_type,
            check_params:     p.check_params,
            title_vi:         p.title_vi,  title_en: p.title_en,
            authored_by:      "genie".into(),
            genie_confidence: Some(p.confidence),
            needs_clause_ref: needs_ref,
            ..Default::default()
        }).await?;
        audit::emit_tx(tx, "eval.rubric_drafted", json!({
            "rubric_version_id": version_id, "item_id": item.id,
            "authored_by": "genie", "source_doc": item.source_doc,
            "clause_ref": item.clause_ref, "needs_clause_ref": needs_ref,
        })).await?;
        out.push(item);
    }
    Ok(out)   // all in 'draft'; a human approves + publishes before TASK-EVAL-003 ever sees them
}
```

---

## §4 — Acceptance criteria

1. **Tables created, tenant-scoped, RLS on** — migration creates `rubric`, `rubric_version`, `rubric_item`; each has a tenant-isolation policy; tenant A's rubric is invisible to tenant B (AC for §1 #1).
2. **Uncited item rejected** — POST a `rubric_item` with empty `clause_ref` → `422 rubric_item_uncited`; with a `clause_ref` → accepted (AC for §1 #2).
3. **Source doc constrained to the three** — `source_doc='handbook'` → CHECK violation; `source_doc='nda_ip'` → accepted (AC for §1 #2).
4. **Obligation requires obligation_kind** — `item_kind='obligation'` with null `obligation_kind` → rejected; with `confidentiality` → accepted (AC for §1 #3).
5. **Check shape validated per check_type** — `threshold_numeric` with `{"target":0.9,"operator":">="}` accepted; `threshold_numeric` with `{}` → 422 (AC for §1 #4).
6. **`_vi` required** — item with `title_vi=""` → `422 rubric_item_missing_vi`; `_en` absent is allowed (AC for §1 #5).
7. **Published version immutable** — attempt UPDATE on a published `rubric_item` as `cyberos_app` → permission denied (AC for §1 #6).
8. **Re-curation makes a new version** — edit-after-publish creates `version_no + 1`, prior version → `superseded` with `effective_to` set (AC for §1 #6).
9. **`resolve_effective` returns the in-force version** — two published versions (Jan–Jun, Jul–open); `resolve_effective(_, 2026-03-15)` returns the Jan–Jun version (AC for §1 #7).
10. **Human approver mandatory** — `publish_version` with a service-account approver → `403 rubric_requires_human_approver`; with a human subject → published (AC for §1 #8).
11. **Draft/unpublished item not evaluable** — TASK-EVAL-003's effective-version read returns only items of a `published` version; a `draft` item is absent (AC for §1 #8).
12. **GENIE drafts into draft only** — `draft-from-documents` inserts items with `authored_by='genie'`, `state` of the version stays `draft`; no item becomes effective without a later human publish (AC for §1 #9).
13. **GENIE never fabricates a clause_ref** — a proposed item with no grounding is stored `needs_clause_ref=true` with empty `clause_ref`, not an invented citation; publish is blocked until a human fills it (AC for §1 #9 #13).
14. **Access-gated by TASK-EVAL-001** — a caller without the rubric-admin grant gets `403` on authoring; without any grant gets `403` on read (AC for §1 #10).
15. **Audit row per mutation** — draft, edit, approve, publish, supersede each emit the matching `eval.rubric_*` row, chained into `l1_audit_log` with the actor and citation (AC for §1 #11).
16. **Publish blocks on incoherence** — a version with one uncited item → publish `422` naming the item; an empty version → `422 rubric_version_empty`; overlapping `effective_from` → `409` (AC for §1 #13).
17. **No per-person data stored** — schema inspection confirms no employee/score/evidence column on any rubric table; per-person scoring is TASK-EVAL-003's (AC for §1 #14).
18. **OTel metrics emitted** — publishing a 4-item version sets `eval_rubric_items_total`; a rejected publish increments `eval_rubric_publishes_total{result="rejected"}` (AC for §1 #15).

---

## §5 — Verification

```rust
#[tokio::test]
async fn uncited_item_is_rejected() {
    let env = EvalTestEnv::new().await;
    let (rid, vid) = env.new_draft_version().await;
    let res = env.post_item(vid, ItemDraft { clause_ref: "".into(), ..good_obligation() }).await;
    assert_eq!(res.status_code(), 422);
    assert_eq!(res.error_code(), "rubric_item_uncited");

    let ok = env.post_item(vid, ItemDraft { clause_ref: "§4.2".into(), ..good_obligation() }).await;
    assert!(ok.is_ok());
}

#[tokio::test]
async fn published_version_is_immutable_to_app_role() {
    let env = EvalTestEnv::new().await;
    let vid = env.publish_minimal_version(env.founder()).await;
    let res: Result<_, _> = sqlx::query("UPDATE rubric_item SET weight = 99 WHERE rubric_version_id = $1")
        .bind(vid).execute(env.pool_as("cyberos_app")).await;
    assert!(res.is_err());   // REVOKE UPDATE denies it
}

#[tokio::test]
async fn resolve_effective_picks_the_in_force_version() {
    let env = EvalTestEnv::new().await;
    let rid = env.rubric_with_versions(&[
        ("2026-01-01", Some("2026-07-01")),   // v1
        ("2026-07-01", None),                 // v2 open-ended
    ]).await;
    let v = resolve_effective(&env.pool, rid, "2026-03-15".parse().unwrap()).await.unwrap();
    assert_eq!(v.version_no, 1);
    let v2 = resolve_effective(&env.pool, rid, "2026-09-01".parse().unwrap()).await.unwrap();
    assert_eq!(v2.version_no, 2);
}

#[tokio::test]
async fn publish_requires_a_human_approver() {
    let env = EvalTestEnv::new().await;
    let vid = env.new_coherent_draft_version().await;
    let svc = env.service_account_subject();
    let err = publish_version_tx(&env.pool, vid, svc, "2026-01-01".parse().unwrap()).await.unwrap_err();
    assert!(matches!(err, RubricError::RequiresHumanApprover));

    let human = env.founder();
    assert!(publish_version_tx(&env.pool, vid, human, "2026-01-01".parse().unwrap()).await.is_ok());
}

#[tokio::test]
async fn genie_draft_never_fabricates_a_clause_ref() {
    let env = EvalTestEnv::new().await;
    let (_, vid) = env.new_draft_version().await;
    env.lumi.stub_proposal_without_clause();   // model returns an item it cannot ground
    let items = draft_from_documents_tx(&env.pool, vid, &env.signed_docs()).await.unwrap();
    let ungrounded = items.iter().find(|i| i.title_vi.contains("ungrounded")).unwrap();
    assert!(ungrounded.needs_clause_ref);
    assert!(ungrounded.clause_ref.is_empty());                 // not an invented citation
    // publish is blocked while a needs_clause_ref item remains
    let err = env.publish(vid, env.founder()).await.unwrap_err();
    assert_eq!(err.error_code(), "rubric_item_uncited");
}

#[tokio::test]
async fn rubric_is_tenant_isolated() {
    let env = EvalTestEnv::new().await;
    let vid_a = env.as_tenant("A").publish_minimal_version(env.founder()).await;
    let rows = env.as_tenant("B").list_rubric_items(vid_a).await.unwrap();
    assert!(rows.is_empty());   // RLS hides tenant A's rubric from tenant B
}
```

---

## §6 — Implementation skeleton

(Model, versioning, GENIE draft path, and migration above. `authoring.rs` wires the create/edit/approve flow over `model::validate_item`; `lib.rs` mounts the routes behind the TASK-EVAL-001 access guard.)

---

## §7 — Dependencies

- **TASK-EVAL-001** — governance, consent, access grants, retention. Hard dependency: the rubric tables ship only after it exists, and rubric authoring/reads are gated by its grants (DEC-2601).
- **TASK-EVAL-003** — the evaluation engine that consumes the published rubric: it calls `resolve_effective(at)` and reads `rubric_item` rows to score evidence. This task `blocks` it.
- **TASK-MEMORY-123** — brain recall; the same audit-chain + memory substrate the rubric mutations chain into.
- **TASK-AUTH-003** — RLS / roles; the tenant isolation and the `cyberos_app` vs `cyberos_ops` role split the immutability REVOKE relies on.
- **TASK-PROJ-008 / TASK-MEMORY-123** — the established hash-chained audit-row pattern the `eval.rubric_*` rows follow.

---

## §8 — Example payloads

```json
{
  "kind": "eval.rubric_published",
  "payload": {
    "rubric_id":          "rub-...",
    "rubric_version_id":  "rv-...",
    "version_no":         2,
    "actor_subject_id":   "stephen-...",
    "effective_from":     "2026-07-01",
    "trace_id":           "0af..."
  }
}
```

```json
{
  "item": {
    "source_doc":     "nda_ip",
    "clause_ref":     "§4.2",
    "source_quote_vi":"Người lao động phải giữ bí mật...",
    "source_quote_en":"The employee shall keep confidential...",
    "item_kind":      "obligation",
    "obligation_kind":"confidentiality",
    "check_type":     "evidence_presence",
    "check_params":   { "signal": "no_unauthorized_disclosure" },
    "weight":         10.0,
    "title_vi":       "Bảo mật thông tin",
    "title_en":       "Confidentiality",
    "authored_by":    "human"
  }
}
```

---

## §9 — Open questions

Resolved by Stephen's 2026-06-29 decisions (governance first; GENIE drafts, human approves; versioned + effective-dated; clause-cited). Deferred:
- Whether `check_params` schemas get a formal JSON-Schema registry per `check_type` (slice 2) — this task validates shape by `check_type` in code; a declared schema registry is additive.
- A diff view between two rubric versions (which clause changed, which item was added) for the TASK-EVAL-004 console — surfacing only, a later additive screen.
- Multi-document rubrics beyond the three signed documents (e.g. a code-of-conduct addendum) — out of scope until such a document is signed; the `source_doc` enum is deliberately closed to the three for now.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Item written with no `clause_ref` | `validate_item` | 422 rubric_item_uncited | Author cites the clause |
| `source_doc` outside the three | CHECK constraint | 422 / constraint error | Author uses one of the three docs |
| Obligation item without `obligation_kind` | `validate_item` | rejected | Author sets confidentiality/non_compete/ip_assignment |
| `check_params` shape wrong for `check_type` | `check_params_shape_for` | 422 | Author fixes the params |
| `title_vi` missing | `validate_item` | 422 rubric_item_missing_vi | Author supplies Vietnamese title |
| Publish with a service-account approver | `is_service_account` guard | 403 rubric_requires_human_approver | A human approves |
| Publish an empty version | `assert_version_coherent` | 422 rubric_version_empty | Add at least one item |
| Publish a version with an uncited item | coherence check | 422 naming the item | Cite or remove the item |
| Overlapping `effective_from` | `assert_no_effective_overlap` | 409 | Adjust the effective date |
| UPDATE on a published row by app role | REVOKE / permission | denied | Make a new version instead |
| GENIE fabricates a clause_ref | `needs_clause_ref` flag + publish coherence | item flagged; publish blocked | Human supplies the real clause |
| GENIE proposes an item for a non-existent doc | `source_doc` CHECK | rejected on insert | Human corrects/drops it |
| Rubric authored before TASK-EVAL-001 exists | dependency gate (DEC-2601) | task not shippable | Ship TASK-EVAL-001 first |
| Read by a caller without a grant | TASK-EVAL-001 access guard | 403 | Grant access per governance |
| `resolve_effective` finds no in-force version | NoEffectiveVersion | error to TASK-EVAL-003 | Publish a version covering the period |
| Two published versions overlap (data drift) | publish-time overlap guard prevents | n/a at write; sweep flags if introduced by admin | Admin corrects-of-record |
| Audit emit fails mid-tx | sqlx tx rollback | mutation not committed | Caller retries |
| Cross-tenant read attempt | RLS | 0 rows | None — by design |
| `weight` negative | CHECK (weight >= 0) | constraint error | Author fixes weight |
| Bilingual drift (`_en` says something `_vi` does not) | human review at approve | caught by approver | Approver corrects translation |
| OTel exporter down | buffered then dropped | logged | Restore TASK-OBS-001 |
| Admin (`cyberos_ops`) edits a published row | allowed by design for corrections-of-record | emits `eval.rubric_edited` audit | Audited; reviewable |

---

## §11 — Implementation notes

- The three tables separate the framework (`rubric`), the immutable published cut (`rubric_version`), and the checkable items (`rubric_item`) so that re-curation is "new version", not "mutate rows" — the same append-only discipline as the TASK-PROJ-008 history layer.
- `resolve_effective(at)` is the single seam TASK-EVAL-003 uses; everything else about versioning is internal. Anchoring an assessment to the version in force on its period date is what keeps a past assessment interpretable after a contract amendment.
- The HITL gate is enforced at two places: the model can only ever write `state='draft'` items (it has no path to `publish`), and `publish_version` rejects a non-human approver. Defence in depth: even a mis-wired caller cannot publish without a human subject.
- `authored_by` provenance is sticky — a later human edit adds `edited_by_subject_id` but never rewrites `authored_by`, so "this item started as a GENIE draft" stays visible in the audit trail forever.
- The `needs_clause_ref` flag is the anti-fabrication mechanism. The model is instructed to leave `clause_ref` empty when it cannot ground an item, and publish coherence (clause 13) refuses to let an uncited item become operative. A grounded-looking but invented citation is the failure we design against; an honest gap that a human fills is the intended path.
- Vietnamese is primary because the documents are governed by Vietnamese law and the Vietnamese text is the operative one; `_vi NOT NULL`, `_en` optional captures that without forcing a translation to exist before the standard can.
- The `cyberos_app` REVOKE makes published rows immutable to the runtime; `cyberos_ops` retains the ability to correct a record (rare, audited via `eval.rubric_edited`). This mirrors the TASK-PROJ-008 role split.
- `check_type` is intentionally a small closed enum so TASK-EVAL-003's evaluation logic is bounded: five evaluation shapes cover obligations (evidence presence / attestation), working terms (periodic review), KPIs (numeric threshold), and career milestones (milestone reached). A new shape is a deliberate schema change, not a free-form field.
- `weight` is relative within a version; the roll-up math (how item weights compose into a section and an overall draft score) belongs to TASK-EVAL-003, not here. This task stores the weight; it does not compute a score.
- No per-person column exists anywhere in this schema by design (clause 14). If a future need tempts adding one, that is the signal to put it in TASK-EVAL-003 against `rubric_version_id`, keeping the standard and the judgement separate.
- The audit rows reuse the `services/eval/src/audit.rs` emitter (the TASK-MEMORY-123 chain), so the rubric's curation history lives in the same tamper-evident ledger as every other CyberOS event — a reviewer reconstructs how the standard was set from one place.
- Effective-dating uses `[effective_from, effective_to)` half-open intervals so adjacent versions meet exactly on a boundary date with no gap and no overlap; the publish-time overlap guard enforces it.
- This is a high-risk task under the EU AI Act framing (the rubric feeds an employment-evaluation system); see the AI Risk Assessment section. The model's role is bounded to drafting, every operative item is human-approved, and the standard is clause-grounded and auditable.

---

## AI Risk Assessment

### Risk classification

High-risk under the EU AI Act framing: the rubric is the standard a later system (TASK-EVAL-003) uses to assess employees, and employment evaluation that can influence pay and progression is a high-risk use. This task does not itself score anyone, but it defines what "good" and "compliant" mean, so it is held to the high-risk bar: human oversight, traceability, and grounding in the signed source.

### Data sources

The rubric is built only from the three signed employment documents — the Labor Contract, the NDA/non-compete/IP agreement, and the Total Rewards & Career Path Appendix (bilingual VN/EN, dated 2026-01-01, under Labor Code 45/2019/QH14 and Decree 145/2020). It stores no per-employee data, no evidence, and no scores. Reading the documents to draft items happens through the TASK-EVAL-001-governed analysis path, under that task's consent and access basis.

### Human oversight

GENIE/Lumi MAY draft rubric items but MUST NOT approve or publish them (EU AI Act Article 14). A human approver — the founder or a designated rubric admin holding the TASK-EVAL-001 grant — must approve and publish before any item becomes the operative standard; a service-account approver is rejected (clause 8). Every item is clause-cited so the human can check the model's reading against the exact signed text, and an item the model cannot ground is flagged `needs_clause_ref` for the human to fill rather than published on an invented citation.

### Traceability

Every rubric mutation — draft, edit, approve, publish, supersede — emits a hash-chained memory audit row (`eval.rubric_*`) carrying the actor, the version, and the citation, into the same tamper-evident `l1_audit_log` as the rest of CyberOS. Published versions are immutable and effective-dated, so the exact standard in force on any date, and the full history of how it was curated, is reconstructable and provable. The `authored_by` provenance keeps GENIE-drafted vs human-authored items distinguishable forever.

### Failure modes

The model fabricating a clause citation is caught by `needs_clause_ref` plus the publish-time coherence check, which refuses to make an uncited item operative (clause 13). The model's reading becoming the standard without a human is prevented structurally — it has no path past `draft` (clause 8). A contract amendment silently rewriting the past is prevented by immutable, effective-dated versions (clause 6, clause 7). In every case the safe state is "no operative standard changed without a human grounding it in the signed text".

---

*End of TASK-EVAL-002.*
