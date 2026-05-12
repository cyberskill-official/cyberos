---
memory_id: mem_019e1a50-0002-7abc-9000-000000000003
scope: memories/people
classification: personnel
authority: human-edited
version: 1
created_at: 2026-05-12T08:00:00+07:00
created_by: subject:test
last_updated_at: 2026-05-12T08:00:00+07:00
updated_by: subject:test
supersedes: null
superseded_by: null
expires_at: null
provenance:
  source: manual
  source_ref: mutation-test-fixture-person
  confidence: 1.0
consent:
  has_consent: true
  consent_event: null
  consent_scope: [personnel-record]
tags: [test, fixture, person]
relationships: []
retention:
  rule: indefinite
  earliest_delete: null
embedding: {model: null, version: null, vector_id: null}
sync_class: local-only
source_freshness_tier: 18
---

# PERSON-001 mutation-test fixture

Personnel-class record. Properly carries `sync_class: local-only` per
the scope-rules plugin for `memories/people`.
