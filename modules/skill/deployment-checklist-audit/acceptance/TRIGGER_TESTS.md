---
skill_id: deployment-checklist-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for deployment-checklist-audit

## Positive triggers (MUST route here)

- "Audit this deploy checklist"
- "Verify the rollback path is covered"
- "Check the deploy plan meets SDP §4(c)"
- "Re-audit the deploy checklists for the AUTH wave"

## Negative triggers (MUST NOT route here)

- "Draft a deploy checklist" → deployment-checklist-author
- "Create the production deploy plan" → deployment-checklist-author
- "Deploy v0.2.6 to prod" → none
- "What's tonight's deploy ETA?" → none

## Authoring notes

- Positives anchor on "audit", "verify", "check", "re-audit".
- Negatives catch sibling-author + imperative-deploy queries.
- Re-author when classifier_version MAJOR-bumps.
