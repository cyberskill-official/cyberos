# `dor_dod_rubric@1.0` — machine-checkable Definition of Ready + Definition of Done rubric

> Sourced from `cyberos/docs/Software Development Process.md` Templates §4.1 (DoR — story-level entry criteria) + §4.2 (DoD — story-level exit criteria). Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `definition-of-ready-and-done@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `project` | required, string | error | false |
| `FM-104` | `engagement_model` | required, one of: fixed_price, time_and_materials, dedicated_team, staff_augmentation, managed_services | error | false |
| `FM-105` | `effective_date` | required, ISO 8601 | error | true |
| `FM-106` | `dor_dod_version` | required, SemVer | error | true |
| `FM-107` | `provenance.source_path` | required, exists | error | false |
| `FM-108` | `provenance.source_hash` | required, SHA-256 | error | false |
| `FM-109` | `approved_by` | required — array of operator handles with `role:` (PO, TL, EM, QA) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Definition of Ready` | error |
| `SEC-002` | `## 2. Definition of Done` | error |
| `SEC-003` | `## 3. Scope of Application` (story / spike / bug / epic — which DoR/DoD applies to which type) | error |
| `SEC-004` | `## 4. Waivers and Exceptions` (when DoR/DoD can be bypassed, by whom, and how it's recorded) | warning |
| `SEC-005` | `## 5. Review Cadence` (when this DoR/DoD itself is reviewed, e.g. quarterly) | warning |
| `SEC-901` | Each required section is non-empty | error |
| `SEC-902` | Section ordering matches SEC-001..005 | warning |

## §4  Definition of Ready — mandatory items (per SDP §4.1)

The §1 body MUST enumerate ALL of the following items as bullets or rows. Missing items emit `DOR-NNN` errors.

| rule_id | DoR item | Severity |
| ------- | -------- | -------- |
| `DOR-001` | Clear user value statement (who benefits, how) | error |
| `DOR-002` | Acceptance criteria explicit (Given/When/Then or equivalent) | error |
| `DOR-003` | Dependencies identified (other teams, vendors, hardware, external APIs) | error |
| `DOR-004` | NFRs noted (perf, security, accessibility, etc.) — even if "none beyond defaults" | error |
| `DOR-005` | Security/privacy implications flagged (data classification, PII, regulatory scope) | error |
| `DOR-006` | Designs attached (Figma link / wireframe / mock-up) or N/A noted with reason | error |
| `DOR-007` | Estimable in one sprint (or marked "spike" with timeboxed budget) | error |
| `DOR-008` | Demoable (success can be shown to a stakeholder in <5 minutes) | error |

## §5  Definition of Done — mandatory items (per SDP §4.2)

The §2 body MUST enumerate ALL of the following items as bullets or rows. Missing items emit `DOD-NNN` errors.

| rule_id | DoD item | Severity |
| ------- | -------- | -------- |
| `DOD-001` | Code merged to main / mainline branch | error |
| `DOD-002` | Unit tests passing | error |
| `DOD-003` | Integration tests passing | error |
| `DOD-004` | Code coverage threshold met (the threshold value SHALL be declared, e.g. ≥75%) | error |
| `DOD-005` | SAST scan clean (no new high-severity findings) | error |
| `DOD-006` | SCA scan clean (no new high-severity dependency vulns) | error |
| `DOD-007` | Documentation updated (API docs / user docs / ADR if applicable) | error |
| `DOD-008` | Deployed to staging | error |
| `DOD-009` | Product owner accepted (UAT or async sign-off) | error |
| `DOD-010` | Observability hooks present (logs / metrics / traces) | error |

## §6  Conditionally-required items

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `engagement_model = fixed_price` | DoD includes "stage-gate sign-off captured" (per SDP Template §4.3) | error |
| `COND-002` | Project handles personal data | DoD includes "privacy review passed" + DoR includes "data class flagged" | error |
| `COND-003` | Project is AI-driven | DoD includes "AI-use disclosure label on PR" + DoR includes "EU AI Act class assessed" | error |
| `COND-004` | Project is multi-region | DoD includes "tested in ≥2 regions" | warning |
| `COND-005` | Project is safety-critical | DoD includes "hazard analysis updated" + DoR includes "safety case applies" | error |

## §7  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | Any non-boilerplate paragraph lacks a `source_ref` marker | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | Any non-boilerplate paragraph lacks `authority:` | error |
| `QA-WAIVE-001` | Waiver in §4 without operator handle and reason | A waiver block lacks `waived_by:` and `reason:` | error |
| `QA-CADENCE-001` | Review cadence in §5 unspecified | §5 lacks a `next_review_date:` ISO 8601 date | warning |
| `QA-VAGUE-001` | Vague threshold | A DoD item has a threshold like "high coverage" without a number | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | not wrapped | warning |
| `QA-TODO` | Skeleton TODO marker remaining | literal `TODO:` | warning |

## §8  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands | warning |

## §9  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The DoR/DoD's `provenance.source_path` matches the author's manifest | warning |
| `XCHAIN-002` | The DoR/DoD's `provenance.source_hash` matches at write time | error |
| `XCHAIN-003` | Linked SOW (if any) declares the same `engagement_model` as the DoR/DoD frontmatter | error |

## §10  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source brief / SOW hash differs from DoR/DoD's `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate | warning → needs_human (`stale_artefact_disposition`) |
| `STALE-002` | `effective_date` is >180 days old | Suggest review cycle | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `REPORT_FORMAT.md`, `INVARIANTS.md`
- `cyberos/docs/Software Development Process.md` §4.1, §4.2 — DoR/DoD sources
