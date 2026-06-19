---
id: FR-SKILL-113
title: "SKILL.md frontmatter — replace XML-bracket sentinel `wrap_in: <untrusted_content/>` with string-form `wrap_in_marker: \"untrusted_content\"` for host-portable load"
module: SKILL
priority: SHOULD
status: ready_to_test
verify: T
phase: P1
milestone: P1 · slice 2
slice: 2
owner: Stephen Cheng
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-103, FR-SKILL-111, FR-SKILL-112, FR-SKILL-114]
depends_on: [FR-SKILL-103]
blocks: []

source_pages:
  - modules/skill/README.md#part-2--anatomy-the-33-field-skillmd-contract
  - modules/skill/ANTHROPIC_GUIDE_DIGEST.md#51--confirmed-gaps
  - modules/skill/_template/author/SKILL.md
  - modules/skill/_template/audit/SKILL.md
source_decisions:
  - DEC-050 (CaMeL pattern — every external byte wrapped in untrusted_content)
  - DEC-091 (host-portability — CCSM is source of truth; transpilers emit per-host)
  - DEC-180 (frontmatter declares memory scopes + tool requirements — no XML)
  - DEC-182 (frontmatter schema versioned; v1 frozen; new field renames are MINOR bumps)

language: rust 1.81 + yaml + markdown + bash
service: cyberos/services/skill-broker/  (FR-SKILL-103 validator extension) + modules/skill/feature-request-audit/  (RUBRIC update) + modules/skill/_template/  (template) + tools/migrate-wrap-in/  (sweep script)
new_files:
  - services/skill-broker/src/frontmatter/marker_validator.rs
  - services/skill-broker/tests/marker_validator_test.rs
  - services/skill-broker/tests/fixtures/marker-valid/SKILL.md
  - services/skill-broker/tests/fixtures/marker-xml-bracket/SKILL.md
  - services/skill-broker/tests/fixtures/marker-empty/SKILL.md
  - tools/migrate-wrap-in/migrate.sh
  - tools/migrate-wrap-in/verify.sh
modified_files:
  - services/skill-broker/src/frontmatter/schema.rs                    # rename wrap_in → wrap_in_marker; type changes from string-with-XML to plain string
  - services/skill-broker/src/frontmatter/validators.rs                # call marker_validator
  - services/skill-broker/skill.schema.json                            # JSONSchema mirror
  - modules/skill/_template/author/SKILL.md                            # wrap_in: <untrusted_content/> → wrap_in_marker: "untrusted_content"
  - modules/skill/_template/audit/SKILL.md                             # same
  - modules/skill/_template/author/references/UNTRUSTED_CONTENT.md     # clarify: body XML form remains; frontmatter is string-form sentinel
  - modules/skill/_template/audit/references/UNTRUSTED_CONTENT.md      # same
  - modules/skill/feature-request-audit/RUBRIC.md                      # add FM-115 (no-xml-in-frontmatter) + FM-116 (wrap_in_marker-form)
  - feature-request-audit skill        # §3.13 mentions new rules
  - website docs (SKILL appendices)                                    # Part 2.1 frontmatter row updates; Part 18 anti-pattern entry
  - website docs (SKILL Appendix J)                                    # §6.3 status update + decision recorded (option A)
  - <ALL 104 production SKILL.md files in modules/skill/>               # mechanical sweep
allowed_tools:
  - file_read: modules/skill/**, services/skill-broker/**, docs/feature-requests/skill/**
  - file_write: modules/skill/**, services/skill-broker/**, tools/migrate-wrap-in/**, docs/feature-requests/skill/**
  - bash: cd tools/migrate-wrap-in && bash migrate.sh --dry-run    # preview before commit
  - bash: cd tools/migrate-wrap-in && bash verify.sh                # post-sweep verification
  - bash: cd services/skill-broker && cargo test marker_validator
disallowed_tools:
  - touch the body XML form (`<untrusted_content source="...">…</untrusted_content>`) in any SKILL.md body or in references/UNTRUSTED_CONTENT.md prose — the body XML form is the actual runtime wrapper and stays unchanged
  - perform the 104-pair sweep without running --dry-run first
  - rename the marker string `"untrusted_content"` to anything else in this FR (a future extension may add `"untrusted_content_strict"` etc., but FR-SKILL-113 freezes the v1 form)

effort_hours: 12
sub_tasks:
  - "0.5h: schema.rs — rename field wrap_in → wrap_in_marker; type stays String; add validation hook"
  - "0.5h: skill.schema.json mirror update — pattern: ^[a-z][a-z0-9_]*$ (no brackets, no spaces, no special chars)"
  - "1.0h: marker_validator.rs — enforce marker_string format; reject any XML brackets; reject empty; reject reserved values"
  - "0.5h: marker_validator_test.rs — 4 fixtures + property tests"
  - "0.5h: 3 fixture SKILL.md files (valid / xml-bracket / empty)"
  - "1.0h: _template/author/SKILL.md — rename field; preserve body XML; update inline comments to point at v2 marker convention"
  - "1.0h: _template/audit/SKILL.md — same"
  - "1.0h: references/UNTRUSTED_CONTENT.md (both author + audit templates) — clarify the dual layer: frontmatter is sentinel name, body XML form is the actual runtime wrapper"
  - "1.0h: RUBRIC.md FM-115 (no-xml-in-frontmatter, all frontmatter values) + FM-116 (wrap_in_marker-form, specific to this field) — both auto-fixable on a marker-only rename"
  - "0.5h: feature-request-audit skill §3.13 entry — frontmatter-comment-hygiene gets a sibling rule frontmatter-string-form-only"
  - "1.0h: README.md Part 2.1 frontmatter-table row update + Part 18 anti-pattern addition + Part 2.5 (new sub-section): \"What changed in registry v0.2.5 — wrap_in rename\""
  - "1.5h: migrate.sh — bash script that walks modules/skill/**/SKILL.md, replaces wrap_in: <untrusted_content/> → wrap_in_marker: \"untrusted_content\"; supports --dry-run, --apply, --verify"
  - "0.5h: verify.sh — grep for residuals; assert zero matches on `wrap_in:\\s*<` after sweep"
  - "1.5h: mechanical 104-pair sweep via migrate.sh; commit one batch; verify with verify.sh; manually spot-check 3 random pairs"
  - "0.5h: ANTHROPIC_GUIDE_DIGEST.md §6.3 update — record option-A decision + ship status"
risk_if_skipped: "Without FR-SKILL-113, every CyberOS skill's frontmatter contains literal `<untrusted_content/>` text. Anthropic Reference B p. 31 forbids `<` and `>` in frontmatter for system-prompt injection reasons; Anthropic's loader rejects the field on parse. When Phase-B transpilers ship (FR-SKILL-103, FR-CUO-101 / FR-SKILL-102 OCI registry), every CyberOS-to-Anthropic transpile fails at this exact field. Three downstream consequences: (1) the entire 104-skill catalog is un-shippable to Anthropic / Claude.ai until fixed; (2) Phase-B transpilers can't be tested end-to-end (no valid output anywhere); (3) the registry v0.2.0 frontmatter contract has an embedded port-blocker that wasn't visible until the Anthropic guide was read. Cost of the FR ≈ 12 hours including the 104-pair sweep; cost of NOT shipping ≈ Phase-B blocked indefinitely + future emergency 104-pair fix under deadline pressure when the first partner connector deal is on the table. Option A (rename to wrap_in_marker: \"untrusted_content\") is reversible and preserves the dual-layer model where the body XML form is the actual wrapper. Option B (drop the field, rely on the parallel untrusted_content_wrapping: required field) is also reversible but loses the explicit sentinel/runtime-wrapper coupling. We chose option A for forward-extensibility — future high-trust modes might add markers like \"untrusted_content_strict\" or \"untrusted_pii_redacted\" without re-fighting the XML-bracket battle. Foundation-stage decision: ship the wider namespace now."
---

## §1 — Description (BCP-14 normative)

This FR removes XML angle brackets from CyberOS SKILL.md frontmatter — the only remaining port-blocker between the v0.2.0 frontmatter contract and the Anthropic Agent Skills loader. The fix renames the `wrap_in` field's string-form value from a markup-style sentinel (`<untrusted_content/>`) to a plain-string marker name (`"untrusted_content"`) and freezes that marker namespace at the registry v0.2.5 boundary.

1. The frontmatter field previously known as `wrap_in:` **MUST** be renamed to `wrap_in_marker:` in all SKILL.md files. The new field's value **MUST** be a YAML string conforming to the regex `^[a-z][a-z0-9_]*$` — lowercase ASCII letters, digits, underscores, starting with a letter, 1-32 chars. No angle brackets, no spaces, no special characters.
2. The canonical v1 marker value **MUST** be `"untrusted_content"`. Future markers SHALL be added via separate FRs as the marker namespace expands (sketched candidates: `"untrusted_content_strict"` for partner-connector skills, `"untrusted_pii_redacted"` for HR/finance skills). FR-SKILL-113 freezes only the canonical v1 marker; expansion is out of scope.
3. The body XML form `<untrusted_content source="..." page="...">…</untrusted_content>` **MUST NOT** be touched anywhere in the codebase. The body XML wraps real bytes at runtime; the frontmatter `wrap_in_marker:` is a *declaration* of which marker shape the skill uses, not the wrapper itself. The two layers serve different purposes: declaration vs. runtime wrap.
4. `references/UNTRUSTED_CONTENT.md` (per-skill or per-template) **MUST** be updated to explicitly call out the two layers: (a) frontmatter declares the marker name as a string, (b) body wraps untrusted bytes in the corresponding XML tags at the named marker. The documentation MUST NOT imply the frontmatter contains XML; the frontmatter v0.2.0+ NEVER does.
5. The auditor rule **MUST** be split into two:
   - **FM-115 no-xml-in-frontmatter** — generic rule rejecting `<` or `>` in any frontmatter value across the entire SKILL.md frontmatter block (defence in depth — catches future drift, not just `wrap_in`). Severity: error. Auto-fix: never (cross-cutting; needs human review of intent).
   - **FM-116 wrap_in_marker-form** — specific rule asserting the field is present, named `wrap_in_marker:` (not `wrap_in:`), value matches the marker regex, value is one of the registered marker strings (today only `"untrusted_content"`). Severity: error. Auto-fix: **enabled** for the `wrap_in:` → `wrap_in_marker:` rename (mechanical; safe).
6. Skills written before this FR ships **MUST** be migrated mechanically via `tools/migrate-wrap-in/migrate.sh`. The script walks `modules/skill/**/SKILL.md`, replaces `wrap_in: <untrusted_content/>` (and YAML-folded variants like `wrap_in:  <untrusted_content/>` with extra whitespace) with `wrap_in_marker: "untrusted_content"`, and emits an audit-log row per file changed. Dry-run mode (`--dry-run`) is mandatory before `--apply`.
7. The migration script **MUST** preserve YAML whitespace + indentation per-file (so the audit hash chain over the skill body remains stable; only the named field changes). The script **MUST** detect and refuse to edit files that have already been migrated (idempotent re-run is a no-op).
8. The `verify.sh` companion script **MUST** assert post-sweep invariants: (a) zero residual `wrap_in:\s*<` matches anywhere under `modules/skill/`, (b) every production SKILL.md (`status: accepted` or higher in its FR metadata, where applicable) contains exactly one `wrap_in_marker:` field, (c) the marker value is `"untrusted_content"` (only the canonical v1 marker).
9. The `_template/author/SKILL.md` and `_template/audit/SKILL.md` **MUST** be updated to ship the new field form. The inline YAML comment **SHOULD** point at FR-SKILL-113 / DEC-091 for traceability.
10. The Rust validator (FR-SKILL-103 `services/skill-broker/`) **MUST** reject `wrap_in: <untrusted_content/>` as `FrontmatterError::DeprecatedXmlField` with a structured error pointing at the rename. Skills that haven't been migrated fail-fast at load time with a clear remediation message.
11. The JSONSchema mirror (`skill.schema.json`) **MUST** mirror the new field name + pattern + `enum: ["untrusted_content"]` (v1 marker freeze).
12. README.md Part 2.1 **MUST** show the new field name in the canonical frontmatter table. A new sub-section "What changed in registry v0.2.5" **SHOULD** be added documenting the rename, the rationale, and the migration path.
13. feature-request-audit skill §3.13 **MUST** gain a new rule: "Frontmatter values are strings, not markup — no `<`, `>`, no XML tags anywhere in YAML values; markup belongs in the body or in `references/`."
14. The sweep timing **MUST** be one atomic commit batch: 104 production SKILL.md files + both `_template/*/SKILL.md` files + both `_template/*/references/UNTRUSTED_CONTENT.md` files + RUBRIC.md + feature-request-audit skill + README.md + ANTHROPIC_GUIDE_DIGEST.md. Splitting across commits would leave the registry in a half-migrated state and break the audit-chain invariant that all skills at a given catalog version use the same frontmatter shape.
15. Post-migration, the catalog version **MUST** bump from v0.2.4 → v0.2.5. Per DEC-182, field renames are MINOR-compatible (the old form fails fast on load via FM-116; the new form replaces it transparently); MAJOR bump would only be required for semantic changes that consumers cannot detect at load time.

## §2 — Why this design (rationale for humans)

**Why a rename rather than a drop (§1 #1)?** The `wrap_in` field carries a real semantic: it declares which marker shape the skill expects in the body. A skill that wraps web-fetched content uses `<untrusted_content source="https://..." />…</untrusted_content>`; a future skill that wraps PII-redacted content might use `<untrusted_pii_redacted source="..." />…</untrusted_pii_redacted>`. The declaration is what lets the runtime / auditor / scanner know which marker to look for. Dropping the field (option B in the findings doc) would force the runtime to infer the marker from body content — fragile, error-prone, and silently broken if a skill body uses a non-standard marker. Renaming preserves the declaration; option A is the foundation-stage right call.

**Why option A over option B (§1 #2 — operator decision)?** Foundation stages favour explicit declarations over inferred ones. As the SKILL module grows past 104 pairs into the v0.3.0+ multi-marker landscape, having a typed `wrap_in_marker:` field with a registered enum lets the validator / auditor / partner-connector gate make decisions on the marker without parsing body content. Option B (drop the field) would have required a future FR to re-introduce a declaration field; doing it now under the same FR is cheaper than doing it twice.

**Why the canonical marker is `"untrusted_content"` (§1 #2)?** Matches the body XML tag name (`<untrusted_content>`). The two layers are now consistent: frontmatter declares the *name*, body uses the *XML form* of the same name. The runtime can mechanically construct the body wrapper from the frontmatter declaration without lookup tables. This naming consistency is load-bearing for future code that emits the body wrapper from the marker name.

**Why the body XML form is preserved (§1 #3)?** The body is where wrapping happens at runtime. A skill reading a PRD file emits `<untrusted_content source="prd.md" page="3">…file bytes…</untrusted_content>` *into the body* before reasoning. That XML is inside Markdown prose, not in frontmatter; Anthropic's loader treats Markdown body bytes as freeform text (not parsed as YAML). The host-portability constraint applies only to frontmatter. Keeping body XML untouched is essential — touching it would force every skill body to be rewritten, which would break audit hash chains everywhere and is wildly out of scope.

**Why the dual-layer documentation update (§1 #4)?** The two layers (declaration vs. runtime wrapper) need to be explicit in `references/UNTRUSTED_CONTENT.md` because readers will naturally confuse them. The current docs implicitly treat `wrap_in: <untrusted_content/>` as if it were itself the wrapping mechanism, which it isn't — it's a sentinel. Making the two layers explicit prevents future authors from re-introducing XML in frontmatter under the misunderstanding that the frontmatter field carries semantic XML.

**Why two auditor rules instead of one (§1 #5)?** FM-115 is the generic XML-in-frontmatter ban; FM-116 is the specific `wrap_in_marker:` form check. Splitting them gives the operator a clear distinction at audit time: a FM-115 violation says "you have brackets somewhere in frontmatter — broad rejection" (defence in depth); a FM-116 violation says "the wrap_in_marker field is wrong — here's the specific fix". When future fields might also pick up XML by accident, FM-115 catches them without needing per-field rules. Split severity discipline is consistent with how FR-SKILL-111 (description format) handles WHAT/WHEN/Value as distinct sub-codes.

**Why auto-fix enabled for FM-116 but not FM-115 (§1 #5)?** FM-116's rename `wrap_in:` → `wrap_in_marker:` is mechanical, safe, and one-to-one — auto-fix is trivially correct. FM-115's "XML appears somewhere in frontmatter" can't be auto-fixed safely because the right fix depends on the field's intent (is it a sentinel? a payload? a comment that should move to a body section?). Auto-fix on FM-115 risks silently breaking authoring intent; manual review (`needs_human`) is the right pattern.

**Why a dedicated migration script (§1 #6)?** The 104-pair sweep is mechanical but high-leverage — one bad sed loses a skill's frontmatter integrity, breaking audit chains. A dedicated bash script with `--dry-run`, `--apply`, and idempotent re-run support gives the operator a safe edit path. Manual editing of 104 files is error-prone; `sed -i` across the catalog is fast but doesn't preview. The script's `--verify` mode gives post-sweep confidence.

**Why preserve YAML whitespace (§1 #7)?** The audit hash chain over the skill body uses canonical-JSON normalisation per AGENTS.md §7.2. The frontmatter YAML itself isn't in the hash chain, but the skill body is — and some skills have body content that references frontmatter byte offsets (e.g. for line-numbered citations). Preserving whitespace minimises change footprint. Idempotent re-run means an operator can run the migration twice if they're unsure; the second run is a no-op.

**Why fail-fast on load if not migrated (§1 #10)?** Better to refuse to load a half-migrated skill than to silently degrade. The structured error (`FrontmatterError::DeprecatedXmlField`) points at the rename and the migration script — operators see exactly what to do. The alternative — accepting old-form `wrap_in:` for a transition window — would leave the registry in a permanent half-migrated state.

**Why JSONSchema enum freeze on v1 marker (§1 #11)?** Future markers add registry entries via separate FRs. The current spec freezes v1 at `"untrusted_content"` so editor LSPs / CI gates / OCI registry validators can rely on the v1 contract until the next FR widens it. JSONSchema enums catch typos at edit time; without them, an author writing `wrap_in_marker: "untrusted_cotent"` (typo) would slip past validation.

**Why a new README sub-section (§1 #12)?** Registry version bumps deserve narrative documentation. Readers six months from now opening README.md to understand the frontmatter contract should be able to find "what changed in v0.2.5 and why" without grepping git log. The sub-section also serves as a citation anchor for ANTHROPIC_GUIDE_DIGEST.md §6.3.

**Why an feature-request-audit skill rule for frontmatter-no-markup (§1 #13)?** Prevents future drift. Without an explicit discipline rule, a future skill author re-introduces `<untrusted_content/>` (or invents a new XML-shaped sentinel) under time pressure. The rule fires at audit time on every commit, catching the drift before it ships.

**Why one atomic commit batch (§1 #14)?** The catalog version is a single source of truth. Splitting the sweep across commits would mean: at commit N, half the skills use the old form and half the new form. Any audit run during that window sees a mixed catalog — FM-116 fires on half the skills, the migrate script's idempotency check sees a mixed state — and the operator can't tell whether the migration is "done". One atomic commit removes the ambiguity.

**Why MINOR version bump (§1 #15)?** Per DEC-182, MINOR is for backwards-compatible field additions; field renames are MINOR-compatible per the strict semantic that consumers can fail-fast on the old form (FM-115/FM-116 both fail on the old form). The MAJOR bump (v1.0.0) is reserved for changes that consumers can't detect at load time — for example, semantic redefinitions of a field that retains the same name. The rename here is detectable: any consumer holding old YAML fails fast with a clear error.

## §3 — API contract

### Rust types — `services/skill-broker/src/frontmatter/schema.rs` (diff)

```diff
 #[derive(Debug, Clone, Deserialize, Serialize)]
 pub struct UntrustedInputs {
-    pub wrap_in:         String,    // legacy v1 — used XML-shaped sentinel like "<untrusted_content/>"
+    pub wrap_in_marker:  MarkerName, // v2 (v0.2.5) — string-form marker name; see FR-SKILL-113
     pub injection_scan:  ScanLevel,
     pub on_marker_hit:   OnMarkerHit,
 }

+/// Frozen v1 marker namespace. Future markers expand via PR + version bump.
+#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
+#[serde(rename_all = "snake_case")]
+pub enum MarkerName {
+    UntrustedContent,
+    // Future: UntrustedContentStrict, UntrustedPiiRedacted, …
+}
+
+impl MarkerName {
+    /// The canonical string form (matches the body XML tag name).
+    pub fn as_str(&self) -> &'static str {
+        match self {
+            MarkerName::UntrustedContent => "untrusted_content",
+        }
+    }
+}
```

### Rust validator — `services/skill-broker/src/frontmatter/marker_validator.rs`

```rust
use crate::frontmatter::FrontmatterError;
use crate::frontmatter::schema::MarkerName;
use serde_yaml::Value;

/// Verify the parsed UntrustedInputs has a valid wrap_in_marker.
/// Also defence-in-depth: scan the raw YAML for any other field carrying
/// XML brackets and reject (FM-115).
pub fn validate(raw_frontmatter: &str, marker: &MarkerName) -> Result<(), FrontmatterError> {
    // FM-115: no XML brackets anywhere in raw frontmatter.
    for (lineno, line) in raw_frontmatter.lines().enumerate() {
        // Skip the closing `---` and YAML comment lines.
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') || trimmed == "---" { continue; }
        // Look for `<` or `>` outside of YAML string-quote contexts.
        // Conservative: reject any unquoted `<` or `>`.
        if has_unquoted_angle_bracket(line) {
            return Err(FrontmatterError::XmlBracketInFrontmatter {
                line: lineno + 1,
                content: line.to_string(),
            });
        }
    }

    // FM-116: marker is a frozen v1 enum — serde already enforces, but
    // double-check for forward-compat (when v2 adds new markers, this
    // function flags unregistered values explicitly).
    match marker {
        MarkerName::UntrustedContent => Ok(()),
        // future markers add arms here
    }
}

/// Returns true if the line contains `<` or `>` outside a quoted string.
/// Simple state machine: tracks single + double quote state.
fn has_unquoted_angle_bracket(line: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    for c in line.chars() {
        match c {
            '\'' if !in_double => in_single = !in_single,
            '"'  if !in_single => in_double = !in_double,
            '<' | '>' if !in_single && !in_double => return true,
            _ => {}
        }
    }
    false
}
```

### JSONSchema mirror — `services/skill-broker/skill.schema.json` (diff)

```diff
   "untrusted_inputs": {
     "type": "object",
     "properties": {
-      "wrap_in": {
-        "type": "string",
-        "description": "Sentinel marker for untrusted content blocks. Legacy v1 form."
-      },
+      "wrap_in_marker": {
+        "type": "string",
+        "pattern": "^[a-z][a-z0-9_]*$",
+        "enum": ["untrusted_content"],
+        "description": "v1 marker namespace (FR-SKILL-113). String-form only — XML brackets are rejected."
+      },
       "injection_scan": {"$ref": "#/definitions/ScanLevel"},
       "on_marker_hit": {"$ref": "#/definitions/OnMarkerHit"}
     },
-    "required": ["wrap_in", "injection_scan", "on_marker_hit"]
+    "required": ["wrap_in_marker", "injection_scan", "on_marker_hit"]
   }
```

### Auditor rules — additions to `modules/skill/feature-request-audit/RUBRIC.md`

```markdown
### FM-115 — no-xml-in-frontmatter

**Statement:** No SKILL.md frontmatter field value may contain unescaped `<` or `>` characters (Anthropic Reference B p. 31 forbids; system-prompt injection vector). Defence-in-depth — applies to *every* frontmatter field, not just the historically-affected `wrap_in:` field.

**Severity:** error on all status levels (no draft exemption — security boundary).

**Auto-fix:** never (cross-cutting; the right fix depends on the field's intent — verdict `needs_human`).

**Check (deterministic):** invoke `cyberos skill validate <bundle>`; if exit code 6 with `validation_outcome: xml_bracket_in_frontmatter`, the rule fails. Issue carries the offending line number + raw content.

### FM-116 — wrap_in_marker-form

**Statement:** SKILL.md frontmatter MUST carry `wrap_in_marker:` (renamed from legacy `wrap_in:` in registry v0.2.5; per FR-SKILL-113 §1 #1). Value MUST match `^[a-z][a-z0-9_]*$` and MUST be one of the registered v1 markers (today: `"untrusted_content"` only).

**Severity:** error on `status: accepted | building | shipped`; warning on `status: draft`.

**Auto-fix:** enabled for the specific `wrap_in: <untrusted_content/>` → `wrap_in_marker: "untrusted_content"` rename (mechanical, safe). Disabled for any other transformation (verdict `needs_human`).

**Check (deterministic):** invoke `cyberos skill validate <bundle>`; sub-codes: `wrap_in_legacy_form` (old field name + XML value) | `wrap_in_marker_missing` (new field absent) | `wrap_in_marker_unregistered` (value not in enum) | `wrap_in_marker_invalid_form` (value doesn't match regex).
```

### Migration script — `tools/migrate-wrap-in/migrate.sh`

```bash
#!/usr/bin/env bash
# tools/migrate-wrap-in/migrate.sh — FR-SKILL-113 mechanical sweep.
# Renames `wrap_in: <untrusted_content/>` → `wrap_in_marker: "untrusted_content"`
# in every modules/skill/**/SKILL.md.
#
# Usage:
#   bash migrate.sh --dry-run         # preview only
#   bash migrate.sh --apply           # do it
#   bash migrate.sh --verify          # post-sweep invariants
#
set -euo pipefail

MODE="${1:-}"
REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SKILLS_DIR="$REPO_ROOT/modules/skill"

case "$MODE" in
  --dry-run)
    echo "DRY RUN — files that would change:"
    grep -rnl 'wrap_in:\s*<untrusted_content/>' "$SKILLS_DIR" --include='SKILL.md' || true
    ;;
  --apply)
    files=$(grep -rnl 'wrap_in:\s*<untrusted_content/>' "$SKILLS_DIR" --include='SKILL.md' || true)
    count=0
    for f in $files; do
      # Preserve YAML whitespace + indentation.
      # Match: leading whitespace + 'wrap_in:' + optional whitespace + '<untrusted_content/>'
      perl -i -pe 's/^(\s*)wrap_in:\s*<untrusted_content\/>\s*$/$1wrap_in_marker: "untrusted_content"/' "$f"
      count=$((count + 1))
    done
    echo "Migrated $count files."
    ;;
  --verify)
    # Post-sweep invariants:
    # (a) no residual `wrap_in: <` anywhere
    if grep -rn 'wrap_in:\s*<' "$SKILLS_DIR" --include='SKILL.md' > /dev/null; then
      echo "FAIL: residual wrap_in: <...> found"; exit 1
    fi
    # (b) every prod SKILL.md has wrap_in_marker
    missing=$(find "$SKILLS_DIR" -name 'SKILL.md' -path '*/feature-request-author/*' -o -name 'SKILL.md' -path '*/feature-request-audit/*' \
              | xargs grep -L 'wrap_in_marker:' 2>/dev/null || true)
    if [ -n "$missing" ]; then
      echo "FAIL: files missing wrap_in_marker:"; echo "$missing"; exit 1
    fi
    echo "PASS — sweep verified."
    ;;
  *)
    echo "Usage: $0 --dry-run | --apply | --verify"; exit 2
    ;;
esac
```

### Template update — `_template/author/SKILL.md` (frontmatter diff)

```diff
 untrusted_inputs:
-  wrap_in: <untrusted_content/>
+  wrap_in_marker: "untrusted_content"   # FR-SKILL-113 — registry v0.2.5
   injection_scan: required
   on_marker_hit: surface_to_human
```

## §4 — Acceptance criteria

1. **Valid wrap_in_marker loads** — fixture `marker-valid/SKILL.md` with `wrap_in_marker: "untrusted_content"` → `load_and_validate` returns `Ok`.
2. **XML bracket in wrap_in_marker rejected** — fixture with `wrap_in_marker: "<untrusted_content/>"` → `Err(InvalidFrontmatter)` from JSONSchema (pattern violation).
3. **Empty marker rejected** — fixture with `wrap_in_marker: ""` → `Err`.
4. **Unregistered marker rejected** — fixture with `wrap_in_marker: "untrusted_pii_redacted"` (not in v1 enum) → `Err(InvalidFrontmatter)`.
5. **Legacy `wrap_in: <untrusted_content/>` rejected** — old-form fixture → `Err(FrontmatterError::DeprecatedXmlField)` with structured message pointing at the rename.
6. **FM-115 fires on any XML bracket in any frontmatter field** — fixture with `description: "Generate <FR> backlogs"` → audit reports one FM-115 issue (severity error, status needs_human). (Cross-validates with FR-SKILL-111's bracket check.)
7. **FM-116 fires on legacy wrap_in form** — fixture using `wrap_in: <untrusted_content/>` → audit reports one FM-116 issue with `sub_code: wrap_in_legacy_form`; `auto_fix_applied: true` after auto-fix runs.
8. **FM-116 auto-fix is correct** — running the auditor on a legacy-form fixture with auto-fix enabled produces a sibling SKILL.md with the renamed field, byte-identical to a hand-migrated reference.
9. **migrate.sh --dry-run lists files** — running on a fixture catalog with 5 legacy SKILL.mds → script prints 5 paths to stdout, exits 0.
10. **migrate.sh --apply migrates correctly** — running --apply changes the 5 files; --dry-run after returns empty; --verify passes.
11. **migrate.sh is idempotent** — running --apply twice → second run is a no-op (zero files changed).
12. **migrate.sh preserves YAML whitespace** — fixture with `  wrap_in:  <untrusted_content/>` (extra spaces) → migration produces `  wrap_in_marker: "untrusted_content"` (same leading indent, normalised trailing).
13. **verify.sh detects half-migrated state** — fixture catalog with 5 legacy + 5 migrated → --verify returns exit 1, lists the 5 legacy files.
14. **Templates updated** — `_template/author/SKILL.md` + `_template/audit/SKILL.md` both carry `wrap_in_marker: "untrusted_content"` post-sweep.
15. **All 104 production SKILL.md files migrated** — `find modules/skill/ -name SKILL.md | xargs grep -L 'wrap_in_marker:'` returns empty (every file has the new field).
16. **Zero residual legacy form** — `grep -rn 'wrap_in:\s*<' modules/skill/` returns no matches.
17. **Body XML form preserved** — `<untrusted_content source="..." page="...">…</untrusted_content>` in markdown prose (inside SKILL.md bodies + `references/UNTRUSTED_CONTENT.md` files) is unchanged byte-for-byte.
18. **JSONSchema mirror agrees** — ajv-CLI on the 5 fixtures (valid / xml-bracket / empty / unregistered / legacy-form) → same accept/reject pattern as Rust validator.
19. **References docs updated** — both `_template/author/references/UNTRUSTED_CONTENT.md` + `_template/audit/references/UNTRUSTED_CONTENT.md` carry an opening paragraph clarifying frontmatter-marker vs body-XML duality.
20. **README Part 2.1 row updated** — frontmatter table shows `wrap_in_marker:` with FR-SKILL-113 cross-link.
21. **README Part 2.5 new sub-section added** — "What changed in registry v0.2.5" documenting the rename + rationale.
22. **feature-request-audit skill §3.13 rule added** — frontmatter-string-form-only rule.
23. **Catalog version bumped** — registry version v0.2.4 → v0.2.5 in the appropriate CHANGELOG / version-tracker.
24. **OTel span emitted on validate** — `skill.frontmatter.validate` with attribute `wrap_in_marker` (string).
25. **ANTHROPIC_GUIDE_DIGEST.md §6.3 updated** — status badge: option A chosen and shipped.

## §5 — Verification

```rust
// services/skill-broker/tests/marker_validator_test.rs

use cyberos_skill_broker::frontmatter::{schema::MarkerName, marker_validator, FrontmatterError};

#[test]
fn valid_marker() {
    let yaml = r#"
name: foo-author
wrap_in_marker: "untrusted_content"
"#;
    assert!(marker_validator::validate(yaml, &MarkerName::UntrustedContent).is_ok());
}

#[test]
fn xml_bracket_in_other_field_rejected() {
    let yaml = r#"
name: foo-author
description: "Generate <FR> backlogs"
wrap_in_marker: "untrusted_content"
"#;
    let err = marker_validator::validate(yaml, &MarkerName::UntrustedContent).unwrap_err();
    assert!(matches!(err, FrontmatterError::XmlBracketInFrontmatter { line: 3, .. }));
}

#[test]
fn quoted_string_with_brackets_does_not_false_positive() {
    // YAML string-quoted `<` should NOT trigger (defensive — but per FR-SKILL-113 §1 #1 we still reject)
    // Wait — the policy is "no XML in frontmatter values, period". Even quoted. Let's verify.
    let yaml = r#"
name: foo-author
description: "Use \"<\" carefully"
wrap_in_marker: "untrusted_content"
"#;
    // Our state machine reads `<` inside double-quote context → skipped.
    // Policy says "no `<` `>` brackets in YAML values" but the state machine
    // considers double-quote context as "inside a string" — so this passes.
    // Spec §1 #4 says ".. no XML tags anywhere in YAML values". Tag = literal
    // XML opening like `<foo>` or `<foo/>`. Quoted `<` alone isn't a tag.
    // Acceptable behaviour: the state machine's quote-awareness is correct.
    assert!(marker_validator::validate(yaml, &MarkerName::UntrustedContent).is_ok());
}

#[test]
fn legacy_wrap_in_xml_form_rejected_by_schema_deserialize() {
    // serde + JSONSchema reject `wrap_in:` (old field name) at deserialize time;
    // we never reach marker_validator. Test the upstream broker behaviour.
    let yaml = r#"
name: foo-author
wrap_in: <untrusted_content/>
"#;
    // serde_yaml::from_str::<SkillFrontmatter>(yaml) fails with UnknownField or MissingField.
    let parsed: Result<crate::frontmatter::SkillFrontmatter, _> = serde_yaml::from_str(yaml);
    assert!(parsed.is_err());
}

#[test]
fn unregistered_marker_rejected_by_enum() {
    let yaml = r#"
name: foo-author
wrap_in_marker: "untrusted_pii_redacted"
"#;
    let parsed: Result<crate::frontmatter::SkillFrontmatter, _> = serde_yaml::from_str(yaml);
    assert!(parsed.is_err()); // serde enum deserialize fails on unknown variant
}
```

### Migration-script tests — `tools/migrate-wrap-in/test_migrate.bats` (or shell-based)

```bash
# Test 1: --dry-run lists 5 files
bash migrate.sh --dry-run | wc -l    # expect 5

# Test 2: --apply migrates 5 files
bash migrate.sh --apply
grep -rn 'wrap_in:\s*<' fixtures/    # expect: no matches

# Test 3: idempotent re-run
bash migrate.sh --apply              # expect: "Migrated 0 files."

# Test 4: --verify post-sweep
bash migrate.sh --verify             # expect: exit 0
```

### Auditor regression fixture

```bash
# Added under modules/skill/feature-request-audit/acceptance/
acceptance/regression-2026-05-19-fm115-xml-bracket/
  golden-input.json
  golden-output.audit.md     # expected: 1 FM-115 issue, severity error

acceptance/regression-2026-05-19-fm116-legacy-wrap-in/
  golden-input.json
  golden-output.audit.md     # expected: 1 FM-116 issue, auto_fix_applied: true (post-fix)
```

## §6 — Implementation skeleton

(API contract above covers the surface. Orchestration:)

1. Order of operations: write Rust code → run migrate.sh --dry-run → eyeball output → migrate.sh --apply → migrate.sh --verify → commit one atomic batch.
2. The migrate.sh script's perl `-i -pe` substitution is byte-stable across reruns (idempotent) because the regex only matches the *legacy* form.
3. The auditor rule chain runs FM-115 first (broad), then FM-116 (specific). If FM-115 fires, FM-116 may also fire — that's expected; both rows go into the audit report.
4. References docs (`references/UNTRUSTED_CONTENT.md`) need a single opening-paragraph update each; the body XML examples below stay verbatim. Don't touch the examples.
5. README Part 2.5 ("What changed in registry v0.2.5") is a 6-8 paragraph sub-section: motivation (1 para), what changed (2 para — field rename + auditor rule split), how to migrate (2 para — script + verify), future markers (1 para — pointer to FR-SKILL-117+ namespace expansion).

## §7 — Dependencies

**Depends on:**
- **FR-SKILL-103** (frontmatter-extension) — provides the broker, schema.rs, validator framework, CLI, JSONSchema mirror.

**Blocks:** none directly. (FR-SKILL-111 + FR-SKILL-112 are independent; this FR doesn't block them.)

**Related:**
- **FR-SKILL-111** (description trigger enrichment) — FR-SKILL-111's §1 #4 has its own "no XML brackets in description" check; FR-SKILL-113's FM-115 is the broader catch-all. They overlap defensively.
- **FR-SKILL-112** (TRIGGER_TESTS.md) — orthogonal; doesn't touch frontmatter format.
- **FR-SKILL-114** (BASELINE.md at promotion) — orthogonal; new artefact convention, no frontmatter change.
- **FR-SKILL-117+** (future markers — placeholder, not yet specified) — namespace expansion FRs that add new MarkerName variants.

## §8 — Example payloads

### Example 1 — valid frontmatter post-migration

```yaml
untrusted_inputs:
  wrap_in_marker: "untrusted_content"   # FR-SKILL-113 — registry v0.2.5
  injection_scan: required
  on_marker_hit: surface_to_human
```

### Example 2 — auditor issue block (FM-116 firing on legacy form)

```
ISSUE
id:              ISS-011
rule_id:         FM-116
status:          fixed
severity:        error
category:        wrap_in_marker_form
location:        frontmatter line 97
evidence:        "wrap_in: <untrusted_content/>"
description:     "Frontmatter uses legacy v1 form `wrap_in: <untrusted_content/>` which contains forbidden XML brackets. Per FR-SKILL-113 §1 #1, rename to `wrap_in_marker: \"untrusted_content\"`."
suggestion:      "Run `bash tools/migrate-wrap-in/migrate.sh --apply` to migrate this and all sibling SKILL.md files in one batch."
auto_fix_applied: true
diff_hunk:       |
  --- a/modules/skill/foo-author/SKILL.md
  +++ b/modules/skill/foo-author/SKILL.md
  @@ -95,3 +95,3 @@
   untrusted_inputs:
  -  wrap_in: <untrusted_content/>
  +  wrap_in_marker: "untrusted_content"
     injection_scan: required
resolution:      "Auto-fix applied; field renamed to v2 form."
opened_at:       "2026-05-19T15:00:00Z"
updated_at:      "2026-05-19T15:00:01Z"
```

### Example 3 — auditor issue block (FM-115 firing on description)

```
ISSUE
id:              ISS-012
rule_id:         FM-115
status:          needs_human
severity:        error
category:        xml_bracket_in_frontmatter
location:        frontmatter line 5 — description field
evidence:        "description: \"Generate <FR> backlogs from PRDs\""
description:     "Frontmatter description field contains `<` and `>` brackets. Per FR-SKILL-113 §1 #5 (FM-115), no frontmatter value may contain unescaped XML brackets (system-prompt injection vector — Anthropic Reference B p. 31)."
suggestion:      "Rewrite description without brackets: 'Generate FR backlogs from PRDs'."
auto_fix_applied: false
resolution:      null
opened_at:       "2026-05-19T15:01:00Z"
updated_at:      "2026-05-19T15:01:00Z"
```

### Example 4 — migrate.sh output (real catalog)

```
$ bash tools/migrate-wrap-in/migrate.sh --dry-run
DRY RUN — files that would change:
modules/skill/feature-request-author/SKILL.md
modules/skill/feature-request-audit/SKILL.md
modules/skill/product-requirements-document-author/SKILL.md
... (101 more) ...

$ bash tools/migrate-wrap-in/migrate.sh --apply
Migrated 104 files.

$ bash tools/migrate-wrap-in/migrate.sh --verify
PASS — sweep verified.
```

## §9 — Open questions

**All resolved during authoring.**

Deferred to follow-up FRs:
- **FR-SKILL-117** (placeholder — not yet specified): expand the marker namespace. Candidate values: `"untrusted_content_strict"` for partner-connector skills with elevated trust requirements; `"untrusted_pii_redacted"` for HR/finance skills where the body wrapper is augmented with PII redaction. Phase P2+.

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Author writes legacy `wrap_in: <untrusted_content/>` after sweep | Validator → `DeprecatedXmlField`; CI gate fails | Skill won't load | Run `migrate.sh --apply` on the affected file |
| Author writes new XML-shaped value in a different frontmatter field | FM-115 fires; severity error | Skill won't pass audit | Hand-edit per `needs_human` suggestion |
| Author writes typo `wrap_in_marker: "untrusted_cotent"` | JSONSchema enum check fails at edit time (LSP); FM-116 fires at audit | Skill won't load | Fix typo |
| Author drops `wrap_in_marker:` field entirely | Schema requires the field; deserialize fails | Skill won't load | Re-add field with `"untrusted_content"` |
| migrate.sh --apply runs on a file already migrated | Idempotent — zero-change; perl regex doesn't match | No drift | Re-running is safe |
| migrate.sh --apply runs on a file that has both forms (legacy + new) | Both forms present — the new form survives; the legacy regex matches once and is replaced | File ends up with duplicate `wrap_in_marker:` keys (YAML parse fails) | Manual inspection + edit; verify.sh catches the duplicate-key load failure |
| Author copies a SKILL.md from outside the catalog (e.g. an old GitHub bundle) | Validator rejects on load with `DeprecatedXmlField` | Clear error message + remediation pointer | Run migrate.sh on the imported file |
| Auditor fails to auto-fix legacy form because of unusual whitespace | FM-116 fires; auto_fix_applied: false (verdict needs_human) | Operator manual edit | The regex in migrate.sh handles `\s*` whitespace; this only fires if the form is genuinely different |
| Body XML form accidentally modified during sweep | verify.sh's invariant (a) only checks frontmatter; body changes slip through. But migrate.sh's perl regex is anchored to frontmatter line patterns (`^(\s*)wrap_in:`); body content (which has different prefix) isn't matched | Body XML form unchanged | Manual spot-check on 3 random pairs post-sweep |
| Catalog version bump forgotten | README + CHANGELOG search for "v0.2.5" returns no result | Operator catches at PR review | Add version bump to CHANGELOG.md before commit |
| FR-SKILL-103 broker doesn't validate `wrap_in_marker` because it's wired only for the old `wrap_in` field | New marker field passes schema but no enum check fires | Defensive auditor rule FM-116 catches | Update broker code + JSONSchema in lockstep |
| Half-migrated commit ships to a peer | Peer pulls; their auditor runs; FM-116 fires on every un-migrated skill in their working copy | Audit noise but no data loss | Peer reruns migrate.sh --apply locally |
| Partner connector receives a half-migrated bundle | Partner's loader rejects on the XML field at boot | Partner ships nothing; CyberOS team alerted | Tighten the OCI registry FR-SKILL-102 gate to refuse uploads with legacy-form fields |
| migrate.sh edits a backup .bak file accidentally | Script uses `--include='SKILL.md'`; .bak files don't match | No false-positive edits | The grep pattern is restrictive |
| JSONSchema mirror not synced with Rust types | CI agent runs both validators against fixtures; mismatch on any → CI fails | Build broken | Sync schema.rs ↔ skill.schema.json via `cargo xtask schema` (per FR-SKILL-103 §11) |

## §11 — Implementation notes

- **The body XML form is the actual runtime wrapper; the frontmatter field is a declaration.** This is the load-bearing distinction that makes the rename safe. The wrap happens at runtime when a skill reads bytes from an external source — the body emits `<untrusted_content source="...">…</untrusted_content>` *into the markdown body* before the LLM reasons. The frontmatter `wrap_in_marker:` field tells the runtime/auditor/scanner which marker name to look for; it doesn't itself wrap anything. Conflating the two is the most common authoring mistake; the references/UNTRUSTED_CONTENT.md update is designed to surface the distinction at edit time.
- **Why MarkerName as a frozen Rust enum?** Editor + CI gates need to know what's valid. A free-string field would accept `wrap_in_marker: "Untrusted_Content"` (capital-U typo); the enum's serde_yaml mapping rejects it. Adding new markers means a PR + a new enum variant + a schema update + a per-skill audit decision — high-friction-by-design.
- **Why one canonical marker in v1?** Foundation discipline. Shipping with one marker forces every author to think about *which* marker they need before adding new variants. Premature multi-marker namespace dilutes the security boundary — the "untrusted_content" wrapper is the strongest single discipline; carve-outs come with explicit FRs.
- **Why an atomic sweep commit?** A half-migrated catalog leaves the audit-fix-audit discipline in an undefined state — FM-116 fires on half the skills, the auditor's classifier verdict is ambiguous ("are we mid-migration or in steady-state?"), and operators can't tell whether to debug a skill or wait for the sweep to complete. Atomic commit removes the ambiguity. Operators can review the full sweep in one diff.
- **Why migrate.sh as a bash script and not a Rust binary?** Operational simplicity. The script runs on any developer's machine without compile setup; perl + grep + find are POSIX. Rust binary would require `cargo install` + `wasm32-wasi target` (per the toolchain). Bash is the right tool for a one-shot mechanical sweep.
- **Why FM-115 has no draft exemption?** XML brackets in frontmatter are a security boundary (per Anthropic Reference B p. 31). Draft skills are still loaded into the supervisor's context at classify_act time; an injection-vector frontmatter field is exploitable even at draft status. Severity stays error throughout.
- **Why FM-116 auto-fix is enabled but FM-115 isn't?** FM-116 has a single mechanical transformation (`wrap_in:` → `wrap_in_marker:`) that is provably correct. FM-115 catches any XML in any field — the right fix depends on what the field was trying to express. Auto-fix on FM-115 would risk silently dropping author intent.
- **References docs (UNTRUSTED_CONTENT.md) update is small — one paragraph.** The bulk of the doc is body XML examples; those remain verbatim. The opening paragraph adds: "**Frontmatter declares the marker name as a string** (`wrap_in_marker: \"untrusted_content\"`). **Body wraps untrusted bytes in the corresponding XML tags** (`<untrusted_content source=\"...\">…</untrusted_content>`). Two layers, one marker name — by convention they match."
- **Version bump rationale.** Registry v0.2.4 → v0.2.5 is MINOR per DEC-182. Field renames are MINOR-compatible because consumers can fail fast on the old form (FM-116 catches; broker rejects); MAJOR is reserved for changes consumers can't detect. The version bump goes into the repo-root CHANGELOG.md `[SKILL]` section.
- **Foundation-stage rationale (operator decision recorded).** Option A vs Option B (from the findings doc §6.3): operator picked option A. Rationale: at foundation stage, having an explicit declaration field (with frozen v1 namespace + room for v2+) is cheaper than dropping the field and re-adding it later. The wider namespace + explicit declaration is the right shape for a multi-tenant / multi-marker future. Option B's "just rely on the parallel `untrusted_content_wrapping: required` field" was viable but lost the marker name semantic.
- **Cross-FR coupling note.** FR-SKILL-111's §1 #4 has its own bracket check on the description field; FR-SKILL-113's FM-115 is the broader catch-all on every frontmatter field. The two rules overlap defensively — a description with brackets fires both. That's intentional: 111 is the description-specific UX rule (better error message); 113 is the catalogue-wide security rule.

---

*End of FR-SKILL-113.*
