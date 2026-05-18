# `skill_bundle_rubric@1.0` — machine-checkable SKILL.md rubric

> **What this rubric audits:** the `SKILL.md` bundle (every skill folder under `modules/skill/`), NOT the artefacts the skill produces. For artefact-level rubrics see `feature-request-audit/RUBRIC.md` (`audit_rubric@2.0` — FR documents), `product-requirements-document-audit/RUBRIC.md` (PRDs), etc.
>
> **Source FRs:** FR-SKILL-103 (frontmatter v1 schema + fields), FR-SKILL-111 (description trigger enrichment), FR-SKILL-112 (TRIGGER_TESTS.md), FR-SKILL-113 (XML-free frontmatter), FR-SKILL-114 (BASELINE.md at v1.0+). This rubric is consumed by future skill-bundle-audit skills + the Rust broker (FR-SKILL-103) + the Python `cuo.baseline` + `cuo.trigger_tests` validators.
>
> **Version:** locked at `skill_bundle_rubric@1.0` post-2026-05-19. Rule IDs use the `SKB-` prefix to avoid namespace collision with the FR rubric's `FM-`.

The key words **MUST**, **MUST NOT**, **REQUIRED**, **SHOULD**, **MAY** in this document are to be interpreted as described in BCP 14 (RFC 2119, RFC 8174) when in all capitals.

---

## §1  Identity rules

| rule_id | Check | Severity | Auto-fixable |
|---|---|---|---|
| `SKB-001` | Skill folder is `kebab-case` (`^[a-z][a-z0-9-]*$`, no underscores, no spaces, no capitals) | error | false |
| `SKB-002` | `SKILL.md` filename is exactly `SKILL.md` (case-sensitive) | error | false |
| `SKB-003` | Folder contains no `README.md` (Anthropic guide Chapter 2 p. 10 — repo-level READMEs are fine, in-folder READMEs are not) | error | true |

## §2  Frontmatter — structural

| rule_id | Check | Severity | Auto-fixable | Source |
|---|---|---|---|---|
| `SKB-010` | File begins with `---` on line 1; closing `---` exists; YAML between fences parses | error | false | FR-SKILL-103 §1 #1 |
| `SKB-011` | All keys are `snake_case` (lowercase ASCII letters, digits, underscores; no leading digit) | error | true | FR-SKILL-103 §1 #2 |
| `SKB-012` | No duplicate keys | error | false | FR-SKILL-103 §1 #2 |
| `SKB-013` | Unknown fields are rejected UNLESS prefixed with `x-` (forward-compat escape hatch) | error | false | FR-SKILL-103 §1 #5 |

## §2.5  Frontmatter — placeholder-free (FR-SKILL-115)

| rule_id | Check | Severity | Auto-fixable |
|---|---|---|---|
| `SKB-030` | **placeholder-free-frontmatter** — no frontmatter field may contain literal template-scaffold placeholder syntax like `<SDP §2 stage letter or "cross">`, `<artifact>`, `<input>`, `<fr_id>` etc. (i.e. tokens matching `<[a-zA-Z]...>` that are NOT in the SAFE_TAGS list `{br, b, i, em, strong, sub, sup, span, div}`). | error (accepted+); warning (draft); EXEMPT under `_template/` | false |

Auto-fix is never enabled — substitution requires operator domain knowledge (the right substitute depends on what the skill actually does). The validator detects + reports + suggests via `tools/sweep-placeholders/suggest.py` but never auto-applies. Per FR-SKILL-115 §1 #4.

Distinct from SKB-040 (the security boundary catch-all): SKB-030 targets the operator-UX + portability boundary — template-scaffold leftovers that never got substituted with real values. SKB-040 fires on any `<` or `>` in any frontmatter field including the placeholder tokens SKB-030 also catches. The two rules overlap defensively.

## §3  Frontmatter — description format (FR-SKILL-111)

| rule_id | Field | Rule | Severity | Auto-fixable |
|---|---|---|---|---|
| `SKB-020` | `description` | required, length 80–1024 chars (flattened single-line equivalent after YAML folding) | error | false |
| `SKB-021` | `description` | MUST NOT contain unescaped `<` or `>` characters (forbidden frontmatter chars per Anthropic Reference B + FR-SKILL-113 SKB-040) | error | false |
| `SKB-022` | `description` | MUST contain at least one verb stem from the canonical list (`generate \| author \| audit \| review \| draft \| emit \| build \| propose \| render \| extract \| classify \| tag \| score \| track \| enforce \| validate \| orchestrate \| chain \| select \| pin \| halt \| resume \| escalate \| wrap \| publish \| deliver \| test \| simulate`) | error | false |
| `SKB-023` | `description` | MUST carry ≥2 distinct trigger-phrase quotations (form: `Use when user asks to "<phrase>"` or `Triggers on "<phrase>"`) | error | false |
| `SKB-024` | `description` | Trigger phrases MUST be paraphrase-distinct (Levenshtein distance > 3 between any pair of positives) | warning | false |
| `SKB-025` | `description` | MAY carry negative triggers (`Do NOT use for "<phrase>"`) — these don't count toward the ≥2 positive floor | info | false |
| `SKB-026` | `description` | SHOULD mention file types or input artefacts by canonical name when the skill consumes a specific format | warning | false |
| `SKB-027` | `description` | SHOULD name the skill's principal output artefact when `produces.output_kind: artefact` | info | false |

Severity scheme for SKB-020..023: `error` for skills at `status: accepted` or higher; `warning` for `status: draft`. **Auto-fix is never enabled** for description rules — description reflects human intent.

## §4  Frontmatter — XML-bracket discipline (FR-SKILL-113)

| rule_id | Check | Severity | Auto-fixable |
|---|---|---|---|
| `SKB-040` | **no-xml-in-frontmatter** — no unescaped `<` or `>` in ANY frontmatter value (catch-all defence-in-depth; Anthropic Reference B p. 31 security boundary) | error (all status levels) | false |
| `SKB-041` | **wrap_in_marker-form** — `untrusted_inputs.wrap_in_marker:` MUST be present (renamed from legacy `wrap_in:` in registry v0.2.5; SKB-041 catches both: missing-field and legacy-form) | error (accepted+); warning (draft) | true (legacy → new rename only) |
| `SKB-042` | **wrap_in_marker enum** — value MUST match `^[a-z][a-z0-9_]*$` and MUST be one of the registered v1 markers (today: `"untrusted_content"` only) | error | false |

## §5  Triggering tests (FR-SKILL-112)

| rule_id | Check | Severity | Auto-fixable |
|---|---|---|---|
| `SKB-050` | **trigger-tests-present** — `acceptance/TRIGGER_TESTS.md` exists | error (accepted+); warning (draft) | false |
| `SKB-051` | Fixture frontmatter declares `skill_id`, `min_confidence`, `classifier_version` | error | false |
| `SKB-052` | Fixture contains `## Positive triggers (MUST route here)` section with ≥3 bulleted phrases | error | false |
| `SKB-053` | Fixture contains `## Negative triggers (MUST NOT route here)` section with ≥3 bulleted phrases | error | false |
| `SKB-054` | Positive triggers are paraphrase-distinct (Levenshtein > 3 between any pair) | error | false |
| `SKB-055` | Negative triggers carry inline `→ <target-skill>` or `→ none` annotation | warning | false |
| `SKB-056` | `min_confidence ≥ confidence_band.defer_below` (cannot test against a confidence the skill itself would reject) | error | false |
| `SKB-057` | Classifier routing matches fixture (positive triggers route here; negative triggers route elsewhere) — verified by `python -m cuo.trigger_tests <skill_path>` | error | false |

## §6  Baseline at v1.0 promotion (FR-SKILL-114)

| rule_id | Check | Severity | Auto-fixable |
|---|---|---|---|
| `SKB-060` | **baseline-present-at-v1** — when `skill_version >= 1.0.0`, `BASELINE.md` MUST exist as sibling of `SKILL.md` | error (v1.0+); info (v0.x — advisory) | false |
| `SKB-061` | `BASELINE.md` frontmatter has all required keys (`skill_id`, `baseline_version`, `baseline_measured_at`, `attested_by`, `next_review_due`) | error | false |
| `SKB-062` | All 6 body sections present (`## Workflow under test`, `## Without-skill baseline`, `## With-skill measurements`, `## Token-budget transparency`, `## Trust calibration`, `## Authoring notes`) | error | false |
| `SKB-063` | `attested_by` matches `^(cuo-[a-z]+\|human:[a-z][a-z0-9_-]*)$` | error | false |
| `SKB-064` | `next_review_due` is valid ISO 8601 with timezone | error | false |
| `SKB-065` | `next_review_due` is in the future (warning) or <365 days overdue (warning); >365 days overdue (error) | warning / error | false |
| `SKB-066` | When `exposable_as.partner_connector: true` AND `skill_version >= 1.0.0`, `BASELINE.md` MUST be present (broker enforces; trust-exposability link per FR-SKILL-103 Part 5.3) | error | false |

## §7  Existing v0.2.0 contract rules (referenced for completeness)

These rules ship with FR-SKILL-103 (already accepted; this rubric is the post-103 + post-111-114 consolidation). They're not new; they're listed here so the rubric is self-contained.

| rule_id | Check | Severity | Source FR |
|---|---|---|---|
| `SKB-100` | `name:` matches kebab-case + matches folder name | error | FR-SKILL-103 §1 #2 |
| `SKB-101` | `description:` ≤1024 chars (raised by FR-SKILL-111 from prior 200-char baseline) | error | FR-SKILL-103 §1 #2 + FR-SKILL-111 §1 #2 |
| `SKB-102` | `allowed_brain_scopes:` globs validate via `globset@0.4` | error | FR-SKILL-103 §1 #2 |
| `SKB-103` | `allowed_mcp_tools:` values are in canonical tool enum (Bash, Read, Write, Edit, Glob, Grep, BrainRead, BrainSearch, HttpFetch + MCP names from FR-SKILL-104 registry) | error | FR-SKILL-103 §1 #2 |
| `SKB-104` | `version:` is valid SemVer | error | FR-SKILL-103 §1 #2 |
| `SKB-105` | `signature:` (when present) verifies ed25519 over `SHA-256(frontmatter_yaml_canonical) \|\| SHA-256(body_markdown_canonical)` | error | FR-SKILL-103 §1 #7 |
| `SKB-106` | `min_broker_version` / `max_broker_version` SemVer-compatible with current broker | error | FR-SKILL-103 §1 #3 |

## §8  Severity legend

- **error** — bundle fails to load (broker) or fails audit (auditor). Cannot promote to `status: accepted`.
- **warning** — bundle loads but audit reports issue. Operator review required for promotion.
- **info** — advisory only; surfaces in audit report but doesn't block.

## §9  Auto-fix policy

- **`true`** — auditor applies a mechanical, safe transformation (e.g. SKB-041 legacy-form rename). Verdict: `fixed`.
- **`false`** — auditor never auto-edits the field; verdict is always `needs_human`. Description / trigger phrases / measurement numbers / attestation chains all require human authorship.

## §10  Cross-rubric coordination

This rubric coexists with per-artefact rubrics (FR, PRD, SOW, etc.). When a skill bundle's artefact-rubric and bundle-rubric disagree, the bundle-rubric wins (the bundle is the unit of distribution; artefact rubrics live inside skill bundles).

When `feature-request-audit/RUBRIC.md` (the FR artefact rubric) and `SKILL_BUNDLE_RUBRIC.md` (this file) reference the same FR-SKILL-NNN, the bundle rubric carries the source-of-truth contract. The FR artefact rubric's rules apply only to FR.md documents; they never apply to SKILL.md bundles.

## §11  Roadmap

Pending future FRs:
- **FR-SKILL-117** — marker namespace expansion (SKB-042 enum grows beyond `"untrusted_content"`).
- **FR-SKILL-118** — automated baseline re-measurement at 12-month review-due (SKB-065 escalation paths).
- Future: a skill-bundle-audit skill that consumes this rubric end-to-end (today the rules are enforced piecemeal: SKB-010..013 by the Rust broker, SKB-020..023 by `cuo.description_validator`, SKB-050..057 by `cuo.trigger_tests`, SKB-060..066 by `cuo.baseline`).
