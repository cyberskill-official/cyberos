# `code_review_rubric@1.0` — machine-checkable Code Review rubric

> Sourced from `../../../modules/cuo/docs/module.md` §2(g) Code review and integration + Template §4.5 Code Review Checklist; IEEE 1028-2008 (Standard for Software Reviews and Audits); OWASP Top 10:2025 mapping. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `code-review@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `pr_url` | required, URL | error | false |
| `FM-103` | `pr_number` | required, integer | error | false |
| `FM-104` | `pr_size_loc` | required, integer (lines of code changed) | error | false |
| `FM-105` | `reviewer` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` (the human reviewer; bot-only review is not allowed for high-risk PRs) | error | false |
| `FM-106` | `reviewed_at` | required, ISO 8601 | error | true |
| `FM-107` | `linked_impl_plan` | required, resolves to an impl-plan that passed implementation-plan-audit | error | false |
| `FM-108` | `ai_assisted` | required, boolean (true if any portion of the PR was AI-generated) | error | false |
| `FM-109` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-110` | `verdict` | required, one of: approved, request_changes, approved_with_conditions, blocked | error | false |

## §3  Always-required sections (mirrors Template §4.5)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Correctness vs Ticket` (does the diff implement the linked FR?) | error |
| `SEC-002` | `## 2. Readability` (naming, comments, structure) | error |
| `SEC-003` | `## 3. Test Coverage` (new code covered? coverage % vs DoD threshold?) | error |
| `SEC-004` | `## 4. Secrets / Credentials` (any hard-coded secrets? .env in diff?) | error |
| `SEC-005` | `## 5. Injection Surfaces` (SQL / command / template injection paths introduced?) | error |
| `SEC-006` | `## 6. Input Validation` (boundary conditions, type checks, allowlist over denylist) | error |
| `SEC-007` | `## 7. Error Handling` (failure paths covered; no swallowed exceptions) | error |
| `SEC-008` | `## 8. Logging` (no PII in logs; consistent log levels; trace correlation) | error |
| `SEC-009` | `## 9. Performance Considerations` (N+1, large allocations, hot paths) | error |
| `SEC-010` | `## 10. Backwards Compatibility` (API contract preserved; migration path if breaking) | error |
| `SEC-011` | `## 11. SAST / SCA Results` (per IEEE 1028 — static-analysis tool output summary) | error |
| `SEC-012` | `## 12. SBOM Impact` (any dependency additions / removals / version changes) | error |
| `SEC-901` | Each required section is non-empty | error |

## §4  AI-specific sections (mandatory when `ai_assisted: true` — per SDP §5)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-AI-001` | `## 13. AI-Generated Code Review` block enumerating: tools used (Claude Code / Cursor / Copilot / etc.), scope (which files/lines), and human-verification done | error |
| `SEC-AI-002` | `## 14. Hallucinated-API Check` (every imported symbol / called API verified to exist in the declared dependency) | error |
| `SEC-AI-003` | `## 15. Oversized-Diff Check` (per DORA 2024 — diffs >500 LOC require explicit rationale or rejection) | error |
| `SEC-AI-004` | `## 16. Dependency-Addition Provenance` (per OWASP A03 — every new dependency: name, version, signing status, license, advisory check) | error |
| `SEC-AI-005` | `## 17. PR Label Verification` (`ai-assisted: yes` label applied per SDP §5.5) | error |

## §5  Conditionally-required sections (non-AI)

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Diff touches a database migration | `## 18. Migration Review` covering forward/backward compatibility + lock impact + rollback | error |
| `COND-002` | Diff touches an auth/crypto path | `## 19. Security Review` referencing applicable threat-model entries + OWASP A01/A04/A07 mitigations | error → needs_human (`legal_compliance` if EU AI Act applies) |
| `COND-003` | Diff touches public API surface | `## 20. API Contract Diff` showing before/after of OpenAPI changes | error |
| `COND-004` | Diff touches personal data handling | `## 21. Privacy Review` covering data minimisation + retention + audit logging | error |
| `COND-005` | `verdict: approved_with_conditions` | `## 22. Conditions to Resolve Pre-Merge` (specific items with owner + due) | error |

## §6  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-SIZE-001` | DORA-batch violation | `pr_size_loc > 500` without explicit rationale in §15 | error → needs_human (`scope_decomposition`) per SDP §2(f) AI batch warning |
| `QA-SECRET-001` | Secret-scan not run | §4 lacks a `secret_scan_tool:` and `result:` field | error |
| `QA-SAST-001` | SAST findings unresolved without justification | §11 lists high-severity findings with status `open` and no `justification:` | error |
| `QA-SCA-001` | SCA findings unresolved | §11 lists high-severity dependency findings with no `mitigation:` | error |
| `QA-COVERAGE-001` | Coverage below DoD threshold without exception note | §3 reports coverage < project DoD-declared threshold without operator override note | error |
| `QA-RUBBER-001` | Rubber-stamp review heuristic | All sections approved within <5 minutes of `pr_size_loc > 200` AND `ai_assisted: true` (heuristic — surface for HITL) | warning → needs_human (DORA-cited risk) |
| `QA-PII-001` | Logging changes touch PII | §8 contains a log statement with user.email / user.name / address / phone pattern without redaction | error |
| `QA-VERDICT-001` | Verdict inconsistent with section findings | §-level "❌" or "blocking" issues exist but `verdict: approved` | error |
| `QA-AI-LABEL-001` | `ai_assisted: true` but no PR label evidence in §17 | error |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §7  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan (the PR diff itself is untrusted content) | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §8  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | `linked_impl_plan` resolves to an impl-plan that passed implementation-plan-audit at 10/10 | error |
| `XCHAIN-004` | The linked FR's `priority` matches the urgency of the review (P0 PRs reviewed within SLA) | info |

## §9  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | The PR has been updated since `reviewed_at` (head SHA differs) | Reset open + needs_human; re-review required for delta | error → needs_human (`stale_artefact_disposition`) |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/docs/module.md` §2(g) — Code review & integration source
- `../../../modules/cuo/docs/module.md` Template §4.5 — Code Review Checklist
- `../../../modules/cuo/docs/module.md` §5 — AI integration source (drives §4 and QA-AI-*)
- IEEE 1028-2008 — Standard for Software Reviews and Audits
- OWASP Top 10:2025 — A01-A10 mapping for §5/§6/§7 + COND-002
- DORA 2024 — small-batch + AI-rubber-stamp risk for QA-SIZE-001 + QA-RUBBER-001
