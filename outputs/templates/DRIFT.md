---
memory_id: mem_${UUID7}
scope: memories/drift
classification: operational
authority: agent:${AGENT_ID}
version: 1
created_at: ${TS_NOW}
created_by: agent:${AGENT_ID}
last_updated_at: ${TS_NOW}
updated_by: agent:${AGENT_ID}
supersedes: null
superseded_by: null
expires_at: null
provenance:
  source: inference
  source_ref: ${ORIGINAL_MEMORY_ID}
  confidence: 0.95
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["drift-detection", "automated"]
tags: [drift, ${TAGS}]
relationships:
  - kind: drift_signal
    target: ${ORIGINAL_MEMORY_ID}
retention:
  rule: indefinite
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: local-only
source_freshness_tier: 80
---

# DRIFT-${NEXT_NNN} ${SLUG_TITLE}

## Source SHA before
${SOURCE_SHA_BEFORE}

## Source SHA after (current)
${SOURCE_SHA_AFTER}

## Affected memory
${ORIGINAL_MEMORY_ID} — ${ORIGINAL_MEMORY_PATH}

## Detected by
- **Skill:** §8.6 source-coverage validator
- **Consolidation run:** ${CONSOLIDATION_RUN_ID}

## Response options (pick one)

### 1. Re-ingest
Walk new source sequentially per §4.10; write v2 digest; supersedes v1.

### 2. Accept drift
Source moved on in ways that don't affect the digest's purpose. Leave digest as-is; this drift record explains why.

### 3. Update source to match
Digest captured the right answer; source got it wrong. Edit source; re-validate SHA.

## Diff summary
[High-level changes between source-before and source-after, if extractable]

## Status
[unresolved | resolved-reingest | resolved-accept | resolved-update]
