# Feature Request Template — CyberSkill v1

> Turn Your Will Into Real.

## 1. Purpose

This template captures feature proposals in a way that lets product, engineering, and sales argue from the same set of facts, and that surfaces EU AI Act risk classification early enough to influence the design rather than the launch.

## 2. When to use this template

Use it when proposing new behaviour, a new module, a new integration, or a non-trivial expansion of existing scope. For a defect, use the Bug Report template. For a one-line tweak with no user-visible behaviour, an issue comment is enough.

## 3. Field reference

| Field | Type | Required | Allowed values | Filled by | Why it exists |
|---|---|---|---|---|---|
| `title` | string | yes | <= 72 chars | author | One-line proposal |
| `author` | string | yes | `@handle` | author | Attribution |
| `department` | enum | yes | engineering, design, product, sales, operations, hr, client_success | author | Routes review |
| `status` | enum | yes | draft .. closed | triage | Lifecycle |
| `priority` | enum | yes | p0..p3 | triage | Work-order priority |
| `created_at` | string (date) | yes | ISO 8601 | scaffolder | Anchors review SLA |
| `ai_authorship` | enum | yes | none, assisted, co_authored, generated_then_reviewed | author | Transparency |
| `template` | enum | yes | `feature_request@1` | scaffolder | Schema-pin |
| `feature_type` | enum | yes | user_facing, internal_tooling, integration, infrastructure | author | Routing & classification |
| `eu_ai_act_risk_class` | enum | yes | not_ai, minimal, limited, high | author | EU AI Act Articles 5–7. `unacceptable` is not allowed by the schema |
| `target_release` | string | optional | SemVer or quarter (`2026-Q3`) | author | Roadmap pin |
| `client_visible` | boolean | yes | true / false | author | Drives Sales/CS Summary requirement |

## 4. Section reference

| Section | Required? | When required | What good looks like | Common mistake |
|---|---|---|---|---|
| Summary | yes | always | One paragraph repeatable from memory | A wishlist |
| Problem | yes | always | Cited evidence: tickets, NPS, telemetry | Hypothetical user |
| Customer Quotes | conditional | `client_visible=true` | Verbatim, attributed where possible | Paraphrasing |
| Proposed Solution | yes | always | User-visible behaviour, not implementation | Implementation detail |
| Alternatives Considered | yes | always | What you rejected and why | "We considered nothing" |
| Success Metrics | yes | always | One primary + one guardrail | Vanity counts |
| Scope | yes | always | Explicit out-of-scope list | Vague boundaries |
| Dependencies | yes | always | Other modules, teams, vendors | "None" |
| AI Risk Assessment | conditional | `eu_ai_act_risk_class` is `limited` or `high` | Three subsections fully filled | Skipping Failure Modes |
| Sales/CS Summary | conditional | `client_visible=true` | One paragraph a non-engineer can pitch | Internal jargon, module codes |
| AI Authorship Disclosure | conditional | `ai_authorship != none` | Three required bullets | Vague scope |

## 5. Required-when rules

The validator enforces:

1. `eu_ai_act_risk_class` is `limited` or `high` ⇒ `## AI Risk Assessment` H2 with `### Data Sources`, `### Human Oversight`, `### Failure Modes` subsections.
2. `client_visible: true` ⇒ `## Customer Quotes` H2 and `## Sales/CS Summary` H2 present.
3. `ai_authorship != none` ⇒ `## AI Authorship Disclosure` H2 with three bullets.
4. The schema rejects `eu_ai_act_risk_class: unacceptable` outright — features in that bucket must not be filed.

## 6. Example (fully-filled realistic artifact)

```markdown
---
title: "AI-assisted invoice categorisation in the export tool"
author: "@thuy-pham"
department: product
status: ready_for_review
priority: p2
created_at: "2026-04-28"
ai_authorship: co_authored
feature_type: user_facing
eu_ai_act_risk_class: limited
target_release: "2026-Q3"
client_visible: true
template: feature_request@1
---

# Feature Request

> Turn Your Will Into Real.

## Summary
Add an opt-in step to the invoice export flow that suggests
categories using a small classifier; the user accepts or edits
each suggestion before download.

## Problem
CS data: 38% of support tickets in the export workflow are about
"why isn't my export categorised?" Telemetry shows median export
review time is 23 minutes; users abandon at 18 minutes.

## Customer Quotes
<untrusted_content source="customer_email">
"We pay an accountant for two days every month just to label
the export. If your tool could pre-fill 80% of it we would not
need that step."  — Finance Lead, PV-A
</untrusted_content>

## Proposed Solution
Add a step "Suggested categories" between filter selection and
download. Each row shows a suggested category, a confidence
indicator, and an inline "edit" control. User must confirm
before download proceeds.

## Alternatives Considered
- Server-side automatic categorisation, no UI step.
  Rejected: removes the human-in-the-loop required for limited-risk
  AI under EU AI Act Article 14.
- Rule-based categoriser with no model.
  Rejected: rules cover 31% of categories; the other 69% need
  fuzzy matching that rules cannot do well.

## Success Metrics
- Primary: median export-review time < 8 minutes (from 23).
- Guardrail: user-corrected suggestion rate < 25% (else the model
  is hurting more than helping).

## Scope
In: invoice export only; suggestion + accept/edit UI; per-tenant
opt-in; offline cache of last 1000 categories per tenant.
Out: other export types (contracts, reports); bulk-accept;
training UI for end users.

## Dependencies
- Auth: scope `exports:write` already exists.
- AI module: needs the small-classifier serving primitive (P0
  CyberOS roadmap, already shipped).
- Compliance review by @legal-vn before alpha.

## AI Risk Assessment

### Data Sources
The classifier is fine-tuned per tenant on that tenant's own
historical labelled invoices. No cross-tenant training data.
Personal-data implications: the invoice metadata used contains
counterparty names, which under PDPL are personal data — handled
inside the tenant's residency boundary.

### Human Oversight
Every suggestion requires explicit user confirmation. Bulk-accept
is intentionally out of scope. The user can override any
suggestion before download. Audit log records the suggestion,
the user's decision, and the model version.

### Failure Modes
- Model offline: the step is skipped with a banner; user
  categorises manually as today. No download blocked.
- Confidence below 30%: no suggestion shown, free-text field
  presented instead.
- Model wrong: user corrects; correction is logged; quarterly
  review by Product checks correction patterns.

## Sales/CS Summary
AI-assisted invoice categorisation cuts customer review time
roughly two-thirds. It is opt-in, per-tenant, and never
auto-confirms. We can position this as the first AI feature in
our finance suite without triggering high-risk EU AI Act
obligations.

## AI Authorship Disclosure
- **Tools used:** Claude Sonnet 4.6
- **Scope:** Drafted Alternatives Considered, Success Metrics,
  and the AI Risk Assessment subsections
- **Human review:** @thuy-pham reviewed and edited every section;
  @legal-vn reviewed the AI Risk Assessment
```

## 7. Anti-patterns

- Setting `eu_ai_act_risk_class: minimal` to dodge the AI Risk Assessment when the feature actually emits AI-generated content to a client. The bucket is `limited` (Article 50 transparency obligation).
- Skipping `Alternatives Considered` because "the answer is obvious." If the answer were obvious, the section would be one sentence — write that sentence.
- `client_visible: true` with no Customer Quotes block. The validator catches this; do not work around it by lying about the flag.

## 8. Cross-departmental usage

| Department | What you fill | What you skip |
|---|---|---|
| Product | All sections; you own the spec | (nothing) |
| Engineering | Often co-author with Product; helpful on Dependencies and Failure Modes | Sales/CS Summary unless you wrote it for Product |
| Sales / CS | Customer Quotes (verbatim), Problem evidence, the Sales/CS Summary | Proposed Solution implementation details |
| Design | Can author user-facing features; mockups go in Proposed Solution | Compliance unless you have legal context |

## 9. Vietnamese version

This README is the canonical English documentation for the feature request template. The Vietnamese-language version lives at [README_VI.md](./README_VI.md) — separate file, kept in sync manually. The feature request body itself is English-only; do not interleave Vietnamese into the artifact the validator parses.

If the feature is shipped to Vietnamese-speaking users, ship a localised version of the user-facing copy at the surface where it appears (in-product strings, marketing copy). The feature request itself remains in English.

## 10. Compliance notes

EU AI Act Articles 5–7 govern risk classification; Article 14 governs human oversight; Article 50 governs transparency obligations for AI-generated content visible to natural persons. The `eu_ai_act_risk_class` field plus the conditionally-required `## AI Risk Assessment` section are the structural enforcement.

See [docs/compliance/eu-ai-act-risk-classes.md](../../docs/compliance/eu-ai-act-risk-classes.md) for the full mapping and the bucket-selection rules.

## 11. AI authorship guidance for this artifact

Feature requests are heavily AI-assisted in practice — this is fine and we encourage it. Set `ai_authorship` accurately and fill the disclosure block. The disclosure is per-artifact, not per-tool: if Claude drafted prose and Cursor refactored an inline code sketch, list both.

## 12. Migration from legacy v1.0

For migration from legacy YAML issue forms or v1.0 feature templates, see [docs/migration/from-v1-yaml-forms.md](../../docs/migration/from-v1-yaml-forms.md).

## 13. Validation contract (what the validator checks)

The validator enforces:

- All required frontmatter fields are present.
- Frontmatter keys are snake_case.
- Enum values are inside the allowed set.
- `eu_ai_act_risk_class` cannot be `unacceptable` (schema reject).
- `eu_ai_act_risk_class` is `limited` or `high` ⇒ `## AI Risk Assessment` H2 with the three required subsections.
- `client_visible: true` ⇒ `## Customer Quotes` and `## Sales/CS Summary` H2 sections present.
- `ai_authorship != none` ⇒ `## AI Authorship Disclosure` H2 with three bullets.
- `<untrusted_content>` blocks are not nested and do not contain prompt-injection markers.

Exit codes: `0` pass, `1` errors, `2` warnings only.
