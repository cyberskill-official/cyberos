# `threat_model_rubric@1.0` — machine-checkable STRIDE threat-model rubric

> Sourced from `cyberos/docs/Software Development Process.md` §2(d) Architecture security review; OWASP Top 10:2025 (A01-A10); OWASP ASVS; STRIDE methodology. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---`; closing `---` exists; YAML parses | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `threat-model@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `system_under_threat` | required, string (the system / product / service being modelled) | error | false |
| `FM-103` | `tm_version` | required, SemVer | error | true |
| `FM-104` | `modelled_at` | required, ISO 8601 | error | true |
| `FM-105` | `modelled_by` | required, array of operator handles with `role:` (Architect, SEC) | error | false |
| `FM-106` | `linked_srs`, `linked_adrs` | recommended, array of paths | warning | false |
| `FM-107` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-108` | `asvs_level` | required, one of: L1, L2, L3 (the OWASP ASVS verification level being targeted) | error | false |
| `FM-109` | `next_review_date` | required, ISO 8601 (max +180 days from `modelled_at`) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. System Overview` (1-2 paragraphs + context diagram reference) | error |
| `SEC-002` | `## 2. Trust Boundaries` (named boundaries with privilege deltas) | error |
| `SEC-003` | `## 3. Data Flow Diagram` (reference to DFD asset; required, even if external) | error |
| `SEC-004` | `## 4. Threats by STRIDE Category` (one H3 per STRIDE letter) | error |
| `SEC-005` | `### 4.1 Spoofing` | error |
| `SEC-006` | `### 4.2 Tampering` | error |
| `SEC-007` | `### 4.3 Repudiation` | error |
| `SEC-008` | `### 4.4 Information Disclosure` | error |
| `SEC-009` | `### 4.5 Denial of Service` | error |
| `SEC-010` | `### 4.6 Elevation of Privilege` | error |
| `SEC-011` | `## 5. OWASP Top 10:2025 Coverage` (table mapping A01-A10 to applicable mitigations) | error |
| `SEC-012` | `## 6. OWASP ASVS Controls Mapping` (per-control rows for the declared `asvs_level`) | error |
| `SEC-013` | `## 7. Residual Risk Register` (threats accepted, with owner + review date) | error |
| `SEC-014` | `## 8. Mitigations and Linked ADRs` (mitigation → ADR-NNNN reference) | error |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | System processes personal data | `## 9. Privacy Threat Analysis` block per LINDDUN (or equivalent privacy threat-modelling framework) | error |
| `COND-002` | System uses AI/ML | `## 10. ML-Specific Threats` covering model evasion, model inversion, training-data poisoning, supply-chain compromise of pretrained models | error → needs_human (`ai_act_risk_boundary` if EU AI Act applies) |
| `COND-003` | System exposes a public API | `## 11. API-Specific Threats` covering rate-limit bypass, auth bypass, IDOR, mass-assignment, business-logic abuse | error |
| `COND-004` | `asvs_level: L3` | Each ASVS L3 control in §6 must have a "status: implemented | compensated | accepted-risk" with evidence link | error |

## §5  STRIDE-specific rules (per category)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `STRIDE-S-001` | §4.1 Spoofing enumerates each authentication boundary identified in §2 | error |
| `STRIDE-T-001` | §4.2 Tampering covers each data store / transit path identified in §3 | error |
| `STRIDE-R-001` | §4.3 Repudiation discusses logging/audit trail per OWASP A09 Security Logging & Alerting Failures | error |
| `STRIDE-I-001` | §4.4 Information Disclosure covers each personal-data flow identified in §2 (if COND-001 fires) | error |
| `STRIDE-D-001` | §4.5 Denial of Service covers each ingress identified in §2 | error |
| `STRIDE-E-001` | §4.6 Elevation of Privilege covers each privilege boundary identified in §2 | error |

## §6  OWASP Top 10:2025 coverage rules

| rule_id | OWASP risk | Required treatment | Severity |
| ------- | ---------- | ------------------ | -------- |
| `OWASP-A01` | Broken Access Control | §5 must declare access-control approach + reference STRIDE-E threats | error |
| `OWASP-A02` | Security Misconfiguration (elevated in 2025) | §5 must declare hardening posture + reference STRIDE-T threats | error |
| `OWASP-A03` | Software Supply Chain Failures (elevated in 2025) | §5 must declare SBOM strategy + dependency scanning + provenance verification | error |
| `OWASP-A04` | Cryptographic Failures | §5 must declare crypto-suite policy + key-management approach | error |
| `OWASP-A05` | Injection | §5 must declare input-validation strategy + reference STRIDE-T threats | error |
| `OWASP-A06` | Insecure Design | §5 must reference at least one threat addressed by design choices (not just controls) | error |
| `OWASP-A07` | Authentication Failures | §5 must declare auth approach (federation / passwordless / MFA) + reference STRIDE-S threats | error |
| `OWASP-A08` | Software & Data Integrity Failures | §5 must declare update-channel integrity + CI/CD integrity controls | error |
| `OWASP-A09` | Security Logging & Alerting Failures | §5 must reference STRIDE-R treatment | error |
| `OWASP-A10` | Mishandling of Exceptional Conditions | §5 must declare fail-safe defaults + error-handling policy | error |

## §7  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-THREAT-001` | Threat without mitigation OR explicit accepted-risk note | A row in §4 lacks `mitigation:` or `accepted_risk_owner:` | error |
| `QA-THREAT-002` | Mitigation references an ADR that doesn't exist | An ADR-NNNN in §8 doesn't resolve | error |
| `QA-RESID-001` | Residual risk without owner or review date | §7 row lacks `owner:` or `review_date:` | error |
| `QA-CVE-001` | Hallucinated CVE | A CVE-YYYY-NNNN identifier referenced in §5/§6/§7 doesn't match the NVD/MITRE CVE format pattern AND lacks a `source_ref` to a primary advisory | error (anti-fabrication critical for CVEs) |
| `QA-LINDDUN-001` | Privacy section invokes LINDDUN but doesn't enumerate the seven categories (Linkability, Identifiability, Non-repudiation, Detectability, Disclosure of information, Unawareness, Non-compliance) | warning |
| `QA-ML-001` | ML threats block missing supply-chain considerations for pretrained models | COND-002 fires but §10 lacks "pretrained model provenance" subsection | warning |
| `QA-TODO` | Skeleton TODO marker remaining | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | warning |

## §8  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §9  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches author manifest at write time | error |
| `XCHAIN-003` | Every ADR-NNNN in §8 resolves to an accepted ADR in the project | error |
| `XCHAIN-004` | Every linked SRS REQ-ID resolves | warning |

## §10  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source SRS/ADR hash differs | Reset open + needs_human to open | warning → needs_human |
| `STALE-002` | `modelled_at` is past `next_review_date` | Trigger review cycle | warning |
| `STALE-003` | A new accepted ADR exists that affects a `linked_adrs` ADR not enumerated in this threat model | Suggest threat-model update | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `REPORT_FORMAT.md`, `INVARIANTS.md`
- `cyberos/docs/Software Development Process.md` §2(d) — Architecture security review source
- OWASP Top 10:2025
- OWASP ASVS (Application Security Verification Standard)
- STRIDE (Microsoft) — threat-modelling framework
- LINDDUN (KU Leuven) — privacy threat-modelling framework
