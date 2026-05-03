---
title: "HR — onboarding workflow, Genie onboarding checklist generation, Vietnamese probation rules, day-1 / week-1 / 30-60-90 plans"
author: "@stephen-cheng"
department: human_resources
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q1"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the HR onboarding workflow: **structured onboarding tasks** (pre-start, day 1, week 1, 30-day, 60-day, 90-day milestones); **per-role onboarding templates** (Engineering / Design / Account Management / HR-Ops / Founder-equivalent) authored by HR/Ops Lead and reusable; **Genie onboarding checklist generation** — when a new Member is hired, CUO/CHRO drafts a personalised onboarding checklist combining the role template + the Engagement assignments + relevant KB pages + buddy assignment; **Vietnamese probation-period mechanics** (30-180-day probation per Labour Code Article 25; auto-transitions current_status from `probation` to `active` on probation end with optional probation-review gate); **buddy-pair assignment** + **first-week meeting schedule** auto-generated; **onboarding signal collection** (1:1 feedback prompts at week 1, week 4, week 8 surfaced by CUO); **welcome email draft** sent to the new Member's personal email pre-start. The PRD §14.3.1 P2 scope includes "onboarding workflows, Genie onboarding checklist generation"; this FR ships both.

## Problem

The team's current onboarding is "sit next to an old Member" — described in PRD §1.1's Origin. Three failure modes:

- **Inconsistent onboarding.** New Member A gets a comprehensive day-1 walkthrough; new Member B is shown one repo and left to figure out the rest. Quality varies by which Member happens to onboard them.
- **Probation-period silence.** Vietnamese Labour Code Article 25 mandates probation period rules (30-180 days depending on contract type); without structured tracking, probation ends invisibly and the company misses the legal opportunity to evaluate before commitment.
- **Lost institutional knowledge.** Onboarding is the highest-leverage moment to read the right KB pages, meet the right people, understand the right Engagements; without a structured plan, a new hire takes months to reach productivity.

## Proposed Solution

The shape of the answer is a `hr.onboarding_*` schema, the role-template library, the CUO/CHRO checklist generator, and the buddy-pair + meeting-schedule automation.

**Schema.**

```sql
CREATE TABLE hr.onboarding_template (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  role_kind TEXT NOT NULL,                              -- "engineering" | "design" | "account_management"
                                                        -- | "hr_ops" | "founder_track" | "general"
  display_name TEXT NOT NULL,
  description_md TEXT,
  is_active BOOLEAN NOT NULL DEFAULT true,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  archived_at TIMESTAMPTZ
);

CREATE TABLE hr.onboarding_template_task (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  template_id UUID NOT NULL REFERENCES hr.onboarding_template(id) ON DELETE CASCADE,
  position INT NOT NULL,
  milestone TEXT NOT NULL,                               -- "pre_start" | "day_1" | "week_1" | "day_30" | "day_60" | "day_90"
  title TEXT NOT NULL,
  description_md TEXT,
  task_kind TEXT NOT NULL,                               -- "checklist" | "read_kb_page" | "meeting" | "system_access"
                                                        -- | "submit_form" | "self_evaluation"
  default_owner_role TEXT,                                -- who's responsible for ensuring it happens: "self" | "buddy"
                                                        -- | "manager" | "hr_ops" | "founder"
  default_due_after_days INT,                             -- days after start_date
  is_required BOOLEAN NOT NULL DEFAULT true,
  external_ref JSONB,                                     -- e.g. { kb_page_id: ..., kb_space_slug: "engineering" }
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

CREATE TABLE hr.onboarding_run (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  employee_id UUID NOT NULL UNIQUE REFERENCES hr.employee(id) ON DELETE CASCADE,
  template_id UUID REFERENCES hr.onboarding_template(id),
  buddy_member_id UUID,
  status TEXT NOT NULL DEFAULT 'in_progress',             -- "draft" | "in_progress" | "completed" | "abandoned"
  generated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  generated_by_persona_version TEXT,                       -- the CUO persona that authored
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  completed_at TIMESTAMPTZ
);

CREATE TABLE hr.onboarding_task (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  tenant_id UUID NOT NULL,
  run_id UUID NOT NULL REFERENCES hr.onboarding_run(id) ON DELETE CASCADE,
  position INT NOT NULL,
  milestone TEXT NOT NULL,
  title TEXT NOT NULL,
  description_md TEXT,
  task_kind TEXT NOT NULL,
  owner_member_id UUID NOT NULL,
  due_at DATE NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',                  -- "pending" | "in_progress" | "completed" | "skipped" | "blocked"
  completed_at TIMESTAMPTZ,
  completed_by UUID,
  external_ref JSONB,
  ai_assisted_drafted BOOLEAN NOT NULL DEFAULT false,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX onboarding_task_run_idx ON hr.onboarding_task (tenant_id, run_id, milestone, position);
CREATE INDEX onboarding_task_owner_idx ON hr.onboarding_task (tenant_id, owner_member_id, status, due_at);
```

**Default role templates seed.**

`onboarding_template` rows seeded for the canonical role kinds; each carries 30-50 `onboarding_template_task` rows. Example **engineering template** (abbreviated):

| Milestone | Title | Owner | Due |
|---|---|---|---|
| pre_start | Personal email + welcome packet sent | hr_ops | -3d |
| pre_start | Hardware provisioned (laptop + Yubikey) | hr_ops | -1d |
| pre_start | GitHub + Linear + CyberOS account created | hr_ops | -1d |
| day_1 | Welcome 1:1 with founder (30 min) | founder | day_1 |
| day_1 | Buddy 1:1 (60 min) | buddy | day_1 |
| day_1 | Read CyberOS onboarding KB page | self | day_1 |
| day_1 | Read Total Rewards & Career Path Appendix | self | day_1 |
| day_1 | Set up local dev environment (cyberos-platform repo) | self | day_1 |
| day_1 | First commit (typo fix or doc update) | self | day_1 |
| week_1 | Read team's recent Engagement summaries | self | day_5 |
| week_1 | 1:1 with each direct teammate | self | day_5 |
| week_1 | Pair-program session with buddy | buddy | day_3 |
| week_1 | First 1:1 with manager | manager | day_5 |
| day_30 | First non-trivial PR merged | self | day_30 |
| day_30 | 30-day self-review questionnaire | self | day_30 |
| day_30 | 30-day check-in with manager | manager | day_30 |
| day_60 | Lead a small feature end-to-end | self | day_60 |
| day_60 | 60-day check-in with manager | manager | day_60 |
| day_90 | Probation review (if probation_period_months >= 3) | manager | day_90 |
| day_90 | 90-day self-review + 360 feedback | self | day_90 |

Similar templates for design / account management / HR-ops / founder-track. Templates are versionable; HR/Ops Lead can edit live (next runs use the new version; in-flight runs keep their snapshot).

**CUO/CHRO onboarding checklist generation.**

When a new `hr.onboarding_run` is created (typically by HR/Ops Lead at hire-time), CUO/CHRO drafts a personalised checklist:

1. Inputs: the role template + the new Member's hire-date + their assigned Engagement(s) (from FR-PROJ-007's `proj.engagement.primary_owner_member_id` updates) + their team + their direct manager.
2. Output: an array of `hr.onboarding_task` rows with personalised titles + descriptions + `external_ref`s (e.g. links to specific KB pages relevant to the assigned Engagement; links to the team's Notion handbook; links to the right Slack channels).
3. Per-task: `ai_assisted_drafted: true`; rendered in the UI with the EU AI Act Article 50 disclosure chip.
4. Due dates computed from start_date + each template task's `default_due_after_days`.

The HR/Ops Lead reviews and edits before the run starts; the new Member sees the run starting on day-1.

**Buddy-pair assignment.**

CUO/CHRO suggests a buddy from the team based on:
- Same team / similar role.
- Shared timezone (helpful for the Vietnamese remote team).
- Buddy's recent buddy-load (rotate; not the same Member every time).
- Onboarding-track overlap (e.g. a recent hire makes a good buddy because they remember).

The HR/Ops Lead confirms before assignment. The buddy receives a Notify card explaining their responsibilities.

**First-week meeting schedule.**

CUO/CHRO drafts a first-week 1:1 schedule:
- Day 1: founder (30 min); buddy (60 min); manager (30 min).
- Day 2-5: each direct teammate (15-30 min each, staggered).
- The drafts open in the Member's calendar; the new Member confirms or reschedules.

**Probation-period mechanics.**

When `hr.contract.contract_kind = 'definite_term'` or `'indefinite_term'` with `probation_period_months > 0`, a scheduled job:
1. On `hire_date + probation_period_months`, surfaces a Notify card to the manager + HR/Ops Lead: "Probation period for [Member] ends in 7 days. Schedule probation review."
2. The manager + HR/Ops Lead conduct a probation review (the day-90 onboarding task scaffolds this).
3. The review records a decision: pass / extend / terminate.
4. On `pass`: `hr.employee.current_status` transitions `probation → active`. On `extend`: probation extended by N days (capped at the 180-day Vietnamese statutory ceiling). On `terminate`: triggers FR-REW-007 termination flow with `termination_reason: 'probation_failed'`.

The Vietnamese Labour Code statutory probation rules are pre-loaded:
- Definite-term contract ≤ 12 months: max probation 6 days.
- Definite-term contract 1-3 years: max probation 30 days.
- Indefinite-term: max probation 60 days for non-management, 180 days for senior management roles.

The schema validates `probation_period_months` against the statutory ceiling at contract-create time.

**Onboarding signal collection.**

At week 1, week 4, week 8 milestones, CUO/CHRO surfaces a 5-question feedback survey to the new Member ("How clear is your role? Are you stuck on anything? Is your buddy responsive? What's not working?"). Responses ingest as Layer 3 docs in BRAIN with the appropriate ACL (visible to manager + HR/Ops Lead + Founder; never to the buddy). Patterns across new hires inform template improvements.

**Welcome email draft.**

3 days before start_date, CUO/CHRO drafts a welcome email to the new Member's personal email (pre-CyberOS account creation). The draft includes: confirmation of start details, login URL + first-time setup instructions, what to expect on day 1, who their buddy is. HR/Ops Lead reviews and sends.

**Frontend surfaces.**

`/hr/onboarding` view (HR/Ops Lead + Founder + manager + the new Member):
- **HR/Ops Lead view.** All in-flight runs; per-run progress; flagged blockers.
- **Manager view.** Their direct reports' runs; quick "schedule check-in" actions.
- **Member view.** Their own run; checklist with checkboxes; "I'm stuck" button (creates a Notify to buddy + manager).

**MCP tool surface.**

- `cyberos.hr.list_onboarding_templates(role_kind?)` — read.
- `cyberos.hr.get_onboarding_run(employee_id)` — read.
- `cyberos.hr.list_open_onboarding_tasks(member_id?, milestone?)` — read.
- `cyberos.hr.draft_onboarding_run(employee_id, template_id, buddy_id?)` — read; CUO drafts; not committed.
- `cyberos.hr.start_onboarding_run(draft_token)` — `destructive: true; requires_confirmation: true`.
- `cyberos.hr.complete_task(task_id)` — `destructive: false`.
- `cyberos.hr.skip_task(task_id, reason)` — `destructive: true; requires_confirmation: true`.
- `cyberos.hr.suggest_buddy(employee_id)` — read.
- `cyberos.hr.draft_welcome_email(employee_id)` — read.

CUO scope contracts: read + draft allowed; commit operations forbidden.

## Alternatives Considered

- **No structured onboarding; rely on managers.** Rejected: PRD §14.3.1 explicitly scopes onboarding workflows; the failure mode is empirically observed.
- **Single template for all roles.** Rejected: engineering and account-management onboarding diverge sharply.
- **Auto-assign buddy via algorithm only.** Rejected: HR/Ops Lead confirmation is the floor; a wrong buddy is a 90-day frustration.
- **Skip probation tracking; manual.** Rejected: Vietnamese Labour Code mandates explicit probation handling; structured tracking is the floor.
- **Send welcome email automatically.** Rejected: a generic-looking welcome email from the platform (vs. signed by the founder personally) is a cold start. Draft + human-send is the floor.

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) HR/Ops Lead creates a synthetic new-hire run from the engineering template; (2) CUO/CHRO drafts a personalised checklist with KB-page links + buddy suggestion; (3) probation review fires 7 days before probation-end; (4) onboarding signal survey delivers at week 1.
- **Adoption metric.** 100% of P2+ hires use the onboarding workflow end-to-end.
- **Quality metric.** New-hire week-4 self-report ≥ 4/5 on "I felt prepared and supported during onboarding."
- **Latency NFR.** Checklist draft p95 ≤ 6 s.

## Scope

**In-scope.**
- The `hr.onboarding_*` schema (4 tables).
- 5 default role templates seeded (engineering / design / account_management / hr_ops / founder_track).
- CUO/CHRO checklist generator.
- Buddy-pair suggestion.
- First-week meeting schedule drafter.
- Probation-period scheduled job + review flow.
- Onboarding signal surveys at week 1 / 4 / 8.
- Welcome email drafter (pre-start; sent to personal email by HR/Ops Lead).
- `/hr/onboarding` views (HR/Ops Lead / manager / Member).
- The 9 MCP tools.
- Audit integration in scope `hr.onboarding.{tenant}`.

**Out-of-scope (deferred).**
- Automated equipment provisioning (P3 — IT module if it ever exists).
- Termination workflow (FR-REW-007 covers the comp side; the HR side is a thinner cousin in P2 batch-08).
- Multi-language onboarding content (vi-VN parity; en-US for international hires in P3).
- Performance-improvement plans (PIPs) — separate FR in batch-07 LEARN.

## Dependencies

- FR-HR-001.
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002 (Notify/Question/Review modes; persona-scope; CUO/CHRO skill).
- FR-KB-001..003 (KB pages referenced by tasks).
- FR-PROJ-001 / FR-PROJ-007 (Engagement linkage for personalisation).
- FR-EMAIL-001..010 (welcome email path; subsequent 1:1 invites; survey delivery).
- FR-OBS-001 / FR-OBS-002.
- Compliance: Vietnamese Labour Code Articles 24-25 (probation rules); PDPL Decree 13 (onboarding signals are personal data); EU AI Act Article 14 (the AI-drafted checklist is human-reviewed before start; Article 50 disclosure chip on AI-drafted content).
- Locked decisions referenced: DEC-159 (5 default role templates), DEC-160 (probation-period rules at contract-create time enforce statutory ceilings), DEC-161 (welcome email is human-sent not auto-sent).

## AI Risk Assessment

The CUO/CHRO checklist generator + buddy-suggestion + welcome-email-draft + onboarding-signal surveys are AI surfaces visible to a natural person joining the company. EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: HR data + KB pages + Engagement context + recent buddy assignments. No third-party. CUO/CHRO runs through the AI Gateway with persona-stamping.

### Human Oversight

- HR/Ops Lead reviews the AI-drafted checklist before run starts.
- Buddy assignment requires HR/Ops Lead confirmation.
- Welcome email is human-sent.
- Probation review is human-conducted; AI does not decide pass/extend/terminate.
- Onboarding signal responses go to humans for review; AI does not interpret them autonomously.

### Failure Modes

- **Checklist mis-personalisation** (linking to wrong Engagement / wrong KB page). Mitigation: HR/Ops Lead review before run start; Member can flag.
- **Buddy mismatch** (suggested buddy on PTO that overlaps day-1). Mitigation: the suggestion accounts for TIME-002 leave records; HR/Ops Lead has the calendar override.
- **Probation auto-end without review.** The system surfaces 7-day-warning Notify; even if the manager misses, the status transition does not auto-trigger termination — it transitions to `active` (the safer side); explicit termination requires the FR-REW-007 flow.
- **Onboarding signal survey fatigue.** Limited to 3 surveys at week 1/4/8; opt-out per Member.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted onboarding flow, schema, role templates, CUO/CHRO generator, probation mechanics, failure modes.
- **Human review:** `@stephen-cheng` reviewed; HR/Ops Lead role + first new-hire onboarding to be co-authored at PR-review.
