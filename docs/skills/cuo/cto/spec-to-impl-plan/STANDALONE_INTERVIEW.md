# `spec-to-impl-plan` standalone-mode interview (2-3 questions)

## Q1 — Target sprint

> "Which sprint should these tickets land in? Options: (a) next sprint (top of backlog, ready), (b) the sprint after (refinement first), (c) icebox (planned but not yet sized for sprint commitment), (d) custom timeline (you specify)."

## Q2 — PROJ backend

> "Which ticket system? Default (from `manifest.mcp_backends`): `{detected_backend}`. Press Enter to accept or specify: linear / jira / github / none (markdown only)."

## Q3 — Create tickets now?

> "Should I create the tickets in `{proj_backend}` now, or just write the impl-plan markdown for you to review first? Options: (a) write markdown only — I'll create tickets after you approve, (b) write markdown AND prompt me again before creating tickets, (c) skip ticket creation (markdown-only forever)."

INV-002 forces a final HALT_BEFORE_CREATE prompt regardless of (a) or (b). (c) sets `tickets_created: false` permanently.

## When the interview is skipped

Chained mode: supervisor passes the input envelope; this skill validates + proceeds without re-prompting EXCEPT for the INV-002 HALT_BEFORE_CREATE prompt which always fires.
