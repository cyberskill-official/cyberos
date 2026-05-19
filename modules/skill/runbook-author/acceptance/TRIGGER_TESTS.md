---
skill_id: runbook-author
min_confidence: 0.7
classifier_version: 3.0.0-a4
---

# TRIGGER_TESTS for runbook-author

## Positive triggers (MUST route here)

- "Draft a runbook for high memory ingest lag"
- "Create the on-call runbook for AUTH 5xx spikes"
- "Document the recovery procedure for stale audit chain"
- "Author the runbook for Postgres connection-pool exhaustion"

## Negative triggers (MUST NOT route here)

- "Audit this existing runbook" → runbook-audit
- "Verify the runbook covers the rollback path" → runbook-audit
- "Page the on-call now" → none
- "What's our current ingest lag?" → none

## Authoring notes

- Positives anchor on "draft", "create", "document", "author" + "runbook"/"recovery procedure"/"on-call".
- Negatives catch sibling-auditor + imperative-action + runtime-query.
- Re-author when classifier_version MAJOR-bumps.
