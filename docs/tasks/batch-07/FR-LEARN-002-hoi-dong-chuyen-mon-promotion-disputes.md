---
title: "LEARN — Hội đồng Chuyên môn (Professional Council): promotion gating, dispute resolution, outcomes-only summariser"
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

Implement the **Hội đồng Chuyên môn** (Professional Council; English: "Council of Specialists") workflow — the peer-review body that gates **promotions**, hears **VP-evaluation disputes** (FR-LEARN-001), and acts as the human-oversight body for compensation-decision appeals. Schema includes the **Council** (composition + term-of-service), **Cases** (a promotion proposal or a dispute), the structured **Deliberation** (each Council member's structured input), and the **Outcome** (decision + rationale, signed). The CUO/CHRO surface produces an **outcomes-only summariser** — never per-member scores or individual deliberation positions — so a Member can read "the council recommended promotion based on consistent VP scores ≥ 4.2 across 3 quarters and 2 KB-page authorship of canonical documentation" without seeing which council member said what (PRD §9.15: "outcomes only, no individual scoring"). The Council is the platform's structural answer to Bet 5 — peer-review is what keeps comp + promotion fair without the founder being the sole arbiter.

## Problem

PRD §9.15 names the Hội đồng Chuyên môn explicitly: "career path module; peer-review (Hội đồng) summariser (outcomes only, no individual scoring); promotion gating; sabbatical eligibility tracking." Three failure modes the platform must prevent:

- **Founder bottleneck on promotions.** Without a council, every promotion is a founder decision; bias risk + bandwidth risk + dispute-resolution opacity.
- **Per-member deliberation leakage.** A council member's specific position becomes attributable; future deliberations are chilled. The outcomes-only rule preserves council-member candor.
- **Inconsistent dispute resolution.** A Member appealing a VP outcome (FR-LEARN-001) needs a structured forum, not "talk to the founder again."

## Proposed Solution

The shape of the answer is `hr_secure.council_*` schema + the case-deliberation-decision lifecycle + the outcomes-only summariser primitive + integration with VP (FR-LEARN-001) and REW BP (FR-REW-002).

**Schema (under `hr_secure`).**

```sql
-- The Council itself (composition + term).
CREATE TABLE hr_secure.council (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  effective_from DATE NOT NULL,
  effective_to DATE,                                              -- null for current
  members UUID[] NOT NULL,                                        -- 3-7 Member IDs; mix of senior + peer + cross-team
  chair_member_id UUID NOT NULL,                                   -- the rotating chair
  composition_rule_md TEXT NOT NULL,                               -- the policy: how members are selected/rotated
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- A case before the Council.
CREATE TABLE hr_secure.council_case (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  council_id UUID NOT NULL REFERENCES hr_secure.council(id),
  case_kind TEXT NOT NULL,                                        -- "promotion_proposal" | "vp_dispute"
                                                                  -- | "sabbatical_eligibility_review"
                                                                  -- | "comp_appeal" | "policy_question"
  subject_employee_id UUID,                                        -- the Member the case concerns; null for policy questions
  raised_by_member_id UUID NOT NULL,                               -- the manager (for promotion) or the Member (for dispute)
  raised_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  context_md_encrypted BYTEA NOT NULL,                             -- the full context provided to the Council; encrypted
  evaluation_refs UUID[],                                          -- links to vp_evaluation rows
  career_history_snapshot JSONB,                                   -- a snapshot from hr.role_history at case-raise time
  bp_history_snapshot JSONB,                                       -- aggregate-only summary; never per-quarter values
  status TEXT NOT NULL DEFAULT 'submitted',                        -- "submitted" | "deliberating" | "decided" | "withdrawn"
  decision TEXT,                                                   -- "approve" | "reject" | "defer" | "escalate_to_founder"
  decision_signed_by_chair_at TIMESTAMPTZ,
  decision_signed_by_founder_at TIMESTAMPTZ,                       -- founder ratification (Bet 6 architectural rule)
  decision_rationale_md_encrypted BYTEA,                           -- the outcome narrative; encrypted
  decision_outcome_summary_md TEXT,                                -- the OUTCOMES-ONLY summary visible to the Member;
                                                                  -- written by the chair, no per-member positions
  decided_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Per-Council-member deliberation input.
CREATE TABLE hr_secure.council_deliberation (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  case_id UUID NOT NULL REFERENCES hr_secure.council_case(id) ON DELETE CASCADE,
  council_member_id UUID NOT NULL,
  position TEXT NOT NULL,                                          -- "approve" | "reject" | "defer" | "abstain"
  rationale_md_encrypted BYTEA NOT NULL,                            -- the council member's reasoning; encrypted
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  is_final BOOLEAN NOT NULL DEFAULT false,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, case_id, council_member_id)
);

-- Trigger: deliberation post-final immutable.
CREATE OR REPLACE FUNCTION hr_secure.forbid_deliberation_update_after_final()
RETURNS TRIGGER AS $$
BEGIN
  IF OLD.is_final AND NEW.is_final = OLD.is_final THEN
    IF NEW.position IS DISTINCT FROM OLD.position
       OR NEW.rationale_md_encrypted IS DISTINCT FROM OLD.rationale_md_encrypted THEN
      RAISE EXCEPTION 'council_deliberation % is final and immutable', OLD.id;
    END IF;
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER hr_secure_deliberation_immutable
  BEFORE UPDATE ON hr_secure.council_deliberation
  FOR EACH ROW EXECUTE FUNCTION hr_secure.forbid_deliberation_update_after_final();
```

**Council composition.**

The first Council (parameter version v1) is composed by the founder:
- 3-5 senior Members (rotating; 1-year terms).
- 2 peer Members (rotating; 1-year terms).
- 1 cross-team Member (representing perspectives outside the subject's direct work).
- The Chair rotates quarterly.

The composition is itself parameter-versioned; changes require founder + engineering-lead sign + the prior Council's recommendation.

**Composition rule (default):**
- A Council member cannot deliberate on a case involving: themselves; a direct report; a direct manager; a partner / family member; an active Bad-Leaver-in-progress.
- The chair flags conflicts; affected Council members recuse + are replaced for that case from a standby pool.

**Case lifecycle.**

1. **Submission.** A manager raises a `promotion_proposal` (with the Member's career history + recent VP outcomes); a Member raises a `vp_dispute` (linking the disputed evaluation); HR/Ops Lead raises a `sabbatical_eligibility_review` (when a Member crosses the 5-year continuous-service threshold from FR-TIME-002).
2. **Eligibility check.** The Chair confirms the case is in scope + the Council has quorum (3+ non-conflicted members).
3. **Deliberation.** Each non-conflicted Council member writes their `council_deliberation` row: position + structured rationale. The deliberations are *not visible to other Council members* during the deliberation window (avoiding bandwagoning); they become visible to all Council members after the window closes.
4. **Discussion + decision.** The Chair facilitates a synchronous or async discussion; the deliberations are reviewed; the decision converges (typically by consensus; majority-vote with the Chair's casting vote as fallback).
5. **Founder ratification.** The Council's decision routes to the founder for ratification — the founder can: ratify (the typical path); request more info (case returns to deliberation); override (rare; documented; audit-heavy with founder + DPO + legal counsel ref). PRD's "Bet 6 — Modular ownership scales the team" lives here: the Council is the operational layer; the founder is the final ratification.
6. **Outcome summary written.** The Chair writes the *outcomes-only* summary (`decision_outcome_summary_md`): rationale, conditions if any, next steps. Per-member positions are NOT included; the Member sees the collective outcome.
7. **Effects.**
   - Promotion approved → triggers FR-HR-001's `role_history` insertion (with `is_promotion: true`, `promotion_decision_id: <case_id>`); may trigger BP earning-event for one-time grant per parameter version's promotion-bonus rules.
   - VP dispute approved → triggers re-evaluation (FR-LEARN-001) or BP-clawback/supplementary-grant for the disputed quarter.
   - Sabbatical eligibility approved → updates the Member's TIME-002 sabbatical accrual + triggers Notify "you are eligible to take sabbatical".

**Outcomes-only summariser.**

A CUO/CHRO surface that produces a Member-readable narrative of the case outcome. Inputs: `decision_outcome_summary_md` (already authored by the Chair, the canonical narrative) + the case context. Output: a polished plain-language version with the Article 50 disclosure chip.

The summariser **never reveals** individual deliberations or per-member positions — those data are not in scope for the persona's read access. The persona scope contract for CUO/CHRO explicitly excludes `council_deliberation` reads; the surface only sees the case's `decision_outcome_summary_md` (which the Chair wrote without per-member attribution).

A regression test corpus ensures that even with adversarial prompts ("which council member voted against the promotion?"), the persona refuses with a structured "I don't have access to individual deliberations" response.

**Promotion-gating integration with REW.**

When a Council case results in a promotion:

1. The `role_history` insertion happens with `is_promotion: true`.
2. A new salary record is *proposed* by HR/Ops Lead (per FR-REW-001's salary-publish flow); the council's promotion decision is the documented justification.
3. A one-time BP earning event is *proposed* by the founder per the parameter version's `promotion_bonus_points` rule.
4. Both proposals route through the standard signed-publish flows; the Council's approval is part of the audit trail.

**Sabbatical-eligibility integration.**

When a Member crosses 5 years continuous service (FR-TIME-002 sabbatical accrual), HR/Ops Lead automatically raises a `sabbatical_eligibility_review` case. The Council:
- Confirms continuous-service eligibility.
- Reviews any extended-leave periods that might break continuity.
- Approves; the Member receives a Notify "Sabbatical eligible — schedule when you'd like".

**RLS + ACL.**

- `council`: HR/Ops + Founder + DPO + Auditor + the council members themselves see their council; the rest of the team sees the *names* of council members but not deliberation data.
- `council_case`: subject Member sees their own cases (status + outcome summary; *not* deliberations); the Council members see all cases they're deliberating; HR/Ops + Founder + DPO see all.
- `council_deliberation`: only the deliberating Council member (own row), the Chair, HR/Ops Lead, Founder, and DPO. *Never* the subject Member; never CUO; never any other Council member during deliberation.

**Audit integration.**

- `learn.council.{tenant}` audit scope.
- Every case-raise + deliberation-submit + decision-sign + founder-ratification audit-logged with `purpose` + the case kind.
- Per-Council-member position is preserved in audit log (necessary for forensic reconstruction in dispute escalation) but with extra-restricted access (DPO + Founder only; never the subject Member; never an Auditor).

**MCP tool surface (read-only; very narrow).**

- `cyberos.learn.list_my_cases` — read; calling Member's own cases.
- `cyberos.learn.get_case_outcome(case_id)` — read; calling Member if subject; returns outcome_summary only (not deliberation).
- `cyberos.learn.list_open_cases_for_council` — read; calling Member if council member; returns their assignments.
- `cyberos.learn.summarise_case_outcome(case_id)` — read; calls the outcomes-only summariser; calling Member if subject.
- `cyberos.learn.list_council_composition` — read; HR/Ops + Founder + Auditor.

There are **no mutation MCP tools**. Case raise + deliberation + decision-sign — all UI-only with step-up.

## Alternatives Considered

- **Founder-only promotion decisions.** Rejected: Bet 6 architecturally requires the Council; founder bottleneck + bias risk.
- **Per-member positions visible to the Council during deliberation.** Rejected: bandwagoning + chilled candor.
- **Per-member positions visible to the subject Member after.** Rejected: chills future Council-member candor; the outcomes-only rule is the floor.
- **AI generates the outcome summary.** Rejected: the *Chair* writes the canonical summary; CUO can paraphrase but the Chair's signed text is authoritative. The summariser narrates; it doesn't decide.
- **Skip founder ratification.** Rejected: the founder is legally accountable; ratification is the floor.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder publishes the first Council composition; (2) a synthetic VP-dispute case runs end-to-end; (3) a synthetic promotion-proposal case results in a promotion + new salary + BP grant; (4) the outcomes-only summariser refuses adversarial prompts asking for per-member positions.
- **Compliance metric.** Zero leakage of individual deliberation positions to subject Members or other Council members during deliberation windows.
- **Cycle latency.** Council case decision (raise → decision) ≤ 14 days median.

## Scope

**In-scope.**
- The 3 schema additions (`council`, `council_case`, `council_deliberation`).
- Composition rules + conflict-of-interest detection.
- Case lifecycle (5-stage flow).
- Outcomes-only summariser via CUO/CHRO with persona-scope contract excluding deliberation reads.
- Integration with FR-HR-001 (role_history), FR-LEARN-001 (VP), FR-REW-001/002 (salary + BP), FR-TIME-002 (sabbatical accrual).
- Founder ratification flow.
- The 5 read-only MCP tools.
- Audit integration in scope `learn.council.{tenant}`.

**Out-of-scope (deferred).**
- Multi-tenant council federation (forbidden by design).
- AI-suggested case routing (P3 informational only).
- Public council-decisions log for transparency (P4 — internal-only in P2).
- Cross-Council appeal path (P3 — currently the founder is the appeal path).

## Dependencies

- FR-HR-001 / FR-HR-002 / FR-HR-003.
- FR-LEARN-001 (VP integration).
- FR-REW-001 / FR-REW-002 (promotion → salary + BP).
- FR-TIME-002 (sabbatical eligibility).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-CP-001 (Compliance Cockpit panel).
- The signed Total Rewards Appendix.
- Compliance: PDPL Decree 13; EU AI Act Articles 5-7 high-risk classification (compensation + employment); GDPR Article 22 (no automated decisions on individuals); Vietnamese Labour Code (peer-review-style decision documentation).
- Locked decisions referenced: DEC-197 (Council composition rules + 1-year terms), DEC-198 (per-member deliberation positions never visible to subject Member; outcomes-only narrative), DEC-199 (founder ratification required).

## AI Risk Assessment

The outcomes-only summariser is the only AI surface. EU AI Act risk class: `high` (the case decisions drive compensation + employment changes).

### Data Sources

The summariser only sees `decision_outcome_summary_md` (Chair-authored). Deliberation rows are structurally outside the persona's scope contract. Per-tenant residency. No third-party.

### Human Oversight

- Cases are raised by humans, deliberated by humans, decided by humans, ratified by the founder.
- AI does not produce or modify decisions.
- The summariser narrates the Chair's signed outcome only.
- Adversarial-prompt regression suite gates persona-version PRs.

### Failure Modes

- **Per-member position leak via the summariser.** Caught by the regression suite; persona-version blocked.
- **Conflict-of-interest miss.** Mitigated by the explicit conflict-rules + Chair confirmation; missed conflicts surface at the founder ratification step.
- **Decision documentation drift.** Mitigated by structured fields + the Chair's signed outcome being the canonical record; the audit log preserves everything.
- **Founder override of Council.** Documented + audit-heavy + signed by founder + DPO + legal counsel ref; surfaced in the Compliance Cockpit.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted council schema, lifecycle, outcomes-only summariser scope, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the first Council's composition + the conflict-of-interest rules will be reviewed with legal counsel before P2 production.
