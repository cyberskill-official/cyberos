# `release_notes_rubric@1.0` — machine-checkable Release Notes rubric

> Sourced from `../../../modules/cuo/docs/module.md` §2(i) release management; Keep-a-Changelog 1.1.0; SemVer 2.0.0. Rubric version `1.0` is locked.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `release-notes@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string | error | skeleton |
| `FM-102` | `release_id` | required, SemVer or release tag | error | false |
| `FM-103` | `release_date` | required, ISO 8601 | error | true |
| `FM-104` | `prior_release_id` | required, the previous release tag (so diff scope is known) | error | false |
| `FM-105` | `audience` | required, one of: customer_public, customer_enterprise, internal_only, partner | error | false |
| `FM-106` | `provenance.source_path`, `provenance.source_hash` | required (typically the CHANGELOG diff range or commit-list bundle) | error | false |
| `FM-107` | `breaking` | required, boolean (true if this release contains any breaking change) | error | false |
| `FM-108` | `security_advisories` | optional, array of `{cve_id, severity, summary, mitigation}` | error (each entry must validate per Schema) | false |
| `FM-109` | `compat_target_version` | optional; if `breaking: true`, MUST specify the minimum-supported prior version that can upgrade directly | error | false |

## §3  Always-required sections (Keep-a-Changelog format)

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## Highlights` (2-5 bullet summary at the top — what most customers care about) | error |
| `SEC-002` | `## Added` (new features) | warning (required if any) |
| `SEC-003` | `## Changed` (non-breaking changes to existing behaviour) | warning (required if any) |
| `SEC-004` | `## Deprecated` (about-to-be-removed features) | warning (required if any) |
| `SEC-005` | `## Removed` (now removed features) | error (required if any) |
| `SEC-006` | `## Fixed` (bug fixes) | warning (required if any) |
| `SEC-007` | `## Security` (security fixes — required if any CVE was patched) | error (required if any) |
| `SEC-008` | `## Upgrade Notes` (steps customers must take; mandatory when `breaking: true`) | error (required when `breaking: true`) |
| `SEC-009` | `## Known Issues` (carried-over or newly-discovered) | warning |
| `SEC-901` | Section ordering matches Keep-a-Changelog (Highlights → Added → Changed → Deprecated → Removed → Fixed → Security → Upgrade Notes → Known Issues) | warning |
| `SEC-902` | Each present section is non-empty | error |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | `breaking: true` | `## Upgrade Notes` populated AND `## Migration Guide` with code examples | error |
| `COND-002` | Any `security_advisories` entry | `## Security` section enumerates each CVE with severity + mitigation | error |
| `COND-003` | `audience: customer_public` | `## Acknowledgements` (community contributors, security reporters) | warning |
| `COND-004` | `audience: customer_enterprise` | `## Compliance Notes` (any change relevant to SOC 2 / ISO 27001 / GDPR / Vietnam Decree 13/2023 PDPD) | warning |
| `COND-005` | Release contains AI-model update | `## AI Model Update Notes` (model card delta, behaviour changes, eval-result summary) | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | non-boilerplate paragraph lacks `authority:` | error |
| `QA-BREAK-001` | Breaking change in §Changed instead of explicit upgrade notes | A bullet in §Changed contains "breaking", "incompatible", "removed", "renamed" | error → needs_human (`scope_decomposition`) |
| `QA-CVE-001` | Security advisory uses fabricated CVE | A CVE-YYYY-NNNN in §Security or `security_advisories` doesn't match NVD/MITRE format pattern AND has no `source_ref` | error |
| `QA-CVE-002` | Security advisory without severity | A `security_advisories` entry missing `severity:` (low/medium/high/critical) | error |
| `QA-JARGON-001` | Customer-facing audience receives engineering jargon | `audience: customer_public` AND body contains: webhook, schema migration, RBAC, JWT, raw HTTP verbs, regex, latency p99, kubernetes, dashmap (engineering jargon list per task audit QA-009) | warning |
| `QA-SEMVER-001` | Release ID and breaking flag inconsistent | `release_id` differs from `prior_release_id` only at patch level but `breaking: true` (or vice-versa) | error → needs_human |
| `QA-COMPAT-001` | Breaking but no `compat_target_version` | error |
| `QA-EMPTY-001` | Highlights empty or generic | §Highlights contains "various improvements" / "bug fixes" only with no specific item | warning |
| `QA-DATE-001` | `release_date` in the future | warning (acceptable for pre-announce; flag for verification) |
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
| `XCHAIN-003` | If a deploy-checklist exists for this release, it references this artefact at DEP-002 | warning |
| `XCHAIN-004` | Every merged task since `prior_release_id` is represented in at least one of §Added / §Changed / §Fixed (or has `release_notes_excluded: true` in its task frontmatter) | warning |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | New merged PR since `release_date` AND `release_date` ≥ now() | Reset open + needs_human; re-summarise diff | warning → needs_human |
| `STALE-002` | `prior_release_id` resolves to a tag that no longer exists | error |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `../../../modules/cuo/docs/module.md` §2(i) — Release management source
- Keep-a-Changelog 1.1.0
- SemVer 2.0.0
- NVD / MITRE CVE — security-advisory format
