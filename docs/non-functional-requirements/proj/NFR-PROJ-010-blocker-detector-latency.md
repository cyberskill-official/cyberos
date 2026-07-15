---
id: NFR-PROJ-010
title: "PROJ blocker detector latency — blocker MUST surface within 4h of cause"
module: PROJ
category: observability
priority: SHOULD
verification: T
phase: P1
slo: "p95 < 4h from blocker condition to dashboard surfacing"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-PROJ-011]
---

## §1 — Statement (BCP-14 normative)

1. The blocker detector (`TASK-PROJ-011`) **MUST** surface blockers (issues stalled > 3d, missing assignee, missing estimate, etc.) within 4h of the triggering condition becoming true.
2. The detector runs on a fixed cadence (default 1h) plus event-triggered re-runs on relevant mutations.
3. Surfaced blockers **MUST** appear in the team's blocker dashboard + notification stream within 2 minutes of detection.
4. False positives (issue correctly flagged but reason no longer valid) **MUST NOT** linger > 1 cycle — the detector self-clears.
5. Blocker count per team is a tracked metric; sustained high counts (> 20 / 50 issues) trigger sev-3 product review.

## §2 — Why this constraint

Blockers that hide for days compound silently. The 4h window is the operational sweet spot — fast enough to catch stalls, slow enough to filter noise. The auto-clear rule prevents stale blocker entries cluttering the dashboard. The high-count escalation is a feature: it signals that the team's workflow itself needs review.

## §3 — Measurement

- Histogram `proj_blocker_detection_latency_seconds` — condition true → surfaced.
- Counter `proj_blocker_detected_total{blocker_kind}`.
- Gauge `proj_blocker_active_count{team}` — surfaces sustained high counts.

## §4 — Verification

- Integration test (T) — create blocker condition; assert detected + surfaced within 4h (test-mode accelerates clock).
- Property test (T) — drive transitions through blocker conditions; assert detector tracks accurately.

## §5 — Failure handling

- Latency > 4h p95 → sev-3; investigate detector cron.
- Active blockers > 20 for 2 weeks → sev-3; workflow review.
- Stale-blocker count > 5% → sev-3; self-clear logic has a bug.

---

*End of NFR-PROJ-010.*
