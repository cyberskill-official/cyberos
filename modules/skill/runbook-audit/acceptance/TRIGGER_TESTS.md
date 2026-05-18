---
skill_id: runbook-audit
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for runbook-audit

## Positive triggers (MUST route here)

- "Audit this runbook"
- "Verify the runbook covers the rollback path"
- "Check the on-call procedure meets SDP §4(d)"
- "Re-audit the runbooks in docs/ops/"

## Negative triggers (MUST NOT route here)

- "Draft a runbook for high ingest lag" → runbook-author
- "Create the on-call runbook" → runbook-author
- "Run the runbook now" → none
- "What's tonight's on-call rotation?" → none

## Authoring notes

- Positives anchor on "audit", "verify", "check", "re-audit".
- Negatives catch sibling-author + imperative-action queries.
- Re-author when classifier_version MAJOR-bumps.
