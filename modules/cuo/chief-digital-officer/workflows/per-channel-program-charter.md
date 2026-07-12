---
workflow_id: chief-digital-officer/per-channel-program-charter
workflow_version: 1.0.0
purpose: Charter a digital-channel program — web / mobile / app / IoT / kiosk / voice modernization.
persona: cuo/chief-digital-officer
cadence: per-event
status: shipped

inputs:
  - { name: channel_brief,         source: requestor, format: markdown }
  - { name: roadmap_context,       source: cuo/chief-digital-officer/annual-digital-transformation-roadmap, format: transformation-roadmap@1 }
  - { name: function_inputs,       source: impacted function heads, format: markdown }

outputs:
  - { name: digital_channel_charter, format: program-charter@1, recipient: cuo/chief-digital-officer + cuo/cto + program owner + impacted function heads }

skill_chain:
  - { step: 1, skill: program-charter-author, inputs_from: { channel_brief: channel_brief, roadmap_context: roadmap_context, function_inputs: function_inputs }, outputs_to: charter_draft }
  - { step: 2, skill: program-charter-audit,  inputs_from: charter_draft, outputs_to: digital_channel_charter }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "channel scope > $500K OR cross-business-unit" }

consults:
  - { persona: cuo/chief-technology-officer,            when: "engineering capacity + platform alignment" }
  - { persona: cuo/chief-product-officer,    when: "product-surface overlap" }

audit_hooks:
  - workflow_complete row on PASS with digital_channel_charter hash + channel scope
  - HITL pause at step 2 on QA-OWNER-001 or QA-VALUE-001
---

# Per-channel program charter — `chief-digital-officer/per-channel-program-charter`

CDO-Digital's per-channel modernization charter per PMI charter + Bain digital-program framework. Triggered per major channel program (web replatform / app rewrite / IoT rollout / kiosk modernization).

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.7 — Chief Digital Officer role profile
- `./annual-digital-transformation-roadmap.md` — upstream parent
- `../../../skill/program-charter-{author,audit}/SKILL.md`
