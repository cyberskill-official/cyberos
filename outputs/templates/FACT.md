---
memory_id: mem_${UUID7}
scope: memories/facts
classification: ${CLASSIFICATION:public}
authority: ${AUTHORITY:human-confirmed}
version: 1
created_at: ${TS_NOW}
created_by: ${SUBJECT_ID}
last_updated_at: ${TS_NOW}
updated_by: ${SUBJECT_ID}
supersedes: null
superseded_by: null
expires_at: null
provenance:
  source: ${PROV_SOURCE:doc}
  source_ref: ${PROV_SOURCE_REF}
  confidence: ${CONFIDENCE:0.95}
consent:
  has_consent: true
  consent_event: null
  consent_scope: ["fact"]
tags: []
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding:
  model: null
  version: null
  vector_id: null
sync_class: ${SYNC_CLASS:shared}
source_freshness_tier: ${FRESHNESS_TIER:10}
ingestion_coverage:
  source_path: ${SOURCE_PATH}
  source_sha256: ${SOURCE_SHA}
  source_lines: ${SOURCE_LINES}
  processed_lines: ${PROCESSED_LINES}
  first_ts: null
  last_ts: null
  intentional_summary: false
  summary_reason: null
---

# FACT-${NEXT_NNN} ${SLUG_TITLE}

## Claim
[The fact being recorded, one sentence.]

## Source
[Where this came from: file path, URL, doc section.]

## Evidence
[Quote or paraphrase from source with line/section ref.]

## Freshness
- **Source last updated:** [ts]
- **Drift detection:** §8.6 source_sha256 above; re-validate on consolidation
- **Confidence:** ${CONFIDENCE}

## Related
- See also: [FACT-NNN, DEC-NNN]
