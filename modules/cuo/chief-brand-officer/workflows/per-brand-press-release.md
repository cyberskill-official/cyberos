---
workflow_id: chief-brand-officer/per-brand-press-release
workflow_version: 1.0.0
purpose: Author a brand-milestone press release — rebrand, identity refresh, brand-value milestone, brand purpose statement.
persona: cuo/chief-brand-officer
cadence: per-event
status: shipped

inputs:
  - { name: announcement_brief,    source: CBO / CMO / CEO, format: markdown }
  - { name: brand_strategy,        source: cuo/chief-brand-officer/annual-brand-strategy, format: brand-strategy@1 }
  - { name: quotes_inventory,      source: pre-approved exec quotes, format: markdown }

outputs:
  - { name: brand_press_release,   format: press-release@1, recipient: cuo/chief-brand-officer + cuo/cco-communications + media list + cuo/cmo }

skill_chain:
  - { step: 1, skill: press-release-author, inputs_from: { announcement_brief: announcement_brief, brand_strategy: brand_strategy, quotes_inventory: quotes_inventory }, outputs_to: release_draft }
  - { step: 2, skill: press-release-audit,  inputs_from: release_draft, outputs_to: brand_press_release }

escalates_to:
  - { persona: cuo/chief-legal-officer,      when: "rebrand triggers trademark filings or domain transition" }

consults:
  - { persona: cuo/chief-communications-officer, when: "release needs earned-media outreach" }

audit_hooks:
  - each step emits artefact_write
  - workflow_complete row on PASS with brand_press_release hash + brand-element changes flagged
  - HITL pause at step 2 on QA-TRADEMARK-001 (rebrand without trademark review)
---

# Per brand press release — `chief-brand-officer/per-brand-press-release`

Chief Brand Officer's brand-milestone press release workflow. For announcements where brand IS the news (rebrand, identity refresh, purpose statement, brand-value milestone). Distinct from CMO's product/partnership press release.

## When to invoke

- "Press release for [rebrand]"
- "Brand announcement: [milestone]"
- "Identity-refresh PR"

## How to invoke

```bash
cyberos-cuo run cuo/chief-brand-officer/per-brand-press-release \
  --input announcement_brief=./brand/announcements/2026-rebrand/brief.md \
  --input brand_strategy=./brand/2026/strategy.md \
  --input quotes_inventory=./brand/quotes.md \
  --output-dir ./brand/announcements/2026-rebrand/
```

## Expected duration

- **Happy path:** 2-4 hours runtime + 1 week for legal + CCO-Communications coordination
- **Worst case:** trademark blockers add 1-2 quarter

## Skill chain

- **Step 1 `press-release-author`** — drafts brand-milestone variant.
- **Step 2 `press-release-audit`** — validates per `press_release_rubric@1.0`.

## Failure modes

| Step | Code | Recovery |
|---|---|---|
| 2 | QA-TRADEMARK-001 | Rebrand no trademark review | Escalate to CLO-Legal |

## Cross-references
- `../../../../modules/cuo/README.md` §5.4 — Chief Brand Officer role profile
- `../../chief-marketing-officer/workflows/per-press-release.md` — product/promo peer
- `../../../skill/press-release-{author,audit}/SKILL.md`
