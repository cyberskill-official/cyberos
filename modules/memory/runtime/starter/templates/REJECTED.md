---
memory_id: mem_${UUID7}
scope: memories/refinements
classification: operational
authority: ${AUTHORITY:human-edited}
version: 1
created_at: ${TS_NOW}
created_by: ${SUBJECT_ID}
last_updated_at: ${TS_NOW}
updated_by: ${SUBJECT_ID}
supersedes: null
superseded_by: null
expires_at: null
provenance:
  source: chat
  source_ref: ${CHAT_TURN_REF}
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["refinement-rejection"]
tags: [refinement-rejected, wont-fix, ${TAGS}]
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: publishable
source_freshness_tier: 25
---

# REJECTED-${NEXT_NNN} ${SLUG_TITLE}

> Tracks a §0.4 refinement candidate that was considered and explicitly rejected. Prevents the same candidate from resurfacing forever without context.

## Pattern flagged
[What pattern triggered the candidate? How many occurrences?]

## Proposal rejected
[What was the proposed refinement? Tier? AGENTS.md section?]

## Why rejected (pick one or multiple)
- [ ] Out of scope (better solved elsewhere)
- [ ] Premature (not enough evidence yet)
- [ ] Wrong shape (right concern, wrong proposal)
- [ ] Risk too high (cure worse than disease)
- [ ] Already covered by existing rule
- [ ] Speculation (no real failure observed)
- [ ] Other: [explain]

## What would change the decision
[Conditions under which this should be reconsidered. E.g., "reconsider if pattern recurs ≥10 times in next 90 days", or "reconsider once memory module P1 ships and multi-machine becomes real".]

## Related
- Pattern memory: [DRIFT-NNN if auto-detected]
- See also: [DEC-NNN, REF-NNN]
