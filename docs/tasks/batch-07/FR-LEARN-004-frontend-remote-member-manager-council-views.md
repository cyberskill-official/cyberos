---
title: "LEARN — frontend remote at /learn (Member career profile, manager view, Council view, training records)"
author: "@stephen-cheng"
department: design
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q2"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the LEARN Module-Federation remote at `/learn` consuming FR-LEARN-001 (VP), FR-LEARN-002 (Council), and FR-LEARN-003 (career path + 360 + next-step). Four primary surfaces: **Member career profile** (`/learn/my`) showing current level + level history, latest VP outcome, latest 360 synthesis, latest next-step recommendation, training records, sabbatical accrual + eligibility (read from FR-TIME-002), pending VP evaluator assignments, open Council cases as subject; **Manager view** (`/learn/team`) showing direct reports' levels + recent VP outcomes (aggregate; no per-criterion details unless explicitly authorised) + 360 cycle progress + next-step recommendations; **Council view** (`/learn/council`) for Council members showing cases assigned for deliberation + their own deliberation drafts + finalised cases; **HR/Ops admin** (`/learn/admin`) for level catalogue management + 360 cycle orchestration + Council composition + rubric publishing. The frontend is the surface that ties together the LEARN cluster + integrates with REW + HR + TIME for context.

## Problem

Three modules (FR-LEARN-001/002/003) without a frontend produce data nobody reads. Three failure modes:

- **Member opacity.** A Member who can't see their level + VP history + next-step recommendation in one place loses the growth-conversation thread.
- **Manager fragmentation.** A manager checking each report's growth across separate UIs (HR for 1:1, REW for VP feed, LEARN for level) wastes time + misses signal.
- **Council friction.** Council members deliberating without a single canonical surface lose context across cases.

## Proposed Solution

The shape of the answer is a Vite + React 19 Module-Federation remote at `/learn` consuming the GraphQL surfaces from FR-LEARN-001/002/003.

**Member career profile (`/learn/my`).**

Default home for any Member.

- **Header.** "Hi <preferred_name>. You're at <level> in <role_kind> for <duration>."
- **Level card.** Current level (chip) + months at level + the level's competency framework (clickable; opens the level definition).
- **Career history.** Chronological list of level changes (from `learn.level_assignment`); each entry shows date + reason (Council case ref clickable; opens read-only Council outcome) + signed-by.
- **Recent VP outcomes.** Last 4 quarters' weighted scores (encrypted; revealed on step-up). Each row clickable; opens the outcome summary (Member sees the outcome narrative; not per-criterion-per-evaluator scores unless rubric's `score_visibility_to_member: detail`).
- **Next-step recommendation card.** The latest `next_step_recommendation`; expanded on first view; collapsible. Includes the "useful / partially useful / not useful" feedback action.
- **360 feedback.** Latest cycle's status + synthesis (when delivered); past cycles in a history accordion.
- **Training records.** Self-recorded + HR/Ops-validated; "+ Add training" button (FR-LEARN-003 mutation MCP).
- **Sabbatical accrual.** Read from FR-TIME-002; "you've accrued X.Y years; next eligibility on YYYY-MM-DD".
- **Pending actions.** VP evaluator assignments (cases where the Member must score someone else); open Council cases where the Member is the subject; pending 360 reviewer assignments.

Every amount-bearing reveal (compensation amounts via VP score + corresponding BP) requires step-up auth (FR-AUTH-003).

**Manager view (`/learn/team`).**

For Members with direct reports.

- **Team grid.** Direct reports as cards; per-report: name + current level + last-VP-score chip + 360 cycle status + next-step "key growth area" excerpt + pending actions count.
- **Click a report.** Opens the report's career profile in a side-drawer (manager-scoped: aggregate VP scores + 360 themes; *no* per-criterion-per-evaluator data).
- **Promotion proposal.** "Propose promotion" CTA per report; opens the Council case-raise form pre-populated with the report's recent VP + 360 + level history.
- **VP cycle dashboard.** Open VP cycles in the team; per-report status (open / scoring / review / signed / fed_to_bp); manager actions (review evaluators' scores, write outcome summary, sign).
- **360 cycle dashboard.** Same shape for 360 cycles.
- **1:1 prep cross-link.** Clicking a report's name jumps to FR-HR-003's 1:1 surface with the report's career profile pre-loaded as context.

**Council view (`/learn/council`).**

For Council members only.

- **Cases inbox.** Cases where the Council member is non-conflicted + has not yet submitted final deliberation. Per-case: subject's name, case kind, raised date, time-to-decision target.
- **Case detail.** Click a case; see the structured context: career history snapshot, evaluation refs (the Member's recent VP outcomes; per-evaluation summary level, not per-criterion-per-evaluator), BP history snapshot (aggregate; never quarterly amounts), the raiser's case context.
- **Deliberation form.** Position picker (approve / reject / defer / abstain) + structured rationale Markdown. Saves drafts; "submit final" applies the immutability trigger (FR-LEARN-002).
- **All-deliberations view.** After the deliberation window closes, the Council member sees other members' deliberations (this is the synchronous-or-async discussion phase). The chair facilitates discussion + decision.
- **Decided-cases archive.** Cases the Council member participated in; the outcome narrative; the chair's signed summary.

**HR/Ops admin (`/learn/admin`).**

For HR/Ops Lead + Founder + DPO.

- **Level catalogue management.** View + edit drafts of new level catalogue versions; founder + engineering-lead sign + publish flow.
- **360 orchestration.** Open / monitor / close 360 cycles; reviewer assignment; synthesis authoring; delivery confirmation.
- **Council composition.** View current composition; manage rotation; publish new compositions (founder + engineering-lead sign).
- **VP rubric management.** Same shape (FR-LEARN-001's rubric publishing flow).
- **Reports.** Aggregate dashboards: level distribution by role; VP-outcome distribution; promotion velocity; 360-completion rate; next-step-feedback distribution.

**Founder views.**

A subset of HR/Ops admin focused on:
- Council case ratification queue.
- Promotion proposal sign queue.
- Rubric publish queue.
- Compliance Cockpit deep-link for LEARN-related metrics.

**Vietnamese-locale rendering.**

- vi-VN default for the canonical CyberSkill tenant.
- Level + competency text rendered in vi-VN by default; en-US fallback for international Members (P3+).

**Performance.**

- Initial JS bundle ≤ 50 KB gzipped.
- `/learn/my` first-paint ≤ 1.5 s on 4G.
- Council case-detail open ≤ 800 ms.
- Step-up reveal of encrypted VP-score detail ≤ 2 s.

**Empty states.**

- New Member with no level history yet: card prompts manager to set initial level.
- Member with no VP outcomes yet: card explains the cadence + expected first cycle.
- Member with no 360 cycle yet: card explains the annual cadence.

**MCP tool surface.**

(All read-only; no mutation MCP for LEARN-004 — the parent FRs ship the few Member-self-serve mutations like training records.)

- `cyberos.learn.my_career_profile` — read; calling Member's own; step-up.
- `cyberos.learn.team_career_summary(member_ids?)` — read; manager (their reports) + HR/Ops + Founder.
- `cyberos.learn.council_inbox` — read; calling Member if council member.
- `cyberos.learn.admin_dashboard_summary` — read; HR/Ops + Founder + DPO; aggregate.

## Alternatives Considered

- **Embed LEARN inside HR-001's frontend.** Rejected: LEARN is a substantive surface deserving its own URL space + sidebar; mixing with HR onboarding clouds the daily-driver UX.
- **Skip Council view; deliberate via email.** Rejected: structured deliberation + immutability + audit are the floor.
- **Skip Member career profile; manager-only views.** Rejected: Members' self-direction toward growth requires their own dashboard.
- **Use Notion-like KB pages for level catalogue.** Rejected: structured GraphQL queryable + parameter-versioned data is the floor; KB pages can mirror for human-readable browsing (P3).

## Success Metrics

- **Primary metric.** P2 sprint demo passes: (1) every Member opens `/learn/my` once during the synthetic Q3; (2) a manager runs the VP cycle through `/learn/team` for their direct reports; (3) a Council member completes a synthetic case via `/learn/council`; (4) HR/Ops Lead orchestrates a 360 cycle via `/learn/admin`.
- **Adoption metric.** ≥ 70% of Members open `/learn/my` at least monthly; managers use `/learn/team` ≥ weekly; Council members use `/learn/council` for ≥ 80% of their assigned cases (vs. external email/Slack).
- **Latency NFR.** Per the budgets above; bundle ≤ 50 KB.

## Scope

**In-scope.**
- The Module-Federation remote at `/learn`.
- All 4 primary surfaces (Member / manager / Council / HR/Ops admin) + Founder subset.
- Cross-FR integration (HR-001/003 + REW-001/002 + TIME-002).
- Vietnamese-locale rendering.
- Step-up auth on every encrypted reveal.
- The 4 read-only MCP tools.
- Audit integration in scope `learn.ui.{tenant}`.
- Mobile-responsive layouts.

**Out-of-scope (deferred).**
- AI-suggested deliberation drafts for Council members (P3 — Review mode; persona-scope contract very narrow).
- Public level-catalogue surface for prospective hires (P4 — recruiting integration).
- Mobile native (P3).
- Analytics on level-distribution drift over time (P3 — informational only).

## Dependencies

- FR-LEARN-001 / FR-LEARN-002 / FR-LEARN-003.
- FR-HR-001 / FR-HR-003.
- FR-REW-001 / FR-REW-002.
- FR-TIME-002 (sabbatical accrual surface).
- FR-INFRA-001 / FR-AUTH-001 / FR-AUTH-002 / FR-AUTH-003 / FR-MCP-001 / FR-AI-001.
- FR-DESIGN-001.
- FR-GENIE-001 / FR-GENIE-002.
- FR-OBS-001 / FR-OBS-002.
- FR-CP-001 (Compliance Cockpit deep-link).
- Compliance: PDPL Decree 13 (career data is personal); EU AI Act Article 50 (next-step recommendation surface renders the disclosure chip).
- Locked decisions referenced: DEC-204 (4-surface frontend layout), DEC-205 (Council deliberation visibility post-window), DEC-206 (manager view aggregate-only on direct-reports' VP detail unless authorised).

## AI Risk Assessment

The frontend itself is deterministic UI; the AI surfaces (next-step recommendation, outcomes-only summariser) inherit FR-LEARN-002/003's classification. EU AI Act risk class: `limited` for the consumer surfaces.

### Data Sources

UI consumes ACL-scoped GraphQL data from the LEARN subgraphs. Per-tenant residency.

### Human Oversight

The frontend is the consumer; mutations go through the parent FRs' sign-and-publish flows. AI-derived elements (next-step card, outcomes summary) carry the disclosure chip + the human-feedback loop.

### Failure Modes

- **Per-reviewer 360 leak through the UI.** Mitigated structurally: the GraphQL queries don't return per-reviewer responses to the subject Member; the synthesis is the only authorised surface.
- **Manager sees too much detail on a peer's career profile.** Mitigated by ACL: a manager only sees direct reports; cross-team views require explicit permission.
- **Member sees their own data without step-up.** Mitigated by FR-AUTH-003 step-up enforcement on encrypted-reveal queries.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted 4-surface layout, mutation flows, persona-scope inheritance, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the founder + first manager + first Council chair will validate the surfaces in a usability walkthrough before P2 production.
