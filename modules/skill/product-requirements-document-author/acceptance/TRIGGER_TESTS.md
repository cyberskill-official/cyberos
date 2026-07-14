---
skill_id: product-requirements-document-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for product-requirements-document-author

> Verifies the supervisor classifier routes PRD-authoring requests correctly. Per TASK-SKILL-112.

## Positive triggers (MUST route here)

- "Draft a PRD from this brief"
- "Turn this SOW into a Product Requirements Document"
- "Expand the project-brief into product requirements"
- "Author the PRD for the email-bounce handling feature"

## Negative triggers (MUST NOT route here)

- "Audit this existing PRD" → product-requirements-document-audit
- "Check the PRD against acceptance criteria" → product-requirements-document-audit
- "Turn this PRD into a backlog of FRs" → task-author
- "Draft an SRS from this PRD" → software-requirements-specification-author
- "What's our Q4 hiring plan?" → none

## Authoring notes

- Positive 1-3 anchor on the canonical PRD inputs: brief, SOW,
  project-brief. Verb cues are "draft", "turn into", "expand".
- Positive 4 is a specific-feature framing — operators often request a
  PRD for a named feature rather than a generic "draft a PRD".
- Negative 1-2 catch the most common confusion: PRD author vs auditor.
  These ARE the auditor's positive triggers — verb cues ("audit",
  "check") MUST route to the auditor.
- Negative 3 catches the downstream-chain confusion: PRD → FR backlog is
  a separate skill (task-author), not this one.
- Negative 4 catches the upstream-chain confusion: PRD → SRS is also a
  separate skill (software-requirements-specification-author).
- Negative 5 is canonical "no skill" sanity.
- Re-author when classifier_version MAJOR-bumps.
