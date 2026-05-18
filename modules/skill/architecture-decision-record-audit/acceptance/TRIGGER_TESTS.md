---
skill_id: architecture-decision-record-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for architecture-decision-record-audit

## Positive triggers (MUST route here)

- "Audit this ADR for completeness"
- "Check the rubric on ADR-007"
- "Tell me which ADRs would fail acceptance"
- "Re-audit the ADRs in docs/adr/"

## Negative triggers (MUST NOT route here)

- "Draft an ADR for the database choice" → architecture-decision-record-author
- "Document this architecture decision" → architecture-decision-record-author
- "Draft a tech spec from this ADR" → software-design-document-author
- "What's the on-call rotation?" → none

## Authoring notes

- Positives anchor on "audit", "check", "rubric", "re-audit".
- Negatives catch (a) sibling author confusion + (b) downstream-chain confusion (ADR-audit vs tech-spec-author).
- Re-author when classifier_version MAJOR-bumps.
