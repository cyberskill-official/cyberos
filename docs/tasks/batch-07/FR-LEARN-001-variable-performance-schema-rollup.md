---
title: "LEARN — Variable Performance schema + quarterly roll-up; rubric versioning; feeds REW Bonus Points fund"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: high
target_release: "P2 / 2027-Q2"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the LEARN module's **Variable Performance (VP)** primitive: the structured quarterly evaluation that produces the **earning events** consumed by the REW Bonus Points fund (FR-REW-002 §"Earning-event ingestion"). Schema includes the **Rubric** (the criteria + weights, parameter-versioned per FR-REW-001's anti-retroactive discipline), the **Evaluation** (per-Member per-quarter), the **Evaluator** (the people performing the assessment — typically the manager + 1-2 peers + the Member's self-assessment), the **Outcome** (the structured points + the rationale, signed). The roll-up runs at quarter-close + 14 days (giving evaluators a window to complete); produces a `bp_earning_event` (FR-REW-002) per Member with `source: 'vp_evaluation'`. The rubric is **versioned + immutable post-publish** (same trigger pattern as FR-REW-001/002); changes mid-quarter require a parameter-version bump that takes effect *next* quarter, never retroactively. AI is **read-only** in the path: CUO/CHRO can summarise an evaluation's outcome to the Member after sign (FR-LEARN-004 frontend), but cannot score, weight, or compute. Lives in `hr_secure`.

## Problem

CyberSkill's Total Rewards Appendix specifies that P3 Performance pay derives from VP evaluations + Hội đồng Chuyên môn promotion decisions. PRD §9.15 names "Variable Performance (VP) roll-up; Hội đồng Chuyên môn (Professional Council) workflow; promotion gating; sabbatical eligibility tracking." Three failure modes the platform must structurally avoid:

- **Inconsistent evaluation criteria.** Without a structured rubric, every manager scores differently; outcomes are not comparable across teams; appeal/dispute resolution becomes "Stephen's judgement."
- **Retroactive rubric changes.** A rubric updated mid-quarter that retroactively rescores prior evaluations breaks the contractual fairness expectation. The anti-retroactive parameter-version contract from REW applies here.
- **AI in the scoring path.** PRD §6.4 + §2.5: AI never decides compensation. VP scoring drives BP, which drives P3 cash — AI cannot score.

## Proposed Solution

The shape of the answer is `hr_secure.vp_*` schema + the rubric-version primitive + the evaluation lifecycle + the roll-up pipeline that feeds FR-REW-002.

**Schema (under `hr_secure`).**

```sql
-- The rubric — versioned + immutable post-publish.
CREATE TABLE hr_secure.vp_rubric (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  effective_from_quarter TEXT NOT NULL,                          -- "2026-Q3"; rubric applies to evaluations of THIS quarter onward
  description_md TEXT NOT NULL,
  criteria JSONB NOT NULL,                                       -- structured rubric:
                                                                 -- [{
                                                                 --   id: "delivery_quality",
                                                                 --   display_name: "Delivery quality",
                                                                 --   weight: 0.30,
                                                                 --   scale: { 1: "...", 2: "...", 3: "...", 4: "...", 5: "..." },
                                                                 --   guidance_md: "How to score..."
                                                                 -- }, ...]
                                                                 -- weights must sum to 1.0 (validated)
  points_per_score JSONB NOT NULL,                               -- mapping from weighted-score to BP points:
                                                                 -- { "1.0-1.99": 0, "2.0-2.99": 10, "3.0-3.49": 25,
                                                                 --   "3.5-3.99": 40, "4.0-4.49": 60, "4.5-4.99": 85, "5.0": 110 }
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  signed_by_legal_counsel_ref TEXT,
  superseded_by UUID REFERENCES hr_secure.vp_rubric(id),
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (tenant_id, effective_from_quarter)
);

-- Trigger: rubric is immutable post-publish.
CREATE OR REPLACE FUNCTION hr_secure.forbid_vp_rubric_update()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.signed_by_founder_at IS NOT NULL AND OLD.signed_by_engineering_lead_at IS NOT NULL THEN
    RAISE EXCEPTION 'vp_rubric % is published and immutable; create a new rubric version', OLD.id
      USING ERRCODE = 'check_violation';
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_vp_rubric_immutable
  BEFORE UPDATE ON hr_secure.vp_rubric
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_vp_rubric_update();

-- Per-Member per-quarter evaluation.
CREATE TABLE hr_secure.vp_evaluation (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  quarter TEXT NOT NULL,                                          -- "2026-Q3"
  rubric_id UUID NOT NULL REFERENCES hr_secure.vp_rubric(id),
  status TEXT NOT NULL DEFAULT 'open',                            -- "open" | "evaluators_scoring" | "self_assessment_pending"
                                                                  -- | "review" | "signed" | "fed_to_bp"
  open_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  evaluators_due_at TIMESTAMPTZ NOT NULL,                          -- typically quarter-end + 7d
  signed_by_manager_at TIMESTAMPTZ,
  signed_by_employee_at TIMESTAMPTZ,                                -- the Member countersigns having seen the outcome
  weighted_score_encrypted BYTEA,                                   -- the final weighted score; derived; encrypted
  bp_points_encrypted BYTEA,                                        -- derived from points_per_score table
  bp_earning_event_id UUID,                                         -- references hr_secure.bp_earning_event when fed
  fed_to_bp_at TIMESTAMPTZ,
  outcome_summary_md_encrypted BYTEA,                                -- the rationale narrative; encrypted
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, employee_id, quarter)
);

-- Per-evaluator scores for a single evaluation (manager + peers + self).
CREATE TABLE hr_secure.vp_evaluator_score (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  evaluation_id UUID NOT NULL REFERENCES hr_secure.vp_evaluation(id) ON DELETE CASCADE,
  evaluator_member_id UUID NOT NULL,
  evaluator_kind TEXT NOT NULL,                                     -- "manager" | "peer" | "self" | "skip_level"
  scores_encrypted BYTEA NOT NULL,                                  -- JSONB encrypted: { criterion_id -> score (1-5) + comment_md }
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  is_final BOOLEAN NOT NULL DEFAULT false,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE INDEX vp_evaluator_score_eval_idx ON hr_secure.vp_evaluator_score (tenant_id, evaluation_id);

-- Trigger: scores are immutable post-final.
CREATE OR REPLACE FUNCTION hr_secure.forbid_vp_score_update_after_final()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.is_final AND NEW.is_final = OLD.is_final THEN
    -- Allow only setting is_final from false → true; everything else immutable.
    IF NEW.scores_encrypted IS DISTINCT FROM OLD.scores_encrypted THEN
      RAISE EXCEPTION 'vp_evaluator_score % is final and immutable', OLD.id;
    END IF;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_vp_score_immutable
  BEFORE UPDATE ON hr_secure.vp_evaluator_score
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_vp_score_update_after_final();
```

**Default rubric seed (CyberSkill, 2026-Q3).**

The first rubric (parameter version v1's `vp_rubric`) carries the founder-signed criteria for the canonical engineering / design / account-management roles. Sample structure (criteria are role-aware via the `metadata.role_specific` flag — engineering rubric differs from account-management rubric):

| Criterion | Weight | Description |
|---|---|---|
| Delivery quality | 0.30 | Did the work meet the spec + acceptance criteria? Defects, rework rate, time-in-review. |
| Collaboration | 0.20 | How effectively did the Member work with peers + clients? Communication clarity, conflict resolution. |
| Initiative | 0.15 | Did the Member surface problems + propose solutions proactively? Self-direction. |
| Customer impact | 0.20 | (For client-facing roles only) Did the Member's work move the engagement forward? Customer satisfaction signals. |
| Internal contribution | 0.15 | KB authorship, mentorship, internal-tooling improvements. |

Each criterion has a 1-5 scale with explicit anchors per level. `points_per_score` mapping starts at 0 BP for sub-2.0 weighted score and tops out at 110 BP for a 5.0.

The rubric is reviewed annually; legal counsel + DPO + the Hội đồng (FR-LEARN-002) all sign before publish.

**Evaluation lifecycle.**

1. **Open the evaluation.** A scheduled job at the start of each quarter creates `vp_evaluation` rows for every active Member; status `open`. The active rubric (matching the quarter) is locked.
2. **Evaluator nomination.** The Member's manager, 1-2 peers (the Member can suggest peers; the manager confirms), and the Member themselves are the default evaluators. For senior roles, a skip-level evaluator is added.
3. **Evaluators score.** Each evaluator opens `/learn/vp/<evaluation_id>` and submits scores per criterion (1-5) with optional comments. Scores are written to `vp_evaluator_score` with `is_final: false` initially; the evaluator can revise until `evaluators_due_at`. After due date, scores auto-finalise.
4. **Self-assessment.** The Member submits their self-scores in the same window.
5. **Manager review.** Manager opens the evaluation; sees all evaluator scores + self-assessment side-by-side; writes the **outcome summary** (Markdown) covering: the weighted-score rationale, key strengths, areas for development, suggested next-quarter focus. The summary is the authoritative narrative.
6. **Compute.** A deterministic compute step:
   - Aggregate scores per criterion: weighted-mean across evaluators (manager + skip-level: 1.5×; peers + self: 1.0×).
   - Compute weighted-score (sum of `criterion_score * weight`).
   - Map to BP points via the rubric's `points_per_score` table.
   - Store `weighted_score_encrypted` + `bp_points_encrypted`.
7. **Sign.** Manager signs (`signed_by_manager_at`); Member countersigns having seen the outcome (`signed_by_employee_at`). The Member's countersign is **acknowledgement of receipt + understanding**, not necessarily agreement; the Member can simultaneously open a dispute (which routes to the Hội đồng — FR-LEARN-002).
8. **Feed to BP.** A scheduled job at quarter-close + 21 days walks all `signed` evaluations, creates a corresponding `bp_earning_event` row in REW (FR-REW-002), and sets `fed_to_bp_at`. The earning event carries `source: 'vp_evaluation'` and `source_ref: <evaluation_id>`.

**Compute + AI prohibition.**

- The compute is deterministic. The aggregation function + the points mapping are the parameter-version-locked rubric.
- AI never scores. AI never recommends a score adjustment. The persona-scope contract for CUO/CHRO explicitly excludes the VP scoring path.
- AI may *narrate* a completed evaluation to the Member (the FR-LEARN-004 surface) — read-only on the signed outcome.

**Rubric publishing flow.**

Same shape as FR-REW-001:
1. HR/Ops Lead drafts the new rubric for next quarter.
2. Founder + Engineering Lead + Legal Counsel review.
3. Founder + Engineering Lead sign; legal counsel ref recorded.
4. Publish; immutability trigger thereafter rejects updates.
5. The new rubric is auto-consumed by evaluations whose `quarter` matches.

**Dispute handling.**

If a Member disputes the outcome:
1. The Member opens a dispute via `/learn/vp/<id>/dispute`.
2. The dispute routes to the Hội đồng Chuyên môn (FR-LEARN-002).
3. The Hội đồng can: confirm the outcome, recommend a re-evaluation, or escalate to founder for a decision.
4. A re-evaluation creates a new `vp_evaluation` row; the prior one is marked `superseded_by` (the prior `bp_earning_event` is *not* clawed back automatically; if the re-evaluation's points differ, a new clawback or supplementary earning event is signed by founder + DPO).

**RLS + ACL.**

- `vp_evaluation`: a Member sees their own; their manager sees their reports'; HR/Ops + Founder + DPO see all.
- `vp_evaluator_score`: only the evaluator (their own scores), the Member (after sign), the manager, HR/Ops + Founder + DPO. Per-evaluator score visibility to the Member is controlled by the rubric's `score_visibility_to_member` flag (default: aggregate visible, per-evaluator-anonymous visible only after Hội đồng confirmation).
- All reads audit-logged with `field_kind` + `purpose`.

**MCP tool surface (read-only).**

- `cyberos.learn.list_my_evaluations` — read; calling Member's own; step-up.
- `cyberos.learn.get_my_evaluation(quarter)` — read; step-up.
- `cyberos.learn.get_evaluator_assignment` — read; the evaluations where the calling Member is an evaluator.
- `cyberos.learn.list_open_evaluations(team?)` — read; manager + HR/Ops + Founder.
- `cyberos.learn.get_active_rubric(quarter)` — read; HR/Ops + Founder + Auditor.

There are **no mutation MCP tools**. Score submission, manager review, signing — all UI-only with step-up.

## Alternatives Considered

- **Skip the structured rubric; let managers score freeform.** Rejected: the consistency + comparability + dispute resolution properties collapse.
- **Allow rubric mid-quarter changes.** Rejected: anti-retroactive contract is the floor.
- **Use Lattice / 15Five.** Rejected: residency + per-tenant Total-Rewards-Appendix integration not viable hosted; comp-related data leakage risk.
- **AI-suggested scoring with manager confirmation.** Rejected: even with confirmation, the AI's anchor effect biases scores; explicit prohibition.
- **Self-assessment optional.** Rejected: self-assessment is the second-most-consistent signal across the rubric (after the manager); making it optional damages the calibration.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder publishes the first rubric (parameter version + sign chain); (2) a synthetic Q3 evaluation runs end-to-end for the 10-employee team; (3) the deterministic compute matches a hand-computed expected weighted score for a sample Member; (4) the BP earning-event feed creates the right `hr_secure.bp_earning_event` rows; (5) immutability triggers reject post-final score updates.
- **Compliance metric.** Zero retroactive rubric changes; zero AI in the scoring path; zero unauthorised P1-protection breaches via VP outcomes.
- **Cycle latency.** Quarterly evaluation cycle completes (open → fed_to_bp) within 28 days of quarter close.

## Scope

**In-scope.**
- The `hr_secure.vp_rubric`, `vp_evaluation`, `vp_evaluator_score` tables.
- Anti-retroactive rubric immutability trigger.
- Score-immutability-post-final trigger.
- Default rubric seed (5 criteria; role-aware).
- Evaluation lifecycle scheduled job + manual override.
- Deterministic compute + BP-feed pipeline.
- Dispute path stub (full path in FR-LEARN-002).
- The 5 read-only MCP tools.
- Audit integration in scope `learn.vp.{tenant}`.

**Out-of-scope (deferred to FR-LEARN-002 / FR-LEARN-003 / FR-LEARN-004).**
- Hội đồng Chuyên môn workflow + outcomes-only summariser (FR-LEARN-002).
- Career path module + 360 feedback (FR-LEARN-003).
- Frontend remote at /learn (FR-LEARN-004).
- Multi-quarter trend analysis per Member (P3).
- Calibration sessions (P3 — when the team grows beyond 30 and inter-manager calibration becomes necessary).

## Dependencies

- FR-HR-001 / FR-REW-001 / FR-REW-002 (the substrate + the BP earning-event consumer).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001.
- FR-CP-001 (Compliance Cockpit panel).
- The signed Total Rewards Appendix.
- Compliance: PDPL Decree 13 (evaluation outcomes are personal data); EU AI Act Articles 5-7 high-risk classification (compensation-decision domain — no AI in scoring); GDPR Article 22 (no automated decisions); Vietnamese Labour Code (transparent evaluation criteria).
- Locked decisions referenced: DEC-193 (rubric anti-retroactive immutability), DEC-194 (deterministic compute; no AI in scoring), DEC-195 (VP feeds REW BP via signed earning events), DEC-196 (Member countersign is acknowledgement, not agreement; dispute path available).

## AI Risk Assessment

This FR explicitly forbids AI in the scoring path. EU AI Act risk class: `high` (compensation domain).

### Data Sources

The schema stores deterministic scores + the signed outcome narrative. AI does not produce or modify either. CUO/CHRO can read the signed outcome to narrate it to the Member (FR-LEARN-004) — that surface is `limited` risk class; this FR is `high` because the data drives compensation.

### Human Oversight

- Every score is human-submitted.
- Manager writes the outcome summary.
- Member countersigns receipt.
- Disputes go to the Hội đồng (a human council).
- Rubric publish requires founder + engineering-lead + legal-counsel-ref signatures.

### Failure Modes

- **Mid-quarter rubric change attempted.** Caught by immutability trigger.
- **Late evaluator scores post auto-finalise.** Mitigation: manager can extend the window once with a documented reason; subsequent extensions require HR/Ops Lead sign.
- **Self-assessment misalignment** (self gives 5; manager gives 2). The outcome summary is the authoritative narrative; the Hội đồng can review.
- **Re-evaluation post-BP-feed.** A clawback or supplementary BP earning event is signed by founder + DPO.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted schema, rubric structure, evaluation lifecycle, compute pipeline, failure modes.
- **Human review:** `@stephen-cheng` reviewed; legal counsel + Vietnamese-labour-law specialist will review the rubric encoding before P2 production.
