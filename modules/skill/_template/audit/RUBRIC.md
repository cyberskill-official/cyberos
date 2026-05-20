# `<artifact>_rubric@1.0` — machine-checkable audit rubric

> Sourced from `cyberos/skill/contracts/<artifact>/CONTRACT.md` (the artefact contract) and `../../../../modules/cuo/README.md` (the SDP stage definition). Rubric version `1.0` is locked; bumping requires a coordinated update of the contract body and this skill's CONTRACT_ECHO. Each rule has a stable `rule_id`. Rule IDs MUST appear verbatim in the audit report so reports are diffable across iterations and operators.

This file is a template. When copying to a real `<artifact>-audit/RUBRIC.md`:

1. Replace `<ARTIFACT>` and `<artifact>` placeholders throughout.
2. Add rules to each family that are specific to the artefact's contract.
3. Add skill-specific families if the artefact has domain-specific concerns (e.g. `IEEE-NNN` for `software-requirements-specification-audit`, `OWASP-NNN` for `threat-model-audit`).
4. Document the auto-fixable flag for every rule.

---

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false |
| `FM-002` | All keys are `snake_case` (lowercase ASCII letters, digits, underscores; no leading digit) | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` key present and equals `<artifact>@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1–<MAX> chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `created_at` | required, ISO 8601 with timezone | error | true |
| `FM-104` | `version` | required, SemVer `^\d+\.\d+\.\d+(-[A-Za-z0-9.-]+)?$` | error | true |
| `FM-105` | `provenance.source_path` | required, exists, readable | error | false |
| `FM-106` | `provenance.source_hash` | required, matches `^[0-9a-f]{64}$` | error | false |
| `FM-107` | (skill-specific field) | (skill-specific rule) | error | varies |

## §3  Always-required sections

| rule_id | Heading | Severity |
| ------- | ------- | -------- |
| `SEC-001` | `## Summary` | error |
| `SEC-002` | `## <Artefact-specific section>` | error |
| `SEC-XXX` | (skill-specific) | error |
| `SEC-998` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-999` | Heading hierarchy well-formed (no H2→H4 jumps; one or zero H1s) | warning |

## §4  Conditionally-required sections

| rule_id | Trigger | Required | Severity |
| ------- | ------- | -------- | -------- |
| `COND-001` | (skill-specific trigger) | (skill-specific required section) | error |
| `COND-002` | (skill-specific trigger) | (skill-specific required section) | error |

## §5  Quality heuristics (anti-patterns)

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-001` | (skill-specific anti-pattern) | (skill-specific detection) | warning → needs_human |
| `QA-CITE-001` | Claim without `source_ref` | Any paragraph in a non-boilerplate section lacks a `source_ref` marker | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | Any paragraph in a non-boilerplate section lacks an `authority` marker | error |
| `QA-NUM-001` | Unsourced numeric target | Metric uses a target value not derivable from inputs | error → needs_human (`success_metric_targets`) |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | Quoted passage not wrapped | warning |
| `QA-TODO` | Skeleton TODO marker remaining | Body contains literal `TODO:` from a skeleton stub | warning (open until human resolves) |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | `<untrusted_content>` blocks not nested | error |
| `SAFE-002` | No unclosed `<untrusted_content>` block at EOF | error |
| `SAFE-003` | Interior of `<untrusted_content>` scanned for prompt-injection markers (per `references/UNTRUSTED_CONTENT.md` §3) | warning (error if ≥3 matches) |
| `SAFE-004` | Quote outside `<untrusted_content>` contains second-person commands targeting auditor | warning |

## §7  Cross-skill rules (when chained from `<artifact>-author`)

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | The artefact's `provenance.source_path` matches the author's manifest's `source_files[].path` | warning |
| `XCHAIN-002` | The artefact's `provenance.source_hash` matches the author's manifest's `source_files[].hash` at write time | error |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source artefact hash differs from `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate. Surface diff to operator. | warning → needs_human (`stale_artefact_disposition`) |

---

## Rule auto-fix behaviour catalogue

| auto-fixable value | Audit behaviour |
| ------------------ | --------------- |
| `true` | Minimal textual change; mark `fixed`. |
| `false` | Leave `open` or mark `needs_human` per severity. |
| `skeleton` | Insert TODO marker; mark `open` with `todo_inserted: true`. |

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md` — the 8-step algorithm that walks this rubric.
- `cyberos/skill/docs/RUBRIC_FORMAT.md` — the rubric column format every audit skill follows.
- `REPORT_FORMAT.md` (sibling file) — the on-disk shape of `.audit.md` reports.
- `INVARIANTS.md` (sibling file) — invariant catalog that includes the `deterministic_drift` check.
