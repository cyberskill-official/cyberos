# `test_strategy_rubric@1.0` — machine-checkable Test Strategy rubric

> Sourced from `cyberos/docs/Software Development Process.md` §2(h) Testing + Template §4.6 Test Strategy outline; OWASP Top 10:2025 (threat-led pen-test); WCAG 2.2 (accessibility); ISO/IEC 25010:2023 (NFR coverage). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `test-strategy@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `strategy_version` | required, SemVer | error | true |
| `FM-103` | `linked_srs` | required, resolves to an SRS that passed srs-audit | error | false |
| `FM-104` | `risk_class` | required, one of: low, medium, high (per the SDP §3 risk heatmap) | error | false |
| `FM-105` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-106` | `effective_date` | required, ISO 8601 | error | true |
| `FM-107` | `qa_owner` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |

## §3  Always-required sections (mirror Template §4.6)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Scope` (what is in/out of scope for testing this release/feature) | error |
| `SEC-002` | `## 2. Risk-Based Test Priorities` (mapping each high-priority risk to a test approach) | error |
| `SEC-003` | `## 3. Test Levels` covering H3s: `### 3.1 Unit`, `### 3.2 Integration`, `### 3.3 System`, `### 3.4 UAT` | error |
| `SEC-004` | `## 4. Test Types` covering H3s: `### 4.1 Functional`, `### 4.2 Performance`, `### 4.3 Security`, `### 4.4 Accessibility`, `### 4.5 Regression` | error |
| `SEC-005` | `## 5. Environments and Data` (envs, refresh policy, anonymisation) | error |
| `SEC-006` | `## 6. Tooling` (concrete tools per type — e.g. Playwright/k6/OWASP ZAP/axe) | error |
| `SEC-007` | `## 7. Entry Criteria` (what must be true for testing to begin) | error |
| `SEC-008` | `## 8. Exit Criteria` (what must be true for testing to be declared complete) | error |
| `SEC-009` | `## 9. Defect Management` (severity scale, SLA, triage cadence, escape rate calc) | error |
| `SEC-010` | `## 10. Metrics` (defect density, defect leakage, automation coverage, MTTD/MTTR for prod-found defects) | error |
| `SEC-901` | Each required section is non-empty | error |
| `SEC-902` | Each H3 in §3 / §4 is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Linked SRS declares any UI surface | `### 4.4 Accessibility` references WCAG 2.2 conformance level (A / AA / AAA) + axe / Pa11y tool | error |
| `COND-002` | Linked SRS declares any public API surface | `### 4.3 Security` declares OWASP Top 10:2025 coverage + ZAP/Burp scan policy + auth/authz test cases | error |
| `COND-003` | `risk_class: high` | `## 11. Threat-Led Pen-Test Plan` referencing the threat-model artefact + per-STRIDE-category test cases | error |
| `COND-004` | Linked SRS declares any performance NFR | `### 4.2 Performance` declares load-test scenarios + tool + target percentile + soak duration | error |
| `COND-005` | Product handles personal data | `## 12. Data Privacy Test Cases` covering data-minimisation + retention + DSAR flows | error |
| `COND-006` | Product is AI-driven | `## 13. AI-Specific Test Cases` covering input adversarial robustness + bias evaluation + fall-back behaviour | error → needs_human (`ai_act_risk_boundary`) |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-NFR-COV-001` | NFR not covered | An NFR in the linked SRS §3.2 has no corresponding test type in §4 | error → needs_human (`nfr_coverage`) |
| `QA-OWASP-001` | Security testing missing OWASP mapping | §4.3 lacks any reference to A01..A10 | error |
| `QA-AUTOMATE-001` | Automation coverage target missing | §10 lacks `automation_coverage_target:` percentage | warning |
| `QA-ENTRY-EXIT-001` | Entry/exit criteria vague | §7 or §8 has fewer than 3 measurable bullets | warning |
| `QA-DEF-001` | Defect SLA missing | §9 lacks per-severity response/resolution SLA | warning |
| `QA-TOOL-001` | Tool named without version | A tool in §6 lacks a `version:` or `version_pin:` | warning |
| `QA-ENV-001` | Test environment lacks data-anonymisation note when using prod-like data | §5 mentions prod-derived data without anonymisation | error |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | Every NFR in linked SRS maps to at least one test type | warning → needs_human (`nfr_coverage`) |
| `XCHAIN-004` | If `risk_class: high` AND a threat-model exists, §11 references it | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Linked SRS hash differs | Reset open + needs_human to open | warning → needs_human |
| `STALE-002` | `effective_date` >180 days old | Suggest review cycle | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `cyberos/docs/Software Development Process.md` §2(h) + Template §4.6 — Test Strategy source
- OWASP Top 10:2025 — security-testing coverage
- WCAG 2.2 — accessibility-testing coverage
- ISO/IEC 25010:2023 — NFR-mapping coverage
