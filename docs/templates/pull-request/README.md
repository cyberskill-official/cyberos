# Pull Request Template — CyberSkill v1

> Turn Your Will Into Real.

## 1. Purpose

This template is the contract every pull request opened in a CyberSkill repository must satisfy. It exists to make reviews shorter, to make change-management auditable for SOC 2 CC8.1, and to make the AI-authorship trail legible without slowing humans down. It is the canonical PR template; departments do not fork their own.

## 2. When to use this template

Use it for every code-bearing PR — `feat`, `fix`, `refactor`, `perf`, `chore`, the lot. It is also used for documentation PRs (`docs`) and CI changes (`ci`), with most fields left at sensible defaults. The only PRs exempt are auto-generated dependabot/renovate bumps, which carry their own machine-format description.

## 3. Field reference

| Field | Type | Required | Allowed values | Filled by | Why it exists |
|---|---|---|---|---|---|
| `title` | string | yes | <= 72 chars, Conventional Commits subject | author | Forces a one-line intent statement; powers the changelog |
| `author` | string | yes | `@handle` (GitHub) | author | Attribution, CODEOWNERS routing |
| `department` | enum | yes | engineering, design, product, sales, operations, hr, client_success | author | Routes review automation; cross-departmental reporting |
| `status` | enum | yes | draft, ready_for_review, in_review, approved, merged, closed | author / reviewer | Lifecycle signal independent of GitHub's own state |
| `priority` | enum | yes | p0, p1, p2, p3 | author | Work-order priority; distinct from severity |
| `created_at` | string (date) | yes | ISO 8601 `YYYY-MM-DD` | scaffolder | Anchors SLA windows |
| `ai_authorship` | enum | yes | none, assisted, co_authored, generated_then_reviewed | author | EU AI Act Article 50 transparency |
| `template` | enum | yes | `pull_request@1` | scaffolder | Schema-pin so old PRs don't fail under new schemas |
| `pr_type` | enum | yes | feat, fix, docs, refactor, perf, test, build, ci, chore, revert | author | Conventional Commits routing |
| `breaking_change` | boolean | yes | true / false | author | Triggers `## Migration` requirement |
| `linked_issues` | array | optional | `#123` or `org/repo#123` | author | Cross-link |
| `soc2_change_class` | enum | yes | standard, expedited, emergency | author | SOC 2 CC8.1 classification |

## 4. Section reference

| Section | Required? | When required | What good looks like | Common mistake |
|---|---|---|---|---|
| Summary | yes | always | Two sentences explaining intent without referencing files | "Refactor X" with no rationale |
| Context | yes | always | Link to issue or doc, plus a paragraph of plain prose | A bare ticket link |
| Changes | yes | always | Grouped list, not a file-by-file enumeration | Pasting `git diff --stat` |
| How to verify | yes | always | Concrete commands and expected output | "Tested locally" |
| Risk and rollback | yes | always | Names rollback migration or feature flag | "Low risk" with no plan |
| Migration | conditional | `breaking_change=true` | Before/after code, version targeting | Empty section with the flag set |
| Post-Incident Review Plan | conditional | `soc2_change_class=emergency` | Owner, date, ticket, control reference | Skipping the SOC 2 reference |
| AI Authorship Disclosure | conditional | `ai_authorship != none` | Three required bullets, no padding | Claiming "AI did the typing" without scoping |

## 5. Required-when rules

The validator enforces the following conditional rules — fail conditions are blocking, not warnings:

1. `breaking_change: true` ⇒ body contains `## Migration` H2 with at least one non-empty paragraph.
2. `soc2_change_class: emergency` ⇒ body contains `## Post-Incident Review Plan` H2.
3. `ai_authorship != none` ⇒ body contains `## AI Authorship Disclosure` H2 with the three-bullet shape.

## 6. Example (fully-filled realistic artifact)

```markdown
---
title: "fix(auth): refresh tokens after tenant switch"
author: "@nguyen-tran"
department: engineering
status: ready_for_review
priority: p1
created_at: "2026-04-28"
ai_authorship: assisted
pr_type: fix
breaking_change: false
linked_issues: ["cyberskill-official/auth#412"]
soc2_change_class: standard
template: pull_request@1
---

# Pull Request

> Turn Your Will Into Real.

## Summary
After a tenant switch, the refresh token kept the old tenant claim,
so background jobs hit 401 within five minutes.

## Context
See cyberskill-official/auth#412. Reproduced on staging with two
tenants under the same user. The root cause is the cached claim
in `TokenService.cache`.

## Changes
- `TokenService.switchTenant` now invalidates the cache before issuing.
- New unit test in `token_service_test.ts` covers the regression.

## How to verify
1. `pnpm test packages/auth`
2. Manual: log in, switch tenant in the UI, wait 6 minutes, refresh.
   Expected: still authenticated.

## Risk and rollback
Low blast radius — touches one service. Rollback: revert this PR.
No DB migrations involved.

## AI Authorship Disclosure
- **Tools used:** Claude Sonnet 4.6, in Cursor
- **Scope:** Drafted the regression test in `token_service_test.ts`
- **Human review:** @nguyen-tran reviewed the test and the assertion order
```

## 7. Anti-patterns

- "Trivial change, no tests needed" without saying *why* it's trivial. Either it's worth saying out loud, or it's worth a test.
- Migration section with `breaking_change: true` but no code samples — the validator fails this; do not work around it.
- Claiming `ai_authorship: none` when AI wrote the description. The disclosure exists for the prose, not just the code.
- `soc2_change_class: standard` for a hot-fix that bypassed code review. If you skipped review, classify it `expedited` or `emergency` and own the post-review.

## 8. Cross-departmental usage

| Department | What you fill | What you skip |
|---|---|---|
| Engineering | All technical fields, full body | (nothing) |
| Design | `department: design`, body focused on UX changes; mockup links in Context | Code-specific verification |
| Product | `department: product`, used for spec PRs against the product repo | Migration unless spec breaks downstream tools |
| Sales / CS | Almost never opens a PR — file an issue instead | Most fields |

## 9. Vietnamese version

This README is the canonical English documentation for the PR template. The Vietnamese-language version of the same content lives at [README_VI.md](./README_VI.md) — separate file, not interleaved. The two files are kept in sync; if you change one, change the other.

The template body itself is English-only. Vietnamese localisation belongs in the documentation, not in the artifact the validator parses.

## 10. Compliance notes

This template carries the SOC 2 CC8.1 change-management evidence. The `soc2_change_class` field plus the conditionally-required `## Post-Incident Review Plan` section is the audit trail. Do not edit those fields out of the template even on PRs you believe are out of scope.

See [docs/compliance/soc2-change-management.md](../../docs/compliance/soc2-change-management.md) for the full mapping.

## 11. AI authorship guidance for this artifact

If AI helped author the PR description, the code, the tests, or the migration notes, set `ai_authorship` to the most-applicable of `assisted`, `co_authored`, or `generated_then_reviewed`, and fill the disclosure block with the three required bullets. The disclosure is not a confession; it is a scope statement.

## 12. Migration from legacy v1.0

For migration from legacy YAML issue forms or v1.0 PR templates, see [docs/migration/from-v1-pr-template.md](../../docs/migration/from-v1-pr-template.md).

## 13. Validation contract (what the validator checks)

The validator (`@cyberskill/templates validate`) enforces:

- All required frontmatter fields are present.
- Frontmatter keys are snake_case (no kebab-case, no camelCase).
- Enum values are inside the allowed set.
- `breaking_change=true` ⇒ `## Migration` H2 with at least one non-empty paragraph.
- `soc2_change_class=emergency` ⇒ `## Post-Incident Review Plan` H2 present.
- `ai_authorship != none` ⇒ `## AI Authorship Disclosure` H2 with three bullets.
- `title` parses as a Conventional Commits subject when `--pr-title` is supplied.

Exit codes: `0` pass, `1` errors (blocks merge), `2` warnings only.
