# Plan-approval render

> Sourced verbatim from `feature-request/FR_CREATE_AND_AUDIT.md` v2.0.0 §11.

Emitted to chat at the end of PLAN phase. The persistent backlog lives in
`manifest.json`; this render is the human-facing summary.

```
PROPOSED FR BACKLOG  (from <K> requirements files, sha256:<first-12-hex>...)
=============================================================================
Run scope:                    generate up to N=<batch_size> FRs this batch.
Total backlog:                <int> FRs identified.
EU AI Act flags (escalate):   <int>.
EU AI Act flags (high):       <int>.
Open planning questions:      <int> (listed at bottom).

FR-001  P<x>  <slug>
        <one-liner ≤140 chars>
        Depends on: <list | —>
        EU AI Act tentative: <class>
        client_visible tentative: <bool>   ai_authorship tentative: <value>
        Source: <file.pdf, p.N>

(repeat for every backlog entry — full backlog, not capped at batch_size)

Open questions identified during planning
-----------------------------------------
  Q1 (FR-NNN): <question>
  Q2 (FR-NNN): <question>

Approval options
----------------
  A) APPROVE          → generate FRs 1..N (N = <batch_size>) this run
  B) REVISE           → reply with edits (add / remove / reorder / re-prioritise / merge / split)
  C) ABORT            → set plan.status = INVALIDATED and stop

Reply with one of:  APPROVE  |  REVISE: <your edits>  |  ABORT
```

This emission appends one `genie.action_log` row with `row_kind: question`
(per SRS §6.6.2 — plan approval IS a Question primitive).
