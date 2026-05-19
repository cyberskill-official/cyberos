---
skill_id: test-strategy-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for test-strategy-author

## Positive triggers (MUST route here)

- "Draft a test strategy for the auth slice"
- "Outline the test plan for this feature"
- "Design the testing approach for the memory ingest pipeline"
- "Author the test-strategy document for v1.0 release"

## Negative triggers (MUST NOT route here)

- "Audit this existing test strategy" → test-strategy-audit
- "Review the test plan for completeness" → test-strategy-audit
- "Run the tests now" → none
- "Why did test X fail?" → none

## Authoring notes

- Positives anchor on "draft", "outline", "design", "author" + "test strategy"/"test plan"/"testing approach".
- Negatives catch sibling-auditor + runtime queries.
- Re-author when classifier_version MAJOR-bumps.
