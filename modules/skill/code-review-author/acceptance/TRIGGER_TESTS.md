---
skill_id: code-review-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for code-review-author

## Positive triggers (MUST route here)

- "Review this PR diff"
- "Draft a code review for the auth changes"
- "Audit the implementation against TASK-AUTH-003"
- "Write a code-review write-up for PR-1247"

## Negative triggers (MUST NOT route here)

- "Audit this existing code-review.md" → code-review-audit
- "Check the code-review verdict for completeness" → code-review-audit
- "Triage this customer bug" → none
- "What does the SDP §5 AI-check rule say?" → none

## Authoring notes

- Positives anchor on "review", "code review", "PR diff", "PR-NNNN".
- Negative 1-2 catch sibling-auditor confusion.
- Re-author when classifier_version MAJOR-bumps.
