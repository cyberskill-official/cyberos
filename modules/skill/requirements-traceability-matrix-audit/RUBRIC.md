# `rtm_rubric@1.0` — machine-checkable Requirements Traceability Matrix rubric

> Sourced from `../../../modules/cuo/README.md` §3 Traceability matrix + Template §4.4. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `requirements-traceability-matrix@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `project` | required, string | error | false |
| `FM-103` | `rtm_version` | required, SemVer | error | true |
| `FM-104` | `generated_at` | required, ISO 8601 (when the matrix was last regenerated) | error | true |
| `FM-105` | `source_set` | required, object listing each source artefact path + hash (SRSes, PRDs, FRs) | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required (typically the source_set canonical concat hash) | error | false |
| `FM-107` | `release` | optional; if present, ties the matrix to a specific release tag | warning | false |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## 1. Summary` (counts: total REQs, traced REQs, untested REQs, orphan code) | error |
| `SEC-002` | `## 2. Matrix` (the columns per Template §4.4: REQ-ID, Description, Source, Priority, Linked Design, Linked Code/PR, Linked Test, Status, Release) | error |
| `SEC-003` | `## 3. Orphans` (REQs with no design / no code / no test linkage) | error |
| `SEC-004` | `## 4. Untested` (REQs with design + code but no test linkage) | error |
| `SEC-005` | `## 5. Untraceable Code` (PRs / commits not linked to a REQ-ID) | warning |
| `SEC-006` | `## 6. Coverage Stats` (per-source-doc REQ coverage %, per-priority traced %, per-status counts) | error |
| `SEC-901` | Each required section is non-empty (matrix MAY be empty in §2 only if `source_set` is empty AND `summary.total_reqs = 0`) | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `release` set | `## 7. Release-Scope Filter` showing only REQs targeted for this release | error |
| `COND-002` | Project is regulated (per project-plan COND-004) | `## 8. Regulatory Mapping` showing each REQ's link to applicable regulation (GDPR / HIPAA / Vietnam Decree 13/2023 PDPD / etc.) | error → needs_human (`legal_compliance`) |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-ORPHAN-001` | Orphan rate >5% of total REQs | warning → needs_human (`scope_decomposition`) |
| `QA-UNTESTED-001` | Untested rate >10% of P0/P1 REQs | error → needs_human (`nfr_coverage`) |
| `QA-UNTRACE-001` | Untraceable code rate >15% (PRs without REQ-IDs) | warning (process hygiene) |
| `QA-LINK-001` | A `Linked Design` reference doesn't resolve | error |
| `QA-LINK-002` | A `Linked Code/PR` reference doesn't resolve | error |
| `QA-LINK-003` | A `Linked Test` reference doesn't resolve | error |
| `QA-STATUS-001` | Status enum violation | A row's `Status` not in {drafted, designed, in_dev, in_test, shipped, deferred} | error |
| `QA-PRIORITY-001` | Priority enum violation | A row's `Priority` not in {p0, p1, p2, p3} | error |
| `QA-DUP-001` | Duplicate REQ-ID across sources | error → needs_human (`scope_decomposition`) |
| `QA-COVERAGE-001` | Coverage stats inconsistent with §2/§3/§4 counts | error |
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
| `XCHAIN-003` | Every REQ-ID in §2 resolves in at least one source in `source_set` | error |
| `XCHAIN-004` | Every `Linked Test` resolves to a test in the project's declared test layout | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Any source in `source_set` hash differs from current source on disk | Regenerate matrix; reset coverage stats | error → needs_human (`stale_artefact_disposition`) |
| `STALE-002` | `generated_at` >14 days old | Suggest re-generation | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/README.md` §3 + Template §4.4 — RTM source
