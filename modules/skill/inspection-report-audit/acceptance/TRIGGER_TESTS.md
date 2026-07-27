# TRIGGER_TESTS: inspection-report-audit

## Must trigger

- "audit this inspection report"
- "check the inspection against the rubric"
- "is this report trustworthy"
- "score this inspection"
- "did the inspection miss anything structural"
- "lint this report"

## Must not trigger

- "inspect this repository" -> inspection-report-author
- "audit this repo" -> inspection-report-author
- "fix what the report found" -> /harden
- "audit this PRD" -> product-requirements-document-audit

## Boundary cases

"lint this report" triggers this skill, which then runs only the machine floor
if that is all the user asked for, and says so rather than silently scoring the
judgement rules too.

A report that fails the machine floor is not scored. Say which INSL rule failed
and stop; a partial judgement score on a structurally invalid report is worse
than no score.
