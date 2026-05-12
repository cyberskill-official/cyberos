---
memory_id: mem_019e1968-d810-7601-8aef-27a30a400e7d
scope: memories/preferences
classification: operational
authority: human-edited
version: 1
created_at: 2026-05-11T23:15:49+07:00
created_by: subject:stephen-cheng
last_updated_at: 2026-05-11T23:15:49+07:00
updated_by: subject:stephen-cheng
provenance: {source: chat, source_ref: aspect-7.1 layer1-improvements catalog, confidence: 1.0}
consent: {has_consent: true, consent_event: null, consent_scope: [preference, voice-standard]}
tags: [preference, voice, no-em-dash, no-ai-vocab, gstack-codex]
relationships: []
retention: {rule: indefinite, earliest_delete: null}
embedding: {model: null, version: null, vector_id: null}
sync_class: publishable
source_freshness_tier: 25
---

# PREF-001 CyberSkill voice standard

## Preference
All CyberOS protocol docs (AGENTS.md, AGENTS-CORE.md, AGENTS.README.md) and all new CyberSkill external materials use the gstack /codex voice standard:
- **No em dashes (—) or en dashes (–)** — use commas, parens, or sentence rewrite
- **No AI vocabulary:** delve, crucial, robust, comprehensive, nuanced, multifaceted, furthermore, moreover, additionally, pivotal, landscape, tapestry, underscore, foster, showcase, intricate, vibrant, fundamental, significant
- **Lead with the point.** Name files, line numbers, commands, outputs.
- **Builder-to-builder tone**, not consultant-to-client.

## Scope
- **Applies to:** all CyberOS protocol docs going forward (additive — old text not retroactively rewritten per §0.5)
- **Applies to:** all new memories Stephen writes (DECs, REFs, FACTs)
- **CHANGELOG and protocol-history are exempt** (descriptive, may quote LLM outputs verbatim)
- **Override-rule:** if a technical term unambiguously requires "robust" or similar, use it but flag with comment

## Rationale
Voice imprecision is correctness imprecision in protocol docs. AGENTS.md MUST reach identical accept/reject decisions on two agents on two machines — that requires precise unambiguous prose. AI vocabulary and em dashes are correctness hazards.

## Examples
- ✓ "validator rejects writes that fail §4.2"
- ✗ "the robust validator comprehensively rejects writes which—through nuanced analysis—fail the multifaceted §4.2 framework"

## Enforcement
- runtime/tools/voice_check.py (lints em dashes + AI vocab)
- runtime/tools/cyberos voice (CLI wrapper)
- .github/workflows/voice-check.yml (CI gate)

## Related
- See also: REF-NNN-voice-discipline (when adopted as §0.5 amendment)
