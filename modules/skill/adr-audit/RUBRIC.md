# `adr_rubric@1.0` — machine-checkable Architecture Decision Record rubric

> Sourced from `cyberos/docs/Software Development Process.md` §2(d) System architecture and high-level design; Michael Nygard's ADR format; arc42 documentation; ISO/IEC 25010:2023 portability/maintainability sub-characteristics. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---`; closing `---` exists; YAML parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `adr@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–120 chars | error | skeleton |
| `FM-102` | `adr_id` | required, `^ADR-\d{4}$` (zero-padded 4 digits) | error | false |
| `FM-103` | `status` | required, one of: proposed, accepted, deprecated, superseded | error | false |
| `FM-104` | `supersedes` | optional, array of `ADR-NNNN`; if present, the named ADRs MUST exist with `status: superseded` | error | false |
| `FM-105` | `superseded_by` | optional; mutually exclusive with `status: accepted` (if `superseded_by` present, status must be `superseded`) | error | false |
| `FM-106` | `decision_date` | required, ISO 8601 | error | true |
| `FM-107` | `decided_by` | required — array of operator handles with `role:` | error | false |
| `FM-108` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-109` | `linked_srs_reqs` | recommended — list of REQ-IDs this ADR affects | warning | false |
| `FM-110` | `iso_25010_impacted_chars` | required — array of ISO/IEC 25010:2023 quality characteristics this decision affects (e.g. performance_efficiency, maintainability, security) | error | false |

## §3  Always-required sections (Nygard ADR format)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Context` (the forces in tension, the problem to solve) | error |
| `SEC-002` | `## 2. Options Considered` (≥2 distinct alternatives with pros/cons) | error |
| `SEC-003` | `## 3. Decision` (the chosen option, in a single sentence) | error |
| `SEC-004` | `## 4. Consequences` (positive, negative, neutral — see SDP §2(d)) | error |
| `SEC-005` | `## 5. Compliance / Quality Impact` (mapping to `iso_25010_impacted_chars`) | error |
| `SEC-006` | `## 6. Notes / References` (links to RFCs, prior ADRs, benchmarks) | warning |
| `SEC-901` | Each required section is non-empty | error |
| `SEC-902` | Section ordering matches SEC-001..006 | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | Decision touches security boundary (auth, crypto, data flow, attack surface) | `## 7. Security Impact` block aligned with OWASP ASVS sections | error → needs_human (`legal_compliance` if EU AI Act applies) |
| `COND-002` | Decision touches data residency / personal data | `## 8. Data Residency and Privacy Impact` | error |
| `COND-003` | Decision is reversible only at high cost (per Nygard's "two-way door" framing) | `## 9. Reversal Cost Estimate` with rationale | warning |
| `COND-004` | `status: superseded` | `## 10. Why Superseded` referencing the successor ADR | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-OPT-001` | Single-option ADR | §2 lists <2 distinct alternatives (excluding "do nothing" placeholder) | error |
| `QA-OPT-002` | Pros-cons asymmetry | An option in §2 has only pros or only cons | warning |
| `QA-DEC-001` | Vague decision | §3 longer than 3 sentences or contains "may", "could", "should consider" | warning |
| `QA-CONSEQ-001` | Consequences only positive | §4 has no negative bullets | warning |
| `QA-ISO-001` | Missing ISO/IEC 25010 mapping | `iso_25010_impacted_chars` empty AND §5 empty | error → needs_human (`nfr_coverage`) |
| `QA-LINK-001` | Linked REQ-ID doesn't resolve | A REQ-ID in `linked_srs_reqs` isn't found in any SRS in the project | warning |
| `QA-ADR-001` | Cyclic supersedes | `supersedes` ↔ `superseded_by` cycles between two ADRs | error |
| `QA-OWASP-001` | Security-relevant decision but no ASVS reference | COND-001 fires but §7 lacks any `ASVS-V*` reference | warning |
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
| `XCHAIN-002` | `provenance.source_hash` matches author manifest at write time | error |
| `XCHAIN-003` | Every `linked_srs_reqs` REQ-ID resolves in some SRS | warning |
| `XCHAIN-004` | If COND-001 fires and a threat-model exists for this project, the ADR is referenced from the threat model OR the threat model was updated within 14 days of this ADR's `decision_date` | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source SRS hash differs from ADR's `provenance.source_hash` | Reset open + needs_human to open | warning → needs_human |
| `STALE-002` | ADR `decision_date` is >365 days old AND `status: accepted` | Suggest review or supersession | info |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `REPORT_FORMAT.md`, `INVARIANTS.md`
- `cyberos/docs/Software Development Process.md` §2(d) — Architecture stage source
- Michael Nygard, "Documenting Architecture Decisions" — ADR template origin
- arc42 — documentation conventions
- ISO/IEC 25010:2023 — quality-characteristic mapping for §5
