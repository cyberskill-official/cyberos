---
title: "OBS — full SLO definitions, alerting routes, per-module dashboards, NFR measurement reports"
author: "@stephen-cheng"
department: engineering
status: ready_for_review
priority: p0
created_at: "2026-05-03"
ai_authorship: co_authored
feature_type: infrastructure
eu_ai_act_risk_class: not_ai
target_release: "P0 / 2026-Q3"
client_visible: false
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary

Extend the OBS skeleton (FR-OBS-001) into the full P0-exit observability surface: per-module **Service-Level Objectives (SLOs)** with explicit error budgets, the **alerting matrix** routing every alert class to its right destination (PagerDuty for sev-0; CHAT for sev-1; Genie panel for sev-2), per-module **Grafana dashboards** authored by each module's owner against the canonical metrics contract, the **NFR measurement reports** that PRD §17.6 requires for the P0 → P1 exit gate (every PRD §11.2 NFR has a green/yellow/red status with a measurement source), and the **on-call runbook library** with one runbook per alert class. This FR is the difference between "the platform observes itself" and "the platform produces the evidence stream that lets us pass a gate review".

## Problem

FR-OBS-001 ships the skeleton — Loki, Prometheus, Tempo, the Compliance Cockpit shell, the Trust Center page. P0 → P1 exit (PRD §14.1.3, §17.6) requires the substantive content: every NFR measured, every alert routed, every module's panel populated, every runbook written. Without those, a phase-gate review has no evidence to evaluate.

Three failure modes a small team must avoid:

- **Alert fatigue from misrouted noise.** A sev-1 alert that reaches the founder's pager at 03:00 ICT trains him to silence the pager; the next sev-0 sleeps. The routing matrix is the discipline.
- **A green dashboard with no SLO definition.** "Latency is fine" is meaningless without a target; the SLO is the contract that turns observation into accountability.
- **Runbook gaps.** An alert without a runbook means the on-call invents the response under duress; a wrong response in the first minute is the most expensive minute of an incident.

## Proposed Solution

The shape of the answer is a per-module SLO definition file + an alerting routing tree + per-module Grafana dashboard authored against the canonical contract + a runbook library.

**SLO definitions.** Each module's repo carries an `obs/slo.yaml` declaring its SLOs:

```yaml
module: brain
slos:
  - id: brain.retrieval.latency
    description: "Layer 2 hybrid-retrieval p95 latency"
    sli:
      type: latency
      query: histogram_quantile(0.95, sum(rate(cyberos_brain_retrieval_duration_seconds_bucket[5m])) by (le, tenant_id))
      threshold_ms: 600
    objective:
      target_pct: 99.0
      window_days: 30
    error_budget_burn_alerts:
      - rate: 14.4   # exhausts budget in 1 hour
        severity: sev-0
      - rate: 6      # exhausts budget in 4 hours
        severity: sev-1
      - rate: 1      # exhausts budget in 30 days
        severity: sev-2
  - id: brain.fact.denylist_violation
    description: "Compensation/equity values appearing in brain.fact"
    sli:
      type: count
      query: cyberos_brain_denylist_violation_total
    objective:
      target_value: 0
      window: forever
    error_budget_burn_alerts:
      - rate: any
        severity: sev-0
  …
```

The SLO config lands in a single shared Grafana SLO dashboard plus per-module panels. The error-budget burn-rate model (Google SRE) is applied: fast burn = sev-0 page; medium burn = sev-1 CHAT alert; slow burn = sev-2 Genie panel Notify.

**P0 SLO baseline by module.**

| Module | SLO | Target | Window |
|---|---|---|---|
| INFRA / Federation | Apollo Router availability | 99.95% | 30d |
| INFRA / Federation | Cross-tenant leakage | 0 | forever |
| AUTH | Login success rate | ≥ 99.9% | 30d |
| AUTH | Audit chain integrity | 100% | forever |
| AI Gateway | p95 latency budget breach | < 1% | 30d |
| AI Gateway | Persona scope violation | 0 | forever |
| AI Gateway | Monthly LLM spend | ≤ $150 internal | per month |
| MCP | Tool-call p99 proxy overhead | ≤ 12 ms | 30d |
| MCP | Audience-bound token rejection failure | 0 | forever |
| BRAIN | L2 retrieval p95 | ≤ 600 ms | 30d |
| BRAIN | Citation drift | 0 | forever |
| BRAIN | Denylist violation | 0 | forever |
| BRAIN | L1→L2 mirror lag p95 | ≤ 5 s | 30d |
| GENIE | Acceptance rate (rolling 7d, all modes) | ≥ 40% | rolling |
| GENIE | Eval-suite pass rate | ≥ 95% per category | per release |
| GENIE | Persona scope violation | 0 | forever |
| GENIE | Kill-switch propagation | ≤ 30 s | per event |
| CHAT | Message-post p95 | ≤ 300 ms | 30d |
| CHAT | CaMeL escape | 0 | forever |
| CHAT | Whisper transcription p95 (60 s msg) | ≤ 8 s | 30d |
| OBS | Trust Center uptime | 99.9% | 30d |

**Alerting matrix.**

| Severity | Trigger examples | Destination | Response SLA |
|---|---|---|---|
| **sev-0** | Cross-tenant leak; audit chain break; persona scope violation; CaMeL escape; production data loss; gateway down > 60 s | PagerDuty page (founder + Engineering Lead) | 5 min ack; 30 min mitigation start |
| **sev-1** | Latency SLO budget burning fast; AI Gateway 5xx > 5% / 60 s; signing-key rotation failure; auto-pause triggered on a persona | CHAT `#ops-alerts` channel + Genie Notify card | 30 min ack; 4 h mitigation |
| **sev-2** | Background-task failures; rate-limit hot paths; LLM cost approaching ceiling | Genie panel Notify card | reviewed in next standup |
| **sev-3** | Informational; capacity trend alerts | OBS dashboard only; no notification | reviewed weekly |

PagerDuty rotation in P0 is a two-person rotation (founder + Engineering Lead, when hired); the founder covers both slots until the hire. PagerDuty's escalation policy: ack-by-2-min or escalate to the next person; ack-by-5-min or escalate to the team channel.

**Per-module Grafana dashboards.** Every module ships a dashboard JSON in its repo at `obs/dashboards/<module>.json`; CI imports the dashboards via Grafana provisioning. Required panels per dashboard:

- "Service health" (request rate, error rate, p50/p95/p99 latency, by tenant).
- "SLO burn" (error budget remaining + burn-rate trend).
- "Saturation" (CPU, memory, IO, connection pool).
- "Top errors" (the last 50 distinct error messages with counts).
- "Audit volume" (rows written to `audit.entry` in scope `<module>.<tenant>`).
- Module-specific panels (BRAIN: retrieval source mix; GENIE: persona acceptance; CHAT: CaMeL drop count; etc.).

The dashboards are reviewed by the Engineering Lead during the weekly Friday demo (PRD §17 cadence).

**NFR measurement reports.** Every PRD §11.2 NFR has a measurement source declared in `obs/nfr_map.yaml`:

```yaml
NFR-PERF-AUTH-001:
  description: "Authenticated request p95 latency ≤ 250 ms"
  measured_by: prometheus_query
  query: histogram_quantile(0.95, sum(rate(cyberos_request_duration_seconds_bucket{module="auth"}[5m])) by (le))
  threshold: 0.250
  status: green | yellow | red
NFR-SEC-AUDIT-001:
  description: "Audit chain integrity = 100%"
  measured_by: nightly_verifier_job
  query: SELECT COUNT(*) FROM audit_chain_breaks_24h
  threshold: 0
  status: green
NFR-USAB-004:
  description: "Founder daily review session ≤ 10 minutes (median)"
  measured_by: trace_query
  query: …
  threshold_seconds: 600
…
```

The `cyberos-nfr-reporter` job runs daily; produces `obs/nfr_report_{date}.md` checked into the platform repo for audit. P0 exit-gate review reads this report.

**Runbook library.** Every alert class has a runbook at `obs/runbooks/<alert_id>.md` with:

- TL;DR of what the alert means.
- First diagnostic steps (commands to run, dashboards to open, queries to check).
- Common root causes with mitigation steps.
- Rollback procedure (if applicable).
- Escalation path.
- Related alerts (so a cluster of alerts is not chased separately).

The runbook is itself audit-logged on every read so the on-call's path is reconstructable for the post-incident review.

**Synthetic on-call drill.** Once per week during P0, a synthetic alert is fired (configured in PagerDuty's test mode); the on-call must acknowledge within SLA and walk the runbook end-to-end. Drills are required for the P0 → P1 exit gate; the drill log is attached to the gate-readiness report.

**Compliance Cockpit data feed.** The cockpit shell from FR-OBS-001 is now fed by:

- `cp.regime` rows from the CP module (FR-CP-001 in this batch).
- `cyberos_meta.dpia` rows for Article 35 GDPR + PDPL DPIA evidence.
- The audit chain verification job's nightly status.
- The NFR report's per-row status.
- The persona-quality dashboard's acceptance rate.

The cockpit's row-by-row status updates reactively as the underlying signals change; no manual YAML maintenance.

**Trust Center extensions.** The status page from FR-OBS-001 is extended with:

- A 90-day uptime SLA report per major service.
- An incident archive with lessons-learned summaries.
- A subscribe-via-email + RSS + JSON feed for status changes.
- A "for procurement" page linking to the public-facing compliance summary (regime → status, no internal evidence).

**MCP tools (read-only, extending FR-OBS-001).**

- `cyberos.obs.list_slos(module?)` — read.
- `cyberos.obs.get_slo_status(slo_id)` — read.
- `cyberos.obs.list_active_alerts` — read.
- `cyberos.obs.get_runbook(alert_id)` — read.
- `cyberos.obs.list_nfr_status` — read; for the founder's gate-readiness review.

## Alternatives Considered

- **Single global SLO instead of per-module SLOs.** Rejected: per-module ownership is the architectural backbone; SLOs must be per-module to drive accountability.
- **Alertmanager → Slack only (no PagerDuty).** Rejected: sev-0 alerts need a paging surface that survives a Slack outage; PagerDuty is the redundant path.
- **One mega-dashboard for the whole platform.** Rejected: a single dashboard with 200 panels is not consultable under incident pressure; module-scoped dashboards are the floor + a "Platform Overview" composite (already shipped in FR-OBS-001).
- **Skip runbooks; let the on-call read code under duress.** Rejected: the first 15 minutes of an incident are where most damage happens; runbooks compress the response.
- **Run the synthetic drill quarterly instead of weekly.** Rejected: weekly drills surface drift in alerting + dashboards rapidly; quarterly is too slow for a 12-week phase.

## Success Metrics

- **Primary metric.** S0-6 demo passes: (1) every P0 module has an SLO YAML committed and a dashboard imported into Grafana; (2) the alerting matrix routes a synthetic sev-0 to PagerDuty in ≤ 60 seconds; (3) every PRD §11.2 NFR has a row in the NFR-report with a measured status; (4) the runbook library covers every alert class in the matrix; (5) one synthetic on-call drill has been completed end-to-end with the founder responding within SLA; (6) the Compliance Cockpit shows green on Decree 20 SME regime fed by real CP data (FR-CP-001).
- **NFR-coverage metric.** 100% of PRD §11.2 NFRs have a measurement source; ≥ 90% are green at P0 exit; any yellow/red has a documented mitigation plan in the gate-readiness report.
- **MTTD / MTTR.** MTTD (mean time to detection) ≤ 5 minutes for sev-1+; MTTR (mean time to mitigate) ≤ 30 minutes for sev-1; ≤ 4 hours for sev-2.

## Scope

**In-scope (S0-5 + S0-6).**
- Per-module `obs/slo.yaml` declarations + Grafana SLO dashboard.
- Alerting routing tree configured in Alertmanager; PagerDuty integration.
- Per-module Grafana dashboards authored and provisioned.
- `obs/nfr_map.yaml` covering every PRD §11.2 NFR.
- `cyberos-nfr-reporter` daily job producing the NFR report.
- Runbook library at `obs/runbooks/`.
- Synthetic on-call drill schedule and tooling.
- Compliance Cockpit fed by real CP module data (depends on FR-CP-001 in this batch).
- Trust Center extensions (90-day uptime SLA, incident archive, subscribe).
- The five new MCP tools.

**Out-of-scope (deferred).**
- SLI ingestion from external services (P1; e.g. Bedrock SLA tracking via CloudWatch).
- Customer-facing SLA reporting per tenant (P4 PORTAL).
- Cost-anomaly detection beyond the existing budget alerts (P2).
- Synthetic load generators (P1; the load-test rig lands as a separate FR alongside PROJ stress testing).

## Dependencies

- FR-OBS-001 (skeleton).
- FR-INFRA-001 (Loki + Prometheus + Tempo).
- FR-AUTH-002 (audit-log driven panels).
- FR-AI-001 (AI cost panels).
- FR-MCP-001 (read tools).
- FR-CP-001 (Compliance Cockpit data feed).
- FR-GENIE-002 (acceptance-rate panels).
- All other P0 modules ship their own SLO YAML + dashboard JSON — required for "Module Ready" (PRD §7.2).
- PagerDuty account; subscription cost included in the $380/month internal budget envelope.
- Locked decisions referenced: DEC-066 (PagerDuty for sev-0; Slack-Mattermost for sev-1; Genie panel for sev-2).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. OBS observes the platform deterministically; no AI-derived behaviour is in the alerting or SLO path. (Cost-anomaly detection in P2 will introduce limited-AI risk classification at that point.)

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
