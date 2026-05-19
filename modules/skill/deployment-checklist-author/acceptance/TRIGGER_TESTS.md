---
skill_id: deployment-checklist-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for deployment-checklist-author

## Positive triggers (MUST route here)

- "Draft a deploy checklist for the AUTH rollout"
- "Create the production deploy plan for v0.2.6"
- "Outline the rollout for the memory service"
- "Author the deployment checklist for next Tuesday's deploy"

## Negative triggers (MUST NOT route here)

- "Audit this existing deploy checklist" → deployment-checklist-audit
- "Verify the checklist covers the rollback path" → deployment-checklist-audit
- "Deploy now" → none
- "What's our SLA for AUTH?" → none

## Authoring notes

- Positives anchor on "draft", "create", "outline", "author" + "deploy checklist"/"deploy plan"/"rollout".
- Negatives catch sibling-auditor + imperative-action queries.
- Re-author when classifier_version MAJOR-bumps.
