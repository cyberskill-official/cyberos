# `knowledge_taxonomy_rubric@1.0` — machine-checkable Knowledge Taxonomy rubric

> Per Session-B Tier-2 rebuild (2026-05-17). Sourced from C-Suite Reference §5 + the matching persona's §5/§7/§8 blocks.

## §1  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable |
| ------- | ----- | -------- | ------------ |
| `FM-001` | YAML parses; closing `---` present | error | false |
| `FM-002` | All keys are `snake_case` | error | true |
| `FM-003` | No duplicate keys | error | false |
| `FM-004` | `template` equals `knowledge-taxonomy@1` | error | true |

## §2  Frontmatter — per-field

| rule_id | Field | Rule | Severity | Auto-fixable |
| ------- | ----- | ---- | -------- | ------------ |
| `FM-101` | `title` | required, string, length 1-160 chars after trimming | error | skeleton |
| `FM-102` | `author` | required, matches `^@[A-Za-z0-9_.-]{1,38}$` | error | false |
| `FM-103` | `provenance.source_path` | required, path exists, readable | error | false |
| `FM-104` | `provenance.source_hash` | required, matches `^[0-9a-f]{64}$` | error | false |

(Skill-specific per-field rules in v1.1.)

## §3  Always-required sections

Per the `contracts/knowledge-taxonomy/template.md` H2 list — every H2 in template MUST be present + non-empty in the authored artefact.

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SEC-001` | Every H2 declared in `contracts/knowledge-taxonomy/template.md` is present in the authored artefact | error |
| `SEC-002` | Each required H2 is non-empty (≥1 non-blank line of body) | error |
| `SEC-003` | Section ordering matches the template | warning |
| `SEC-004` | Heading hierarchy well-formed (no H2→H4 jumps; one H1) | warning |

## §4  Conditionally-required sections

Per the template's `<!-- comment-blocks -->` for conditional sections. Each comment-block declares the trigger; the audit fires `COND-NNN` when the trigger is true but the section is absent.

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `COND-001` | When trigger condition fires per template metadata, the conditional section MUST be present | error |

## §5  Quality heuristics

| rule_id | Anti-pattern | Detection | Severity |
| ------- | ------------ | --------- | -------- |
| `QA-CITE-001` | Claim without `source_ref` | Non-boilerplate paragraph lacks `source_ref` | error |
| `QA-AUTH-001` | Paragraph without `authority` marker | Non-boilerplate paragraph lacks `authority:` | error |
| `QA-NUM-001` | Unsourced numeric target | Any numeric goal lacks a citable source | error → needs_human (`success_metric_targets`) |
| `QA-VAGUE-001` | Vague placeholder | Body contains `TBD` / `TODO` / `<placeholder>` from the template | warning |
| `QA-OWNER-001` | Action item without owner | Any actionable row lacks `owner:` | error |
| `QA-DUE-001` | Action item without due date | Any actionable row lacks `due:` | error |
| `QA-TODO` | Skeleton TODO marker remaining | literal `TODO:` from a stub | warning |
| `QA-QUOTE-001` | Quote outside `<untrusted_content>` | Quoted passage not wrapped | warning |

## §6  Untrusted-content safety

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `SAFE-001` | Nested `<untrusted_content>` blocks | error |
| `SAFE-002` | Unclosed `<untrusted_content>` at EOF | error |
| `SAFE-003` | Injection-marker scan | warning (error if ≥3 matches) |
| `SAFE-004` | Second-person commands outside `<untrusted_content>` | warning |

## §7  Cross-skill rules

| rule_id | Check | Severity |
| ------- | ----- | -------- |
| `XCHAIN-001` | `provenance.source_path` matches author manifest | warning |
| `XCHAIN-002` | `provenance.source_hash` matches at write time | error |

## §8  Staleness

| rule_id | Trigger | Action | Severity |
| ------- | ------- | ------ | -------- |
| `STALE-001` | Source artefact hash differs from `provenance.source_hash` | Reset open + needs_human issues to open; re-evaluate | warning → needs_human (`stale_artefact_disposition`) |

---

## Cross-references

- `cyberos/skill/docs/AUDIT_LOOP.md`, `cyberos/skill/docs/RUBRIC_FORMAT.md`
- `cyberos/skill/contracts/knowledge-taxonomy/CONTRACT.md` + `template.md`
- `../../../modules/cuo/docs/module.md` §4 — the source role profile (persona catalog).

## Skill-specific rules (extend in v1.1)

The above is the Tier-2 Session-B compact rubric. Skill-specific per-field rules (FM-105+), conditional triggers (COND-NNN), and quality heuristics (QA-NNN) are added in v1.1 via fine-tunes per `cyberos/skill/docs/FINE_TUNE.md` discipline + per-skill `FINE_TUNE.md` if shipped.
