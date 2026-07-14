---
id: NFR-OBS-007
title: "Chain-of-custody manifest completeness — every alert carries trace_id + alertmanager_id + runbook_id"
module: OBS
category: compliance
priority: MUST
verification: T
phase: P0
slo: "100% of fired alerts carry the four-field custody manifest; CI verifies on every release"
owner: CSO
created: 2026-05-18
related_tasks: [TASK-OBS-009, TASK-OBS-007]
---

## §1 — Statement (BCP-14 normative)

1. Every Alertmanager-routed alert **MUST** carry the four-field chain-of-custody manifest:
   - `trace_id` — the OTel trace that triggered the alert (or `unknown` if the alert is from a recording rule without a trace).
   - `alertmanager_id` — the Alertmanager fingerprint (deterministic hash of label set).
   - `runbook_id` — the CUO runbook path (see NFR-OBS-004).
   - `tenant_id` — the affected tenant (or `cross-tenant` for platform-wide alerts).
2. The manifest **MUST** survive every routing hop (Alertmanager → router → Slack → CUO supervisor) — no field dropped.
3. Each manifest **MUST** be appended to the memory audit chain as `obs.alert.custody_manifest` with the four fields + the alert's `startsAt` timestamp; this row is immutable and survives consolidation.
4. CI gate **MUST** verify on every `deploy/obs/alerts/*.yml` change that all alert rules carry annotations for `cyberos_runbook` and a templated `tenant_id` (where applicable).
5. The custody manifest **MUST** be the canonical key for post-incident analysis — postmortems reference the manifest, not free-text descriptions.

## §2 — Why this constraint

Without chain-of-custody, an incident's "what happened, when, who responded" timeline is reconstructed from Slack scrollback — unreliable and slow. The four-field manifest gives postmortem authors a deterministic key to join: trace data (`trace_id`), alert data (`alertmanager_id`), runbook version (`runbook_id`), affected scope (`tenant_id`). All four are essential — missing any one breaks a different join. The memory append makes the manifest tamper-evident for compliance reviewers (SOC 2 CC7.3 monitoring evidence).

## §3 — Measurement

- Counter `obs_alert_custody_manifest_complete_total{result}` where result ∈ {`complete`, `incomplete`}. Incomplete should be zero.
- Sev-2 alarm on any `result=incomplete` row.
- memory audit query `view kind=obs.alert.custody_manifest` — every fired alert should produce one row.

## §4 — Verification

- CI gate `tests/obs/alert_manifest_completeness_test.sh` (T) — parses every alert rule YAML; asserts all carry the four annotations.
- Integration test (T) — fires a synthetic alert, fetches the memory row, asserts four fields present.

## §5 — Failure handling

- Manifest incomplete on a fired alert → sev-2; investigate which routing hop dropped the field; postmortem must reconstruct from logs.
- Missing memory audit row → sev-1; chain-of-custody broken for that alert; compliance flag.
- Two consecutive incomplete manifests in CI → block merge until alert-config root cause fixed.

---

*End of NFR-OBS-007.*
