---
skill_id: code-review-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for code-review-audit

## Positive triggers (MUST route here)

- "Audit this code review"
- "Check the code-review rubric"
- "Verify the PR review meets SDP §5 AI-check rules"
- "Re-audit the code-review.md collection"

## Negative triggers (MUST NOT route here)

- "Review this PR diff" → code-review-author
- "Draft a code review for PR-1247" → code-review-author
- "Triage this incident" → none
- "What's deploy day this week?" → none

## Authoring notes

- Positives anchor on "audit", "check", "rubric", "verify", "re-audit".
- Negatives catch sibling-author confusion + unrelated ops queries.
- Re-author when classifier_version MAJOR-bumps.
