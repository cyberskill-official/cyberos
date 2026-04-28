# Bug Report Template — CyberSkill v1

> Turn Your Will Into Real.

## 1. Purpose

This template captures bugs in a way that is repeatable for engineering, defensible for compliance (Vietnam PDPL Article 23), and safe for AI triage tools. It encodes the 72-hour breach-notification clock structurally so it cannot be skipped by accident.

## 2. When to use this template

Use it for any defect — a behavioural defect, a documentation defect, an accessibility defect, a security defect, a performance regression. If the system is doing something it should not be doing, or not doing something it should be doing, this is the template.

For a feature you wish existed, use the Feature Request template instead.

## 3. Field reference

| Field | Type | Required | Allowed values | Filled by | Why it exists |
|---|---|---|---|---|---|
| `title` | string | yes | <= 72 chars | reporter | One-line statement of the defect |
| `author` | string | yes | `@handle` | reporter | Attribution |
| `department` | enum | yes | engineering, design, product, sales, operations, hr, client_success | reporter | Routes triage |
| `status` | enum | yes | draft .. closed | triage | Lifecycle |
| `priority` | enum | yes | p0..p3 | triage | Work-order priority |
| `created_at` | string (date) | yes | ISO 8601 | scaffolder | Anchors SLA |
| `ai_authorship` | enum | yes | none, assisted, co_authored, generated_then_reviewed | reporter | Transparency |
| `template` | enum | yes | `bug_report@1` | scaffolder | Schema-pin |
| `severity` | enum | yes | sev1, sev2, sev3, sev4 | triage | Customer-impact severity, distinct from priority |
| `affected_versions` | array | yes | SemVer ranges, at least one | reporter | Bounds the search |
| `pdpl_breach_suspected` | boolean | yes | true / false | reporter | Triggers PDPL Article 23 conditional fields |
| `discovered_at` | datetime | conditional | ISO 8601 with timezone | reporter | Required when `pdpl_breach_suspected=true`; clock start |
| `reproducible` | enum | yes | always, intermittent, once, unable_to_reproduce | reporter | Triage signal |

## 4. Section reference

| Section | Required? | When required | What good looks like | Common mistake |
|---|---|---|---|---|
| Summary | yes | always | Symptom + area, two sentences | Stack trace dump |
| Reporter Description | yes | always | Verbatim words; untrusted block when from a customer | Paraphrasing customer language |
| Steps to Reproduce | yes | always | Numbered, copy-paste-runnable | "Just open the page" |
| Expected Behaviour | yes | always | Tied to spec or prior behaviour | Opinion |
| Actual Behaviour | yes | always | Verbatim error message + screenshot | "It crashes" |
| Environment | yes | always | Versions, OS, region, tenant ID | "On my laptop" |
| Impact | yes | always | Counts, revenue, contracts | "Important" |
| Breach Containment | conditional | `pdpl_breach_suspected=true` | Immediate containment, residual exposure | Filling later |
| Notification Plan | conditional | `pdpl_breach_suspected=true` | Subjects, regulators, deadline computed from `discovered_at` | Vague timeline |
| AI Authorship Disclosure | conditional | `ai_authorship != none` | Three required bullets | Skipping the human-review bullet |

## 5. Required-when rules

The validator enforces:

1. `pdpl_breach_suspected: true` ⇒ `discovered_at` is set (schema-enforced) AND body contains `## Breach Containment` AND `## Notification Plan` H2 sections (validator body check).
2. `ai_authorship != none` ⇒ `## AI Authorship Disclosure` H2 with three bullets.

## 6. Example (fully-filled realistic artifact)

```markdown
---
title: "Customer can see another tenant's invoices on the export endpoint"
author: "@hoa-le"
department: client_success
status: ready_for_review
priority: p0
created_at: "2026-04-28"
ai_authorship: none
severity: sev1
affected_versions: [">=2.4.0 <2.4.3"]
pdpl_breach_suspected: true
discovered_at: "2026-04-28T09:12:00+07:00"
reproducible: always
template: bug_report@1
---

# Bug Report

> Turn Your Will Into Real.

## Summary
The `/api/v2/invoices/export` endpoint returns invoices belonging to
other tenants when called with a specific query parameter.

## Reporter Description
<untrusted_content source="customer_email">
"I exported my invoices and the spreadsheet contains a column with
company names I don't recognise. We are not supposed to see other
companies' data."
</untrusted_content>

## Steps to Reproduce
1. Log in as any tenant user with the `invoices:read` scope.
2. Call `GET /api/v2/invoices/export?include_archived=true`.
3. Open the resulting CSV.
4. Observe rows with `tenant_id` not equal to the caller's tenant.

## Expected Behaviour
The endpoint must return only invoices where `tenant_id` matches the
caller's tenant.

## Actual Behaviour
The endpoint returns invoices for all tenants when
`include_archived=true` is set. The `tenant_id` filter is dropped in
the archive branch (see `invoice_query.go:147`).

## Environment
- Region: `ap-southeast-1` (Vietnam-resident tenants)
- Versions: 2.4.0 through 2.4.2
- Tenants observed: PV-A (reporter), Acme Co (data leaked)

## Impact
Telemetry shows 12 tenants made the offending API call in the last
72 hours; up to 47 tenants' invoice metadata was exposed. PDPL
Article 23 applies.

## Breach Containment
- 09:12 UTC+7: discovered.
- 09:18: feature flag `invoices_export_v2` flipped to off.
- 09:26: endpoint returns 503 globally.
- Residual exposure: data already downloaded; cannot be recalled.

## Notification Plan
- Data subjects: 47 affected tenants (account owners and primary
  contacts).
- Regulator: Cục An toàn thông tin, MIC (Vietnam).
- Deadline: 2026-05-01T09:12:00+07:00 (72h from discovery).
- Owner: @hoa-le (CS), @nguyen-tran (Eng), @legal-vn (Legal).
```

## 7. Anti-patterns

- Filling `pdpl_breach_suspected: false` to avoid the extra sections. The validator does not catch a lie, but the auditor will. If you are unsure, set it true and let Legal triage.
- Pasting customer email text outside the `<untrusted_content>` block. The block is the boundary that prompt-injection defences depend on.
- Setting `severity: sev1` without setting `priority: p0`. Severity is impact; priority is work order. They usually align, but when they don't, say why in the Impact section.

## 8. Cross-departmental usage

| Department | What you fill | What you skip |
|---|---|---|
| Engineering | All technical fields, full body | (nothing) |
| Client Success | `## Reporter Description` (verbatim from client), `pdpl_breach_suspected` flag, Impact | Technical repro details — leave for Eng |
| Sales | Severity from client perspective, business impact | Stack traces, `affected_versions` |
| Design | UI/accessibility defects: screenshots, expected vs actual, design-system component name | Backend repro |
| Operations | Infra outages: tenant ID, region, observability links | Code-level repro |

This is the "ten-person consultancy" reality — Sales and CS file bugs.

## 9. Vietnamese version

This README is the canonical English documentation for the bug report template. The Vietnamese-language version lives at [README_VI.md](./README_VI.md) — separate file, kept in sync manually. The bug report body itself is English-only; do not interleave Vietnamese into the artifact the validator parses.

For client-facing post-mortem language in Vietnamese, write the post-mortem in a separate document — do not stuff it into this template.

## 10. Compliance notes

Vietnam PDPL Article 23: notification within 72 hours of discovering a personal-data breach. The `pdpl_breach_suspected` flag, the conditionally-required `discovered_at` field, and the conditionally-required `## Breach Containment` and `## Notification Plan` sections are the structural enforcement of that obligation.

See [docs/compliance/pdpl-vietnam-breach-clock.md](../../docs/compliance/pdpl-vietnam-breach-clock.md) for the full text and the deadline-computation rule.

## 11. AI authorship guidance for this artifact

Bug reports drafted with AI tooling must declare it. AI is particularly useful for re-writing terse customer messages into reproducible steps — when you do that, set `ai_authorship: assisted` and name the tool. The Reporter Description must remain verbatim regardless.

## 12. Migration from legacy v1.0

For migration from legacy YAML issue forms or v1.0 bug templates, see [docs/migration/from-v1-yaml-forms.md](../../docs/migration/from-v1-yaml-forms.md).

## 13. Validation contract (what the validator checks)

The validator enforces:

- All required frontmatter fields are present.
- Frontmatter keys are snake_case.
- Enum values are inside the allowed set.
- `pdpl_breach_suspected=true` ⇒ `discovered_at` is set (schema), AND `## Breach Containment` and `## Notification Plan` H2 sections present (body).
- `ai_authorship != none` ⇒ `## AI Authorship Disclosure` H2 with three bullets.
- `<untrusted_content>` blocks are not nested and do not contain prompt-injection markers (see audit §1.5).

Exit codes: `0` pass, `1` errors, `2` warnings only.
