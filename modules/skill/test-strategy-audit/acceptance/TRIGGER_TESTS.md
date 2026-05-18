---
skill_id: test-strategy-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for test-strategy-audit

## Positive triggers (MUST route here)

- "Audit this test strategy"
- "Check the test-plan rubric"
- "Verify the strategy meets ISO 29119-3 coverage"
- "Re-audit the test strategies in docs/"

## Negative triggers (MUST NOT route here)

- "Draft a test strategy" → test-strategy-author
- "Outline the test plan for the auth slice" → test-strategy-author
- "Run the test suite" → none
- "What's our coverage today?" → none

## Authoring notes

- Positives anchor on "audit", "check", "rubric", "verify", "re-audit".
- Negatives catch sibling-author confusion + runtime queries.
- Re-author when classifier_version MAJOR-bumps.
