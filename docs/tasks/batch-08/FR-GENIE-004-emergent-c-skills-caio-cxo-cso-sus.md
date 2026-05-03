---
title: "GENIE — emergent CUO C-skills: CAIO (AI governance), CXO (experience), CSO-Sus (sustainability) shipped behind feature flags"
author: "@stephen-cheng"
department: product
status: ready_for_review
priority: p2
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "P2 / 2027-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Ship the remaining **CUO emergent C-skills** named in PRD §3.2 + PRD §14.3.1 P2 scope: **CUO/CAIO** (Chief AI Officer — for AI-governance + EU AI Act compliance scrutiny + persona-quality oversight), **CUO/CXO** (Chief Experience Officer — for client + Member experience signals from EMAIL/CHAT/CRM/HR), **CUO/CSO-Sus** (Chief Sustainability Officer — for cost-of-cloud + carbon-impact tracking + ESG-readiness placeholder for P3+). All three ship as Anthropic Skills directories (FR-GENIE-001 format) at `~/.cyberos/skills/cuo/{caio|cxo|cso-sus}/SKILL.md`, dual-signed by founder + Engineering Lead, **feature-flagged off** initially per PRD §14.3.1 ("CUO emergent-role skills (CAIO/CXO/CRO/CSO-Sus) shipped behind feature flags") and turned on selectively as the team validates each. CRO already shipped in FR-CRM-003; CFO in FR-INV-002; CSO in FR-OKR-002. This FR completes the emergent-skills set: CAIO + CXO + CSO-Sus.

## Problem

The PRD's CUO architecture (PRD §6) commits to "ten C-level functions" with extensions for emerging roles. P0 shipped the canonical 3 (CEO, COO, CTO from FR-GENIE-001); P2 added CHRO (HR/REW context), CRO (CRM-003), CFO (INV-002), CSO (OKR-002). The remaining emergent skills cover three distinct concerns:

- **CAIO** — AI governance: persona-quality oversight, EU AI Act compliance review, prompt-injection signal review, persona-version evaluation.
- **CXO** — cross-module customer + Member experience signals: NPS-equivalent inference from CRM activities + EMAIL sentiment + 360 themes + KB usage patterns.
- **CSO-Sus** — sustainability + ESG readiness: cloud-cost trends, hosting-region carbon estimation, ESG-disclosure placeholder.

PRD §14.3.1 explicitly schedules these for P2 behind feature flags. Without shipping them, the C-skill set is incomplete + the persona-versioning + scoping discipline isn't applied uniformly.

## Proposed Solution

The shape of the answer is three Anthropic Skills directories + their persona-scope contracts + integration with existing modules + the feature-flag system + per-skill regression suites.

**CUO/CAIO — AI Governance.**

**Persona purpose.** Surface AI governance signals across the platform; review + flag persona-quality drift; surface EU AI Act compliance gaps; review prompt-injection signal patterns.

**Inputs (read-only access via persona-scope contract).**
- `cyberos.genie.persona_acceptance_rate` (per-persona-version per-mode metrics from FR-GENIE-002).
- `cyberos.cp.list_dpias` (FR-CP-003 DPIA library).
- `cyberos.brain.l3_search` (CaMeL-injection-signal logs from FR-EMAIL-003 + FR-CHAT-001).
- `cyberos.obs.list_active_alerts` (AI-related alerts).
- `cyberos.email.list_dropped_messages` (CaMeL drops from FR-EMAIL-003).
- `cyberos.cp.full_regime_status` (compliance signal aggregation).

**Surfaces.**
- Weekly Notify card to the Founder + Engineering Lead: "AI governance summary — persona acceptance rates, drift signals, compliance gaps."
- Quarterly review report rendered as a Review-mode draft for founder publication.
- Real-time Notify on persona-version regression > threshold.

**Tools forbidden.** Any tool that mutates persona configuration (`persona_publish` / `persona_pause` / `persona_resume`) — these remain founder + Engineering Lead UI-only with step-up. CAIO advises; humans configure.

**Persona Skills directory.** `~/.cyberos/skills/cuo/caio/SKILL.md` with the structured fields from FR-GENIE-001.

**CUO/CXO — Experience signals.**

**Persona purpose.** Aggregate customer + Member experience signals across modules; surface trends + outliers; advise on experience-improvement opportunities.

**Inputs (read-only).**
- `cyberos.crm.list_signals` + `cyberos.crm.list_activities` (sentiment + recency).
- `cyberos.email.search` (CaMeL-sanitised summaries; no per-message PII).
- `cyberos.learn.get_my_360_synthesis` (themes-only, never per-reviewer).
- `cyberos.kb.list_pages` + view counts + comment thread sentiment.
- `cyberos.proj.list_issues` (customer-facing PROJ tasks' resolution patterns).
- `cyberos.hr.list_one_on_ones` (1:1 themes from FR-HR-003 — only when the Member opted in for theme-aggregation).

**Surfaces.**
- Monthly client-experience pulse to Founder + Account Manager: "Acme's last 30-day signal trend is positive (+15% from prior 30); themes: appreciation for proactive communication; concern: feature delivery cadence on the Q3 launch."
- Monthly Member-experience pulse to Founder + HR/Ops Lead: "Team sentiment trend; 360 themes recurring across multiple Members; structural patterns (e.g. 'communication latency between teams' shows up in 3 syntheses)."
- Per-account experience dashboard panel in CRM (FR-CRM-002).

**Tools forbidden.** Per-Member 360 per-reviewer responses; per-Engagement compensation context; PII access beyond what other personas already see.

**Persona Skills directory.** `~/.cyberos/skills/cuo/cxo/SKILL.md`.

**CUO/CSO-Sus — Sustainability.**

**Persona purpose.** Surface cloud-cost trends, hosting-region carbon estimation, ESG-disclosure readiness signals.

**Inputs (read-only).**
- `cyberos.obs.query_metric` (Hetzner cloud cost per region; AWS Bedrock cost from FR-AI-001's `cyberos_meta.ai_call_daily_rollup`; Cloudflare Workers cost; Vercel/CDN cost).
- `cyberos.inv.list_invoices(direction: "inbound", vendor_kind: "saas_subscription")` (vendor + subscription cost + usage).
- Static carbon-intensity table per region (informational reference data).
- `cyberos.proj.engagement_dashboard` (Engagement-attributed cloud usage where attributable; informational).

**Surfaces.**
- Monthly cost + sustainability summary: "AI Gateway cost trended at $128/month vs. $150 ceiling; Bedrock-Singapore region carbon-intensity ~0.4 kg CO2/kWh; alternative regions evaluated; ESG-disclosure status: not-yet-applicable for Vietnamese SME tier."
- Cost-anomaly detection (when monthly cost exceeds budget by > 10%).
- Carbon-intensity comparison: "Frankfurt region's grid is greener than Singapore; consider routing-priority for new EU tenants in P3+."

**P3+ readiness.** ESG disclosure (CSRD in EU; SEC climate disclosure in US) is forward-compatible — the persona's metric collection produces evidence the platform can surface to a future external sustainability-reporting flow.

**Persona Skills directory.** `~/.cyberos/skills/cuo/cso-sus/SKILL.md`.

**Feature-flag system.**

A new `cyberos_meta.persona_feature_flag` table:

```sql
CREATE TABLE cyberos_meta.persona_feature_flag (
  tenant_id UUID NOT NULL,
  persona_skill_id TEXT NOT NULL,                                      -- "cuo-caio" | "cuo-cxo" | "cuo-cso-sus"
  status TEXT NOT NULL DEFAULT 'disabled',                              -- "disabled" | "internal_preview" | "enabled"
  enabled_for_member_ids UUID[],                                        -- when status = "internal_preview"; else null
  enabled_at TIMESTAMPTZ,
  disabled_at TIMESTAMPTZ,
  signed_by_founder_at TIMESTAMPTZ NOT NULL,
  signed_by_engineering_lead_at TIMESTAMPTZ NOT NULL,
  metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
  PRIMARY KEY (tenant_id, persona_skill_id)
);
```

The 3 new skills ship at `status: 'disabled'` initially. Per-skill enablement requires founder + Engineering Lead dual-sign. Internal preview lets specific Members test before full rollout. The Genie panel (FR-GENIE-001) only surfaces enabled skills.

**Regression suite per skill.**

Each new skill ships with a curated 30-case regression eval:
- Citation correctness on the inputs the persona consumes.
- Scope adherence (the persona refuses out-of-scope requests — e.g. CAIO refuses to recommend persona-version changes; CXO refuses to reveal per-Member 360 specifics; CSO-Sus refuses to recommend infrastructure changes that aren't its scope).
- Voice coherence with the parent CUO persona.
- Vietnamese-locale adherence.
- Safety (refusing to engage in adversarial reasoning attempts).

A regression on any category gates the skill's persona-version PR.

**Audit + observability.**

- `genie.emergent_skills.{tenant}` audit scope.
- OBS dashboard panel for per-skill acceptance rates + active feature-flag status per tenant.

**MCP tool surface.**

- `cyberos.genie.list_active_personas` — read; everyone (already in FR-GENIE-001; this FR extends).
- `cyberos.genie.get_persona_status(skill_id)` — read.
- `cyberos.genie.draft_caio_governance_summary` — read; via CAIO scope.
- `cyberos.genie.draft_cxo_experience_pulse(account_id?, scope: "client"|"member")` — read; via CXO scope.
- `cyberos.genie.draft_cso_sus_summary(period?)` — read; via CSO-Sus scope.

There are no mutation MCP tools for the personas themselves — feature-flag flips + persona-version publishes go through founder + Engineering Lead UI + step-up + dual-sign.

## Alternatives Considered

- **Skip the emergent skills until P3.** Rejected: PRD §14.3.1 explicitly schedules them for P2; the architectural slot exists.
- **Auto-enable for the canonical CyberSkill tenant.** Rejected: feature-flag with explicit founder enablement is the floor; allows trial-then-rollout.
- **Combine all three into one "advanced CUO" persona.** Rejected: the persona-scope-contract pattern requires per-skill scope precision.
- **Skip the regression suite for emergent skills.** Rejected: persona-version regression is the structural quality gate (FR-GENIE-002).

## Success Metrics

- **Primary metric.** P2 → P3 exit-gate progress: all 3 emergent skills shipped + signed; ≥ 1 of 3 enabled in internal-preview for the founder + Engineering Lead at P2 → P3 cutover; regression suites pass on every persona-version PR.
- **Adoption metric.** Per-skill acceptance rate ≥ 30% on internal preview before promoting to enabled.
- **Compliance metric.** CAIO weekly governance summary drives ≥ 1 documented compliance follow-up per quarter.

## Scope

**In-scope.**
- 3 Anthropic Skills directories (CAIO, CXO, CSO-Sus) authored + dual-signed.
- 3 persona-scope contracts with explicit forbid lists.
- `cyberos_meta.persona_feature_flag` table + UI + sign-flip flow.
- 3 regression eval suites (30 cases each).
- Cross-module data wiring (CAIO from genie + cp + obs + brain; CXO from crm + email + learn + kb + proj; CSO-Sus from obs + inv).
- 5 read-only MCP tools.
- OBS dashboard panel.
- Audit integration in scope `genie.emergent_skills.{tenant}`.

**Out-of-scope (deferred).**
- Multi-tenant per-tenant CUO persona customisation (P3 — when external tenants).
- ESG external-disclosure flow (P3+).
- Public CAIO governance reports (P4 — Trust Center extension).
- Real-time CAIO drift detection beyond the existing FR-GENIE-002 calibration (P3 — auto-rollback on regression).

## Dependencies

- FR-GENIE-001 / FR-GENIE-002 (persona substrate + persona-version + scope-contract pattern).
- FR-AI-001 (AI Gateway with persona-stamping).
- FR-MCP-001 (persona-scope contract enforcement).
- FR-CRM-003 (CRO precedent), FR-INV-002 (CFO precedent), FR-OKR-002 (CSO precedent), FR-HR/REW (CHRO precedent).
- FR-CP-003 (CAIO consumes DPIA + full-regime status).
- FR-OBS-001 / FR-OBS-002.
- All cross-module data sources per skill.
- Compliance: EU AI Act Articles 5-7 (CAIO is the platform's structural answer to AI-governance scrutiny); Article 50 (transparency disclosure on every emergent-skill output); GDPR Article 22 (no automated decisions; advisory surfaces).
- Locked decisions referenced: DEC-250 (CAIO + CXO + CSO-Sus skills shipped behind feature flag in P2), DEC-251 (per-skill regression suite gates persona-version PRs), DEC-252 (founder + engineering-lead dual-sign required for feature-flag flip).

## AI Risk Assessment

Three new AI surfaces visible to natural persons (founder + Engineering Lead + Account Manager + HR/Ops Lead). EU AI Act risk class: `limited`.

### Data Sources

Per-tenant only: as enumerated per skill. No third-party. Each persona runs through the AI Gateway with persona-stamping. Per-tenant residency.

### Human Oversight

- Feature-flag flip requires founder + Engineering Lead dual-sign.
- Internal-preview model lets the founder validate before full enablement.
- Outputs are descriptive (governance summary, experience pulse, sustainability summary); never auto-action.
- Persona regression suite gates every version PR.

### Failure Modes

- **CAIO false-positive on persona drift.** Mitigation: 7-day-rolling-window before flagging; founder validates before action.
- **CXO leaks per-Member 360 specifics.** Mitigation: persona-scope contract restricts source data to themes-only synthesis (FR-LEARN-003 produces themes; per-reviewer data is structurally outside scope).
- **CSO-Sus suggests cost-cutting that breaks SLOs.** Mitigation: descriptive only; the surface doesn't recommend SLO-affecting infrastructure changes — it surfaces facts.

## AI Authorship Disclosure

- **Tools used:** Claude Sonnet 4.6.
- **Scope:** Drafted the 3 skill scopes, persona-scope contracts, feature-flag flow, regression-suite design, failure modes.
- **Human review:** `@stephen-cheng` reviewed; the founder + Engineering Lead will author the 3 SKILL.md files at PR-review time + sign before P2 production.
