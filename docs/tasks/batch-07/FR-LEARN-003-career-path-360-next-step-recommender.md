---
title: "LEARN — career path module, level definitions, competency framework, 360 feedback, next-step recommender"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q2"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the **career path** primitive: explicit **level definitions** per role kind (engineering / design / account-management / hr-ops) with **competency frameworks** (per-level expected behaviours, deliverables, scope of impact); **360 feedback** collection (peer + skip-level + cross-team feedback rounds, separate from VP — focused on growth, not compensation); **next-step recommender** — a CUO/CHRO surface that, given a Member's current level + recent VP outcomes (FR-LEARN-001) + 360 themes + recent KB authorship + recent project leadership signals, surfaces a **growth recommendation** ("you're consistently meeting Senior expectations on delivery and collaboration; the gap to Staff is in cross-team architectural influence; consider leading the Acme migration architecture review next quarter"). Recommendations are **informational, not gates**; promotion decisions remain with the Hội đồng (FR-LEARN-002). Ships the **competency-framework templates** seeded for the 4 P2 role kinds; the **365 feedback** lifecycle (separate cadence from VP — annual for everyone, ad-hoc on request); and the **training records + certifications** primitive (Member-tracked or HR/Ops-recorded, surfaces in the Member's career profile).

## Problem

PRD §9.15 names "career path module; promotion gating; sabbatical eligibility tracking" as P2 scope. Three failure modes the platform must prevent:

- **Opaque levels.** A Member who doesn't know what "Senior" means in CyberSkill cannot self-direct toward it. Without explicit level definitions + competency frameworks, growth conversations are vague.
- **VP overload.** VP (FR-LEARN-001) feeds compensation; using the same instrument for growth conversations creates score-anxiety. 360 feedback, separated structurally, gives Members feedback that is not directly tied to pay.
- **No structured next-step.** A manager's 1:1 (FR-HR-003) might cover growth but inconsistently; the next-step recommender ensures every Member receives a structured growth perspective at least quarterly.

## Proposed Solution

The shape of the answer is `learn.*` schema (mostly *not* in `hr_secure` because career-path data is generally not compensation-secret) + the competency framework + the 360 cycle + the next-step recommender.

**Schema.**

```sql
CREATE SCHEMA learn;

-- Level catalogue per role kind.
CREATE TABLE learn.level (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  parameter_version_id UUID NOT NULL REFERENCES hr_secure.parameter_version(id),
  role_kind TEXT NOT NULL,                                         -- "engineering" | "design" | "account_management" | "hr_ops"
                                                                  -- | "founder_track"
  level_code TEXT NOT NULL,                                        -- "L1" | "L2" | ... | "Senior" | "Staff" | "Principal"
  display_name TEXT NOT NULL,
  position INT NOT NULL,                                           -- ordering: L1 < L2 < Senior < Staff < Principal
  expected_years_typical INT,                                      -- guidance, not gate
  competencies JSONB NOT NULL,                                     -- structured framework:
                                                                  -- [{
                                                                  --   id: "technical_depth",
                                                                  --   display_name: "Technical depth",
                                                                  --   level_indicator_md: "What this looks like at this level...",
                                                                  --   evidence_examples_md: ["Designed X", "Mentored Y on Z"]
                                                                  -- }, ...]
  promotion_criteria_md TEXT NOT NULL,                              -- "To advance from this level, a Member should consistently..."
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, role_kind, level_code)
);

-- 360 feedback cycle.
CREATE TABLE learn.feedback_cycle (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cycle_kind TEXT NOT NULL,                                        -- "annual_360" | "ad_hoc_on_request" | "promotion_360"
  subject_employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE RESTRICT,
  raised_by UUID,                                                   -- HR/Ops Lead or Member self-request
  open_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  due_at TIMESTAMPTZ NOT NULL,
  status TEXT NOT NULL DEFAULT 'open',                              -- "open" | "collecting" | "synthesised" | "delivered" | "closed"
  reviewers UUID[] NOT NULL,                                        -- typically 5-8 Members; mix of peers, manager, cross-team
  synthesis_md TEXT,                                                -- the themes-only synthesis written by HR/Ops Lead
  delivered_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE learn.feedback_response (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  cycle_id UUID NOT NULL REFERENCES learn.feedback_cycle(id) ON DELETE CASCADE,
  reviewer_member_id UUID NOT NULL,
  reviewer_kind TEXT NOT NULL,                                     -- "manager" | "peer" | "skip_level" | "cross_team" | "self"
  responses JSONB NOT NULL,                                        -- structured per-question responses
  free_text_md TEXT,                                               -- open-ended feedback
  submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  is_anonymous_to_subject BOOLEAN NOT NULL DEFAULT true,            -- typical 360: anonymous to subject
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  UNIQUE (tenant_id, cycle_id, reviewer_member_id)
);

-- Per-Member current level + level history.
CREATE TABLE learn.level_assignment (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  level_id UUID NOT NULL REFERENCES learn.level(id),
  effective_from DATE NOT NULL,
  effective_to DATE,                                                -- null for current
  superseded_by UUID REFERENCES learn.level_assignment(id),
  basis_md TEXT,                                                    -- the rationale (typically a council case ref)
  basis_council_case_id UUID REFERENCES hr_secure.council_case(id),
  signed_by_manager_at TIMESTAMPTZ NOT NULL,
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, employee_id, effective_from)
);

-- Training records + certifications.
CREATE TABLE learn.training_record (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,                                               -- "internal_course" | "external_course" | "certification"
                                                                   -- | "conference" | "self_directed_learning"
  title TEXT NOT NULL,
  provider TEXT,
  completion_date DATE,
  expiry_date DATE,
  certificate_blob_id UUID,                                          -- references the content-addressed blob store
  is_company_funded BOOLEAN NOT NULL DEFAULT false,
  validated_by_member_id UUID,                                       -- HR/Ops Lead or manager validation
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Next-step recommendations (per Member, refreshed quarterly).
CREATE TABLE learn.next_step_recommendation (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL REFERENCES hr.employee(id) ON DELETE CASCADE,
  generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  generated_by_persona_version TEXT,
  current_level_id UUID NOT NULL REFERENCES learn.level(id),
  target_level_id UUID REFERENCES learn.level(id),
  recommendation_md TEXT NOT NULL,                                   -- the CUO-generated narrative
  evidence_refs JSONB NOT NULL,                                      -- citations: VP outcomes, 360 themes, KB authorship counts
  member_feedback TEXT,                                               -- "useful" | "not_useful" | "partially_useful"
  member_feedback_at TIMESTAMPTZ,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

**Default level catalogue seed.**

The first parameter version's `learn.level` rows for each role kind. Sample **engineering ladder**:

| Code | Name | Years (typical) | Competencies (abbreviated) |
|---|---|---|---|
| L1 | Junior Engineer | 0-2 | Implements features under guidance; writes tested code; participates in reviews. |
| L2 | Engineer | 2-4 | Owns medium-sized features end-to-end; debugs production issues; reviews peers. |
| Senior | Senior Engineer | 4-7 | Owns subsystems; designs medium-scope architecture; mentors L1/L2. |
| Staff | Staff Engineer | 7+ | Owns cross-team architecture; influences platform direction; mentors Senior. |
| Principal | Principal Engineer | 10+ | Defines technical strategy; cross-platform impact; founder-equivalent technical influence. |

Each level has 5-8 structured competencies (technical depth, architectural scope, mentorship, communication, code quality, customer empathy, on-call ownership, learning velocity). The `level_indicator_md` per competency-per-level describes "what does it look like" in concrete terms.

Similar ladders for design / account-management / hr-ops; the founder-track is a separate primitive (P3+ when the team has subordinate "founder-track" leadership candidates).

**360 feedback cycle.**

- **Annual cadence.** Every Member gets a 360 feedback cycle on their hire-date anniversary (configurable to align with calendar quarters).
- **Reviewer selection.** Manager + 2-3 peers + 1-2 cross-team Members + the Member's self-assessment. The Member nominates peers; the manager confirms.
- **Anonymity.** Per-reviewer responses are anonymous to the subject Member (the standard 360 protocol). HR/Ops Lead synthesises into themes; per-reviewer text is *not* shown to the Member. The exception: the manager's input is *attributed* (the Member knows their manager's input).
- **Structured + free-text questions.** A small set of structured Likert-scale questions (5-10) per role-kind + an open-ended free-text "what should this Member start / stop / continue?" section.
- **Synthesis.** HR/Ops Lead reads all responses + writes a themes-only synthesis: 3-5 paragraphs covering strengths, growth areas, suggested next steps. The synthesis is the canonical artefact the Member sees.
- **Delivery.** Synthesis delivered via 1:1 between Member + manager (FR-HR-003); the Member acknowledges receipt.

**Next-step recommender.**

A scheduled quarterly job (after the VP cycle closes, ~Q+1 month 5) generates a `next_step_recommendation` per active Member:

- **Inputs.**
  - Current level + months at current level.
  - Last 4 VP outcomes (weighted scores + outcome summaries).
  - Most-recent 360 synthesis.
  - Recent KB-page authorship + PROJ-issue leadership + Engagement-led count.
  - Manager's recent 1:1 notes (FR-HR-003) — only the Member's-side notes; manager-private notes excluded.
- **Compute.** CUO/CHRO via the AI Gateway (FR-AI-001) with the next-step persona prompt; reads the inputs through ACL-scoped retrieval. No compensation amounts are visible to the persona; only level + level-progress signals.
- **Output.** A 4-8 sentence narrative recommendation:
  > "You're at L2 and have been for 14 months. Your last 4 VP scores are 4.1, 4.2, 4.3, 4.0 — consistent strong performance on Delivery and Collaboration. Your 360 themes name 'designs systems thoughtfully' as a strength and 'cross-team influence' as a growth area. You authored 6 KB pages this quarter (top-quartile on the team). To progress to Senior, focus next quarter on leading the Acme migration architecture review (Engagement: Acme; suggested by your manager). Consider co-authoring the new dependency-injection RFC with @khoa-le for cross-team visibility. We'll review again at the next quarterly recommendation."
- **Evidence.** Each claim cites the underlying data point.
- **Member feedback.** The Member rates the recommendation as useful / partially useful / not useful with optional comment; HR/Ops Lead reviews quarterly to tune the persona.
- **Not a gate.** The recommendation is informational; promotion decisions remain with the Hội đồng (FR-LEARN-002). The `target_level_id` is a *suggested* next level, not a commitment.

**Persona scope contract for next-step.**

CUO/CHRO declares for the next-step path:
- `tools_allowed`: read VP outcomes; read 360 syntheses (themes-only); read KB authorship; read PROJ leadership signals; read level definitions.
- `tools_forbidden_explicit`: read compensation values; read per-reviewer 360 responses; read Council deliberation positions; read other Members' levels; suggest compensation changes; gate promotions.

**Level-assignment lifecycle.**

A new level assignment is created when:
1. Hire (initial level set by HR/Ops Lead).
2. Promotion (Hội đồng case approved → manager + founder co-sign → `level_assignment` inserted with `superseded_by` chain to prior).
3. Lateral move (between role kinds) — rare; full Hội đồng case.
4. Demotion — exceedingly rare; requires founder + DPO + legal-counsel-ref + Member consent or labour-tribunal directive.

**Frontend integration.**

`/learn` (FR-LEARN-004) surfaces:
- Member's career profile: current level, level history, competency self-assessment, training records.
- Next-step recommendation card.
- 360 cycle status + last synthesis.
- Promotion proposal form (manager-initiated).
- Level catalogue + competency framework reference (read-only).

**MCP tool surface (read-only).**

- `cyberos.learn.list_levels(role_kind?)` — read; everyone.
- `cyberos.learn.my_level` — read; calling Member.
- `cyberos.learn.get_my_level_history` — read; calling Member.
- `cyberos.learn.list_my_360_cycles` — read; calling Member.
- `cyberos.learn.get_my_360_synthesis(cycle_id)` — read; calling Member.
- `cyberos.learn.get_my_next_step` — read; calling Member.
- `cyberos.learn.list_training_records(member_id?)` — read; HR/Ops + Founder + the Member themselves.
- `cyberos.learn.create_training_record(...)` — `destructive: true; requires_confirmation: true`; Member self-records (validated by HR/Ops post-hoc).

There are no mutation MCP tools for level assignments, 360 responses, or recommendations — UI + step-up only.

## Alternatives Considered

- **Skip the level catalogue; let levels be informal.** Rejected: PRD §9.15 explicitly scopes "career path module"; the founder's bottleneck on growth conversations is real.
- **Per-reviewer 360 responses visible to subject.** Rejected: chills candor; standard 360 protocol uses synthesis.
- **Skip the next-step recommender; manager-only growth conversations.** Rejected: inconsistency across managers; the recommender ensures parity.
- **AI generates the 360 synthesis.** Rejected: the synthesis interprets nuanced human input; HR/Ops Lead ownership is the floor. CUO can suggest a draft synthesis (P3 — Review-mode) but the Lead writes the canonical.
- **Allow compensation-amount visibility in next-step recommendations.** Rejected: explicit prohibition; the recommendation is about growth, not pay.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) the founder publishes the level catalogue (parameter version + sign chain) for the 4 role kinds; (2) a synthetic 360 cycle runs end-to-end for one Member with 5 reviewers + synthesis; (3) the next-step recommender produces a structured recommendation with citations for every active Member; (4) the regression suite catches an adversarial prompt asking the persona to reveal a per-reviewer 360 response.
- **Adoption metric.** Every Member receives at least one annual 360 + four quarterly next-step recommendations.
- **Quality metric.** Member rating of next-step recommendations as "useful" or "partially useful" ≥ 70% on a rolling 6-month window.

## Scope

**In-scope.**
- The 5 schema additions (`level`, `feedback_cycle`, `feedback_response`, `level_assignment`, `training_record`, `next_step_recommendation`).
- Default level catalogue seed for 4 role kinds.
- 360 feedback cycle (annual + ad-hoc).
- Synthesis-only delivery; per-reviewer anonymity.
- Quarterly next-step recommender via CUO/CHRO with persona-scope contract.
- Member-feedback loop on recommendations.
- Level-assignment lifecycle.
- Training records.
- The 8 MCP tools.
- Audit integration in scope `learn.{tenant}` (separate from `learn.council.{tenant}`).

**Out-of-scope (deferred).**
- Frontend remote at /learn (FR-LEARN-004).
- AI-suggested 360 synthesis drafts (P3).
- External certification API integrations (Coursera, Pluralsight) — P3.
- Skill-graph cross-team mapping (P3).
- Performance Improvement Plans (PIPs) (P3 — sensitive; needs careful design).

## Dependencies

- FR-HR-001 / FR-HR-003 (employees + 1:1 notes).
- FR-LEARN-001 / FR-LEARN-002 (VP + Council).
- FR-REW-001 (parameter version primitive).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-MCP-001 / FR-AI-001.
- FR-KB-001 / FR-PROJ-001 (signal sources for the recommender).
- FR-CP-001.
- Compliance: PDPL Decree 13; EU AI Act Article 22 (no automated decisions on individuals — recommender is informational, not a gate); Article 50 (transparency disclosure on recommendations).
- Locked decisions referenced: DEC-200 (level catalogue parameter-version-locked), DEC-201 (360 anonymity protocol), DEC-202 (next-step recommender is informational, not a gate), DEC-203 (CUO never sees compensation amounts in the recommender path).

## AI Risk Assessment

The next-step recommender is the AI surface. EU AI Act risk class: `limited` (informational; no automated decision; the Hội đồng + founder gate any actual progression).

### Data Sources

Per-tenant only: the Member's own VP outcomes + 360 syntheses + KB/PROJ signals. CUO/CHRO never sees compensation values or per-reviewer 360 responses. Per-tenant residency.

### Human Oversight

- The recommendation is informational; promotion decisions go to the Hội đồng.
- Members rate the recommendation; HR/Ops Lead tunes the persona.
- The 360 synthesis is HR/Ops Lead-authored, not AI-authored.
- The kill-switch from FR-GENIE-002 silences the recommender.

### Failure Modes

- **Recommender suggests target level too aggressively.** Members rate "not useful"; HR/Ops Lead retunes prompts; quarterly persona-version-eval gates the deployment.
- **Recommender leaks per-reviewer 360 content.** Caught by adversarial regression suite; persona-version blocked.
- **Recommender suggests compensation change.** Caught by persona-scope contract refusing the request; regression test covers.
- **Bias in recommendations across groups.** Mitigation: HR/Ops Lead + DPO quarterly review of recommendation distribution by team / tenure / etc.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted level catalogue, 360 cycle, next-step recommender architecture, persona-scope contract, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the level catalogue + competency framework will be reviewed with the Engineering Lead + senior team members at PR-review.
