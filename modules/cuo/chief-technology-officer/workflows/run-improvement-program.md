---
workflow_id: chief-technology-officer/run-improvement-program
workflow_version: 2.0.0
purpose: RETIRED. Improvement programs no longer run as a separate track. Enterprise-hardening work is now a feature-request (the improvement class) driven by `chief-technology-officer/ship-feature-requests`. This file is a tombstone that redirects.
persona: chief-technology-officer
cadence: per-task
status: retired   # CUO-workflow lifecycle: planned | shipped | retired
pattern: linear
superseded_by: chief-technology-officer/ship-feature-requests
---
# Run an improvement program (RETIRED)

This workflow is retired. On Stephen's 2026-07-08 decision, CyberOS runs a single implementation workflow: `chief-technology-officer/ship-feature-requests`. Improvement, hardening, and audit-remediation work is not a separate track any more: each item is a feature-request carrying `class: improvement` and runs the same lifecycle, with HITL required at the two human-acceptance gates.

Where to go now:

- The workflow: `chief-technology-officer/ship-feature-requests.md` (section 1a covers improvement FRs and their gate profile).
- The improvement-class home and the migration of the old `docs/improvement/` backlogs (`MEM-*`, `T-*`, `IMP-*`) into FR ids: `docs/feature-requests/improvement/README.md`.
- The halt and HITL doctrine: `modules/cuo/EXECUTION-DISCIPLINE.md` §2a.

The two generic skills this workflow used, `cyberos-improve-implement` and `cyberos-improve-review`, have been removed.
