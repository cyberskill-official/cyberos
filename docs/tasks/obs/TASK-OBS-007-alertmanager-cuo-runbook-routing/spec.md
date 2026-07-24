---
id: TASK-OBS-007
title: "obs-router: Alertmanager → CUO triage-alert → CHAT or PagerDuty"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: obs
priority: p0
status: done
entered_via: rework
routed_back_count: 1
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-OBS-001, TASK-OBS-003, TASK-OBS-005, TASK-CUO-101, TASK-KB-008]
depends_on: [TASK-OBS-002, TASK-OBS-003]
blocks: [TASK-KB-008]

source_pages:
  - website/docs/modules/obs.html#alert-routing
  - website/docs/modules/cuo.html#obs-triage-skill

source_decisions:
  - DEC-170 2026-05-15 — CUO triage confidence floor 0.70; below = page on-call
  - DEC-171 2026-05-15 — sev-1 always pages BOTH CHAT + PagerDuty; never trust triage at highest severity
  - DEC-172 2026-05-15 — CHAT post carries ack-button + suggested runbook + trace_id link
  - DEC-173 2026-05-15 — CUO skill failure = safe fallback to PagerDuty; never silent-drop

language: rust 1.81
service: cyberos/services/obs-router/
new_files:
  - services/obs-router/Cargo.toml
  - services/obs-router/src/main.rs
  - services/obs-router/src/lib.rs
  - services/obs-router/src/alertmanager_webhook.rs
  - services/obs-router/src/audit.rs
  - services/obs-router/src/chat_post.rs
  - services/obs-router/src/config.rs
  - services/obs-router/src/cuo_triage.rs
  - services/obs-router/src/dedup.rs
  - services/obs-router/src/error.rs
  - services/obs-router/src/handle.rs
  - services/obs-router/src/notify.rs
  - services/obs-router/src/pagerduty.rs
  - services/obs-router/src/route.rs
  - services/obs-router/src/runbook.rs
  - services/obs-router/src/severity.rs
  - services/obs-router/src/triage.rs
  - services/obs-router/tests/route_decision_test.rs
  - services/obs-router/tests/alertmanager_wiring_test.rs
  - modules/skill/obs-triage-alert/SKILL.md
  - modules/skill/obs-triage-alert/runbooks-corpus/.keep
  - deploy/obs/alertmanager-config.yaml
modified_files: []

allowed_tools:
  - file_read: services/obs-router/**, modules/skill/obs-triage-alert/**
  - file_write: services/obs-router/**, modules/skill/obs-triage-alert/**, deploy/obs/alertmanager-config.yaml
  - bash: cd services/obs-router && cargo test

disallowed_tools:
  - auto-resolve a sev-1 alert without human confirmation (per DEC-171)
  - bypass PagerDuty fallback on CUO failure (per DEC-173)
  - silent-drop any alert (per DEC-173)
  - claim live PagerDuty/CHAT network CI or phantom skills/ path as shipped

effort_hours: 10
subtasks:
  - "0.5h: alertmanager_webhook.rs + severity.rs"
  - "0.5h: route.rs — decide() + CONFIDENCE_FLOOR 0.70"
  - "1.0h: handle.rs — route_alert + §1 #11 fallback chain"
  - "0.5h: cuo_triage.rs + triage.rs traits"
  - "0.5h: chat_post.rs + pagerduty.rs + notify.rs"
  - "0.5h: runbook.rs allowlist fail-closed"
  - "0.5h: dedup.rs + main.rs webhook secret + /ack stub"
  - "0.5h: modules/skill/obs-triage-alert/SKILL.md"
  - "0.5h: deploy/obs/alertmanager-config.yaml (this batch)"
  - "1.5h: route_decision_test.rs + handle.rs / route.rs inline tests"
  - "1.0h: batch/9b-obs re-spec + audit"
risk_if_skipped: "Every alert pages a human regardless of severity. On-call noise overload (~20 alerts/day at 50 tenants). Without sev-1-always-pages, an over-confident triage might silence a real incident. Without ack-button + audit rows, ops correlation between CHAT discussion and alert state breaks."
---

# TASK-OBS-007: obs-router Alertmanager → CUO → CHAT/PagerDuty

## Summary

Route Alertmanager webhook fires through CUO's `obs.triage-alert@1` skill to CHAT (confidence ≥ 0.70) or PagerDuty, with sev-1 always paging both channels. As-built surface is the `services/obs-router/` crate (`handle.rs` orchestration — not `ack_handler.rs`), pure routing in `route.rs` (`CONFIDENCE_FLOOR = 0.70`), the skill at `modules/skill/obs-triage-alert/SKILL.md`, and `deploy/obs/alertmanager-config.yaml` (added in this batch).

## Problem

The original engineering-spec claimed `skills/obs.triage-alert/SKILL.md`, a standalone `ack_handler.rs`, phantom integration test filenames (`triage_test.rs`, `sev1_always_pages_test.rs`), and live PagerDuty/CHAT network tests. The live skill lives under `modules/skill/`; ack is a minimal stub on `main.rs` `/ack/:fingerprint`; routing correctness is proven by `handle.rs` and `route.rs` unit tests plus `route_decision_test.rs`. FM-004 blocked re-entry (`## §N` body + wrong paths).

## Proposed Solution

Adopt the as-built layout:

- `route.rs` — `decide(severity, confidence)` with `CONFIDENCE_FLOOR: 0.70`; sev-1 → `Route::Both`
- `handle.rs` — `route_alert` ties triage → decide → deliver with CHAT↔PagerDuty fallback chain; emits `obs.alert_triaged`
- `runbook.rs` — `sanitize_runbook` drops unverified URLs (fail-closed against `OBS_RUNBOOK_ALLOWLIST`)
- `main.rs` — `POST /alert` webhook + `X-CyberOS-Webhook-Secret`, dedup, `/ack` audit stub
- `modules/skill/obs-triage-alert/SKILL.md` — CUO skill contract (`obs.triage-alert@1`)
- `deploy/obs/alertmanager-config.yaml` — webhook receiver → obs-router:7777

## Alternatives Considered

- **Resume the old engineering-spec as-is.** Rejected: FM-004 blocks re-entry; `skills/` and `ack_handler.rs` paths lie.
- **Page every alert to PagerDuty (no CUO triage).** Rejected: DEC-170 requires confidence-gated CHAT routing to reduce noise.
- **Trust CUO at sev-1.** Rejected: DEC-171 requires both channels regardless of confidence.

## Success Metrics

- Primary: routing table matches DEC-170/DEC-171 for all (severity, confidence) pairs; CUO failure never silent-drops; sev-1 always hits both channels in tests.
- Guardrail: `route_decision_test.rs` exhaustive grid; `handle.rs` tests prove fallback legs fire.

## Scope

In scope (as-built):

- Full `services/obs-router/src/**` layout (`handle.rs`, not `ack_handler.rs`)
- `modules/skill/obs-triage-alert/SKILL.md` + runbooks corpus placeholder
- `deploy/obs/alertmanager-config.yaml` (this batch)
- `tests/route_decision_test.rs` + inline tests in `route.rs` and `handle.rs`

### Out of scope / Non-Goals

- Phantom `skills/obs.triage-alert/` path (live skill is `modules/skill/obs-triage-alert/`)
- `ack_handler.rs` filename (ack is `main.rs::handle_ack` stub — CHAT post update + PagerDuty close deferred)
- Live PagerDuty / CHAT network CI (clients are HTTP env-configured; tests use trait mocks)
- Alert auto-resolve on `resolved` status (main.rs skips resolved alerts; slice 4 follow-up)
- Full escalate-to-PagerDuty post-hoc flow (`/escalate/:fingerprint` is a minimal stub)

## Dependencies

`depends_on: [TASK-OBS-002, TASK-OBS-003]`. Soft: TASK-OBS-005 (trace_id on alert labels for CHAT/audit links); TASK-CUO-101 (CUO runtime); TASK-KB-008 (runbook corpus the skill RAG-searches).

## 1. Description (normative)

- 1.1 `obs-router` MUST accept Alertmanager v2 webhook payloads on `POST /alert` and parse firing alerts (`alertmanager_webhook.rs`).
- 1.2 CUO triage MUST invoke skill `obs.triage-alert@1` per `modules/skill/obs-triage-alert/SKILL.md`, returning confidence + summary + suspected cause + optional runbook (`cuo_triage.rs`, `triage.rs`).
- 1.3 Routing MUST follow `route::decide`: sev-1 → `Route::Both` regardless of confidence; sev-2..4 with clamped confidence ≥ `CONFIDENCE_FLOOR` (0.70) → `Route::Chat`; otherwise → `Route::PagerDuty` (DEC-170, DEC-171).
- 1.4 CUO triage failure or timeout MUST be absorbed as confidence 0.0 and MUST route to PagerDuty, never silent-drop (DEC-173).
- 1.5 Delivery MUST implement the §1 #11 fallback chain in `handle.rs::deliver`: CHAT failure → PagerDuty; PagerDuty failure → last-resort CHAT; sev-1 `Both` delivers each leg independently.
- 1.6 Every routed alert MUST emit an `obs.alert_triaged` audit row with route actually taken (`audit.rs`, `handle.rs::route_alert`).
- 1.7 Webhook ingress MUST authenticate via shared secret header `X-CyberOS-Webhook-Secret` when configured; missing/wrong → 401 (`main.rs`).
- 1.8 Firing alerts with identical fingerprint within the dedup window MUST collapse to a single route decision per window (`dedup.rs`).
- 1.9 Suggested runbook URLs MUST pass `runbook::sanitize_runbook` against `OBS_RUNBOOK_ALLOWLIST`; unlisted URLs are dropped fail-closed in both CHAT post and audit payload.
- 1.10 This adopt MUST NOT claim phantom `skills/` paths, an `ack_handler.rs` module, or live PagerDuty/CHAT network CI as shipped.

## Acceptance criteria

- [ ] AC 1 (traces_to: #1.1) - Alertmanager webhook parses and normalises alerts - test: `services/obs-router/src/alertmanager_webhook.rs::parses_and_normalises_multiple_alerts`
- [ ] AC 2 (traces_to: #1.2) - CUO client invokes obs.triage-alert@1 skill id - verify: `services/obs-router/src/cuo_triage.rs` + `modules/skill/obs-triage-alert/SKILL.md`
- [ ] AC 3 (traces_to: #1.3) - routing table matches spec §1 #3 at floor and boundaries - test: `services/obs-router/tests/route_decision_test.rs::routing_table_matches_spec_section_1_3`
- [ ] AC 4 (traces_to: #1.3) - every (severity, confidence) grid cell yields a real route - test: `services/obs-router/tests/route_decision_test.rs::every_alert_routes_somewhere_no_silent_drop`
- [ ] AC 5 (traces_to: #1.3) - sev-1 always Both at any confidence - test: `services/obs-router/src/route.rs::sev1_always_routes_both_regardless_of_confidence`
- [ ] AC 6 (traces_to: #1.3) - non-sev1 at/above 0.70 → Chat - test: `services/obs-router/src/route.rs::non_sev1_at_or_above_floor_goes_to_chat`
- [ ] AC 7 (traces_to: #1.3) - non-sev1 below 0.70 → PagerDuty - test: `services/obs-router/src/route.rs::non_sev1_below_floor_pages_pagerduty`
- [ ] AC 8 (traces_to: #1.4) - CUO failure as zero confidence pages - test: `services/obs-router/src/route.rs::cuo_failure_as_zero_confidence_pages_never_drops`
- [ ] AC 9 (traces_to: #1.5) - sev-1 pages both CHAT and PagerDuty - test: `services/obs-router/src/handle.rs::sev1_pages_both`
- [ ] AC 10 (traces_to: #1.4,#1.5) - triage failure routes PagerDuty - test: `services/obs-router/src/handle.rs::triage_failure_pages_pagerduty`
- [ ] AC 11 (traces_to: #1.5) - CHAT failure falls back to PagerDuty - test: `services/obs-router/src/handle.rs::chat_failure_falls_back_to_pagerduty`
- [ ] AC 12 (traces_to: #1.5) - PagerDuty failure last-resorts to CHAT - test: `services/obs-router/src/handle.rs::pagerduty_failure_last_resorts_to_chat`
- [ ] AC 13 (traces_to: #1.6) - obs.alert_triaged row carries spec fields - test: `services/obs-router/src/audit.rs::triaged_row_carries_the_spec_fields`
- [ ] AC 14 (traces_to: #1.6) - route_alert emits chat route in audit - test: `services/obs-router/src/handle.rs::confident_non_sev1_goes_to_chat_and_audits`
- [ ] AC 15 (traces_to: #1.7) - webhook secret enforced on ingress - verify: `services/obs-router/src/main.rs` `X-CyberOS-Webhook-Secret` check
- [ ] AC 16 (traces_to: #1.8) - fingerprint dedup within 5m window - test: `services/obs-router/src/dedup.rs::repeats_within_window_bump_the_counter`
- [ ] AC 17 (traces_to: #1.9) - non-allowlisted runbook dropped in audit - test: `services/obs-router/src/handle.rs::runbook_is_dropped_unless_allowlisted`
- [ ] AC 18 (traces_to: #1.10) - Out of scope lists skills/ path + ack_handler + live network CI - verify: this spec Scope / Out of scope
- [ ] AC 19 (traces_to: #1.1,#1.10) - Alertmanager wiring file targets `/alert` without phantom skills/ claims - test: `services/obs-router/tests/alertmanager_wiring_test.rs`

## Verification

```bash
cd services && cargo test -p cyberos-obs-router
cd services && cargo test -p cyberos-obs-router --test route_decision_test --test alertmanager_wiring_test
```

| Path | Covers |
|------|--------|
| `tests/route_decision_test.rs` | Full routing table + no-silent-drop grid |
| `tests/alertmanager_wiring_test.rs` | `deploy/obs/alertmanager-config.yaml` residual gate |
| `src/route.rs` (inline tests) | `CONFIDENCE_FLOOR`, clamp, sev-1 Both |
| `src/handle.rs` (inline tests) | End-to-end route_alert + fallback chain + runbook allowlist |
| `src/severity.rs` (inline tests) | Label parsing |
| `modules/skill/obs-triage-alert/SKILL.md` | CUO skill contract |
| `deploy/obs/alertmanager-config.yaml` | Alertmanager → obs-router webhook |

## AI Authorship Disclosure

- **Tools used:** Cursor agent (Composer) on branch `batch/9b-obs`.
- **Scope:** Re-spec/adopt against as-built `obs-router` + `modules/skill/obs-triage-alert/`; cite `handle.rs` not `ack_handler.rs`; add `deploy/obs/alertmanager-config.yaml` to new_files.
- **Human review:** Required at the two HITL gates (`entered_via: rework`, `routed_back_count: 1`).

---

*batch/9b-obs adopt — TASK-OBS-007 re-spec against as-built obs-router.*
