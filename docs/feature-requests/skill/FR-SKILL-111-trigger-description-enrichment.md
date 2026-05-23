---
id: FR-SKILL-111
title: "SKILL.md `description:` field — mandated trigger-phrase enrichment + 1024-char budget for host-portable triggering"
module: SKILL
priority: SHOULD
status: done
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-101, FR-SKILL-103, FR-SKILL-112, FR-SKILL-113]
depends_on: [FR-SKILL-103]
blocks: []

source_pages:
  - modules/skill/README.md#part-2--anatomy-the-33-field-skillmd-contract
  - modules/skill/ANTHROPIC_GUIDE_DIGEST.md#51--confirmed-gaps
  - modules/skill/_template/author/SKILL.md
  - modules/skill/_template/audit/SKILL.md
source_decisions:
  - DEC-091 (host-portability contract — CCSM is source of truth; transpilers emit per-host artefacts)
  - DEC-180 (every .skill bundle MUST declare its memory scopes + tool requirements in frontmatter)
  - DEC-182 (frontmatter schema versioned; v1 frozen at FR-SKILL-103; new validation rules are MINOR bumps that add but don't remove fields)

language: rust 1.81 + yaml + markdown
service: cyberos/services/skill-broker/  (FR-SKILL-103) + modules/skill/feature-request-audit/  (RUBRIC update) + modules/skill/_template/  (template scaffolds)
new_files:
  - services/skill-broker/src/frontmatter/description_validator.rs
  - services/skill-broker/tests/description_validator_test.rs
  - services/skill-broker/tests/fixtures/description-valid/SKILL.md
  - services/skill-broker/tests/fixtures/description-missing-triggers/SKILL.md
  - services/skill-broker/tests/fixtures/description-too-short/SKILL.md
  - services/skill-broker/tests/fixtures/description-too-long/SKILL.md
modified_files:
  - services/skill-broker/src/frontmatter/schema.rs                    # raise max description length 200 → 1024
  - services/skill-broker/src/frontmatter/validators.rs                # call description_validator
  - services/skill-broker/skill.schema.json                            # JSONSchema mirror — description maxLength: 1024
  - modules/skill/_template/author/SKILL.md                            # description block carries trigger phrases per FM-112
  - modules/skill/_template/audit/SKILL.md                             # description block carries trigger phrases per FM-112
  - modules/skill/feature-request-audit/RUBRIC.md                      # add FM-112 (description-format)
  - feature-request-audit skill        # §3.13 mentions description-format rule
  - website docs (SKILL appendices)                                    # Part 2.1 description field row updates; Part 18 anti-pattern entry added
  - website docs (SKILL Appendix J)                                    # §6.1 status badge updates when FR ships
allowed_tools:
  - file_read: modules/skill/**, services/skill-broker/**, docs/feature-requests/skill/**
  - file_write: services/skill-broker/{src,tests,fixtures}/**, modules/skill/{_template,feature-request-audit}, docs/feature-requests/skill/**
  - bash: cd services/skill-broker && cargo test description_validator
  - bash: cd modules/skill && grep -l 'description:' */SKILL.md  | wc -l   # sweep precondition
disallowed_tools:
  - regenerate descriptions for all 104 pairs automatically without operator review (each must be hand-edited because triggers reflect human intent)
  - downgrade max description length below 1024 in v1 schema (DEC-182 — would break newly-conforming skills)
  - delete the body's "## When to invoke this skill" section; it remains the long-form companion to the frontmatter triggers

effort_hours: 12
sub_tasks:
  - "0.5h: schema.rs — raise description maxLength to 1024 (was 200 in FR-SKILL-103 baseline); SemVer-compatible field-loosening"
  - "0.5h: skill.schema.json mirror update"
  - "1.5h: description_validator.rs — Rust validator implementing FM-112 trigger-phrase detection"
  - "1.0h: description_validator_test.rs — happy + 4 negative fixtures + parametric tests for trigger-form variants"
  - "0.5h: 4 fixture SKILL.md files (valid / missing-triggers / too-short / too-long)"
  - "1.0h: _template/author/SKILL.md update — description block carries trigger phrases inline; ## When to invoke section becomes optional explanatory companion"
  - "1.0h: _template/audit/SKILL.md update — analogous changes for auditor template"
  - "1.5h: feature-request-audit/RUBRIC.md — add FM-112 rule (description carries ≥2 distinct trigger-phrase forms; auto-fixable as `needs_human` only — never auto-edit user-facing description)"
  - "0.5h: feature-request-audit skill §3.13 update — add description-format rule and link to FM-112"
  - "1.0h: README.md Part 2.1 frontmatter-table description row update; Part 18 add anti-pattern \"Don't put triggers only in body\""
  - "1.0h: ANTHROPIC_GUIDE_DIGEST.md §6.1 status update — mark FR-SKILL-111 shipped when CI passes"
  - "0.5h: integration test against 3 backfilled exemplar skills (feature-request-author, feature-request-audit, prd-author — confirm new auditor rule fires correctly on un-enriched descriptions)"
  - "1.5h: backfill exemplar enrichment — hand-edit description: blocks of feature-request-author + feature-request-audit + prd-author to carry trigger phrases (validates rule + provides reference patterns for lazy-sweep cohort)"
risk_if_skipped: "Without trigger phrases in the frontmatter description, CyberOS skills under-trigger on every non-CyberOS host (Claude.ai, Codex, Cursor, vanilla MCP) once Phase-B transpilers ship. The host's progressive-disclosure level 1 (frontmatter only) decides whether to load the skill — if the description carries WHAT but not WHEN, the loader returns 'no skill matched' on natural user phrasings the body anticipates. Inside CyberOS the gap is silent because the supervisor reads the body during classify_act; outside CyberOS the skill ships dead. Inventory the failure: a user types 'Turn this PRD into a backlog' on Claude.ai; the host classifier reads feature-request-author's current description (which mentions 'backlog' and 'Feature Request' but not 'Turn this into a backlog' or 'PRD'); skill is not loaded; user falls back to manual prompting. This is exactly the under-triggering pattern Anthropic's guide Chapter 5 lists as the #1 reason skills fail in the wild. Without FR-SKILL-111, every skill we ship to a non-CyberOS host inherits the failure. With FR-SKILL-111, the auditor rule (FM-112) forces every production skill to carry ≥2 trigger-phrase forms in the description; the description budget rises from 200 to 1024 chars to fit; lazy backfill spreads the fix over the next month's natural fine-tune cycle. Cost of the FR ≈ 12 hours; cost of NOT shipping it once Phase-B is live ≈ 104 silent skill failures plus a future emergency sweep when the first partner connector ships and the partner complains."
---

## §1 — Description (BCP-14 normative)

This FR establishes the rules for the `description:` field of every `SKILL.md` frontmatter so that Anthropic-style hosts (and any host whose progressive-disclosure level 1 reads only frontmatter) can correctly route user requests to CyberOS skills.

1. The `description:` field **MUST** carry three structural elements in any order: **(a) WHAT** the skill does (one verb phrase, present tense, third-person), **(b) WHEN** the host should route to it (≥2 distinct trigger-phrase forms quoted in the description prose; trigger phrases mirror natural-language fragments a user might type), and **(c) KEY VALUE** (what the user gets — one outcome phrase). Order is recommended `WHAT + WHEN + KEY VALUE` but not enforced.
2. The `description:` field's character length **MUST** be **≥ 80 chars** AND **≤ 1024 chars** (raised from FR-SKILL-103's baseline of 200; aligns with the Anthropic guide's published cap per Reference B p. 31). Below 80 chars cannot carry both WHAT and ≥2 trigger phrases; above 1024 violates the host system-prompt budget.
3. Trigger phrases **MUST** be quoted in standard double-quotes (`"..."`) inside the description prose. Two acceptable forms:
   - **Use-when form** — `Use when user asks to "<verb phrase>"` or `Use when user mentions "<noun>"`. Example: `Use when user asks to "audit this FR" or "check the rubric"`.
   - **Triggers-on form** — `Triggers on "<phrase>" or "<phrase>"`. Example: `Triggers on "draft a PRD" or "outline the requirements"`.
4. The description **MUST NOT** contain XML angle brackets `<` or `>`. (Restates FR-SKILL-103 §1 #2 and FR-SKILL-113 sketch; included here so FM-112 is self-contained for the auditor — the validator runs the bracket check before the trigger-phrase check so the user gets the more diagnostic error first.)
5. The description **MAY** contain negative triggers in the form `Do NOT use for "<phrase>" (use <other-skill> instead)`. When present, the validator increments the trigger-phrase count for the positive triggers but not the negative ones (negatives are disambiguators, not triggers).
6. The description **MUST** reference file types or input artefacts by their canonical name when the skill consumes a specific format. Examples: `PRD/spec/SRS documents`, `.fig files`, `Postman collections`. This mirrors the Anthropic guide Chapter 2 p. 10 "Mention file types if relevant".
7. The description **MUST NOT** be a single trailing word like `Helps with X.` or `Manages Y.` — these were the canonical bad examples in the Anthropic guide p. 12. The validator rejects any description that fails the WHAT-detection regex (`\b(generate|author|audit|review|draft|emit|build|propose|render|extract|classify|tag|score|track|enforce|validate|orchestrate|chain|select|pin|halt|resume|escalate|wrap|publish|deliver|test|simulate)\b` — verb stems that indicate concrete action).
8. The description **SHOULD** name the skill's principal output artefact when the skill is artefact-producing (i.e. `produces.output_kind: artefact`). Example: `Generates a versioned feature_request@1 markdown`. This anchors trust because the user can verify the artefact appears in the output.
9. The description **MAY** be a multi-line YAML block (using `|` or `>-` folding) for readability. The 80-1024 char budget applies to the **flattened single-line equivalent** (after YAML folding resolves whitespace).
10. Description **MUST** be locale-default English. Localised variants for VN-locale skills live in `description_localized.<lang>` under `metadata:` (out of scope for this FR; documented as future v0.3.0 work in §9).
11. The auditor rule **MUST** be `FM-112 description-format` with severity `error` for production skills (`status: accepted` or higher); `severity: warning` for `status: draft` skills (so authors get feedback during authoring but the rule doesn't block draft work).
12. The auditor rule **MUST NOT** auto-fix the description. Description text reflects human intent; auto-edit would silently rewrite trigger phrases the author chose deliberately. Verdict on description issues is always `needs_human` per `_template/audit/REPORT_FORMAT.md`.
13. The CLI `cyberos skill validate` (FR-SKILL-103 #10) **MUST** report description-format violations with a structured error message naming the missing element (`missing_what` | `missing_triggers` | `missing_value_phrase` | `too_short` | `too_long` | `forbidden_brackets`). The host shim per Part 9 of `modules/skill/README.md` MUST surface the message identically across hosts.
14. Existing skills that pre-date this FR **MUST** be backfilled lazily — the rule fires at `status: accepted` or higher; scaffold/`status: draft` skills get a grace window. The auditor rule fires only on fields that were touched by a fine-tune commit after this FR ships (per `human_fine_tune.signals_to_initiate` — fine-tune session brings the skill into compliance as a side effect).
15. The `## When to invoke this skill` body section **MUST NOT** be deleted; it remains the long-form companion to the frontmatter trigger phrases. The body section serves three purposes the description cannot: (a) negative triggers with prose explanations, (b) disambiguation cross-links to sibling skills, (c) handoff guidance to other personas. The frontmatter triggers are the host's level-1 hook; the body section is level-2 detail. **Both must be kept in sync** — when the description's triggers are updated, the body section MUST be re-audited for consistency (auditor rule SEC-007 enforces).

## §2 — Why this design (rationale for humans)

**Why mandate trigger phrases in description rather than relying on the body (§1 #1)?** Anthropic's progressive-disclosure model (guide Chapter 1 p. 5) loads frontmatter into every system prompt but reads the body only after a skill is matched. The host's relevance classifier sees the description and nothing else. If trigger phrases live only in the body, the classifier never sees them; the skill never matches; the body never loads. Inside CyberOS the supervisor's `classify_act` node reads the body (because we control both ends), but outside CyberOS — where Phase-B transpilers will ship plugins to Claude.ai, Codex, Cursor, etc. — the host follows Anthropic's contract. Trigger phrases MUST be in the description for the contract to hold.

**Why raise the budget from 200 to 1024 chars (§1 #2)?** The 200-char limit in FR-SKILL-103 baseline was inherited from an earlier draft that prioritised terseness. Real CyberOS skill descriptions already exceed 200 chars (e.g. `feature-request-author/SKILL.md` line 4-9 is ~470 chars in single-line equivalent). The Anthropic guide explicitly permits 1024 (Reference B p. 31). Raising the cap is a SemVer-compatible field-loosening — existing descriptions stay valid; the new cap admits enriched descriptions without forcing a v2 frontmatter schema. The 80-char floor is empirical: below 80 chars cannot fit WHAT (≥20 chars), 2 trigger phrases (≥30 chars combined), and a value phrase (≥20 chars) with quotes and separators.

**Why two trigger-phrase forms rather than one (§1 #3)?** Single-trigger descriptions over-fit one user phrasing. Two distinct forms increase classifier robustness — if the user says "draft a PRD" and only `"author a PRD"` is registered, the synonym mismatch breaks routing. Two forms cover the common paraphrase delta. More than two is welcome but not required. The Anthropic guide examples (p. 11) all carry 2-4 forms, validating the two-floor as the right minimum.

**Why both `Use when` and `Triggers on` forms (§1 #3)?** Different skill authors prefer different prose styles; both forms appear in the Anthropic guide examples. The validator accepts either. What matters is that the trigger phrases are *quoted* — the validator's regex looks for `"<phrase>"` to detect triggers; unquoted natural-language sentences pass freely without being recognised as triggers.

**Why explicit XML-bracket rejection (§1 #4)?** The Anthropic guide Reference B p. 31 forbids `<` and `>` in frontmatter for system-prompt injection reasons. Even though FR-SKILL-103 already validates this at the frontmatter level, FM-112's own check provides the more diagnostic error ("found `<` in description text" vs the generic FR-SKILL-103 "InvalidDescription") — operator UX matters.

**Why allow negative triggers (§1 #5)?** The Anthropic guide Chapter 5 p. 25 shows the disambiguation pattern: `Do NOT use for simple data exploration (use data-viz skill instead)`. CyberOS body sections already do this in `## When to invoke this skill`; moving the negative-trigger sentence into the description gives the host classifier the same disambiguation power. Counting them separately prevents the validator from double-counting (a negative trigger isn't a positive trigger).

**Why require file-type references (§1 #6)?** Anthropic guide Chapter 2 p. 10 calls this out specifically: "Mention file types if relevant". A skill that reads .fig files should say `.fig files`; a skill that reads PRDs should say `PRD/spec/SRS documents`. The classifier weights file-extension matches heavily — without them, the skill doesn't trigger on prompts like "I have a Figma file...".

**Why reject single-word-action descriptions (§1 #7)?** The Anthropic guide p. 12 lists "Helps with projects" as the canonical bad description. The verb-stem regex catches the same anti-pattern: descriptions that don't name an action ("Helps with X", "Manages Y", "Implements Z") never match concrete user requests. The verb list is short (29 verb stems) and represents the actual surface of CyberOS skill actions.

**Why name the output artefact (§1 #8)?** Anchors trust. A user reading the description sees what they'll get (`feature_request@1 markdown`); the audit row's `payload_hash` (per `audit:` block) verifies the artefact exists; the round-trip is closed. Without the artefact name, the user can't tell what to expect, and the audit row carries less semantic meaning.

**Why warning on draft, error on accepted (§1 #11)?** Drafting is exploratory; forcing the trigger-phrase format on every draft commit would slow first-pass authoring. Production skills (`status: accepted` or higher) are routed by the supervisor and shipped to hosts — they MUST conform. The two-severity scheme lets authors iterate while keeping production hygienic.

**Why no auto-fix (§1 #12)?** Trigger phrases reflect human intent. An auto-edit might rewrite `"audit this FR"` as `"check the FR"` and silently miss the user-visible verb the author chose. Description issues always require human review.

**Why the body `## When to invoke this skill` stays (§1 #15)?** Three irreducible functions: (a) negative triggers with prose explanations (e.g. "If the user wants `<other skill>`, route to `<other skill>` instead"); (b) disambiguation cross-links to sibling skills that the description's character budget can't accommodate; (c) handoff guidance describing chains. Frontmatter is for the host classifier; body is for the supervisor + human reader. Keeping both keeps the cross-host port honest while preserving the rich-body audit surface.

## §3 — API contract

### Rust types (added to `services/skill-broker/src/frontmatter/schema.rs`)

```rust
// Raised from prior 200-char cap. SemVer-compatible loosen.
pub const DESCRIPTION_MIN_LEN: usize = 80;
pub const DESCRIPTION_MAX_LEN: usize = 1024;

// Inside SkillFrontmatter (FR-SKILL-103 §3):
//   pub description: String,    // unchanged shape — just new validator
```

### Rust validator (`services/skill-broker/src/frontmatter/description_validator.rs`)

```rust
use crate::frontmatter::FrontmatterError;
use regex::Regex;
use once_cell::sync::Lazy;

/// Verb stems that indicate concrete action. Conservative list — operator
/// extends as needed via PR + RUBRIC update.
static VERB_STEMS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(generate|author|audit|review|draft|emit|build|propose|render|extract|classify|tag|score|track|enforce|validate|orchestrate|chain|select|pin|halt|resume|escalate|wrap|publish|deliver|test|simulate)\b")
        .unwrap()
});

/// Quoted trigger phrase: `"..."` with non-empty body.
static QUOTED_TRIGGER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#""([^"]{1,80})""#).unwrap()
});

/// Negative-trigger preamble: matches "Do NOT use for ..." prefix.
static NEGATIVE_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\bdo\s+not\s+use\s+(for|when|with)\b").unwrap()
});

#[derive(Debug, PartialEq)]
pub enum DescriptionViolation {
    TooShort { len: usize },
    TooLong  { len: usize },
    ForbiddenBrackets,
    MissingWhat,
    InsufficientTriggers { found: usize, needed: usize },
}

pub fn validate(description: &str) -> Result<(), DescriptionViolation> {
    // 0. Flatten YAML-folded multi-line into single line for measurement.
    let flat = description.replace('\n', " ").trim().to_string();
    let len  = flat.chars().count();
    if len < DESCRIPTION_MIN_LEN { return Err(DescriptionViolation::TooShort { len }); }
    if len > DESCRIPTION_MAX_LEN { return Err(DescriptionViolation::TooLong  { len }); }

    // 1. Bracket-free (defensive duplicate of FR-SKILL-103 check).
    if flat.contains('<') || flat.contains('>') {
        return Err(DescriptionViolation::ForbiddenBrackets);
    }

    // 2. WHAT detection — at least one verb stem.
    if !VERB_STEMS.is_match(&flat) {
        return Err(DescriptionViolation::MissingWhat);
    }

    // 3. Trigger phrases — count quoted phrases, excluding those preceded
    //    by "Do NOT use for" (negative triggers).
    let mut positive_triggers = 0usize;
    for m in QUOTED_TRIGGER.find_iter(&flat) {
        let preceding = &flat[..m.start()];
        // If a negative-prefix appears in the 40 chars preceding the
        // quoted phrase, treat it as a negative trigger (don't count).
        let window_start = preceding.len().saturating_sub(40);
        let window       = &preceding[window_start..];
        if NEGATIVE_PREFIX.is_match(window) { continue; }
        positive_triggers += 1;
    }
    if positive_triggers < 2 {
        return Err(DescriptionViolation::InsufficientTriggers {
            found: positive_triggers, needed: 2,
        });
    }
    Ok(())
}

impl From<DescriptionViolation> for FrontmatterError {
    fn from(v: DescriptionViolation) -> Self {
        FrontmatterError::InvalidDescription(format!("{v:?}"))
    }
}
```

### JSONSchema mirror (`services/skill-broker/skill.schema.json` — diff)

```diff
   "description": {
     "type": "string",
-    "maxLength": 200
+    "minLength": 80,
+    "maxLength": 1024,
+    "pattern": "(?s)^(?!.*[<>]).*$"
   }
```

(JSONSchema can express min/max length and the bracket-absence guard via pattern; the verb-stem + trigger-count checks live in the Rust validator only — JSONSchema regex can't express "≥2 quoted substrings excluding those after a Do-NOT preamble" cleanly. The mirror documents this in a `$comment` field.)

### Auditor rule (added to `modules/skill/feature-request-audit/RUBRIC.md`)

```markdown
### FM-112 — description-format

**Statement:** SKILL.md frontmatter `description:` field MUST carry WHAT (verb-stem action) + WHEN (≥2 quoted trigger phrases) + KEY VALUE; 80-1024 chars flattened; no XML brackets; per FR-SKILL-111 §1.

**Severity:** error on `status: accepted | building | shipped`; warning on `status: draft`.

**Auto-fix:** never (description reflects human intent — verdict `needs_human`).

**Check (deterministic):** invoke `cyberos skill validate <bundle>`; if exit code 6 with `validation_outcome: description-format`, the rule fails. Specific sub-codes: `too_short` | `too_long` | `forbidden_brackets` | `missing_what` | `insufficient_triggers`.

**Issue template:**

```
ISSUE
id:              ISS-NNN
rule_id:         FM-112
status:          needs_human
severity:        error|warning
category:        description_format
location:        frontmatter "description:" field
evidence:        "<the description text, truncated to 200 chars>"
description:     "Description fails FM-112: <sub-code>. Detail: <validator output>."
suggestion:      "Rewrite description to include WHAT + WHEN (≥2 quoted triggers like \"<phrase>\") + KEY VALUE. See feature-request-audit skill §3.13 for examples."
auto_fix_applied: false
resolution:      null
```
```

### Updated `_template/author/SKILL.md` description block (diff)

```diff
-description: |
-  Author a <ARTIFACT> markdown from <input artefact(s)>. Generates a
-  versioned <artefact>@1 file under output_dir, with per-claim authority
-  markers and provenance to the source. Chains naturally into
-  <artifact>-audit by default. Refuses to author when upstream artefact
-  is in non-pass state.
+description: >-
+  Generate a versioned <artefact>@1 markdown from one or more <input
+  artefact(s)>. Use when user asks to "draft a <ARTIFACT>" or "turn
+  this <input> into a <ARTIFACT>". Halts at PLAN approval and HITL
+  gates; resumable from manifest.json state. Chains into
+  <artifact>-audit by default. Outputs feature_request@1-style
+  markdowns with per-claim authority markers and source provenance.
```

(The folded scalar `>-` flattens newlines to spaces for the validator's length count.)

### Operator-facing CLI

```bash
# Validate a single skill's description (calls into FR-SKILL-103's broker)
cyberos skill validate modules/skill/feature-request-author/

# Validate every production skill — used by CI gate
cyberos skill validate-all --status-min accepted

# Output (human form):
✓ modules/skill/feature-request-author/         — valid (description: 487 chars, 3 triggers)
✗ modules/skill/closure-author/                 — InvalidDescription: insufficient_triggers (found: 1, needed: 2)
✗ modules/skill/stage-gate-author/              — InvalidDescription: missing_what

# Exit code: 0 if all pass, 6 (SchemaViolation) on any failure.
```

## §4 — Acceptance criteria

1. **Valid description with 2 triggers loads** — fixture `description-valid/SKILL.md` with `description: "Generate a feature_request@1 markdown from PRDs. Use when user asks to \"draft an FR\" or \"turn this PRD into a backlog\". Outputs versioned FR-NNN-slug.md files."` → `description_validator::validate` returns `Ok(())`.
2. **Description too short rejected** — 70-char description → `Err(TooShort { len: 70 })`.
3. **Description too long rejected** — 1100-char description → `Err(TooLong { len: 1100 })`.
4. **XML brackets in description rejected** — description containing `<untrusted>` → `Err(ForbiddenBrackets)`.
5. **Description missing verb stem rejected** — `"Helps with FRs. Use when user says \"FR\" or \"backlog\"."` → `Err(MissingWhat)`.
6. **Description with only 1 trigger rejected** — `"Generate FRs. Use when user asks to \"draft an FR\"."` (single quoted phrase) → `Err(InsufficientTriggers { found: 1, needed: 2 })`.
7. **Description with 1 positive + 1 negative trigger rejected** — `"Generate FRs. Use when user asks to \"draft an FR\". Do NOT use for \"audit existing FRs\"."` → `Err(InsufficientTriggers { found: 1, needed: 2 })`. (Negative trigger doesn't count.)
8. **Description with 2 positive + 1 negative trigger accepts** — `"Generate FRs. Use when user asks to \"draft an FR\" or \"turn this PRD into a backlog\". Do NOT use for \"audit existing FRs\"."` → `Ok(())`.
9. **Folded YAML multi-line description flattened correctly** — `description: >-\n  Line one with first trigger \"phrase one\".\n  Line two with second trigger \"phrase two\".\n  Outputs X.\n` → flattened to single line and validated as if one-line; → `Ok(())`.
10. **JSONSchema mirror agrees on length bounds** — running `ajv validate -s skill.schema.json -d <fixture>` against the same 4 length-test fixtures yields the same accept/reject pattern as the Rust validator.
11. **CLI exit codes** — `cyberos skill validate <valid>` → 0; `cyberos skill validate <missing-triggers>` → 6 (SchemaViolation); `cyberos skill validate <too-short>` → 6.
12. **Auditor rule FM-112 fires on production skill with bad description** — running `feature-request-audit` against an `accepted`-status skill whose description has 1 trigger → audit report contains one `rule_id: FM-112, severity: error, status: needs_human` issue.
13. **Auditor rule FM-112 fires as warning on draft skill** — same input but `status: draft` → audit report contains one `rule_id: FM-112, severity: warning, status: needs_human` issue.
14. **Auditor never auto-fixes description** — even when `auto_fix_applied` could be true in principle, the rule's `auto_fix_applied: false` is preserved across all runs.
15. **Backfill exemplar — feature-request-author** — after FR-SKILL-111 ships, `modules/skill/feature-request-author/SKILL.md` description carries ≥2 trigger phrases; running `cyberos skill validate` on it returns Ok; running the auditor returns no FM-112 issues.
16. **Backfill exemplar — feature-request-audit** — analogous; description carries ≥2 trigger phrases including ones distinct from the author's so the classifier disambiguates.
17. **Backfill exemplar — prd-author** — analogous, completing the 3 exemplars cited in the §6.1 of `modules/skill/ANTHROPIC_GUIDE_DIGEST.md`.
18. **README Part 2.1 row updated** — the description-field row in Part 2.1 of `modules/skill/README.md` shows new min/max + cross-references FR-SKILL-111.
19. **feature-request-audit skill §3.13 entry added** — new sub-rule "Description format" with example good + bad descriptions.
20. **Cross-FR reciprocity preserved** — FR-SKILL-103's `blocks:` list updated to include FR-SKILL-111 (since 103 is the parent frontmatter spec).
21. **OTel span emitted** — every validate call emits `skill.description.validate` with attributes `skill_id`, `outcome` (ok | too_short | too_long | forbidden_brackets | missing_what | insufficient_triggers), `length_chars`, `trigger_count`, `duration_ms`.

## §5 — Verification

```rust
// services/skill-broker/tests/description_validator_test.rs

use cyberos_skill_broker::frontmatter::description_validator::{validate, DescriptionViolation};

#[test]
fn valid_description_with_two_triggers() {
    let d = r#"Generate a feature_request@1 markdown from PRDs. Use when user asks to "draft an FR" or "turn this PRD into a backlog". Outputs versioned FR-NNN-slug.md files with anti-fabrication discipline."#;
    assert!(validate(d).is_ok());
}

#[test]
fn too_short() {
    let d = r#"Generate FRs. Use "draft" or "audit"."#; // 38 chars
    assert_eq!(validate(d).unwrap_err(), DescriptionViolation::TooShort { len: 38 });
}

#[test]
fn too_long() {
    let d = "A".repeat(1100);
    let d = format!(r#"Generate FRs. Use when user asks to "draft" or "audit". {d}"#);
    let len = d.chars().count();
    assert_eq!(validate(&d).unwrap_err(), DescriptionViolation::TooLong { len });
}

#[test]
fn forbidden_brackets() {
    let d = r#"Generate <FR> markdowns. Use when user asks to "draft" or "audit"."#;
    assert_eq!(validate(d).unwrap_err(), DescriptionViolation::ForbiddenBrackets);
}

#[test]
fn missing_what_verb() {
    let d = r#"Helps with FRs in the backlog. Useful when user says "FR" or "backlog" or "story". Returns markdown."#;
    assert_eq!(validate(d).unwrap_err(), DescriptionViolation::MissingWhat);
}

#[test]
fn insufficient_triggers_single_positive() {
    let d = r#"Generate FRs from a PRD source. Use when user asks to "draft an FR". Outputs versioned files in a structured backlog directory under output_dir."#;
    assert_eq!(
        validate(d).unwrap_err(),
        DescriptionViolation::InsufficientTriggers { found: 1, needed: 2 }
    );
}

#[test]
fn negative_trigger_does_not_count() {
    let d = r#"Generate FRs from a PRD source. Use when user asks to "draft an FR". Do NOT use for "audit existing FRs". Outputs versioned FR-NNN-slug.md files."#;
    assert_eq!(
        validate(d).unwrap_err(),
        DescriptionViolation::InsufficientTriggers { found: 1, needed: 2 }
    );
}

#[test]
fn two_positive_plus_one_negative_accepts() {
    let d = r#"Generate FRs from a PRD source. Use when user asks to "draft an FR" or "turn this PRD into a backlog". Do NOT use for "audit existing FRs". Outputs versioned files."#;
    assert!(validate(d).is_ok());
}

#[test]
fn folded_yaml_multiline_flattens() {
    // Simulate what serde_yaml emits after parsing a `description: >-` block.
    let d = "Generate FRs from a PRD source. Use when user asks to\n\"draft an FR\" or \"turn this PRD into a backlog\".\nOutputs versioned files.";
    assert!(validate(d).is_ok());
}

#[test]
fn exact_floor_80_chars_with_two_triggers() {
    // Smallest valid string: exactly 80 chars including 2 triggers + verb.
    let d = r#"Audit FRs. Use when user asks to "audit FR" or "check FR". Reports issues."#;
    assert_eq!(d.chars().count(), 74); // sanity — below floor, should fail
    assert_eq!(validate(d).unwrap_err(), DescriptionViolation::TooShort { len: 74 });

    let d = r#"Audit feature_request@1 markdowns. Use when user asks to "audit an FR" or "check rubric". Reports per-rule verdicts."#;
    assert!(d.chars().count() >= 80);
    assert!(validate(d).is_ok());
}

#[test]
fn cli_exit_code_on_violation() {
    use assert_cmd::Command;
    let bundle = tempfile::tempdir().unwrap();
    // ... write a SKILL.md with too-short description ...
    let mut cmd = Command::cargo_bin("cyberos-skill-validate").unwrap();
    cmd.arg(bundle.path()).assert().failure().code(6);
}

#[test]
fn jsonschema_mirror_agrees_on_length_bounds() {
    // Run ajv-CLI against the 4 length-test fixtures and assert
    // accept/reject pattern matches Rust validator.
    use std::process::Command;
    let outputs = ["description-valid", "description-too-short", "description-too-long", "description-missing-triggers"]
        .iter()
        .map(|fix| {
            Command::new("ajv")
                .args(["validate", "-s", "skill.schema.json", "-d", &format!("tests/fixtures/{fix}/skill.json")])
                .status()
                .unwrap()
                .success()
        })
        .collect::<Vec<_>>();
    // valid + too-long-by-pattern → ajv catches length; missing-triggers ajv ignores (rust-only).
    assert_eq!(outputs, vec![true, false, false, true /* ajv can't see triggers */]);
}
```

### Auditor regression fixtures (added to `modules/skill/feature-request-audit/acceptance/`)

```bash
# Three new golden fixtures:
acceptance/regression-2026-05-19-fm112-missing-triggers/
  golden-input.json    # an FR-style fixture with frontmatter that fails FM-112
  golden-output.audit.md   # expected audit report with one FM-112 issue
```

## §6 — Implementation skeleton

Most of the surface is in §3 (Rust types + validator + JSONSchema diff + auditor rule + template diff). The remaining orchestration is small:

1. **Validator wiring** — `services/skill-broker/src/frontmatter/validators.rs`'s top-level `validate(fm, body, broker_version)` calls `description_validator::validate(&fm.description)?` as the first check.
2. **CLI integration** — `cyberos skill validate` already exists (FR-SKILL-103 §3 CLI block). No change needed in the CLI itself — the new validator surfaces through the same `Err` path.
3. **Auditor integration** — `feature-request-audit/RUBRIC.md` is the lookup table; the auditor's 8-step loop (per `_template/audit/AUDIT_LOOP.md`) runs every rule including FM-112.
4. **Lazy-backfill mechanic** — no automation. The new RUBRIC rule fires on `status: accepted` skills; first audit cycle after FR ship surfaces the issue; author addresses during normal fine-tune flow. No bulk sweep.

## §7 — Dependencies

**Depends on:**
- **FR-SKILL-103** (frontmatter-extension) — provides the broker, frontmatter schema, validator framework, CLI binary, JSONSchema mirror, and `InvalidDescription` error variant. FR-SKILL-111 extends FR-SKILL-103's `description` field rules.

**Blocks:** none (independent of FR-SKILL-112 and FR-SKILL-113).

**Related:**
- **FR-SKILL-112** (trigger-tests-fixtures) — defines positive + negative trigger phrases per skill in `acceptance/TRIGGER_TESTS.md`. The two FRs are complementary: 111 puts triggers in the description; 112 validates the description's triggers against actual classifier behaviour. Either can ship first.
- **FR-SKILL-113** (XML-tag-free frontmatter — sketched only, not authored) — both 111 and 113 strengthen the host-portability surface. If 113 ships first, FR-SKILL-111's §1 #4 becomes a duplicate check (acceptable; the duplicate is defensive). If 111 ships first, FR-SKILL-113 still provides the broader sweep across `wrap_in:` and other fields.
- **FR-SKILL-101** (memory integration) — orthogonal; runs in a separate slice.
- **AGENTS.md §15** — the SKILL.md scope contract referenced from the broker.

**Cross-module:**
- **OBS module** (FR-OBS-001..009) — when OBS ships, the `skill.description.validate` OTel span feeds the dashboard; per-skill description-format violation rates are visible.

## §8 — Example payloads

### Example 1 — valid description (the canonical good)

```yaml
description: >-
  Generate a versioned feature_request@1 markdown from one or more PRD/spec/SRS
  documents. Use when user asks to "draft an FR", "turn this PRD into a backlog",
  or "expand this spec into FRs". Halts at PLAN approval and HITL gates;
  resumable from manifest.json. Chains into feature-request-audit by default.
  Outputs FR-NNN-slug.md files with per-claim authority markers + provenance.
  Do NOT use for "audit existing FRs" (use feature-request-audit instead).
```

Flattened length: ~478 chars. Triggers detected: 3 positive (`"draft an FR"`, `"turn this PRD into a backlog"`, `"expand this spec into FRs"`) + 1 negative (`"audit existing FRs"` after `Do NOT use for`). Verb stems: `generate`, `chain` (multiple). Verdict: **OK**.

### Example 2 — auditor issue block (FM-112 firing)

```
ISSUE
id:              ISS-007
rule_id:         FM-112
status:          needs_human
severity:        error
category:        description_format
location:        frontmatter "description:" field
evidence:        "Author a closure markdown from project state. Halts at HITL gates. Outputs versioned closure@1 files."
description:     "Description fails FM-112: insufficient_triggers (found: 0, needed: 2). The description has no quoted trigger phrases — the classifier sees only WHAT, not WHEN."
suggestion:      "Add ≥2 quoted trigger phrases. Example: 'Use when user asks to \"close the project\" or \"draft the closure report\"'. See feature-request-audit skill §3.13."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-05-19T14:00:00Z"
updated_at:      "2026-05-19T14:00:00Z"
```

### Example 3 — OTel span

```json
{
  "name": "skill.description.validate",
  "attributes": {
    "skill_id": "feature-request-author",
    "outcome": "ok",
    "length_chars": 487,
    "trigger_count": 3,
    "duration_ms": 0.4
  }
}
```

### Example 4 — CLI JSON output

```json
{
  "status": "fail",
  "skill_id": "closure-author",
  "violation": {
    "kind": "description_format",
    "sub_code": "insufficient_triggers",
    "found": 0,
    "needed": 2,
    "description_length": 97
  }
}
```

## §9 — Open questions

**All resolved during authoring.**

Deferred to follow-up FRs (out of scope here):
- **FR-SKILL-115** (placeholder — not yet specified): localised descriptions (`description_localized.<lang>`) for VN-locale skills. Per §1 #10. Phase P2+.
- **FR-SKILL-116** (placeholder — not yet specified): classifier-feedback loop — feed real user phrasings that triggered (or failed to trigger) each skill back into the description's trigger phrases. Phase P2+ when OBS dashboards have enough volume.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Author writes description ≤80 chars | `cyberos skill validate` → `TooShort`; CI gate fails | Skill not promoted to `accepted` | Author rewrites; re-runs validate |
| Author writes description >1024 chars | `cyberos skill validate` → `TooLong`; CI gate fails | Skill not promoted | Author trims; re-runs validate |
| Author embeds `<some-marker/>` in description | Validator → `ForbiddenBrackets`; CI gate fails | Skill not promoted | Author rewrites without brackets; if a marker is needed, move it to `metadata:` |
| Author writes `Helps with X` (no verb stem) | Validator → `MissingWhat`; CI gate fails | Skill not promoted | Author rewrites with concrete action verb |
| Author writes single quoted trigger | Validator → `InsufficientTriggers { found: 1, needed: 2 }` | Skill not promoted | Author adds second trigger phrase |
| Author writes 2 negatives + 0 positives | Validator → `InsufficientTriggers { found: 0, needed: 2 }` | Skill not promoted | Author adds positive trigger forms |
| Production skill backfilled wrong — trigger phrase doesn't match classifier behaviour | OBS reports `acceptance_rate` drop on that skill; manual fine-tune triggered (per `human_fine_tune.signals_to_initiate`) | Auto-pause at <40% per DEC-055 | Fine-tune cycle: update description triggers; re-test against TRIGGER_TESTS.md (FR-SKILL-112 when shipped) |
| Two skills' descriptions overlap in trigger phrases | Supervisor classifier returns ambiguous routing; surfaces Question primitive ("which workflow do you mean?") | User clarifies | Fine-tune both skills: differentiate triggers; cross-reference in `## When to invoke this skill` body section |
| Description triggers stale (skill behaviour changed but description didn't) | Auditor rule FM-112 still passes (triggers exist); but OBS shows `acceptance_rate` drop | Operator-initiated fine-tune | Update description AND body section together (§1 #15 keeps them in sync) |
| Author flattens YAML multi-line wrong (e.g. `|-` instead of `>-`) | YAML parser preserves newlines; flattened length still under cap; tests still pass — but the host's prompt receives literal `\n` characters | Cosmetic issue only; no functional break | Documentation note in feature-request-audit skill §3.13 prefers `>-` (folded) |
| JSONSchema mirror drifts from Rust validator | CI gate runs both validators against fixtures; mismatch on any one fixture → CI fails | Build broken | Sync schema.rs ↔ skill.schema.json via `cargo xtask schema` (FR-SKILL-103 §11) |
| OTel span attribute schema drifts | Dashboard panels expecting `outcome` attribute see `validation_outcome` after a rename → no data | Visible in OBS dashboard panels | Rename only via MINOR-bump + dashboard migration; never silently |
| FM-112 rule mis-classifies a verb stem (e.g. `track` matches but author meant another sense) | Rule auto-fix never fires (always `needs_human`); operator reviews and either accepts the suggestion or extends the verb-stem regex via a future PATCH | No silent damage | Extend `VERB_STEMS` regex via PR; bump RUBRIC PATCH version |
| Backfill exemplar (feature-request-author) inadvertently breaks a working chain | Per-skill acceptance fixture catches it on next CI run | Chain works on golden fixture but fails on a real PRD | Rollback description change; re-author with the existing trigger phrases preserved |

## §11 — Implementation notes

- **Why not enforce trigger-phrase format at YAML-parse time?** Could be a JSONSchema regex, but that regex would be unwieldy (`(?s)^.*"[^"]+".*"[^"]+".*$` works for two but breaks on negative-trigger detection). Pushing this to the Rust validator gives us precise sub-code errors and keeps JSONSchema simple. The JSONSchema mirror handles length + bracket guard only; full semantic in Rust.
- **Why 80-char floor and not 100?** Empirical. 74 chars (`"Audit FRs. Use when user asks to \"audit FR\" or \"check FR\". Reports issues."`) hits 2 trigger phrases + WHAT verb + brief value — but feels tight. 80 gives slack for a slightly longer outcome phrase. Below 80 the descriptions read as cryptic; above 80 they breathe.
- **Why verb-stem regex and not a model-based check?** Determinism. CI gates need byte-stable outcomes. A model-based "is this a verb-action description?" check would drift between model versions and break reproducibility. The regex is conservative (29 verb stems); operators extend it via PR when a legitimate new verb appears. False negatives are caught at audit (verdict: `needs_human` — operator can override).
- **Why preserve `## When to invoke this skill` body section?** Three reasons in §1 #15 + §2 — but the deeper one is **audit-trail integrity**. The body is what the supervisor reads at classify_act; the frontmatter is what hosts read at level-1 discovery. The two serve different layers; merging them would force one or the other to compromise. Keeping both is a small cost.
- **Lazy backfill is the right strategy.** A 104-pair sweep in one batch would consume ~30 hours and inject inconsistency into the audit ledger (every skill bumps `skill_version` in one commit batch). Lazy backfill — fire FM-112 only on `status: accepted`+ skills, surface during natural fine-tune — spreads the fix over 4-6 weeks of normal cadence. No big-bang risk.
- **Two trigger-phrase forms is a floor, not a ceiling.** Skills with broad surface (e.g. `chain-selector`) may want 5+ triggers. The validator never complains about >2; it only complains about <2.
- **Authors who hate the verb-stem regex** can propose extensions via PR + RUBRIC bump. The list is intentionally conservative — adding 50 verbs would let `Helps with X` slip through. A tight regex with a clear extension protocol is healthier than a permissive regex that admits anti-patterns.
- **Cross-host portability is the load-bearing reason for this FR.** Inside CyberOS, the supervisor reads the body, and trigger-phrase placement doesn't matter. Outside CyberOS — Phase B transpilers, partner connectors, the eventual OCI registry per FR-SKILL-102 — the host follows Anthropic's contract. FR-SKILL-111 makes CyberOS skills portable on day one of Phase B without an emergency port-surface sweep.
- **FR-SKILL-112's TRIGGER_TESTS.md complements this FR**. Once both ship, the workflow becomes: description carries triggers (111) → TRIGGER_TESTS.md asserts the classifier matches those triggers (112) → CI catches drift. The two FRs are independent but together close the routing gap entirely.

---

*End of FR-SKILL-111.*
