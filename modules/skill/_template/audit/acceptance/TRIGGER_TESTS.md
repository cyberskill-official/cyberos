---
skill_id: <artefact>-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for <artefact>-audit

> **Purpose:** assert routing for auditor skills. Per TASK-SKILL-112.

## Positive triggers (MUST route here)

- "Audit this <ARTIFACT> for completeness"
- "Has <ARTIFACT>-007 changed since the last audit?"
- "Tell me which <ARTIFACT>s would fail acceptance today"

## Negative triggers (MUST NOT route here)

- "Draft a new <ARTIFACT> from this PRD" → <artefact>-author
- "Generate the <ARTIFACT> backlog" → <artefact>-author
- "What's the team's holiday schedule?" → none

## Authoring notes

- Auditor positive triggers anchor on "audit", "check", "re-audit", "rubric", "verdict".
- Negative triggers must include the sibling author skill (most common confusion in pilot).
- Re-author when `classifier_version` MAJOR-bumps.
