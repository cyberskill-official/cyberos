---
id: TASK-<MODULE>-<NNN>
title: ""
template: task@1  # managed by @cyberskill/templates — do not edit
type: feature | improvement | chore   # FM-108. Pick ONE and delete the others.
                                      # improvement.md and chore.md are pointers here,
                                      # so this skeleton serves all three — set `type`
                                      # to the one you were routed for.
module: <module>
author: "@your-handle"
department: product
status: draft
priority: p0 | p1 | p2 | p3           # NOT MoSCoW. MUST/SHOULD/COULD was retired 2026-07-14.
created_at: <ISO 8601 with timezone>
ai_authorship: none | assisted | co_authored | generated_then_reviewed
eu_ai_act_risk_class: not_ai | minimal | limited | high
target_release: ""
client_visible: true | false
depends_on: []
---

<!-- This skeleton carried the PRE-migration schema until 2026-07-15: `feature_type:
     user_facing` (retired by FM-108 — three overlapping axes collapsed to one), no
     `type:` at all, no `id:`, no `module:`. A task authored from it failed FM-108 —
     `type` required, error severity — the moment it was written.

     It survived because nothing executes a template. It is prompt text an LLM renders,
     so no test imports it and no gate parses it. The rubric and this file were written
     in the same change and never checked against each other; `bug.md` is correct only
     because it was authored fresh afterwards.

     `scripts/tests/test_template_schema.sh` now checks every template in this directory
     against RUBRIC.md's FM family, so the two cannot drift apart again silently. -->


# Task

> Turn Your Will Into Real.

## Summary

A single-paragraph summary. The reader should be able to repeat it back without scrolling.

## Problem

What is the user actually trying to do? What is blocking them today? Cite evidence: support tickets, NPS comments, sales calls, telemetry. If you have customer quotes, put them inside the untrusted block.

## Customer Quotes

Required when `client_visible: true`. Verbatim, attributed where possible. Paraphrasing here costs you the signal.

<untrusted_content source="other"> …paste verbatim customer quote here… </untrusted_content>

## Proposed Solution

The shape of the answer. Not the implementation — the user-visible behaviour. Include rough mockups, API sketches, or wireframes if helpful.

## Alternatives Considered

What did you reject and why. Forces clarity on the actual trade-off.

## Success Metrics

How will we know this worked? Pick one primary metric and one guardrail metric. Avoid vanity counts.

## Scope

In-scope vs out-of-scope. Be explicit about what the first release does *not* include. Scope creep starts where this section is vague.

## Dependencies

Other modules, other teams, vendor APIs, contractual constraints, compliance approvals.

## AI Risk Assessment

Required when `eu_ai_act_risk_class` is `limited` or `high`. EU AI Act Articles 5–7. Three subsections, all required:

### Data Sources

What data trains, fine-tunes, or grounds the AI behaviour. Provenance, licensing, and any personal data implications.

### Human Oversight

Where a human approves, overrides, or audits the AI output. Article 14 conformance.

### Failure Modes

What happens when the model is wrong, hallucinates, or is offline. The fallback path and the user-visible behaviour during failure.

## Sales/CS Summary

Required when `client_visible: true`. One paragraph written so a non-engineer can pitch the feature. Plain English. No internal jargon, no module codes, no speculation about future scope.

## AI Authorship Disclosure

Required only when `ai_authorship` is not `none`. Same three-bullet shape:

- **Tools used:**
- **Scope:**
- **Human review:**
