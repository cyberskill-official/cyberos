---
id: NFR-OBS-004
title: "Alert-to-CUO-runbook routing latency — p95 < 30s from rule fire to runbook surface"
module: OBS
category: reliability
priority: MUST
verification: T
phase: P0
slo: "p95 < 30s from Alertmanager rule fire to CUO runbook surfaced to on-call"
owner: CTO
created: 2026-05-18
related_frs: [FR-OBS-007, FR-OBS-001]
---

## §1 — Statement (BCP-14 normative)

1. From the moment a Prometheus alert rule transitions to `firing`, the CyberOS routing pipeline **MUST** surface the matched CUO runbook to the on-call surface (Slack channel + CUO supervisor) at **p95 < 30s** and **p99 < 60s**.
2. Every alert **MUST** carry a `cyberos_runbook` annotation pointing to a path under `cuo/<persona>/runbooks/<slug>.md`; alerts without a runbook annotation **MUST** be rejected by the Alertmanager config validator.
3. The routing pipeline **MUST** include the chain-of-custody fields (NFR-OBS-007): `trace_id`, `alertmanager_id`, `runbook_id`, `tenant_id` (when applicable). All four go into the Slack notification payload.
4. If the runbook lookup fails (path doesn't exist, persona not found), the routing pipeline **MUST** still notify on-call with a fallback `cuo/sre/generic-incident.md` runbook — no alert silently dropped.
5. The runbook content rendered to on-call **MUST** be the version pinned in the alert's annotation, not the latest — same alert seen twice surfaces the same runbook even if the file has changed.

## §2 — Why this constraint

On-call response time is dominated by "what do I do?" — the question the runbook answers. Without sub-minute routing, an incident's first 5 minutes are wasted hunting the right doc. The 30s budget aligns with the platform's MTTA (mean-time-to-acknowledge) target — alerts that arrive with a runbook get acknowledged faster because the responder doesn't have to context-switch. The pinned-version rule prevents "the runbook said one thing yesterday, but now says another" debugging confusion.

## §3 — Measurement

- Histogram `obs_alert_to_runbook_seconds` measured from Alertmanager's `startsAt` to the Slack-webhook delivery timestamp on the routed message.
- Counter `obs_runbook_lookup_fallback_total` — incremented when the primary runbook path 404s; should be near-zero.
- Counter `obs_alert_no_runbook_total` — should always be zero (config validator should reject).

## §4 — Verification

- Integration test `tests/obs/alert_routing_latency_test.sh` (T) — fires a synthetic alert, asserts Slack receives the formatted runbook within 30s.
- Quarterly chaos drill (D) — operator fires a real-but-test alert; on-call must acknowledge within 5 minutes (whole-loop test).

## §5 — Failure handling

- p95 > 30s for 1 hour → sev-3; check Alertmanager → router → Slack webhook latencies individually.
- Fallback runbook used → sev-3 ticket per occurrence; investigate which alert lost its runbook annotation.
- Slack webhook fully down → backup routing to PagerDuty (legacy on-call channel); CTO + CSO notified.

---

*End of NFR-OBS-004.*
