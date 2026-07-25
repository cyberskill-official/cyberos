---
batch: batch/11-wave2-residuals
members:
  - TASK-IMP-145
  - TASK-IMP-146
started: 2026-07-25
ended: 2026-07-25
---

# Batch 11 — post-1.2.0 Wave 2 residuals

Two genuine gaps remained after auditing Waves 1–4 against live main:

1. TASK-IMP-145 — repair SKILL-202 / G7 / G8 citations to the suite that shipped.
2. TASK-IMP-146 — add `memory-append append --dry-run` so verdict payloads can be rehearsed
   without permanent probe rows.

Everything else in Waves 1–4 is already shipped; this batch does not reopen it.

