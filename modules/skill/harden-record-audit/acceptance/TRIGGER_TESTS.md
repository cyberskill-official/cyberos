# TRIGGER_TESTS: harden-record-audit

## Must trigger

- "audit this hardening session"
- "check what /harden actually did"
- "verify the remediation record"
- "audit this hardening-record"

## Must not trigger

- "work the backlog" / "/harden" → harden-record-author
- "inspect this repository" → inspection-report-author
- "audit this inspection report" → inspection-report-audit
- "harden a task" / ship improvement task → ship-tasks (distinct)
