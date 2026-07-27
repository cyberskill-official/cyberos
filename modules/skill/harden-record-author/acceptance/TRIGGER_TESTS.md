# TRIGGER_TESTS: harden-record-author

## Must trigger

- "work the backlog from this inspection"
- "fix what the inspection found"
- "harden this repo"
- "start on the next action"
- "close INS-F-0002"
- "/harden"

## Must not trigger

- "inspect this repository" -> inspection-report-author
- "audit this inspection report" -> inspection-report-audit
- "is this report trustworthy" -> inspection-report-audit
- "review this pull request" -> code-review-author
- "what should we fix first" -> this is a question about the report, answer from
  the report's own section 18 rather than starting a session

## Boundary cases

"inspect and fix" triggers inspection-report-author only. This skill runs after
a report exists, and the combined request is answered by producing the report
and naming this skill as the next step.

"fix everything" starts a session, presents the ordered plan, and halts at the
plan gate like any other. It does not become a licence to skip the gates.

"just push it" is refused. HRD-SAFE-3 holds regardless of phrasing, including
when the instruction to fix and the instruction to push arrive in one sentence.

A request naming a finding whose timeline class is never worked automatically
triggers the skill and works that finding, because naming it is the explicit
instruction HRD-SEQ-1 requires.
