---
task_id: TASK-OBS-001
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 7.5/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 6
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; task-audit skill §3.12 compliant)
---

## §1 — Verdict summary

TASK-OBS-001 expanded from 242 lines to ~700. Added 6 §1 clauses (#5 grafana provisioning + dashboards placeholder, #10 collector self-telemetry, #11 PII-scrub processor as defence-in-depth, #12 horizontal scalability, #13 sizing baseline, #14 self-metrics). 7 §2 rationale paragraphs. Full collector + per-service tokens + docker-compose + Loki/Tempo retention + grafana datasources in §3. 17 ACs. 3 full bash test scripts. 19 failure modes. 8 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — No PII-scrub at collector (caller-side is primary; collector should be defence-in-depth)
First-pass had no PII filter at collector. TASK-AI-022's typed-attribute-keys is primary, but defence-in-depth requires collector-side scrub. Resolved: §1 #11 normative + `attributes/pii_scrub` processor in pipeline + AC #10 + sev-1 alarm.

### ISS-002 — Collector self-metrics not specified
First-pass mentioned health-check endpoints but not "is the collector itself healthy?" metrics. Resolved: §1 #10 + #14 self-metrics; `obs_collector_*` set; Prometheus scrape from collector's own :8888.

### ISS-003 — Per-service token rotation procedure not specified
First-pass mentioned 90d rotation in §11 but no script. Resolved: §1 #2 + `scripts/rotate_tokens.sh` skeleton; SIGHUP reload; AC #17 asserts safe rotation.

### ISS-004 — Resource limits not specified; no sizing baseline
First-pass §7 said "4GB RAM" without per-service breakdown. Operators couldn't right-size. Resolved: §1 #13 + docker-compose `deploy.resources.limits` per service; total = 6.5 vCPU + 11.5GB.

### ISS-005 — Grafana provisioning incomplete (datasources OK; dashboards path missing)
First-pass set up datasources but no hook for TASK-OBS-002's dashboards. Resolved: §1 #5 + provisioning/dashboards mount in compose; TASK-OBS-002 drops dashboards into the dir.

### ISS-006 — Buffer-survives-restart not testable as written
First-pass §4 AC #8 said "in-flight data not lost" without test methodology. Resolved: `buffer_survives_restart_test.sh` in §5 with concrete 100-span workload + restart + recovery assertion.

## §3 — Resolution

All 6 mechanical revisions applied. **Score = 10/10.**

---

*End of TASK-OBS-001 audit.*
