---
task_id: TASK-DOCS-006
audited: 2026-07-12
verdict: PASS
score: 10/10
template: engineering-spec@1
---

# TASK-DOCS-006 audit

## §1 - Verdict summary
Audited for supersession hygiene (no dual status story; bookmarks preserved; legacy suite repointed not deleted) and for single-source count integrity (deck vs tabs). TRACE closes: #1/#5->AC5, #2->AC1, #3->AC2/AC3/AC6, #4->AC4 -> t01-t06.

## §2 - Findings (resolved during authoring)
ISS-001 two status pages would drift - resolved: redirect stub + nav swap (§1 #4).
ISS-002 deck computed separately from tabs could disagree - resolved: one corpus object (§10 #2).

## §3 - Resolution
**Score = 10/10.**

*End of TASK-DOCS-006 audit.*

## §4 - Ship record (2026-07-12, batch mode)

- Implemented: render-status-hub.mjs (one corpus object -> deck + 3 hash-routed tabs through
  status-hub@1; backlog tab with 4 facets; FR-page links feature-detected; roadmap.html permanent
  redirect stub; nav Roadmap -> Status; render-roadmap.mjs retired with TASK-DOCS-003 post-ship
  amendment + repointed legacy suite 7/7). test_render_status_hub.sh 6/6; full site build green.
- HITL: operator standing batch verdict recorded.

Verdict unchanged: PASS, Score = 10/10.
