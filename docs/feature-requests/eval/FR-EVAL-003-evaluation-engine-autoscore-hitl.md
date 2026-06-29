---
template: engineering-spec@1
id: FR-EVAL-003
title: "Evaluation engine — GENIE evidence-linked auto-scoring against the FR-EVAL-002 rubric with a mandatory human-in-the-loop gate before any assessment is final"
module: EVAL
priority: MUST
status: draft
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-06-29
shipped: null
memory_chain_hash: null

author: "@stephen"
department: engineering
ai_authorship: assisted
feature_type: internal_tooling
eu_ai_act_risk_class: high
client_visible: false

related_frs: [FR-EVAL-001, FR-EVAL-002, FR-EVAL-004, FR-MEMORY-123, FR-CUO-204, FR-AUTH-003, FR-PROJ-008]
depends_on: [FR-MEMORY-123, FR-EVAL-002]
blocks: [FR-EVAL-004]

source_decisions:
  - DEC-2510 (evaluation is ACCESS-RESTRICTED + CONTRACT-DISCLOSED: an assessment MUST NOT be drafted for a subject who has not acknowledged the FR-EVAL-001 monitoring notice; outputs are visible only to founder + the subject's managers + the subject)
  - DEC-2511 (GENIE auto-scores against the FR-EVAL-002 rubric AND a human reviewer MUST approve before any assessment is final — both, never only the model)
  - DEC-2512 (HITL is a HARD GATE for consequential outcomes: anything affecting pay, progression, or employment MUST carry an explicit human approval and MUST NOT be auto-applied; the model assists, it never decides those)
  - DEC-2513 (every score MUST cite the specific MEMORY evidence events AND the rubric clause it measured against — no unsourced scores; an uncited score is a defect, not a draft)
  - DEC-2514 (model wrong / hallucinated / offline → fallback is "no score, flag for full human review", never a silent or fabricated score)
  - DEC-2515 (the final assessment AND every state change land in l1_audit_log: who scored, who approved/changed/rejected, what evidence, what rubric version, what changed)

language: rust 1.81 (engine + state machine) + python 3.12 (GENIE inference orchestration)
service: cyberos/services/eval/ + cyberos/services/ai-gateway/ (GENIE / Lumi)
new_files:
  - services/eval/migrations/0001_assessments.sql
  - services/eval/migrations/0002_assessment_scores.sql
  - services/eval/migrations/0003_assessment_state_events.sql
  - services/eval/migrations/0004_assessment_rebuttals.sql
  - services/eval/src/lib.rs
  - services/eval/src/state.rs                  # AssessmentState machine: draft → pending_review → approved|changed|rejected
  - services/eval/src/scoring/mod.rs            # ScoreRow, EvidenceRef, citation-coverage invariant
  - services/eval/src/scoring/citation.rs       # every score cites ≥1 evidence event + exactly 1 rubric clause
  - services/eval/src/hitl.rs                   # mandatory-reviewer gate + consequential-outcome hard gate
  - services/eval/src/access.rs                 # FR-EVAL-001 consent check + RLS visibility (founder|manager|self)
  - services/eval/src/audit.rs                  # l1_audit_log emit per state change (eval.* kinds)
  - services/eval/src/handlers.rs               # HTTP: draft / review / approve / change / reject / rebut / get
  - services/eval/src/metrics.rs               # OTel: drafted, approval/override/reject rate, citation coverage, fallback rate
  - services/eval/genie/score_subject.py        # GENIE orchestration: recall → map-to-rubric → draft scores (via ai-gateway)
  - services/eval/genie/prompts/score_v1.md     # the scoring prompt (evidence-grounded, refuse-if-unsupported)
  - services/eval/genie/fairness.py             # protected-attribute blinding + group-disparity check
  - services/eval/tests/state_machine_test.rs
  - services/eval/tests/citation_coverage_test.rs
  - services/eval/tests/hitl_gate_test.rs
  - services/eval/tests/access_consent_test.rs
  - services/eval/tests/audit_chain_test.rs
  - services/eval/genie/tests/test_score_subject.py
  - services/eval/genie/tests/test_fallback_no_score.py
  - services/eval/genie/tests/test_fairness_blinding.py
modified_files:
  - services/ai-gateway/src/policy.rs           # register the `eval.score` route: spend cap, residency, ZDR=on, fallback=no-score
  - deploy/vps/docker-compose.yml               # add the eval service (single-origin, behind AUTH)
allowed_tools:
  - file_read: services/eval/**
  - file_read: services/ai-gateway/src/policy.rs
  - file_write: services/eval/{src,genie,tests,migrations}/**
  - bash: cd services/eval && cargo test
  - bash: cd services/eval/genie && pytest
disallowed_tools:
  - persist an assessment as `final`/`approved` without a human reviewer subject_id (per DEC-2511 / DEC-2512 — HITL is the gate, not a setting)
  - write a ScoreRow with zero EvidenceRef or zero rubric_clause_id (per DEC-2513 — an uncited score is rejected at the boundary)
  - draft an assessment for a subject with no acknowledged FR-EVAL-001 notice (per DEC-2510)
  - emit a fabricated/default numeric score when GENIE is unsupported/offline (per DEC-2514 — the only legal fallback is "no score, flag for human")
  - auto-apply a consequential outcome (pay/progression/employment) from a model output (per DEC-2512)
  - call any model except via the ai-gateway `eval.score` route with ZDR on (per DEC-2515 / FR-AI residency+ZDR contract)

effort_hours: 14
sub_tasks:
  - "1.0h: 0001..0004 migrations (assessment, assessment_score, assessment_state_event, assessment_rebuttal; RLS; append-only state log)"
  - "2.0h: state.rs — AssessmentState machine (draft → pending_review → approved|changed|rejected) with legal-transition table + reviewer-required edges"
  - "2.0h: scoring/citation.rs — EvidenceRef + rubric_clause_id invariant; reject any ScoreRow that fails citation coverage at the boundary"
  - "1.5h: access.rs — FR-EVAL-001 consent gate (no notice → no draft) + RLS visibility (founder | subject's manager chain | self)"
  - "2.0h: hitl.rs — mandatory-reviewer gate; consequential-outcome hard gate (pay/progression/employment require explicit human approval, never auto-apply)"
  - "1.5h: genie/score_subject.py — recall (FR-MEMORY-123) → map evidence to FR-EVAL-002 clauses → draft scores via ai-gateway eval.score route (ZDR on)"
  - "1.0h: genie/prompts/score_v1.md — evidence-grounded prompt that refuses to score a clause it cannot ground, returns `unsupported` not a guess"
  - "1.0h: genie/fairness.py — blind protected attributes before scoring; post-hoc group-disparity check that flags for human review"
  - "1.0h: audit.rs — eval.* memory audit emit per state change (drafted/approved/changed/rejected/rebutted/fallback_no_score) into l1_audit_log"
  - "1.0h: metrics.rs — assessments_drafted, approval/override/reject rate, citation_coverage, fallback_no_score rate, disparity_flag count"

risk_if_skipped: "Without a mandatory HITL gate, a model output could silently become a pay or progression decision — the single highest legal, ethical, and trust risk in the whole BRAIN workstream, and a banned 'unacceptable risk' pattern under disclosed-monitoring governance (FR-EVAL-001) and an EU-AI-Act high-risk obligation. Without citation coverage, scores are unfalsifiable assertions the subject cannot contest and a reviewer cannot check. Without the consent gate, the system evaluates people who never acknowledged the notice (a PDPD/Labor-Code breach). Without the no-score fallback, an offline or hallucinating model fabricates a number that looks authoritative. Without the audit chain, there is no defensible record of who decided what on what evidence — exactly the record a wrongful-termination or grievance review demands."
---

## §1 — Description (BCP-14 normative)

The evaluation engine **MUST** turn captured platform evidence into an *evidence-linked, human-approved* assessment of a consented subject against the FR-EVAL-002 rubric. GENIE (Lumi, on the ai-gateway) drafts; a human decides. The model assists a decision — it never makes one that affects a person's pay, progression, or employment. The contract:

1. **MUST** run on a cadence AND on demand. A scheduled run (default: per review cycle, configurable per tenant) and an explicit `POST /v1/eval/assessments` MUST both produce the same artifact through the same path. The cadence MUST NOT bypass any gate in this section.

2. **MUST** gate every draft on FR-EVAL-001 consent. Before GENIE is invoked for a subject, the engine MUST verify the subject has an *acknowledged* FR-EVAL-001 monitoring notice on record (`access::has_acknowledged_notice(subject_id, tenant_id)`). No acknowledgement → `403 not_consented`, no recall, no draft, and a `eval.draft_refused_no_consent` audit row. This is **ACCESS-RESTRICTED + CONTRACT-DISCLOSED** (DEC-2510).

3. **MUST** retrieve evidence via FR-MEMORY-123 brain recall only. For a consented subject, the engine MUST pull relevant work-interaction events (with provenance: event id, kind, occurred-at, source module, tenant) from MEMORY recall. It MUST NOT read private-life data, keystroke/screen surveillance, or any non-platform source. Recall scope MUST be the subject's own platform interactions within the review window.

4. **MUST** map evidence to rubric items and draft an assessment. GENIE MUST, per rubric clause from the active FR-EVAL-002 rubric version, select the supporting evidence events and draft a score with a rationale that quotes/refers to those events. The output is a set of `ScoreRow`s, one per scored rubric clause.

5. **MUST** enforce citation coverage at the boundary (DEC-2513). Every `ScoreRow` MUST carry: (a) `rubric_clause_id` (exactly one clause, from the pinned `rubric_version`), and (b) `evidence_refs` (≥ 1 `EvidenceRef`, each a real MEMORY event id resolvable in the subject's recall set). A `ScoreRow` with zero evidence refs or no rubric clause MUST be rejected with `422 uncited_score` and MUST NOT persist. There are no unsourced scores.

6. **MUST** produce a DRAFT only — never a final result. A GENIE run creates an `assessment` in state `draft`, then transitions it to `pending_review`. The legal states are exactly: `draft → pending_review → {approved | changed | rejected}` (DEC-2511). No other terminal state exists. `approved`, `changed`, and `rejected` are the only states reachable solely by a human reviewer.

7. **MUST** enforce a MANDATORY human-in-the-loop gate (DEC-2511, DEC-2512). An assessment MUST NOT become final (`approved` or `changed`) without a `reviewer_subject_id` distinct from the subject, recorded on the transition. The transition into any final state MUST be initiated by a human actor through `POST /v1/eval/assessments/:id/{approve|change|reject}`; the GENIE service identity MUST NOT be accepted as the reviewer. There is no configuration, flag, or fast-path that finalizes an assessment without a human.

8. **MUST** treat consequential outcomes as a HARD GATE (DEC-2512). Any field of the assessment marked `consequential` (a recommendation that affects pay, progression, level, bonus, PIP, or continued employment) MUST be `null`/inert until a human reviewer sets it on an `approve`/`change` transition with `consequential_ack = true`. The engine MUST NOT auto-apply a consequential outcome to any downstream system (payroll, HRIS, level/title), and MUST NOT expose a consequential recommendation as actionable while the assessment is `draft` or `pending_review`. The model proposes; only an approving human's explicit act makes it real.

9. **MUST** let the subject view and respond to their own assessment. Once an assessment is `pending_review` or later, the subject MUST be able to read it (their own, in full, with evidence and rationale) and submit a `rebuttal` (free text + optional clause-level disagreements) via `POST /v1/eval/assessments/:id/rebut`. The rebuttal MUST be captured immutably, surfaced to the reviewer before they finalize, and recorded in the audit chain. A pending rebuttal MUST block transition to `approved` until the reviewer has seen it (reviewer sets `rebuttal_considered = true`).

10. **MUST** write the final result AND every state change to `l1_audit_log` (DEC-2515) via the FR-MEMORY chain. Each transition emits one append-only audit row: `eval.assessment_drafted`, `eval.assessment_pending_review`, `eval.assessment_approved`, `eval.assessment_changed`, `eval.assessment_rejected`, `eval.assessment_rebutted`, `eval.fallback_no_score`, `eval.draft_refused_no_consent`. Each row MUST carry `{assessment_id, subject_id, actor_subject_id, rubric_version, prior_state, new_state, evidence_event_ids, score_digest, change_summary, trace_id}`. The chain is the defensible record.

11. **MUST** run GENIE via the ai-gateway with spend caps + residency + ZDR (DEC-2515). All model calls MUST route through the ai-gateway `eval.score` policy: per-tenant spend cap enforced, data residency honored (no cross-region egress of personal data), zero-data-retention (ZDR) on so the provider does not retain or train on the evidence. A direct provider call bypassing the gateway MUST be impossible from this service (no provider keys in the eval service).

12. **MUST** fall back to "no score, flag for full human review" on any model failure (DEC-2514). If GENIE is offline, times out, returns malformed output, or returns a score for a clause it cannot ground in evidence, the engine MUST NOT fabricate or default a number. It MUST mark the affected clause(s) `score = null, status = needs_human_review`, emit `eval.fallback_no_score`, and route the whole assessment to a human with the failure reason. The safe state is "no score", reached by flagging, never by guessing.

13. **MUST** refuse-to-score at the prompt boundary. The `score_v1` prompt MUST instruct GENIE to return `unsupported` for any rubric clause it cannot ground in the recalled evidence, rather than inferring a score from absence of evidence or from the subject's identity. An `unsupported` return maps to clause status `needs_human_review`, not to a low score.

14. **MUST** apply bias/fairness guardrails. Before scoring, `fairness.py` MUST blind protected attributes (name, gender, age, ethnicity, nationality, disability, and any field marked protected in the tenant config) from the evidence text passed to the model, scoring on behavior and output, not identity. After a batch, a group-disparity check MUST compare score distributions across protected groups and, on a configured threshold breach, flag the batch for human fairness review (`eval.disparity_flagged`) — the flag informs a human; it does not auto-adjust scores.

15. **MUST** provide an appeal path beyond the rebuttal. A subject MUST be able to escalate a finalized assessment to an appeal (`POST /v1/eval/assessments/:id/appeal`) that routes to a second human reviewer (not the original reviewer) and is recorded as `eval.assessment_appealed`. An appeal MUST be possible on any `approved`/`changed` assessment within the tenant-configured appeal window.

16. **MUST** RLS-enforce access (FR-AUTH-003 + FR-EVAL-001 access rules). An assessment row MUST be visible only to: the founder, the subject's manager chain, and the subject themselves (their own). No peer, no unrelated manager, no cross-tenant read. RLS MUST be the spine; the `access.rs` visibility check is defense-in-depth on top. Consequential recommendations MUST be visible to reviewers/founder only until finalized.

17. **MUST** pin the rubric version per assessment. Each assessment records the exact `rubric_version` (FR-EVAL-002) it was scored against, so re-reading an old assessment shows what "good" meant at scoring time. A new rubric version MUST NOT silently re-interpret past scores.

18. **MUST** make the score store append-only-on-finalize. `assessment_score` rows for a finalized assessment MUST NOT be mutated; a reviewer "change" creates a new score-revision linked to the prior, preserving the GENIE draft alongside the human-changed value (the diff is the point — "GENIE said 3, manager changed to 4 because …").

19. **MUST** record reviewer rationale on `change`/`reject`. A `change` or `reject` transition MUST carry a human `reviewer_rationale` (non-empty); a finalization that overrides GENIE without a stated reason MUST be refused with `422 rationale_required`. The override rate and its reasons are a first-class fairness signal.

20. **MUST** emit OTel metrics: `eval_assessments_drafted_total{tenant}`, `eval_assessments_finalized_total{outcome}` (outcome ∈ approved | changed | rejected), `eval_override_rate` (changed ÷ finalized), `eval_citation_coverage_ratio` (scores-with-valid-citations ÷ total scores; MUST be 1.0 by invariant — any dip is an alarm), `eval_fallback_no_score_total`, `eval_disparity_flagged_total`, `eval_rebuttals_total`, `eval_appeals_total`.

---

## §2 — Why this design (rationale for humans)

**Why a draft-only model output with a mandatory human gate (DEC-2511, DEC-2512)?** This is the line between a useful assistant and an unaccountable automated decision about a person's livelihood. The governance plan (Phase 4, "Doing the monitoring responsibly") is explicit: *"Lumi drafts and surfaces evidence; a person decides anything that affects pay, progression, or employment. Never let the model auto-decide those."* The state machine encodes that as structure, not policy: there is literally no transition into a final state that the GENIE identity can take, and consequential fields are inert until a human acts. A reviewer can't be skipped because there's no edge that skips them.

**Why citation coverage as a boundary invariant (DEC-2513)?** An evaluation a person cannot contest is not an evaluation — it's an accusation. Every score that cites the specific events and the specific rubric clause is falsifiable: the subject can look at the same events and argue, the reviewer can check the inference, and an appeal has something concrete to examine. Rejecting an uncited score at the write boundary (not "warning" on it) is what makes "no unsourced scores" a guarantee rather than an aspiration. It also defangs the model's biggest failure mode — confident assertion without basis — because the structure won't store it.

**Why the consent gate before recall (DEC-2510)?** Disclosed monitoring is the defensible path; covert monitoring is the risky one. Vietnam's PDPD (13/2023/ND-CP) expects a lawful basis, notice, and purpose limitation, and the Labor Code (45/2019/QH14) governs the relationship the monitoring sits inside. Gating *recall itself* on an acknowledged notice means the system structurally cannot evaluate someone who was never told — the notice isn't a checkbox after the fact, it's the precondition for the pipeline running at all.

**Why the "no score, flag for human" fallback (DEC-2514)?** A fabricated score is worse than no score because it looks authoritative. When the model is offline or can't ground a clause, the only safe output is an explicit gap routed to a person — the same philosophy as FR-CUO-204's "the safe state is 'no change applied'." Defaulting to a low score on missing evidence would punish people for the model's blind spots; defaulting to a high score would launder a non-answer. Both are wrong; `needs_human_review` is the truthful state.

**Why route exclusively through the ai-gateway with ZDR (DEC-2515)?** The evidence is sensitive employment data about real people. ZDR-on means the model provider doesn't retain or train on it; residency means it doesn't leave the permitted region; the spend cap means a runaway batch can't burn the budget. Keeping provider keys out of the eval service makes the gateway the only door — there's no side channel for a future change to accidentally leak the evidence to an unbounded provider.

**Why the subject sees their record and can rebut + appeal (§1 #9, #15)?** Transparency here is a feature, not an afterthought (governance plan, Phase 5). A person who can read the evidence behind their score, disagree in writing before it's finalized, and escalate to a second reviewer afterward is a person who trusts the system enough to keep using it — which raises data quality, which makes the evaluations better. The rebuttal blocking finalization-until-seen guarantees the reviewer can't rubber-stamp around the disagreement.

**Why blind protected attributes + a disparity check (§1 #14)?** The contracts define performance and compliance; identity is not one of the inputs. Blinding name/gender/age/ethnicity before the model scores keeps the assessment on behavior and output. The post-hoc disparity check is a smoke detector: if approved scores skew across protected groups beyond a threshold, a human looks — the system flags, it never silently "corrects" scores (which would be its own bias).

**Why append-only-on-finalize with revisions, not in-place edits (§1 #18)?** The diff between what GENIE drafted and what the human decided is the most valuable artifact for fairness auditing and for improving the rubric. "GENIE said 3, manager changed to 4 because the client email shows the deadline moved" is a record that teaches. Mutating in place would destroy exactly the signal that makes the human-in-the-loop loop auditable.

**Why reviewer rationale is mandatory on override (§1 #19)?** A high override rate without reasons could mean the rubric is wrong, the model is wrong, or a reviewer is rubber-stamping or freelancing. Forcing a stated reason on every `change`/`reject` turns the override rate into a diagnosable signal and creates the defensible "why" a grievance review will ask for.

**Why pin the rubric version (§1 #17)?** Rubrics evolve. An assessment scored against v3 must keep meaning v3 forever, or every historical review silently rewrites itself when the rubric changes. Pinning makes the record stable and the comparison honest.

**Why a dedicated `services/eval` Rust+Python split?** The state machine, the citation invariant, the access/consent gate, and the audit emit are correctness-critical and benefit from Rust's type system and the existing per-tenant RLS+sqlx patterns (mirrors FR-PROJ-008, FR-CHAT). The GENIE inference orchestration (recall → prompt → parse) is Python because that's where the ai-gateway client and the prompt live (mirrors the CUO/modules layout). The boundary is clean: Python proposes scores, Rust validates and persists them — and Rust is where every gate lives, so a Python bug can't finalize anything.

---

## §3 — API contract

### Migrations

```sql
-- services/eval/migrations/0001_assessments.sql

CREATE TYPE assessment_state AS ENUM
    ('draft', 'pending_review', 'approved', 'changed', 'rejected');

CREATE TABLE assessment (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    subject_id          UUID NOT NULL,                       -- the evaluated person
    review_cycle        TEXT NOT NULL,                       -- e.g. '2026-H1'
    rubric_version      TEXT NOT NULL,                       -- pinned FR-EVAL-002 version (§1 #17)
    state               assessment_state NOT NULL DEFAULT 'draft',
    drafted_by          TEXT NOT NULL,                       -- always the GENIE service identity for the draft
    reviewer_subject_id UUID,                                -- the human reviewer; NULL until pending_review is acted on
    consequential_ack   BOOLEAN NOT NULL DEFAULT FALSE,      -- §1 #8 hard gate
    rebuttal_considered  BOOLEAN NOT NULL DEFAULT FALSE,     -- §1 #9 block
    created_at_ns       BIGINT NOT NULL,
    finalized_at_ns     BIGINT,
    CHECK (subject_id <> reviewer_subject_id)                -- a person cannot review themselves (§1 #7)
);
CREATE INDEX idx_assessment_subject ON assessment (subject_id, review_cycle);
CREATE INDEX idx_assessment_state   ON assessment (tenant_id, state);

ALTER TABLE assessment ENABLE ROW LEVEL SECURITY;
-- §1 #16 — visible to founder, the subject's manager chain, and the subject.
CREATE POLICY assessment_visibility ON assessment
    USING (
        tenant_id = current_setting('app.tenant_id')::uuid
        AND (
            current_setting('app.role') = 'founder'
            OR subject_id = current_setting('app.subject_id')::uuid
            OR eval_is_manager_of(current_setting('app.subject_id')::uuid, subject_id)
        )
    );
```

```sql
-- services/eval/migrations/0002_assessment_scores.sql

CREATE TABLE assessment_score (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id       UUID NOT NULL REFERENCES assessment(id),
    rubric_clause_id    TEXT NOT NULL,                       -- exactly one clause (§1 #5)
    score               NUMERIC,                             -- NULL == needs_human_review (§1 #12)
    clause_status       TEXT NOT NULL DEFAULT 'scored'
                          CHECK (clause_status IN ('scored','needs_human_review','unsupported')),
    genie_score         NUMERIC,                             -- the model's original draft (§1 #18)
    rationale           TEXT NOT NULL,
    evidence_event_ids  UUID[] NOT NULL,                     -- ≥1 real MEMORY event id (§1 #5)
    revision_of         UUID REFERENCES assessment_score(id),-- human "change" links to the prior (§1 #18)
    reviewer_rationale  TEXT,                                -- required on a human change/reject (§1 #19)
    is_consequential    BOOLEAN NOT NULL DEFAULT FALSE,      -- §1 #8
    tenant_id           UUID NOT NULL,
    -- the boundary invariant, enforced in the DB too: a scored clause must cite evidence.
    CHECK (clause_status <> 'scored' OR array_length(evidence_event_ids, 1) >= 1)
);
CREATE INDEX idx_score_assessment ON assessment_score (assessment_id);

ALTER TABLE assessment_score ENABLE ROW LEVEL SECURITY;
CREATE POLICY score_tenant_isolation ON assessment_score
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
-- finalized scores are immutable; a change writes a new revision row (§1 #18).
REVOKE UPDATE, DELETE ON assessment_score FROM cyberos_app;
```

```sql
-- services/eval/migrations/0003_assessment_state_events.sql  (append-only audit spine, §1 #10)
CREATE TABLE assessment_state_event (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id       UUID NOT NULL REFERENCES assessment(id),
    seq                 BIGSERIAL NOT NULL,
    prior_state         assessment_state,
    new_state           assessment_state NOT NULL,
    actor_subject_id    UUID,                                -- NULL only for the GENIE draft; never NULL on a finalize
    actor_kind          TEXT NOT NULL CHECK (actor_kind IN ('genie','human','system')),
    reason              TEXT,
    memory_row_id       TEXT NOT NULL,                       -- l1_audit_log chain anchor (FR-PROJ-008 pattern)
    chain_anchor        TEXT NOT NULL,
    occurred_at_ns      BIGINT NOT NULL,
    tenant_id           UUID NOT NULL,
    -- a finalize MUST be human-driven (§1 #7): the type system + this check both guard it.
    CHECK (new_state NOT IN ('approved','changed','rejected') OR actor_kind = 'human')
);
CREATE UNIQUE INDEX idx_state_event_assessment_seq ON assessment_state_event (assessment_id, seq);
REVOKE UPDATE, DELETE ON assessment_state_event FROM cyberos_app;
```

```sql
-- services/eval/migrations/0004_assessment_rebuttals.sql  (§1 #9 / #15)
CREATE TABLE assessment_rebuttal (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assessment_id       UUID NOT NULL REFERENCES assessment(id),
    subject_id          UUID NOT NULL,                       -- must equal assessment.subject_id
    kind                TEXT NOT NULL CHECK (kind IN ('rebuttal','appeal')),
    body                TEXT NOT NULL,
    clause_disputes     JSONB,                               -- optional clause-level disagreements
    created_at_ns       BIGINT NOT NULL,
    tenant_id           UUID NOT NULL
);
CREATE INDEX idx_rebuttal_assessment ON assessment_rebuttal (assessment_id);
REVOKE UPDATE, DELETE ON assessment_rebuttal FROM cyberos_app;   -- immutable (§1 #9)
```

### Rust — the state machine (the gate lives here)

```rust
// services/eval/src/state.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "assessment_state", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AssessmentState {
    Draft, PendingReview, Approved, Changed, Rejected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorKind { Genie, Human, System }

/// The ONLY legal transitions. There is no edge into a final state for `Genie`.
pub fn legal_transition(from: AssessmentState, to: AssessmentState, actor: ActorKind) -> bool {
    use AssessmentState::*;
    use ActorKind::*;
    match (from, to, actor) {
        // GENIE drafts, then hands to humans — and stops there.
        (Draft, PendingReview, Genie) => true,
        (Draft, PendingReview, System) => true,         // cadence runner
        // Only a human can finalize (§1 #7, DEC-2511/2512).
        (PendingReview, Approved, Human) => true,
        (PendingReview, Changed,  Human) => true,
        (PendingReview, Rejected, Human) => true,
        _ => false,
    }
}

pub struct FinalizeGuard;
impl FinalizeGuard {
    /// Hard gate (§1 #7 #8). Refuses to finalize without a distinct human reviewer,
    /// and refuses to apply a consequential outcome without an explicit ack.
    pub fn check(
        actor: ActorKind,
        reviewer_subject_id: Option<uuid::Uuid>,
        subject_id: uuid::Uuid,
        has_consequential: bool,
        consequential_ack: bool,
        reviewer_rationale: Option<&str>,
        to: AssessmentState,
    ) -> Result<(), EvalError> {
        if matches!(to, AssessmentState::Approved | AssessmentState::Changed | AssessmentState::Rejected) {
            if actor != ActorKind::Human {
                return Err(EvalError::HumanRequired);          // GENIE may never finalize.
            }
            match reviewer_subject_id {
                Some(r) if r != subject_id => {}
                _ => return Err(EvalError::ReviewerRequired),  // distinct human reviewer mandatory.
            }
            if has_consequential && !consequential_ack {
                return Err(EvalError::ConsequentialAckRequired); // §1 #8 hard gate.
            }
            if matches!(to, AssessmentState::Changed | AssessmentState::Rejected)
                && reviewer_rationale.map(str::trim).unwrap_or("").is_empty() {
                return Err(EvalError::RationaleRequired);        // §1 #19.
            }
        }
        Ok(())
    }
}
```

### Rust — citation coverage (the no-unsourced-score boundary)

```rust
// services/eval/src/scoring/citation.rs
pub struct ScoreRow {
    pub rubric_clause_id: String,
    pub score: Option<f64>,               // None == needs_human_review
    pub clause_status: ClauseStatus,      // Scored | NeedsHumanReview | Unsupported
    pub rationale: String,
    pub evidence_refs: Vec<EvidenceRef>,  // each is a real MEMORY event id
}

pub struct EvidenceRef { pub event_id: uuid::Uuid, pub kind: String, pub occurred_at_ns: i64 }

/// DEC-2513: a *scored* clause must cite exactly one rubric clause and ≥1 grounded evidence event.
/// `unsupported` / `needs_human_review` are allowed to carry no score, but never a fabricated one.
pub fn validate_citation(row: &ScoreRow, recall_set: &RecallSet) -> Result<(), EvalError> {
    if row.rubric_clause_id.trim().is_empty() {
        return Err(EvalError::UncitedScore("missing rubric_clause_id"));
    }
    match row.clause_status {
        ClauseStatus::Scored => {
            if row.score.is_none() {
                return Err(EvalError::UncitedScore("scored clause with null score"));
            }
            if row.evidence_refs.is_empty() {
                return Err(EvalError::UncitedScore("scored clause cites no evidence"));
            }
            // every cited event must actually be in the subject's recall set (§1 #5, no hallucinated ids).
            for e in &row.evidence_refs {
                if !recall_set.contains(e.event_id) {
                    return Err(EvalError::UncitedScore("evidence id not in recall set (hallucinated citation)"));
                }
            }
            Ok(())
        }
        // a non-scored clause must NOT carry a number (§1 #12 #13) — that would be a smuggled guess.
        ClauseStatus::NeedsHumanReview | ClauseStatus::Unsupported => {
            if row.score.is_some() {
                return Err(EvalError::UncitedScore("unsupported clause must not carry a score"));
            }
            Ok(())
        }
    }
}
```

### Python — GENIE orchestration (drafts only; every model call via the gateway)

```python
# services/eval/genie/score_subject.py
async def score_subject(subject_id: str, tenant_id: str, cycle: str, rubric_version: str) -> DraftAssessment:
    # 1. consent gate is enforced upstream in Rust (access.rs); GENIE is only reached for consented subjects.
    # 2. recall evidence with provenance (FR-MEMORY-123). Platform interactions only.
    recall = await memory_recall(subject_id, tenant_id, window=cycle)        # [{event_id, kind, text, occurred_at, src}]
    rubric = await load_rubric(tenant_id, rubric_version)                    # FR-EVAL-002 clauses

    # 3. blind protected attributes before the model ever sees the text (§1 #14).
    blinded = fairness.blind_protected_attributes(recall, tenant_id)

    rows: list[ScoreRow] = []
    for clause in rubric.clauses:
        # 4. score one clause, grounded ONLY in the recalled evidence.
        out = await ai_gateway.complete(
            route="eval.score",            # spend cap + residency + ZDR=on enforced by the gateway (§1 #11)
            prompt=render(SCORE_V1, clause=clause, evidence=blinded),
            tenant_id=tenant_id,
        )
        parsed = parse_score(out)
        if parsed is None or parsed.unsupported:
            # 5. refuse-to-score → needs_human_review, NEVER a fabricated number (§1 #12 #13, DEC-2514).
            rows.append(ScoreRow.needs_human_review(clause.id, reason=parsed.reason if parsed else "unparseable"))
            await emit_audit("eval.fallback_no_score", subject_id, clause_id=clause.id, reason="unsupported_or_offline")
            continue
        # the model returns the evidence ids it used; Rust re-validates them against the recall set (§1 #5).
        rows.append(ScoreRow.scored(clause.id, parsed.score, parsed.rationale, parsed.evidence_event_ids))

    return DraftAssessment(subject_id, cycle, rubric_version, rows)   # persisted by Rust in state=draft → pending_review
```

The Python side never writes a final state and never persists a score directly — it returns a draft to the Rust service, which runs `validate_citation` on every row and `FinalizeGuard` on every transition before anything is stored. A model failure or a parser failure yields `needs_human_review`, not an exception that could be swallowed into a default.

---

## §4 — Acceptance criteria

1. **Consent gate blocks unconsented draft** — subject with no acknowledged FR-EVAL-001 notice → `POST /assessments` returns `403 not_consented`; no recall call made; `eval.draft_refused_no_consent` audit row written. (§1 #2)
2. **Consented draft produces pending_review** — consented subject → GENIE drafts → assessment lands in `pending_review` with ≥1 `ScoreRow`. (§1 #4 #6)
3. **Every scored clause cites evidence + a rubric clause** — each `clause_status='scored'` row has `rubric_clause_id` set and `array_length(evidence_event_ids) >= 1`. (§1 #5)
4. **Uncited score rejected at boundary** — a `ScoreRow` with empty `evidence_refs` and `clause_status='scored'` → `422 uncited_score`; row not persisted. (§1 #5)
5. **Hallucinated citation rejected** — a `ScoreRow` citing an `event_id` not in the recall set → `422 uncited_score`. (§1 #5)
6. **GENIE cannot finalize** — a transition `pending_review → approved` attempted with `actor_kind='genie'` → `EvalError::HumanRequired`; state unchanged. (§1 #7)
7. **Reviewer must be distinct from subject** — `approve` with `reviewer_subject_id == subject_id` → `EvalError::ReviewerRequired`; DB CHECK also rejects. (§1 #7)
8. **Mandatory reviewer recorded on finalize** — successful `approve` records a non-null `reviewer_subject_id` and an `assessment_state_event` with `actor_kind='human'`. (§1 #7)
9. **Consequential hard gate** — assessment with `is_consequential=true` score → `approve` with `consequential_ack=false` → `EvalError::ConsequentialAckRequired`; the consequential field stays inert and is not emitted to any downstream. (§1 #8)
10. **No auto-apply of consequential outcome** — at no state does the engine call a payroll/HRIS/level downstream; verified by asserting no such egress on any transition. (§1 #8)
11. **Subject can read own assessment + rebut** — subject GETs their own `pending_review` assessment in full; `POST /rebut` stores an immutable `assessment_rebuttal(kind='rebuttal')`; `eval.assessment_rebutted` audit row. (§1 #9)
12. **Pending rebuttal blocks approval until seen** — `approve` while an unconsidered rebuttal exists and `rebuttal_considered=false` → refused; succeeds after reviewer sets it true. (§1 #9)
13. **Every state change audit-chained** — draft, pending_review, approved/changed/rejected each emit an `assessment_state_event` with a valid `chain_anchor` resolvable in `l1_audit_log`. (§1 #10)
14. **All model calls route via the gateway** — eval service holds no provider keys; `score_subject` calls only `ai_gateway.complete(route='eval.score', ...)`; a direct provider call path does not exist. (§1 #11)
15. **ZDR + residency + cap enforced on eval.score** — the `eval.score` policy in `ai-gateway/src/policy.rs` sets ZDR on, residency pinned, spend cap present; a call over cap is rejected by the gateway. (§1 #11)
16. **Offline model → no-score fallback** — GENIE unreachable → affected clauses `score=null, clause_status='needs_human_review'`; `eval.fallback_no_score` row; assessment routed to human; no number fabricated. (§1 #12, DEC-2514)
17. **Unsupported clause carries no score** — GENIE returns `unsupported` for a clause → `clause_status='unsupported'`, `score=null`; `validate_citation` rejects any attempt to attach a number. (§1 #13)
18. **Protected attributes blinded pre-scoring** — evidence text passed to the model has name/gender/age/ethnicity redacted per tenant config; verified by inspecting the gateway request payload. (§1 #14)
19. **Disparity check flags, never adjusts** — a batch breaching the disparity threshold emits `eval.disparity_flagged` and routes to human review; no score is auto-changed by the check. (§1 #14)
20. **Appeal routes to a second reviewer** — `POST /appeal` on an `approved` assessment creates `assessment_rebuttal(kind='appeal')`, `eval.assessment_appealed`, and assigns a reviewer ≠ original. (§1 #15)
21. **RLS visibility** — assessment visible to founder, subject, and subject's manager; a peer/unrelated-manager GET returns 0 rows; cross-tenant returns 0 rows. (§1 #16)
22. **Rubric version pinned** — assessment records `rubric_version`; bumping the live FR-EVAL-002 rubric does not change a finalized assessment's scores. (§1 #17)
23. **Change writes a revision, not an in-place edit** — reviewer `change` of a score creates a new `assessment_score` with `revision_of` set and `genie_score` preserving the original draft value. (§1 #18)
24. **Override without rationale refused** — `change`/`reject` with empty `reviewer_rationale` → `422 rationale_required`. (§1 #19)
25. **Metrics emitted** — `eval_citation_coverage_ratio == 1.0` on a healthy batch; `eval_override_rate`, `eval_fallback_no_score_total`, `eval_disparity_flagged_total` increment as exercised. (§1 #20)

---

## §5 — Verification

```rust
#[tokio::test]
async fn genie_cannot_finalize_an_assessment() {
    let env = TestEnv::new().await;
    let a = env.draft_assessment_pending_review().await;     // GENIE-drafted, in pending_review
    let res = transition(&env.pool, a.id, AssessmentState::Approved, ActorKind::Genie, /*reviewer*/ None, None).await;
    assert!(matches!(res, Err(EvalError::HumanRequired)));    // the gate holds
    assert_eq!(env.state_of(a.id).await, AssessmentState::PendingReview);  // unchanged
}

#[tokio::test]
async fn scored_clause_without_evidence_is_rejected() {
    let env = TestEnv::new().await;
    let recall = env.recall_set_with(&["e1","e2"]).await;
    let row = ScoreRow::scored("clause.duty.1", 3.0, "looks fine", vec![]);  // zero evidence
    assert!(matches!(validate_citation(&row, &recall), Err(EvalError::UncitedScore(_))));
}

#[tokio::test]
async fn hallucinated_evidence_id_is_rejected() {
    let env = TestEnv::new().await;
    let recall = env.recall_set_with(&["e1","e2"]).await;
    let ghost = uuid::Uuid::new_v4();
    let row = ScoreRow::scored("clause.duty.1", 4.0, "cited a ghost", vec![EvidenceRef::just(ghost)]);
    assert!(matches!(validate_citation(&row, &recall), Err(EvalError::UncitedScore(_))));
}

#[tokio::test]
async fn consequential_outcome_needs_explicit_human_ack() {
    let env = TestEnv::new().await;
    let a = env.pending_review_with_consequential().await;
    // approve without the ack -> refused; the consequential field stays inert.
    let res = FinalizeGuard::check(ActorKind::Human, Some(env.manager()), a.subject_id,
                                   true, /*ack*/ false, Some("ok"), AssessmentState::Approved);
    assert!(matches!(res, Err(EvalError::ConsequentialAckRequired)));
}

#[tokio::test]
async fn unconsented_subject_is_never_drafted() {
    let env = TestEnv::new().await;
    let subj = env.subject_without_notice().await;
    let res = draft_assessment(&env, subj).await;
    assert!(matches!(res, Err(EvalError::NotConsented)));
    assert_eq!(env.recall_call_count().await, 0);            // recall never happened
    assert!(env.audit_has("eval.draft_refused_no_consent", subj).await);
}
```

```python
# services/eval/genie/tests/test_fallback_no_score.py
async def test_offline_model_yields_needs_human_review_not_a_number(monkeypatch):
    monkeypatch.setattr(ai_gateway, "complete", _raise_offline)   # GENIE unreachable
    draft = await score_subject("subj-1", "ten-1", "2026-H1", "rubric-v3")
    assert all(r.score is None for r in draft.rows)               # NO fabricated score
    assert all(r.clause_status == "needs_human_review" for r in draft.rows)
    assert audit_emitted("eval.fallback_no_score")

async def test_unsupported_clause_never_carries_a_score():
    out = parse_score('{"unsupported": true, "reason": "no evidence for this duty"}')
    assert out.unsupported and out.score is None
```

---

## §6 — Implementation skeleton

(Migrations, state machine, citation boundary, GENIE orchestration above. The Rust service owns every gate; Python only drafts.)

---

## §7 — Dependencies

- **FR-MEMORY-123** — brain recall supplies the evidence events with provenance; the recall set is the universe a score may cite (§1 #3 #5). *(depends_on)*
- **FR-EVAL-002** — the rubric (clauses + source contract clause) the engine scores against; `rubric_version` is pinned per assessment (§1 #4 #17). *(depends_on)*
- **FR-EVAL-001** — governance/consent/access/retention; the consent gate (§1 #2) and the founder|manager|self visibility (§1 #16) are this FR's enforcement of FR-EVAL-001's rules.
- **FR-CUO-204 / the ai-gateway** — GENIE (Lumi) is the analysis engine; the `eval.score` route reuses the gateway's spend cap + residency + ZDR machinery (§1 #11).
- **FR-AUTH-003** — per-tenant RLS spine for assessment visibility (§1 #16).
- **FR-PROJ-008** — the chain_anchor / memory-row linkage pattern reused for `assessment_state_event` (§1 #10).
- **FR-EVAL-004** — manager/employee views consume this engine's output. *(blocks)*

---

## §8 — Example payloads

```json
{
  "kind": "eval.assessment_changed",
  "payload": {
    "assessment_id":      "asm-7e57...",
    "subject_id":         "usr-alice-...",
    "actor_subject_id":   "usr-manager-bob-...",
    "actor_kind":         "human",
    "rubric_version":     "rubric-v3",
    "prior_state":        "pending_review",
    "new_state":          "changed",
    "evidence_event_ids": ["evt-0a3...", "evt-9c1..."],
    "score_digest":       "sha256:1b9e...",
    "change_summary":     "clause duty.3: GENIE drafted 3, reviewer changed to 4 — client email evt-9c1 shows the deadline moved, so the slip was external",
    "reviewer_rationale": "Deadline change was the client's, not Alice's; the duty was met under the revised date.",
    "consequential_ack":  false,
    "trace_id":           "0af..."
  }
}
```

```json
{
  "kind": "eval.fallback_no_score",
  "payload": {
    "assessment_id":   "asm-7e57...",
    "subject_id":      "usr-alice-...",
    "rubric_clause_id":"compliance.ip.2",
    "reason":          "ai_gateway_unreachable",
    "resolution":      "routed_to_human_review",
    "trace_id":        "0af..."
  }
}
```

---

## §9 — Open questions

Deferred (do not block slice 1):
- Cadence scheduler wiring (reuse the scheduled-task runner that wakes FR-CUO-204) — slice 2.
- Calibration across reviewers (normalizing two managers who score differently) — a fairness concern for slice 3; out of scope here beyond the disparity flag.
- Multi-language rationale (Vietnamese + English) for the subject view — FR-EVAL-004's surface concern.
- Evidence "freshness" weighting (recency-decay from FR-MEMORY-113) as a recall input — additive, slice 2.

Resolved in-spec: HITL is mandatory and structural (DEC-2511/2512); the fallback is no-score (DEC-2514); citation coverage is a boundary invariant (DEC-2513); consent gates recall (DEC-2510).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| GENIE offline / timeout | gateway error / deadline | clauses → `needs_human_review`; `eval.fallback_no_score`; routed to human | Human scores manually; retry when gateway back |
| GENIE returns a score it can't ground | prompt returns `unsupported`; `validate_citation` | clause `unsupported`, no number stored | Human reviews the clause |
| GENIE cites an event id not in recall | `validate_citation` recall-set check | `422 uncited_score`; row dropped | Re-draft; flag possible prompt drift |
| GENIE identity attempts a finalize | `FinalizeGuard` + DB CHECK (`actor_kind`) | `HumanRequired`; state unchanged | None — by design; a human must act |
| Reviewer == subject | `FinalizeGuard` + DB CHECK (`subject_id <> reviewer`) | `ReviewerRequired` | Assign a different reviewer |
| Consequential outcome without ack | `FinalizeGuard` | `ConsequentialAckRequired`; field inert | Reviewer sets `consequential_ack` deliberately |
| Override with no rationale | `FinalizeGuard` | `422 rationale_required` | Reviewer states a reason |
| Unconsented subject drafted | `access::has_acknowledged_notice` | `403 not_consented`; no recall; audit row | Obtain FR-EVAL-001 acknowledgement first |
| Subject's rebuttal ignored | `rebuttal_considered` gate | `approve` blocked until reviewer sees it | Reviewer reads rebuttal, sets flag |
| Disparity across protected groups | post-hoc `fairness.py` check | `eval.disparity_flagged`; human fairness review | Human investigates; rubric/model adjusted by humans |
| Provider key leaks into eval service | code review + no-key invariant | build/config check fails | Remove key; all calls go via gateway only |
| Cross-region egress of evidence | gateway residency policy | call rejected at gateway | Fix residency config; never bypass gateway |
| Spend cap exceeded mid-batch | gateway cost ledger | over-cap call rejected | Resume next window; raise cap deliberately |
| In-place edit of a finalized score | `REVOKE UPDATE/DELETE` on `assessment_score` | permission denied | Change writes a revision row instead |
| Audit emit fails mid-transition | sqlx tx rollback | state change not committed | Caller retries; no orphan state |
| RLS bypass attempt | RLS policy | 0 rows | None — by design |
| Peer reads someone's assessment | RLS visibility policy | 0 rows | None |
| Rubric bumped after finalize | pinned `rubric_version` | old assessment unchanged | None — by design |
| Malformed GENIE JSON | `parse_score` returns None | `needs_human_review`; `eval.fallback_no_score` | Human scores; fix prompt/parser |
| Two reviewers act concurrently | first transition wins; second sees non-`pending_review` | second gets 409 | Second reviewer re-reads current state |
| Appeal assigned to original reviewer | second-reviewer constraint | rejected; reassigned | System picks a different reviewer |
| Recall returns private-life data (capture bug) | platform-only recall scope + provenance filter | non-platform events excluded before scoring | Fix capture (FR-EVAL-001 scope) |
| Consequential field exposed pre-finalize | visibility rule (reviewer/founder only) | not shown to subject while draft | None — by design |
| Subject denied view of own record | RLS self-visibility | subject always sees own | Fix RLS if regressed |
| OTel exporter down | buffered then dropped | metrics gap logged | Restore FR-OBS-001 |
| `eval_citation_coverage_ratio` < 1.0 | metric invariant breach | SEV-1 alarm (an uncited score escaped) | Investigate boundary bypass immediately |

---

## §11 — Implementation notes

- The state machine is the load-bearing safety control: `legal_transition` has **no edge** into `approved`/`changed`/`rejected` for `ActorKind::Genie`, and the DB `CHECK (new_state NOT IN (...) OR actor_kind='human')` enforces the same thing one layer down. A bug in one layer cannot finalize an assessment because the other layer still refuses.
- `FinalizeGuard::check` is pure and exhaustively unit-tested; every finalize path calls it, so the consequential-ack and rationale-required gates can't be forgotten by a new handler.
- Citation coverage is enforced in three places on purpose: the prompt (`score_v1` refuses to guess), the Rust boundary (`validate_citation` rejects), and the DB (`CHECK (clause_status <> 'scored' OR array_length(evidence_event_ids,1) >= 1)`). Defense in depth because an uncited score is the failure that most undermines trust.
- "No score, flag for human" is the single fallback for every model failure mode — offline, timeout, malformed output, unsupported clause. The engine has no code path that writes a numeric score the model did not ground; a `None` score is a first-class value, not an error to be defaulted away.
- GENIE never persists. Python returns a `DraftAssessment`; Rust validates and writes it. This keeps every gate (consent, citation, finalize, audit) in the typed Rust service where it's testable, and means a Python exception degrades to `needs_human_review` rather than corrupting state.
- The `eval.score` gateway route reuses the existing AI-gateway policy machinery (spend cap, residency, ZDR) rather than re-implementing it — this FR adds the route and its policy, not a second cost/residency engine. ZDR-on is non-negotiable for employment evidence; the route refuses to dispatch to a provider that doesn't honor it.
- `genie_score` on `assessment_score` always preserves the model's original draft even after a human `change`, so the GENIE-vs-human diff is queryable forever. The override rate and its rationales are the primary fairness telemetry; a high rate is a prompt/rubric signal, not a reviewer failing.
- Protected-attribute blinding happens in Python before the gateway call so the model literally never receives identity. The blind list is per-tenant (jurisdictions differ on what's protected) and auditable. The disparity check is post-hoc and advisory: it flags for a human, never mutates a score, because an auto-correcting fairness pass is just a different bias.
- The rebuttal is immutable and must be seen before approval (`rebuttal_considered`). This is deliberately a hard block, not a soft nudge: a person's written disagreement cannot be approved around without a reviewer acknowledging it.
- The appeal path requires a *second* human reviewer, distinct from the one who finalized, mirroring a normal grievance escalation. It's the human safety net beyond the in-cycle rebuttal.
- This is an EU-AI-Act high-risk system (employment evaluation): the conformance posture is disclosed-purpose (FR-EVAL-001 notice), human oversight (Article 14 — the mandatory reviewer and the appeal), transparency to the subject (own-record view + rebuttal), data governance (recall provenance, ZDR, residency), and logging (the audit chain). Stephen reviews and signs off before this FR leaves `draft`, per the high-risk rule.
- `eval_is_manager_of` is a tenant-scoped helper over the org/manager-chain data (FR-AUTH directory); it backs the RLS visibility policy and the appeal-reviewer selection. If the org graph is unavailable, the safe default is *deny* (founder + self only), never *allow*.
- Cadence and on-demand drafts share one code path (§1 #1) so a gate can never be present on one and missing on the other; the scheduler is just a different caller of the same `draft_assessment` entrypoint.

---

## AI Risk Assessment

Required and central — this is an EU-AI-Act **high-risk** system (Annex III: AI used in employment, for evaluating people). EU AI Act Articles 5–7 and 14. All three subsections are load-bearing, not perfunctory.

### Data Sources

The engine grounds its scores in two governed sources and nothing else:

- **Captured platform work-interactions**, retrieved via FR-MEMORY-123 brain recall, each carrying provenance (event id, kind, occurred-at, source module, tenant). Scope is the subject's own platform activity within the review window — chat, module usage, task/project/document activity, sign-ins. It explicitly **excludes** private life, keystroke logging, and screen surveillance (governance plan, "proportionality and minimization"). Recall only runs for a subject who has acknowledged the FR-EVAL-001 monitoring notice (§1 #2).
- **The three signed documents** — the labor contract, the NDA / non-compete / IP agreement, and the total-rewards and career-path appendix — but only as encoded into the FR-EVAL-002 rubric (the clauses + their source contract clause). The engine scores *against* this rubric; it does not read the raw contracts at inference time, and the `rubric_version` is pinned per assessment (§1 #17).

No model weights are trained or fine-tuned on this data. Personal-data handling: evidence reaches a model only via the ai-gateway `eval.score` route with zero-data-retention on and data residency pinned, so the provider neither retains nor trains on it and it does not leave the permitted region (§1 #11). Protected attributes (name, gender, age, ethnicity, nationality, disability) are blinded from the evidence text before the model ever receives it (§1 #14).

### Human Oversight

A human is the decision-maker at every consequential point (EU AI Act Article 14):

- **Mandatory reviewer.** No assessment becomes final without a distinct human `reviewer_subject_id`; the GENIE service identity has no transition into `approved`/`changed`/`rejected`, enforced by the state-machine edge table, a DB `CHECK`, and `FinalizeGuard` (§1 #6 #7).
- **Consequential hard gate.** Anything affecting pay, progression, level, bonus, PIP, or continued employment stays inert until a human sets it with `consequential_ack = true`; the engine never auto-applies it to payroll/HRIS/level (§1 #8). The model assists; only an approving human's explicit act makes it real.
- **Employee rebuttal.** The subject reads their own assessment in full (evidence + rationale) and can rebut in writing; a pending rebuttal hard-blocks approval until the reviewer has seen it (§1 #9).
- **Appeal.** A finalized assessment can be escalated to a second human reviewer, distinct from the first (§1 #15).
- **Override accountability + audit.** A human `change`/`reject` requires a stated rationale (§1 #19); every state change is written append-only to `l1_audit_log` with actor, evidence, rubric version, and change summary (§1 #10), so who decided what on what evidence is always reconstructable. Stephen signs off before this FR leaves `draft`, per the high-risk rule.

### Failure Modes

Every model failure resolves to a human, never to a fabricated or silent score — the safe state is "no score", reached by flagging (the same discipline as FR-CUO-204's "no change applied"):

- **Model wrong / hallucinated.** The prompt returns `unsupported` for any clause it cannot ground (§1 #13); `validate_citation` rejects a score citing an evidence id not in the recall set and rejects any number on a non-scored clause (§1 #5 #12). A wrong-but-grounded score is contestable by the subject's rebuttal and correctable by the reviewer, whose change preserves the GENIE original as a diff (§1 #18).
- **Model offline / timeout / malformed output.** The affected clauses become `score = null, needs_human_review`, `eval.fallback_no_score` is emitted, and the assessment is routed to a human (§1 #12, DEC-2514). No code path defaults a number.
- **Bias / unfairness.** Protected attributes are blinded pre-scoring; a post-hoc group-disparity check flags a batch for human fairness review on a threshold breach — it flags, it never auto-adjusts a score, because an auto-correcting pass is its own bias (§1 #14).
- **Governance failures.** An unconsented subject is never drafted (§1 #2); evidence cannot egress to an unbounded or cross-region provider (§1 #11); a peer cannot read an assessment (RLS, §1 #16). In each case the system refuses rather than proceeds.

## AI Authorship Disclosure

- **Tools used:** Claude (Cowork), authoring this FR + its paired audit from Stephen's 2026-06-29 decisions and the BRAIN/EVAL strategy note (`docs/strategy/cyberos-brain-evaluation-plan.md`, Phase 4 + "Doing the monitoring responsibly"), in the repo's engineering-spec@1 house style.
- **Scope:** full draft of this specification — the normative clauses, the migrations and Rust/Python sketches, the AI Risk Assessment, the acceptance criteria, and the failure-mode inventory.
- **Human review:** Stephen reviews and approves before status moves past `draft`. This is a high-risk FR, so the mandatory-HITL design, the consent gate, and the no-score fallback need his explicit sign-off; the paired audit plus the CAF/AWH gate validate before any implementation merges.

---

*End of FR-EVAL-003.*
