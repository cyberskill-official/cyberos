---
id: NFR-PROJ-007
title: "PROJ a11y-ci gate — every page MUST pass axe-core with 0 critical/serious issues"
module: PROJ
category: usability
priority: MUST
verification: T
phase: P0
slo: "0 critical/serious axe-core violations per page; 100% page coverage in CI"
owner: CTO
created: 2026-05-18
related_tasks: [TASK-PROJ-018]
---

## §1 — Statement (BCP-14 normative)

1. Every PROJ page **MUST** pass `axe-core` with **0 critical and 0 serious** violations in the CI a11y-ci gate.
2. Moderate violations are warnings — they appear in the report but do not block. Sustained moderate counts > 5 per page trigger sev-3 review.
3. Custom components (issue cards, Kanban columns, Gantt bars) **MUST** declare ARIA roles + keyboard interaction patterns; the gate verifies declarations.
4. The CI gate runs on every PR; merging is blocked on any critical/serious violation.
5. New pages **MUST** be added to the CI page list before merge — drift between deployed pages and CI coverage triggers sev-3.

## §2 — Why this constraint

Accessibility is a legal + ethical baseline (WCAG 2.1 AA in most jurisdictions; future-AAA aspiration). A11y bugs accrue silently — moderate violations slip in, then someone with assistive tech can't use the platform. Strict CI gate at the critical/serious bar gives a non-negotiable floor without making every nit a release-blocker. The page-list-drift rule prevents new code from sneaking in without coverage.

## §3 — Measurement

- Per-page counter `proj_axe_violation_count{page, severity}` — critical/serious must be 0.
- CI metric `proj_a11y_uncovered_pages_count` — must be 0.
- Quarterly: manual screen-reader walkthrough of top-10 user flows.

## §4 — Verification

- CI gate `proj-a11y-ci` (T) — runs axe-core against every page in the manifest.
- Manual quarterly screen-reader audit by CTO + design.
- New-page lint: page added to `pages.yaml` before merge.

## §5 — Failure handling

- Critical/serious violation → CI block; contributor fixes.
- Sustained moderate > 5 → sev-3 review; design + engineering on the fix.
- Page drift (deployed but not in CI list) → sev-3; immediate add + scan.

---

*End of NFR-PROJ-007.*
