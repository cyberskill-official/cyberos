---
workflow_id: chief-marketing-officer/per-press-release
workflow_version: 1.0.0
purpose: Author a press release for a launch / partnership / milestone / executive change.
persona: cuo/chief-marketing-officer
cadence: per-event
status: shipped

inputs:
  - { name: announcement_brief,    source: PR requestor (CEO / product / sales / partnerships), format: markdown }
  - { name: prior_releases,        source: last 6 months press-release@1, format: press-release@1 (set) }
  - { name: brand_voice,           source: cuo/chief-marketing-officer/quarterly-brand-strategy messaging architecture, format: brand-strategy@1 }
  - { name: quotes_inventory,      source: pre-approved exec quotes, format: markdown }

outputs:
  - { name: press_release,         format: press-release@1, recipient: cuo/cmo + cuo/cco-communications + wire services + media list }

skill_chain:
  - { step: 1, skill: press-release-author, inputs_from: { announcement_brief: announcement_brief, prior_releases: prior_releases, brand_voice: brand_voice, quotes_inventory: quotes_inventory }, outputs_to: release_draft }
  - { step: 2, skill: press-release-audit,  inputs_from: release_draft, outputs_to: press_release }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "release contains material disclosure (must coordinate with 8-K if public)" }

consults:
  - { persona: cuo/chief-communications-officer, when: "release requires earned-media outreach plan" }
  - { persona: cuo/chief-executive-officer,            when: "release contains exec quote that needs sign-off" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with press_release hash + embargo timestamp + media-list size
  - HITL pause at step 2 on QA-MATERIAL-001 (material disclosure unflagged) or QA-QUOTE-001 (unapproved quote)
---

# Per press release — `chief-marketing-officer/per-press-release`

CMO's per-release press-release workflow. Per PRSA + AP Stylebook standards. Critical material-disclosure boundary: any release that triggers SEC 8-K Item 7.01 requires CLO-Legal sign-off + coordinated filing.

## When to invoke

- "Draft press release for [announcement]"
- "Write the launch PR"
- "Press release for [partnership]"

## How to invoke

```bash
cyberos-cuo run cuo/chief-marketing-officer/per-press-release \
  --input announcement_brief=./pr/2026-acme-partnership/brief.md \
  --input prior_releases=./pr/2026/ \
  --input brand_voice=./brand/2026-Q2/strategy.md \
  --input quotes_inventory=./pr/quotes.md \
  --output-dir ./pr/2026-acme-partnership/
```

## Expected duration

- **Happy path:** 1-2 hours runtime + same-day approval cycle
- **Worst case:** material-disclosure flag adds 1-2 day legal review

## Skill chain

- **Step 1 `press-release-author`** — drafts per PRSA + AP Stylebook.
- **Step 2 `press-release-audit`** — validates per `press_release_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-MATERIAL-001 | Material disclosure unflagged | Escalate to CLO-Legal |
| 2 | QA-QUOTE-001 | Unapproved quote | Operator gets approval |

## Cross-references
- `../../../../modules/cuo/docs/module.md` §5.4 — CMO role profile
- `../../chief-communications-officer/workflows/per-press-release.md` — partner persona's same-skill workflow (different angle: CMO owns content; CCO-Communications owns distribution)
- `../../../skill/press-release-{author,audit}/SKILL.md`
