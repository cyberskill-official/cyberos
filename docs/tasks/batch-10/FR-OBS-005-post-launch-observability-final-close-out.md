---
title: "OBS — post-launch observability, customer-success metrics, anti-regression contract, final P4 close-out"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P4 / 2028-Q4"
client_visible: false
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Stand up the **post-launch observability + customer-success metric stack** + final P4 close-out: (1) extend FR-OBS-002's per-module dashboards with **per-tenant tenant-success dashboards** showing each paying tenant's adoption + AI usage + module activation depth + NPS + support-ticket trail + churn-risk score; (2) launch the **first-paying-customer playbook** — a structured 90-day post-trial-signup customer-success workflow with 30/60/90 milestones, automated CUO/CXO health checks, founder personal-touch outreach at week 1 + week 4, and explicit "graduation" criteria; (3) define and enforce the **anti-regression contract** — a CI-blocking suite of "platform must not regress" tests covering all 22 modules' critical paths, run on every deploy, with auto-rollback on failure; (4) wire **continuous-improvement loops** — weekly KPI review, monthly cohort analysis, quarterly business review (CUO drafts, founder signs), annual platform retrospective; (5) close out the **24-month build arc** with the final P4 → GA exit-gate evidence map (FR-OBS-003 + FR-OBS-004 pattern, fourth and final): SOC 2 Type II report, ISO 42001 certificate, ≥ 10 paying customers, ≥ 30 days of multi-tenant load with all SLOs green, founder-attested CUO autonomy at C-skill level, audit-chain entry covering the 24-month milestone-arc check from PRD §1.4. After this gate, CyberOS is "general availability" — the build phase ends and the operations phase begins.

## Problem

PRD §14.5.2 P4 → GA exit-gate: "≥ 10 paying customers retained for ≥ 30 days; SOC 2 Type II + ISO 42001 issued; multi-tenant load test passes at 500 tenants × 10 users; CUO has produced ≥ 3 quarterly state-of-business reports unaided; founder + Engineering Lead + DPO sign GA-readiness; 24-month milestone-arc check completes."

Three failure modes if the post-launch surface isn't built carefully:

- **Adoption blind spots.** Without per-tenant adoption metrics, churn-risk tenants are invisible until they cancel. Mitigation: per-tenant dashboards + churn-risk scoring + early-warning Notify.
- **Regression surprises.** A platform shipping daily without CI-blocking regression tests will eventually break a critical path. Mitigation: anti-regression contract.
- **Drift from CyberSkill's own dogfood usage.** If CyberSkill stops using CyberOS for its own ops, the platform loses the founder's eyes. Mitigation: dogfood-coverage metric tracked + reviewed weekly.

## Proposed Solution

### Per-tenant tenant-success dashboard

`/obs/tenant-success` — Founder + Engineering Lead + DPO + Account Manager:

For each active paying tenant, surface:
- **Activation depth:** which modules are active (≥ 1 user-action/week).
- **Adoption velocity:** week-over-week growth in active modules + active users.
- **AI usage:** % of plan budget consumed; trajectory.
- **CUO acceptance rate:** rolling-7-day; trend.
- **NPS:** in-product survey at days 14, 30, 60, 90, then quarterly.
- **Support tickets:** opened, resolved, average resolution time, sentiment.
- **Churn-risk score:** composite metric (low CUO acceptance + low module activation + low NPS + open critical tickets); 0-100 with threshold for Notify.
- **Health classification:** healthy / watch / at-risk / churning.
- **Account Manager notes:** free-text per-tenant context.

A "view trend" deep-link per tenant opens a per-tenant detail dashboard with 90-day charts.

Schema:
```sql
CREATE TABLE obs.tenant_health (
  tenant_id UUID NOT NULL,
  evaluation_date DATE NOT NULL,
  activation_depth_pct NUMERIC,                                              -- % of available modules active
  active_users_7d INT,
  ai_usage_pct_of_budget NUMERIC,
  cuo_acceptance_rate_7d NUMERIC,
  nps_latest NUMERIC,
  nps_response_count INT,
  support_tickets_open INT,
  support_tickets_avg_resolution_hours NUMERIC,
  churn_risk_score NUMERIC,                                                   -- 0-100
  health_classification TEXT NOT NULL,                                        -- "healthy" | "watch" | "at_risk" | "churning"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, evaluation_date)
);
```

Daily evaluator job runs at 06:00 ICT.

### First-paying-customer playbook

90-day onboarding-to-success flow for every new paying tenant:

- **Day 0** (signup → trial start): automated welcome email + first-run wizard logged; CUO greets in-product.
- **Day 1**: founder personal-touch email ("hi, I'm Stephen, you're customer #N — what brought you here?"). Manual; tracked.
- **Day 3**: in-product Notify if no module activation yet ("the team helped X get started — want a 15-min walkthrough?").
- **Day 7**: NPS micro-survey (single question: "How likely are you to recommend CyberOS so far?"); CUO check-in chat.
- **Day 14** (trial midpoint): trial-status email; full NPS survey; Account Manager review of `obs.tenant_health` row.
- **Day 30** (post-conversion): 30-day milestone review; CUO produces a "your first month with CyberOS" summary; founder personal email.
- **Day 60**: 60-day NPS; module-expansion suggestions ("you've been heavy on PROJ — try CRM next?").
- **Day 90** (graduation): tenant moves from "first 90 days" cohort to "ongoing" cohort; quarterly review cycle takes over.

Schema:
```sql
CREATE TABLE obs.customer_success_milestone (
  tenant_id UUID NOT NULL,
  milestone_kind TEXT NOT NULL,                                              -- "day_1_email" | "day_3_notify" | …
  scheduled_at TIMESTAMPTZ NOT NULL,
  completed_at TIMESTAMPTZ,
  completed_by_user_id UUID,
  notes_md TEXT,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, milestone_kind)
);
```

Account Manager dashboard surfaces upcoming + overdue milestones.

### Anti-regression contract

A CI-blocking test suite at `infra/anti-regression/`:

- **Per-module critical-path tests**: each of the 22 modules has a Gherkin-formatted critical-path test that exercises the canonical flow.
  - AUTH: passkey signup + login + step-up.
  - AI: AI Gateway round-trip + persona-version stamping.
  - MCP: tool-call with persona-scope contract enforced.
  - BRAIN: PUT + QUERY + UPDATE + DELETE + cross-module retrieval.
  - GENIE: persona invocation + skill-version + dual-sign.
  - CHAT: message + CaMeL anti-injection.
  - EMAIL: send + receive + CaMeL.
  - PROJ: issue create + status change.
  - … etc. across all 22 modules.
- **Cross-tenant invariant tests** (extends FR-TEN-001): every release runs the cross-tenant leakage suite on a synthetic-tenant pair.
- **Cross-module integration tests**: PROJ → TIME → INV → REW chain works end-to-end.
- **Compensation/equity human-decision invariant**: AI-only path to compensation/equity write returns 403; only human + step-up succeeds.
- **Persona-scope contract tests**: every persona × every tool combination tested for permit/deny correctness.
- **EU AI Act invariant**: every high-risk module has a working human-in-the-loop endpoint; AI-only bypass returns 403.
- **Audit-chain integrity**: external CLI verifier passes against the canonical audit chain.

CI policy: any failed test blocks the deploy. Auto-rollback if a test fails post-deploy (canary catches it before full rollout).

Schema:
```sql
CREATE TABLE obs.regression_test_run (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  deploy_id UUID NOT NULL,
  ran_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  total_tests INT NOT NULL,
  passed INT NOT NULL,
  failed INT NOT NULL,
  duration_seconds INT NOT NULL,
  failed_test_ids TEXT[],
  status TEXT NOT NULL,                                                       -- "passed" | "failed"
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);
```

### Continuous-improvement loops

- **Weekly KPI review** (Friday 16:00 ICT): Founder + Engineering Lead + Account Manager + DPO; reviews tenant_health rollup, anti-regression status, AI usage, support queue.
- **Monthly cohort analysis**: per-month-of-signup retention curves; churn drivers analysed.
- **Quarterly business review (QBR)**: CUO drafts; Founder + Engineering Lead + DPO + Account Manager review + sign; published to all employees + Trust Center (redacted version).
- **Annual platform retrospective**: full architectural retrospective; what's surviving, what's drift, what's debt; planning input for next year's roadmap.

The QBR surface is the same `obs.cuo_unaided_report` schema from FR-OBS-004 — the founder-edit-≤-10% threshold continues post-GA as a core measure of CUO autonomy.

### P4 → GA exit-gate evidence map

Reusing FR-OBS-003 + FR-OBS-004's `obs.gate_*` schema with `gate_kind = 'p4_to_ga'`:

| Code | Description | Threshold |
|---|---|---|
| `paying_customers_30day` | ≥ 10 paying tenants retained ≥ 30 consecutive days | count >= 10 |
| `multi_tenant_load_500` | 500 tenants × 10 users × 1,000 BRAIN ops/day × 7 days, zero leakage | passed = true |
| `soc2_type2_issued` | SOC 2 Type II report issued | bool = true |
| `iso_42001_issued` | ISO/IEC 42001 certificate issued | bool = true |
| `eu_ai_act_attested` | EU AI Act conformity declaration signed | bool = true |
| `cuo_3_unaided_quarterly_reports` | CUO has produced ≥ 3 quarterly reports with founder-edit ≤ 10% chars | count >= 3 |
| `cuo_acceptance_60_rolling` | CUO acceptance rate ≥ 60% rolling-7-day for ≥ 60 days | days >= 60 |
| `nfr_full_coverage_green_30d` | All §11.2 NFRs green for ≥ 30 consecutive days | green_streak >= 30 |
| `compliance_cockpit_green_all_9` | All 9 regimes green for ≥ 30 consecutive days | all_green = true |
| `tenant_admin_console_self_service` | ≥ 90% of tenant admin actions completed without CyberSkill support contact | pct >= 90 |
| `enterprise_security_review_self_service` | First 10 enterprise prospects' security reviews completed via Trust Center alone | count >= 10 |
| `external_pilots_nps_8` | First 10 paying tenants' NPS ≥ 8 | count >= 10 |
| `anti_regression_zero_critical` | Zero critical anti-regression test failures in last 30 days | count = 0 |
| `founder_ga_signing` | Founder GA-readiness signed | bool = true |
| `engineering_lead_ga_signing` | Engineering Lead GA-readiness signed | bool = true |
| `dpo_ga_signing` | DPO GA-readiness signed | bool = true |
| `24_month_milestone_arc_check` | The PRD §1.4 milestone-arc audit-log entry at Month 24 | exists = true |

Daily evaluator job; gate-readiness dashboard at `/obs/gate-readiness/p4-to-ga`.

GA Phase-Exit RFC: same 5-party sign as FR-OBS-004 (Founder + Engineering Lead + DPO + auditor letters + first-3-paying-customers letters).

### Final close-out artefacts

After GA sign-off:

- **`docs/build-retrospective.md`** — what shipped vs PRD, what slipped, what surprised; honest pre-mortem of the build phase.
- **`docs/architecture-current-state.md`** — canonical architecture diagram updated to GA state; living doc onward.
- **`docs/operations-runbook.md`** — every operational procedure; on-call routing; incident response.
- **`docs/roadmap-year-3.md`** — the next 12-month roadmap (post-GA continuous-improvement direction).
- **Public blog post** — "How we built CyberOS in 24 months" — Founder-authored, published to cyberos.world/blog.
- **Trust Center final state** — all artefacts in place per FR-GTM-001 + FR-GTM-002.

## Out of Scope

- New module development (post-GA roadmap; handled in `roadmap-year-3.md`).
- AI model training or fine-tuning (CyberOS architecture stays RAG + persona Skills + frontier models; no in-house fine-tuning).
- Multi-region failover / active-active across shards (deferred to Year 3+).
- Public ESOP plan (CyberSkill's internal ESOP is FR-ESOP-001; no public phantom-stock product).
- Acquisition / fundraising activities (separate process).

## Dependencies

- FR-OBS-001/002/003/004 (entire OBS module).
- FR-AUTH-002 (audit chain).
- FR-GENIE-001/004 (CUO unaided-report).
- FR-CP-005 + FR-GTM-002 (compliance certificates).
- FR-TEN-001/002/003/004/005 (tenancy + admin console).
- FR-PORTAL-001/002/003 (PORTAL).
- FR-API-001/002 (public APIs).
- FR-GTM-001 (Trust Center).
- All other FRs (anti-regression coverage spans every module).

## Constraints

- **Anti-regression suite is mandatory.** Cannot be skipped on deploy.
- **CUO unaided-report 10% threshold preserved.** Same DB CHECK constraint from FR-OBS-004.
- **Tenant data sovereignty preserved.** Cross-shard reads/writes forbidden; auto-test on every release.
- **Founder personal-touch at week 1 + week 4 mandatory** for every paying tenant (cannot be delegated to CUO; must be founder-signed). Scaling rule: when paying tenant count > 50, the rule relaxes to "Account Manager personal-touch + founder reviews weekly cohort summary".
- **No silent breaking changes.** Every breaking change requires major-version bump + 12-month deprecation + tenant Notify.
- **GA gate is the LAST gate.** No further phase gates after GA; ongoing operations follow the continuous-improvement cadence.

## Compliance / Privacy

- `obs.tenant_health` aggregates per-tenant data; access scoped to Founder + Engineering Lead + DPO + Account Manager.
- Per-tenant detail views show data internal to one tenant (not aggregated across tenants for that view); cross-tenant aggregation is allowed at Founder + Engineering Lead level only.
- Customer success milestones contain personal data (Account Manager notes); retained 24 months then archived.
- QBR drafts (CUO unaided-reports) follow FR-OBS-004's persona-version + skill-version + audit-chain rules.
- No new compliance regime introduced; all existing controls apply.

## Risk Assessment (AI-emitting features)

- **CUO QBR drafting:** continues from FR-OBS-004 with the same Article 50 + Article 14 controls + 10% founder-edit threshold.
- **Churn-risk scoring:** deterministic threshold rule; not AI; `eu_ai_act_risk_class: not_ai`.
- **CUO/CXO in-product check-ins** during 90-day playbook: covered by FR-GENIE-001 + FR-PORTAL-003 risk classifications.

The FR itself is `eu_ai_act_risk_class: not_ai` (it is observability + process; AI surfaces are inherited from other FRs).

## Vietnamese-locale considerations

- vi-VN tenant-success dashboards.
- Vietnamese 90-day playbook templates (founder personal-touch emails in Vietnamese for vn-shard tenants).
- Vietnamese CUO QBR reports.
- Vietnamese-locale anti-regression test fixtures (PGroonga tokenisation, vi-VN-format dates, VND amounts).
- Vietnamese Tết holidays accounted for in schedule (no day-3 Notify lands during Tết).

## Scope (acceptance criteria — auditable)

- [ ] `obs.tenant_health` schema applied; daily evaluator job populates rows.
- [ ] `/obs/tenant-success` dashboard live with all listed metrics.
- [ ] Per-tenant detail views work; 90-day charts render.
- [ ] Churn-risk score formula documented + tested.
- [ ] `obs.customer_success_milestone` schema applied; 90-day playbook scheduling works.
- [ ] First-paying-customer playbook executes for ≥ 1 real tenant end-to-end (every milestone reached).
- [ ] Anti-regression suite live in CI; covers all 22 modules' critical paths + cross-tenant invariants + persona-scope contracts + EU AI Act high-risk human-in-the-loop + audit-chain integrity.
- [ ] Auto-rollback on canary failure tested in staging.
- [ ] Weekly KPI review held + minutes captured ≥ 4 weeks running.
- [ ] Monthly cohort analysis published ≥ 1 cycle.
- [ ] Quarterly QBR produced via FR-OBS-004's pattern; founder-edit ≤ 10%; all 4-party sign chain.
- [ ] P4 → GA gate criteria seeded into `obs.gate_criterion` for `gate_kind = 'p4_to_ga'`.
- [ ] Daily evaluator updates all 17 GA criteria.
- [ ] Gate dashboard `/obs/gate-readiness/p4-to-ga` live; Founder + Engineering Lead + DPO can see current status.
- [ ] When all criteria green for ≥ 30 days: GA Phase-Exit RFC sign flow available.
- [ ] On RFC sign: `obs.phase_exit_rfc` row created; audit-chain entry includes 24-month milestone-arc check.
- [ ] Compliance Cockpit P4 → GA panel flips green.
- [ ] `docs/build-retrospective.md` + `docs/architecture-current-state.md` + `docs/operations-runbook.md` + `docs/roadmap-year-3.md` published.
- [ ] Public blog post "How we built CyberOS in 24 months" published.
- [ ] Trust Center final state verified.

**Gherkin (PRD §19.18).**

```gherkin
Feature: Anti-regression suite blocks deploy on critical-path failure

  Scenario: A change to AI Gateway breaks the persona-scope contract test
    Given a developer pushes a PR that breaks the persona-scope contract enforcement
    When the anti-regression suite runs in CI
    Then the persona-scope-contract test fails
    And the deploy is blocked
    And the developer is notified with the failing test + stack trace
    And no canary rollout occurs
    When the developer fixes the regression and re-pushes
    Then the suite passes and the deploy proceeds

Feature: P4 → GA exit-gate cannot sign without 24-month milestone-arc check

  Scenario: Founder attempts to sign GA before milestone-arc check
    Given 16 of 17 P4 → GA criteria are green
    And the 17th (`24_month_milestone_arc_check`) requires the audit-chain entry to exist
    And the audit-chain entry has not been written yet (Month 23)
    When Founder navigates to /obs/gate-readiness/p4-to-ga/sign
    Then the "Sign GA Phase-Exit RFC" CTA is disabled
    And the UI shows "1 criterion not yet green: 24_month_milestone_arc_check (target Month 24)"
    When Month 24 arrives and the audit-chain entry is written
    Then the criterion flips green
    And the CTA enables
    And the GA Phase-Exit RFC sign flow proceeds

Feature: CUO QBR preserves the 10% founder-edit threshold

  Scenario: Quarter 5 post-launch — Founder publishes the QBR
    Given CUO has drafted Q5 QBR with char_count = 8,000
    When the Founder finalises with char_count = 8,400 (5% increase)
    Then obs.cuo_unaided_report.founder_edit_pct_chars = 5
    And the report is recorded as unaided
    And the count of unaided quarterly reports increments
    And the criterion `cuo_3_unaided_quarterly_reports` advances toward its threshold
```

## Success Metrics

- 17 / 17 P4 → GA criteria green for ≥ 30 days at sign-time.
- ≥ 10 paying tenants retained ≥ 30 days at gate-window.
- Anti-regression: zero critical failures in last 30 days at gate-window.
- Tenant churn: ≤ 5% in first 90 days post-trial-conversion.
- CUO acceptance rate ≥ 60% rolling-7-day for ≥ 60 days at gate-window.
- 90-day playbook completion: 100% of milestones reached for first 10 paying tenants.
- Weekly KPI review: ≥ 90% attendance over a 12-week rolling window.
- Customer NPS: ≥ 8 average across first 10 paying tenants.

## Open Questions

- **OQ-OBS-005-01.** Should the anti-regression suite run on every PR (slow CI) or only on merges to main (faster PR loop, slower main)? Default: PR for fast subset (AUTH, AI, MCP, audit-chain) + main for full suite + nightly full + weekly full with stress.
- **OQ-OBS-005-02.** Should `obs.tenant_health` aggregate metrics be exposed via the public API? Default: no (admin-internal); each tenant sees its own from `/admin/overview`.
- **OQ-OBS-005-03.** When CyberSkill's tenant count exceeds 50, should the founder-personal-touch rule relax automatically or require explicit founder sign-off? Default: automatic relaxation at 50 with a Notify to founder.
- **OQ-OBS-005-04.** Should we publish an annual transparency report (similar to Slack, Stripe, etc.) covering DSAR statistics, law-enforcement requests, anti-regression-failure incidents? Default: yes — first transparency report at GA + 12 months.

## References

- PRD §1.4 24-month milestone arc.
- PRD §14.5.2 P4 → GA exit-gate.
- PRD §11.2 NFR catalogue.
- SRS Decisions Log: DEC-019..DEC-023, DEC-050..DEC-052.
- FR-OBS-001/002/003/004 (full OBS arc).
- FR-AUTH-002 (audit chain).
- FR-GENIE-001/004 (CUO).
- FR-CP-005, FR-GTM-002 (final compliance close).
- FR-TEN-001..005, FR-PORTAL-001/002/003, FR-API-001/002, FR-GTM-001.
- All other FRs (anti-regression coverage).

---

*ai_authorship: co_authored — drafted by Claude Cowork on 2026-05-03. Closes Batch 10 — the final batch of the CyberOS FR backlog. 100 / 100 FRs complete.*
