---
skill_id: architecture-decision-record-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for architecture-decision-record-author

## Positive triggers (MUST route here)

- "Draft an ADR for the database choice"
- "Record this architecture decision as an ADR"
- "Document the choice between Postgres and CockroachDB"
- "Author the ADR for our event-bus selection"

## Negative triggers (MUST NOT route here)

- "Audit this existing ADR for completeness" → architecture-decision-record-audit
- "Review the technical merit of ADR-007" → architecture-decision-record-audit
- "Draft a tech spec from this task" → software-design-document-author
- "What's the team holiday schedule?" → none

## Authoring notes

- Positive triggers anchor on "draft", "record", "document", "author" + "ADR" or "architecture decision".
- Negative triggers catch (a) sibling auditor confusion + (b) downstream-chain confusion (ADR vs tech-spec).
- Re-author when classifier_version MAJOR-bumps.
