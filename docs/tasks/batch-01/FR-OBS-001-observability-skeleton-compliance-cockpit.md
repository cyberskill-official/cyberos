---
title: "OBS module — observability skeleton (logs, metrics, traces, AI usage), Compliance Cockpit, Trust Center status page"
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

Stand up the OBS module's P0 baseline: a single-pane observability stack (Loki for logs, Prometheus + Grafana for metrics, OpenTelemetry → Tempo for traces, LangSmith for AI-call usage, and the platform's own `audit.entry` table for audit) plus the **Compliance Cockpit** (a red/yellow/green dashboard per regime: PDPL Vietnamese, GDPR, EU AI Act, SOC 2 Type I scope, ISO/IEC 27001 gap-list) plus the public **Trust Center** status page at `https://status.cyberos.world`. OBS is the surface that proves the platform is healthy, that demonstrates the compliance posture to auditors, and that publishes uptime + incident history to anyone who asks. The full SLO definitions, alerting, and dashboards-by-module land in P1 (FR-OBS-002 in batch-02); this FR is the skeleton that every subsequent module feeds into.

## Problem

A platform without observability cannot phase-gate. PRD §17.6 makes the S0-6 sprint exit-criterion explicit: "All P0 NFRs measured and reported; OBS dashboards live; Trust Center status page online; tenant-deletion (RTBE) flow exercised end-to-end on a synthetic tenant; PDPL A05 filing draft prepared; SOC 2 Type I evidence map populated." Without OBS up, none of those evidence streams exist.

Three failure modes a small team must avoid:

- **Discovering an incident from the customer.** Without metrics + alerting, a degradation in BRAIN retrieval latency surfaces as "Genie is slow today" from a Member, not as an automated page. The platform's reliability story degrades fast.
- **Compliance evidence missing at audit.** PDPL A05 filing in P0 plus SOC 2 Type I scope at P3 entry both require evidence streams that started accumulating *at the beginning of P0*. We cannot back-fill.
- **Inability to prove the platform's properties to ourselves.** PRD §4.1 commits to specific NFRs (BRAIN p95 ≤ 600 ms, AI Gateway p95 budgets, Audit chain integrity = 100%, persona scope-violations = 0). Each of those is a number that must come from somewhere; OBS is where they come from.

## Proposed Solution

The shape of the answer is one OBS subgraph + one OBS Module-Federation remote + a Grafana stack + a public status page + a `cyberos-obs-collector` daemon that bridges module signals into the dashboards. Every other module emits OpenTelemetry instrumentation at SDK level; OBS configures the collection.

**Logs — Loki.** Every workload emits structured JSON logs to stdout. A Promtail agent on each node ships logs to a single Loki cluster. Log retention: 30 days hot, 1 year cold (S3 with Object Lock in `Compliance` mode for regulatory queries). Log fields are constrained: `level`, `timestamp`, `service`, `tenant_id`, `request_id`, `subject_id`, `message`, `attributes (object)`. Personal data in logs is redacted by the same Presidio pipeline used in the AI Gateway; the redactor runs in Promtail before the log leaves the node. Logs that fail redaction (e.g. binary uploads accidentally logged) are dropped with a counter incremented.

**Metrics — Prometheus + Grafana.** A Prometheus cluster scrapes every workload via the standard `/metrics` endpoint. Default canonical metrics every CyberOS module emits:

- `cyberos_request_total{module, route, method, status}`
- `cyberos_request_duration_seconds_bucket{module, route, method}`
- `cyberos_persisted_query_unregistered_total{module}`
- `cyberos_audit_chain_break_total{tenant, scope}` (sev-0 alert if > 0)
- `cyberos_rls_denial_total{tenant, table}` (info; spikes investigated)
- `cyberos_mcp_call_total{tool, module, status}`
- `cyberos_ai_call_total{provider, model, route_taken, status}`
- `cyberos_ai_call_cost_usd_micros{tenant, provider, model}`
- `cyberos_ai_persona_violation_total{persona_version, scope_contract}` (sev-1 alert if > 0)
- `cyberos_brain_retrieval_duration_seconds_bucket{tenant}`
- `cyberos_chat_camel_drop_total{tenant}` (info; spikes audited)

Grafana dashboards ship with a P0 baseline set:
- "Platform Overview" — request rate, error rate, p95 latency, tenant count, region split.
- "AI Spend & Quality" — daily cost by tenant by model; persona acceptance rate; latency-budget breach rate.
- "Audit & Compliance" — audit-row volume, chain integrity status, RLS denials, persona scope violations.
- "BRAIN" — retrieval p95, ingestion volume, denylist drops, conflict-detection rate.
- "MCP" — tool-call volume by tool, destructive confirmations, OAuth flow success rate.

The Grafana instance is exposed only inside the cluster + via VPN; not internet-public.

**Traces — OpenTelemetry → Tempo.** Every workload uses the OTel SDK with W3C Trace Context propagation. Sampling: 100% in P0 internal scale; we will sample down to 10% at 50-tenant scale per FR-OBS-002. Span attributes include the canonical CyberOS dimensions: `cyberos.tenant_id`, `cyberos.subject_id`, `cyberos.request_id`, `cyberos.persona_version`, `cyberos.module`, `cyberos.route`. Traces tie back to logs and audit rows by `request_id`; clicking a trace span in Grafana surfaces the matching log lines and audit rows in adjacent panels.

**AI usage — LangSmith.** Every AI Gateway call also emits a LangSmith trace with prompt + response + token counts + persona-version. LangSmith is the operator-facing surface for evaluating persona quality, regression-testing prompts, and reviewing curated cases; the canonical authoritative store remains `cyberos_meta.ai_call` (FR-AI-001). LangSmith is *read* by the founder and Engineering Lead; production decisions reference the in-platform store.

**Audit — Postgres + S3 archival.** The `audit.entry` table (FR-AUTH-002) is queried by OBS for the Compliance Cockpit's audit-volume and chain-integrity panels. Cold archive lands in `s3://cyberos-audit-archive-{region}/` per FR-AUTH-002 §"Retention".

**Compliance Cockpit.** A red/yellow/green dashboard with one column per regulatory regime and one row per evidence requirement:

| Regime | Status indicator | Evidence backing |
|---|---|---|
| **PDPL Decree 13/2023** | green/yellow/red | DPIA template populated; A05 filing draft ready; per-tenant residency verified; consent records present |
| **PDPL Decree 53/2022** | green/yellow/red | Cybersecurity audit trail present; incident response plan filed |
| **PDPL Decree 20/2026 (SME exemption)** | green/yellow/red | SME eligibility declared; exemption claim documented |
| **GDPR (P3+)** | n/a / amber / green | DPIA per high-risk processing; DSAR workflow; right-to-erasure path |
| **EU AI Act Articles 5–7, 14, 50** | green/yellow/red | Risk classification per feature; human-oversight controls; transparency disclosure chips |
| **SOC 2 Type I** | n/a → amber → green | Evidence map population %; security-policy artefacts; control-evidence samples |
| **ISO/IEC 27001** | n/a → amber → green | Gap list closed %; statement of applicability; risk register |

The cockpit reads from the `cp` (Compliance Plane) module's data store (which lands in batch-02 as FR-CP-001); for P0 the cockpit is a shell that renders a hand-curated YAML status file maintained by the founder + DPO until CP-001 ships. The shell is real; the data feed becomes automated as CP-001 lands. The PRD §14.1.3 P0 → P1 exit gate is "Compliance Cockpit shows green on Decree 20 SME regime and Compliance Backlog has zero P0-Sev0 items open."

**Trust Center status page.** A public page at `https://status.cyberos.world` shows:

- Live uptime per major service (Federation gateway, AUTH, AI Gateway, MCP gateway, BRAIN, CHAT) with 90-day rolling SLA reporting.
- Current incidents (with severity + ETR).
- Recent incidents (last 30 days; 12-month archive linked).
- Subscribe to email/RSS.
- Compliance summary: regime → status (no internal evidence shown; only "active / pending / not yet").

The page is a small Next.js static site deployed to Cloudflare Pages, sourced from a `status.yaml` file in the platform monorepo plus live signals from Prometheus blackbox-exporter probes. Incident timelines are managed via PRs to a `incidents/` directory; the runbook lives next to it.

**Per-module instrumentation contract.** Every module shipped from S0-2 onward must implement four properties to be "OBS-ready":

1. Emit the canonical Prometheus metrics for its service.
2. Emit OpenTelemetry traces with the canonical span attributes.
3. Write a `module_health` row every 60 seconds to `obs.module_health` with `version`, `phase`, `module_ready: bool`.
4. Pass a `cyberos-obs-lint` check in CI that verifies (1)–(3) by exercising a synthetic request and checking the resulting telemetry shape.

PRD §7.2 lists this as a "Module Ready" requirement; the lint check is the enforcement.

**Alerting (P0 minimum).** Prometheus Alertmanager routes alerts:
- sev-0 → page the on-call (founder + Engineering Lead) via PagerDuty.
- sev-1 → CHAT alert in `#ops-alerts`.
- sev-2 → Genie panel Notify card.

P0 alerts:
- `audit_chain_break_total > 0`
- `persona_scope_violation_total > 0`
- `cross_tenant_data_leakage > 0` (synthetic test fails)
- `ai_call_error_rate > 5% over 60 seconds`
- `brain_retrieval_p95_ms > 1500 over 10 minutes`
- `gateway_unavailable > 60 seconds`

**MCP tool surface (read-only).**
- `cyberos.obs.list_dashboards` (read).
- `cyberos.obs.query_metric(promql, since, until)` (read; capped to safe-PromQL subset).
- `cyberos.obs.list_recent_incidents(since)` (read).

OBS does not expose a write MCP — incident creation goes through PagerDuty's own flow and is mirrored back into OBS by the alert-manager webhook.

## Alternatives Considered

- **Datadog / New Relic / Honeycomb hosted.** Rejected for P0: residency story, per-tenant cost (Datadog scales with metrics + hosts and would exceed the $380/month internal budget), and the "we own the substrate" property for compliance evidence.
- **Self-host the full stack including Tempo + Loki + Prometheus.** Adopted, but with explicit $380/month total infra ceiling. The stack is small enough at internal scale to fit on the cluster.
- **Skip Trust Center until P3.** Rejected: a status page is the cheapest single trust signal we can publish; not having one when the first external prospect asks is a sales blocker.
- **No LangSmith, only the in-platform `ai_call` store.** Rejected: LangSmith's eval and dataset features are the most productive operator surface for tuning the persona; the in-platform store remains the authoritative source for billing and audit, but LangSmith is the work surface.

## Success Metrics

- **Primary metric.** S0-1 + S0-6 demos pass: (1) every P0 module emits the canonical metrics + traces + module_health rows from the moment it ships, (2) Grafana dashboards are populated for each P0 module, (3) Trust Center is online at `status.cyberos.world` with the seven services listed and 0/0/0/0/… incidents (zero is the P0 reality), (4) Compliance Cockpit shows green on PDPL Decree 20 SME regime with the YAML evidence file populated, (5) the synthetic on-call drill (a fired sev-0 alert) reaches the founder's pager in ≤ 60 seconds.
- **Guardrail metric.** Time-to-detection for a regression: median ≤ 5 minutes for a p95-latency degradation > 25%; ≤ 60 seconds for an audit chain break.
- **NFR coverage.** Every PRD §11.2 NFR has a measurement source by P0 exit (PRD §17.6 cross-sprint theme).

## Scope

**In-scope (S0-1 baseline; S0-6 completion).**
- Loki + Promtail.
- Prometheus + Grafana with the P0 dashboard set.
- OpenTelemetry collector → Tempo (or hosted alternative if approved by the Engineering Lead at S0-2).
- LangSmith integration (project per environment; redaction before send).
- `module_health` table + `obs` GraphQL subgraph + read-only MCP tools.
- Compliance Cockpit shell with the seven-row template.
- Trust Center static site at `status.cyberos.world` with blackbox-probe data feed.
- Alertmanager routing for the P0 alert set.
- `cyberos-obs-lint` CI check (every module CI runs it).
- Synthetic on-call drill scripted and run in S0-6.
- Audit-row + RLS-denial + persona-violation panels live with real data.

**Out-of-scope (deferred).**
- SLO definitions per module (FR-OBS-002 in batch-02; full SLOs land in P1).
- Per-module dashboards beyond the P0 baseline (each module's owner authors theirs in their own FR).
- DPIA + A05 filing artefact authoring (FR-CP-001 in batch-02).
- DSAR + RTBE workflows (P3; the RTBE drill in S0-6 is a synthetic exercise on a synthetic tenant).
- Mobile-app status page client (P3 mobile).

## Dependencies

- FR-INFRA-001 (Kubernetes cluster, Postgres, NATS).
- FR-AUTH-001 / FR-AUTH-002 (audit log feeds the cockpit).
- FR-AI-001 (LangSmith integration through the AI Gateway).
- FR-MCP-001 (read-only OBS tools).
- All other P0 modules (BRAIN, GENIE, CHAT) instrument themselves to the OBS-ready contract.
- PagerDuty account (or equivalent open-source alternative; OQ-OBS-PAGERDUTY tracks the choice).
- Cloudflare Pages account for the status site.
- Compliance: PDPL Decree 13 (logs and metrics may include personal data; redaction at Promtail + retention windows are the controls); SOC 2 Common Criteria CC7 (system operations) and CC8 (change management) — OBS is the evidence stream.
- Locked decisions referenced: DEC-050 (single-pane OBS stack on Loki + Prometheus + Tempo), DEC-051 (LangSmith for AI usage), DEC-052 (Trust Center is public).

## AI Risk Assessment

Not applicable. `eu_ai_act_risk_class: not_ai`. OBS records AI usage but does not itself emit AI-derived behaviour to natural persons; the dashboards are deterministic queries over the platform's own state.

## AI Authorship Disclosure

- **Tools used:** Claude Cowork (Anthropic).
- **Scope:** drafted the FR end-to-end against the PRD + SRS; founder reviews and edits before status changes from `ready_for_review`.
- **Human review:** founder (`@stephen-cheng`) — final wording is the founder's responsibility.
