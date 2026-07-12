# `deploy_checklist_rubric@1.0` — machine-checkable Deployment Readiness Checklist rubric

> Sourced from `../../../modules/cuo/docs/module.md` §2(i) Deployment and release management + Template §4.7 Deployment Readiness Checklist; DORA four key metrics. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `deployment-checklist@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `release_id` | required, SemVer or release tag | error | false |
| `FM-103` | `release_candidate_sha` | required, full git SHA | error | false |
| `FM-104` | `target_environment` | required, one of: staging, canary, production | error | false |
| `FM-105` | `deploy_window` | required, ISO 8601 timestamp range | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required | error | false |
| `FM-107` | `deploy_owner` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-108` | `change_ticket_id` | required (Jira / Linear / GitHub Issue ref) | error | false |
| `FM-109` | `progressive_delivery` | required, one of: canary, blue_green, feature_flag, rolling, direct (with rationale) | error | false |

## §3  Always-required sections (mirror Template §4.7)

Each item in §3 is a checklist row with `status: ✅ | ⏳ | ❌ | n/a` and `evidence:` link.

| rule_id | Item | Severity |
| ------- | ---- | -------- |
| `DEP-001` | All DoDs met (per project DoR/DoD doc) | error |
| `DEP-002` | Release notes drafted (release-notes artefact exists + audit-passed) | error |
| `DEP-003` | Rollback plan documented and rehearsed | error |
| `DEP-004` | Feature flags configured (per `progressive_delivery`) | error |
| `DEP-005` | Database migrations rehearsed in staging (forward + backward) | error |
| `DEP-006` | Monitoring + alerts in place for new code paths | error |
| `DEP-007` | On-call rota notified (calendar invite confirmed) | error |
| `DEP-008` | Security scans clean (SAST + SCA + secret-scan) | error |
| `DEP-009` | Change ticket approved by CAB (or async approver per change policy) | error |
| `DEP-010` | SBOM published for this release-candidate SHA | error |
| `DEP-011` | DORA baseline captured (deployment frequency / lead time / change failure rate / failed-deployment recovery time) | error |
| `DEP-012` | Signed artefacts (per OWASP A08 Software & Data Integrity) | error |

## §4  Conditionally-required items

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `target_environment: production` | `DEP-013` Customer-comms drafted (status page entry, email if breaking) | error |
| `COND-002` | Release contains a breaking change | `DEP-014` Deprecation timeline communicated + migration guide linked | error |
| `COND-003` | Release contains a data migration affecting >1M rows | `DEP-015` DBA approved + execution-plan reviewed | error |
| `COND-004` | Release touches a regulated path (privacy / financial / health) | `DEP-016` Compliance sign-off captured + recorded in change ticket | error → needs_human (`legal_compliance`) |
| `COND-005` | `progressive_delivery: canary` | `DEP-017` Canary success/failure criteria declared + auto-rollback wired | error |
| `COND-006` | Release contains AI-model update | `DEP-018` Model card + eval results + rollback model SHA recorded | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-EVIDENCE-001` | ✅ without evidence link | A DEP-* row marked ✅ has no `evidence:` link | error |
| `QA-EVIDENCE-002` | Evidence link doesn't resolve | broken at audit time | warning |
| `QA-ROLLBACK-001` | Rollback plan vague | §DEP-003 `evidence:` doesn't reference a specific runbook section or rehearsal log | error |
| `QA-RUN-001` | Direct deploy without rationale | `progressive_delivery: direct` without operator rationale | warning → needs_human |
| `QA-SBOM-001` | SBOM missing components | `DEP-010` evidence omits a dependency present in the release artefact | error |
| `QA-WINDOW-001` | Deploy window overlaps with maintenance freeze | warning → needs_human (`stale_artefact_disposition`) |
| `QA-DORA-001` | DORA baseline not captured | `DEP-011` has no metric values | error |
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
| `XCHAIN-003` | Linked release-notes artefact passed release-notes-audit at 10/10 | error |
| `XCHAIN-004` | DoR/DoD doc declares the items checked match the SDP §4.2 DoD minimums | warning |
| `XCHAIN-005` | runbook artefact exists for the system being deployed (so on-call has it) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | The release-candidate SHA changed after this checklist was authored | Reset all ✅ to ⏳ for re-verification | error → needs_human (`stale_artefact_disposition`) |
| `STALE-002` | `deploy_window` has passed and `target_environment: production` deploy not completed | Surface as expired window | warning |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/docs/module.md` §2(i) + Template §4.7 — Deployment source
- DORA four key metrics (deployment frequency / lead time / change failure rate / failed-deployment recovery time)
- OWASP Top 10:2025 — A08 Software & Data Integrity Failures (drives DEP-012)
