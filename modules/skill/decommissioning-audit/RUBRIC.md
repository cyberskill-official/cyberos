# `decomm_rubric@1.0` — machine-checkable Decommissioning rubric

> Sourced from `cyberos/docs/Software Development Process.md` §2(m) Decommissioning. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `decommissioning@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `system_being_retired` | required, string | error | false |
| `FM-103` | `retirement_date` | required, ISO 8601 (the target shutdown date) | error | false |
| `FM-104` | `decomm_version` | required, SemVer | error | true |
| `FM-105` | `data_retention_policy_ref` | required, link to the applicable retention policy (project DPA / corporate policy / regulator-mandated) | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-107` | `decision_owner` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` with `role:` (typically EM or owner team's TL) | error | false |
| `FM-108` | `compliance_owner` | required, handle (the operator responsible for sign-off of regulated-data disposition) | error | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Retirement Decision and Rationale` | error |
| `SEC-002` | `## 2. Affected Stakeholders` (customers, internal teams, vendors, regulators) | error |
| `SEC-003` | `## 3. Customer Communication Timeline` (T-60 / T-30 / T-7 / T-1 / T-0 notifications) | error |
| `SEC-004` | `## 4. Data Retention Plan` (per data class — what is retained, where, for how long, by whom) | error |
| `SEC-005` | `## 5. Data Export Plan` (formats / channels / verification per export target) | error |
| `SEC-006` | `## 6. Data Destruction Certificate` (per data class — destruction method, witness, completion log) | error |
| `SEC-007` | `## 7. DNS and Endpoint Retirement` (per FQDN — sunset HTTP status, redirect target, final removal) | error |
| `SEC-008` | `## 8. License and Vendor Cancellation` (per third-party — cancellation date, residual obligations) | error |
| `SEC-009` | `## 9. Source-Code Archive Manifest` (repo, branch, tag, archive location, checksum) | error |
| `SEC-010` | `## 10. Final Backup` (last full backup location + retention) | error |
| `SEC-011` | `## 11. On-Call and Runbook Decommissioning` (runbook archived; on-call rota removed) | error |
| `SEC-012` | `## 12. Sign-Off` (signers: decision_owner, compliance_owner, ops_owner) | error |
| `SEC-901` | Each required section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | System processed personal data | `## 13. GDPR Article 17 (Right to Erasure) Compliance` + `## 14. Vietnam Decree 13/2023 PDPD Disposition` (where applicable) | error → needs_human (`legal_compliance`) |
| `COND-002` | System held financial / payment data | `## 15. PCI-DSS Decommissioning Steps` (per PCI-DSS requirement 9.8) | error → needs_human (`legal_compliance`) |
| `COND-003` | System held health data | `## 16. HIPAA Disposal Compliance` (45 CFR § 164.310(d)(2)) | error → needs_human (`legal_compliance`) |
| `COND-004` | System integrated with external partners | `## 17. Partner-Notification Log` + revoked API keys / OAuth client_ids | error |
| `COND-005` | System is being replaced by a successor | `## 18. Migration Path to Successor` (linked to successor's SOW/SRS) | error |
| `COND-006` | System is being retired without replacement and customers are paying | `## 19. Refund / Credit Policy` | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-DATA-001` | Data class missing destruction method | §6 row lacks `destruction_method:` (overwrite + verify / crypto-erase / physical) | error → needs_human (`legal_compliance`) |
| `QA-DATA-002` | Data class missing witness | §6 row lacks `witness:` operator handle for high-sensitivity classes | error |
| `QA-DNS-001` | DNS row missing sunset HTTP status | §7 row lacks `sunset_status_code:` (typically 410 Gone) | warning |
| `QA-LICENSE-001` | License cancellation row missing residual-obligations note | §8 row lacks `residual_obligations:` | warning |
| `QA-ARCHIVE-001` | Source archive missing checksum | §9 lacks `archive_sha256:` | error |
| `QA-COMM-001` | Timeline gap | §3 missing T-30 or T-7 milestone | warning |
| `QA-SIGN-001` | Missing required signer | §12 lacks decision_owner / compliance_owner / ops_owner | error |
| `QA-RUNBOOK-001` | Runbook still active after `retirement_date` | warning |
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
| `XCHAIN-003` | If COND-005 fires, the successor's linked SOW exists and references this decomm | error |
| `XCHAIN-004` | Runbooks for `system_being_retired` are archived or marked retired before `retirement_date` | error |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | `retirement_date` passed but §6 destruction not logged complete | error → needs_human (`legal_compliance`) — likely regulatory exposure |
| `STALE-002` | A linked policy (`data_retention_policy_ref`) version changed since decomm `provenance.source_hash` | warning → needs_human |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `cyberos/docs/Software Development Process.md` §2(m) — Decommissioning source
- GDPR Article 17 — Right to Erasure
- Vietnam Decree 13/2023 PDPD
- PCI-DSS Requirement 9.8 — media destruction
- HIPAA 45 CFR § 164.310(d)(2) — health-data disposal
