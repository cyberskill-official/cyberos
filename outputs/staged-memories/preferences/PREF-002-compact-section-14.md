---
memory_id: mem_019e1968-d810-77d3-9b4d-8d06c05956cf
scope: memories/preferences
classification: operational
authority: human-edited
version: 1
created_at: 2026-05-11T23:15:49+07:00
created_by: subject:stephen-cheng
last_updated_at: 2026-05-11T23:15:49+07:00
updated_by: subject:stephen-cheng
provenance: {source: chat, source_ref: Bundle I — Compact §14, confidence: 1.0}
consent: {has_consent: true, consent_event: null, consent_scope: [preference, section-14]}
tags: [preference, section-14, compact, operational-mode]
relationships: []
retention: {rule: indefinite, earliest_delete: null}
embedding: {model: null, version: null, vector_id: null}
sync_class: publishable
source_freshness_tier: 25
---

# PREF-002 Compact §14 output for normal operational_mode

## Preference
`operational_mode: normal` MUST produce the compact §14 format (Bundle I + O + P).
Only switch to verbose §14.2 for debug / maintenance / verbose modes OR when findings exist.

## Scope
- **Applies to:** Stephen's daily cyberos sessions
- **Override-rule:** `cyberos status --verbose` or set `operational_mode: verbose` per-session

## Rationale
Bundle I observation: pre-compact §14 was 14+ lines, most reading "no change". Signal-to-noise was poor.

## Related
- Bundle I (2026-05-06): introduced compact format
- Bundle N (2026-05-10): added §14 omission for fully-silent normal mode
- Bundle O (2026-05-10): three-state triage (silent / files-only-compact / issues-verbose)
- Bundle P (2026-05-10): §14 `📁 Files changed:` = non-BRAIN paths only
