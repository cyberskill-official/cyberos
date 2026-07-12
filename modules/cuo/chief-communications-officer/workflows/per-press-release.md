---
workflow_id: chief-communications-officer/per-press-release
workflow_version: 1.0.0
purpose: Distribute and amplify a press release — wire selection, media-list curation, embargo coordination, follow-up outreach.
persona: cuo/chief-communications-officer
cadence: per-event
status: shipped
pattern: persona_pair
peer_persona: chief-marketing-officer
peer_workflow: per-campaign-plan
shared_artefact: campaign-plan
handoff_step: 2

inputs:
  - { name: release_content,       source: cuo/chief-marketing-officer/per-press-release output, format: press-release@1 }
  - { name: prior_distributions,   source: last 6 months distribution metrics, format: markdown }
  - { name: media_list,            source: PR-tools media DB (Cision / Muck Rack / Prowly), format: csv }

outputs:
  - { name: distributed_release,   format: press-release@1 (with distribution log), recipient: target media + cuo/cmo (post-distribution analysis) }

skill_chain:
  - { step: 1, skill: press-release-author, inputs_from: { release_content: release_content, prior_distributions: prior_distributions, media_list: media_list }, outputs_to: dist_release_draft }
  - { step: 2, skill: press-release-audit,  inputs_from: dist_release_draft, outputs_to: distributed_release }

escalates_to:
  - { persona: cuo/chief-executive-officer,            when: "embargo break or correction needed post-distribution" }

consults:
  - { persona: cuo/chief-marketing-officer,            when: "distribution outcome warrants creative iteration" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with distributed_release hash + media-list size + embargo timestamp
  - HITL pause at step 2 on QA-EMBARGO-001 (embargo not respected) or QA-CORRECTION-001 (typo in distributed text)
---

# Per press release (distribution) — `chief-communications-officer/per-press-release`

CCO-Communications' distribution-side workflow paired with CMO's content-side `per-press-release` workflow. CMO owns the content; CCO-Communications owns the distribution + earned-media follow-through.

## When to invoke

- "Distribute the [release name] PR"
- "Send the [announcement] to wire"
- "Media outreach for [release]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-communications-officer/per-press-release \
  --input release_content=./pr/2026-acme-partnership/release.md \
  --input prior_distributions=./pr/distributions/ \
  --input media_list=./pr/media-list.csv \
  --output-dir ./pr/2026-acme-partnership/distribution/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + same-day distribution
- **Worst case:** embargo break requires same-hour correction sweep

## Skill chain

- **Step 1 `press-release-author`** — augments CMO output with distribution metadata.
- **Step 2 `press-release-audit`** — validates per `press_release_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-EMBARGO-001 | Embargo broken | Same-hour outreach + correction |
| 2 | QA-CORRECTION-001 | Typo in distribution | Issue correction notice |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CCO-Communications role profile
- `../../chief-marketing-officer/workflows/per-press-release.md` — content-side partner
- `../../../skill/press-release-{author,audit}/SKILL.md`
