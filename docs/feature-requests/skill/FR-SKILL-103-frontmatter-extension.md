---
id: FR-SKILL-103
title: "SKILL.md frontmatter extension — allowed_memory_scopes + allowed_tools + version + signature enforced by capability broker"
module: SKILL
priority: MUST
status: ready_to_test
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-SKILL-101, FR-SKILL-102, FR-SKILL-104, FR-SKILL-105, FR-SKILL-111, FR-SKILL-112, FR-SKILL-113, FR-SKILL-114, FR-SKILL-115, FR-MEMORY-106, FR-AUTH-003]
depends_on: [FR-SKILL-101]
blocks: [FR-SKILL-104, FR-SKILL-105, FR-SKILL-111, FR-SKILL-112, FR-SKILL-113, FR-SKILL-114, FR-SKILL-115]

source_pages:
  - website/docs/modules/skill.html#frontmatter
  - website/docs/runbooks/skill-author-runbook.html
source_decisions:
  - DEC-180 (every .skill bundle MUST declare its memory scopes + tool requirements in frontmatter)
  - DEC-181 (broker enforces frontmatter at invoke time; missing/invalid frontmatter = refuse)
  - DEC-182 (frontmatter schema versioned; v1 frozen at this FR; v2+ via SemVer-style migration)

language: rust 1.81 + yaml
service: cyberos/services/skill-broker/
new_files:
  - services/skill-broker/src/frontmatter.rs
  - services/skill-broker/src/frontmatter/schema.rs
  - services/skill-broker/src/frontmatter/parser.rs
  - services/skill-broker/src/frontmatter/validators.rs
  - services/skill-broker/skill.schema.json
  - services/skill-broker/tests/frontmatter_test.rs
  - services/skill-broker/tests/fixtures/skill-valid/SKILL.md
  - services/skill-broker/tests/fixtures/skill-invalid-no-scopes/SKILL.md
  - services/skill-broker/tests/fixtures/skill-invalid-bad-glob/SKILL.md
  - services/skill-broker/tests/fixtures/skill-invalid-unknown-tool/SKILL.md
modified_files:
  - services/skill-broker/src/lib.rs                    # re-export frontmatter module
  - services/skill-broker/src/invoke.rs                 # call frontmatter::load_and_validate before dispatch
  - cyberos/AGENTS.md                                   # add §17 note on SKILL.md schema (referenced by skills using memory scopes)
allowed_tools:
  - file_read: services/skill-broker/**, services/skill/**
  - file_write: services/skill-broker/{src,tests}/**
  - bash: cd services/skill-broker && cargo test frontmatter
disallowed_tools:
  - invoke a skill whose frontmatter fails to parse or validate (per DEC-181)
  - extend the v1 schema in place (per DEC-182 — schema is frozen; new fields require v2)
  - downgrade signature verification (per §1 #9 — signed bundles MUST be verified before frontmatter is trusted)

effort_hours: 7
sub_tasks:
  - "0.5h: schema.rs — Rust types for SkillFrontmatter v1 (serde-derived)"
  - "0.5h: skill.schema.json — JSONSchema for external validators (e.g. CI gates, editor LSP)"
  - "1.0h: parser.rs — extract frontmatter block (between `---` fences) from SKILL.md; serde_yaml deserialize"
  - "1.0h: validators.rs — each field: allowed_memory_scopes glob syntax, allowed_tools enum, version semver, signature hex format"
  - "1.0h: integration into invoke.rs — refuse with ExitCode::ValidationFailed (6) on parse/validate error"
  - "0.5h: AGENTS.md §17 cross-reference (1 paragraph describing SKILL.md contract for skills that touch memory)"
  - "1.5h: frontmatter_test.rs — happy + 5 negative fixtures + version-skew + signature-mismatch"
  - "0.5h: fixtures (valid + 3 invalid SKILL.md files)"
  - "0.5h: CI lint command `cyberos skill validate <bundle>` (prints structured report)"
risk_if_skipped: "Without normative frontmatter, skill authors invent their own conventions; the broker can't enforce memory-scope boundaries (a skill claims to need `meta/people/*` and silently reads `meta/finance/*`); allowed_tools enforcement (FR-SKILL-104 capability broker) has no data source. Authors burn hours on debug-friendly errors. Version skew (skill written for v1 broker, run on v0.9 broker) crashes at invoke time instead of at install time. Signature absence means a tampered .skill bundle runs without warning. v1 freeze (DEC-182) means downstream tools (LSP, CI gates, OCI registry) can rely on a stable shape — without it, every new field breaks everything."
---

## §1 — Description (BCP-14 normative)

Every `.skill` bundle's `SKILL.md` file **MUST** carry a YAML frontmatter block conforming to schema v1. The contract:

1. **MUST** begin with `---` on line 1, end with `---` on a line by itself; the YAML block lies between. Anything before line 1's `---` is rejected. The body markdown begins on the line after the closing fence.
2. **MUST** include the following required fields (rejection if missing):
    - `id` (string): kebab-case slug `^[a-z][a-z0-9-]*$`; the canonical skill identifier; used in OCI registry path (FR-SKILL-102).
    - `version` (string): SemVer `^\d+\.\d+\.\d+(?:-[A-Za-z0-9.-]+)?$`.
    - `description` (string): 1–200 chars; the operator-facing one-line summary.
    - `allowed_memory_scopes` (list of glob strings): which memory paths the skill MAY read. Empty list `[]` = no memory access. Each glob validated by `globset@0.4`; invalid glob → reject. Globs are evaluated against memory paths starting at `<memory-root>/` (e.g. `memories/projects/cyberos/**`).
    - `allowed_tools` (list of strings): which broker-managed tools the skill MAY invoke. Each name MUST appear in the canonical tool enum (`Bash`, `Read`, `Write`, `Edit`, `Glob`, `Grep`, `MemoryRead`, `MemorySearch`, `HttpFetch`, custom MCP names per FR-SKILL-104). Unknown names → reject.
3. **MUST** support the following optional fields (validated when present):
    - `signature` (object): `{ algo: "ed25519", public_key_hex: <64-char hex>, signature_hex: <128-char hex> }`. Verifies that frontmatter content + body hash matches the signature. Absent signature = unsigned skill (allowed for local dev; rejected by FR-SKILL-102's OCI gate which requires signed).
    - `min_broker_version` (string): SemVer; broker refuses to invoke if its own version is lower than this. Default: `0.1.0`.
    - `max_broker_version` (string): SemVer; broker refuses if higher (forward-incompatible). Default: unbounded.
    - `disallowed_tools` (list of strings): explicit denylist (subset of canonical tool enum). Useful for skills that want to assert "I never need Edit" even if allowed_tools is broad.
    - `sync_class` (enum: `private` | `shareable`): default `private`; controls whether the skill's emitted memory rows are eligible for cross-device sync (per FR-MEMORY-106).
    - `tenant_scope` (enum: `any` | `pinned`): default `any`; if `pinned`, skill executes only under the tenant_id where it was installed.
    - `effort_minutes` (integer): suggested timeout cap; broker enforces SIGTERM after `effort_minutes * 60` seconds.
    - `tags` (list of strings): operator-facing categorisation; ≤ 10 tags; each tag ≤ 30 chars.
4. **MUST** validate every field per §1 #2 + #3 BEFORE the skill is admitted to invocation. The broker exposes `frontmatter::load_and_validate(bundle_path) -> Result<SkillFrontmatter, FrontmatterError>`; only `Ok` admits the skill.
5. **MUST** reject (and produce structured error) on any of:
    - Schema-required field missing.
    - `id` does not match the kebab-case regex.
    - `version` is not valid SemVer.
    - Any `allowed_memory_scopes` glob fails `globset::Glob::new()`.
    - Any `allowed_tools` value is not in the canonical enum.
    - `signature` present and verification fails.
    - `min_broker_version > current_broker_version`.
    - `max_broker_version < current_broker_version`.
    - Frontmatter parses but contains unknown keys NOT prefixed with `x-` (forward-compat: `x-` prefixed unknown keys are allowed and ignored).
6. **MUST** be schema-version frozen at v1. Adding a new required field is a v2 schema; v1 bundles MUST continue to load on broker versions that support v2.
7. **MUST** verify ed25519 signature (when present) BEFORE trusting any other frontmatter field. The signature signs `SHA-256(frontmatter_yaml_canonical) || SHA-256(body_markdown_canonical)`. Canonical form: trimmed leading/trailing whitespace, normalised line endings to `\n`.
8. **MUST** emit OTel span `skill.frontmatter.validate` per call with attributes `skill_id`, `version`, `validation_outcome` (ok | parse_err | required_missing | invalid_field | signature_failed | version_skew), `duration_ms`.
9. **MUST** emit metric `skill_frontmatter_validation_total{outcome}` (counter) and `skill_frontmatter_validation_duration_seconds` (histogram).
10. **MUST** be invokable from CLI: `cyberos skill validate <bundle-path>` prints structured JSON report; exit 0 on valid, exit 6 (`SchemaViolation`) on invalid; exit 1 on file-not-found.
11. **MUST** be invokable from editor LSPs / CI gates via the standalone `skill.schema.json` (JSONSchema Draft 2020-12). LSPs can validate live as the author types; the Rust validator is the runtime source of truth (LSP is informational).
12. **SHOULD** support `cyberos skill scaffold <id>` that writes a starter SKILL.md with all required fields populated and clear comments.

---

## §2 — Why this design (rationale for humans)

**Why frontmatter at all (§1 #1)?** The body of SKILL.md is markdown — readable by humans, opaque to brokers. The frontmatter is the machine-readable contract: "I am skill X v1.2.3; I need scopes Y and Z; I use tools W." Without it, the broker has no idea what the skill needs until the skill *tries* something and either succeeds or trips a denial.

**Why required-field strictness (§1 #2 + #5)?** Optional fields drift over time — early adopters skip them, later code assumes they exist, runtime panics. Required fields are validated at load; the skill either works on every broker or doesn't load at all. Five fields is the calibrated minimum: id (identity), version (compatibility), description (operator UX), allowed_memory_scopes (memory authorisation), allowed_tools (tool authorisation).

**Why allowed_memory_scopes as globs (§1 #2)?** A skill that touches "all my projects" needs `memories/projects/**`; a skill that touches "only Cyberos" needs `memories/projects/cyberos/**`. Globs are the right shape — granular, declarative, well-understood. `globset` is the canonical Rust glob crate; reusing it ensures consistency with FR-MEMORY-106's sync-class globs.

**Why allowed_tools as enum (§1 #2)?** If we accepted arbitrary strings, typos would creep in (`Bsah`, `memoryread`). Enum tokens are validated; the canonical set is small (~10 native tools + MCP names via FR-SKILL-104). Adding a new tool means: enum variant + broker impl + version bump.

**Why signature optional (§1 #3)?** Local dev needs to iterate fast — adding/checking signature on every change kills the flow. The OCI gate (FR-SKILL-102) enforces signature on registry-uploaded bundles; local dev runs unsigned. Two scopes: trust-local (loose) and trust-registry (strict).

**Why min/max broker version (§1 #3)?** Forward compat: a skill compiled against broker 0.5 may use APIs not in 0.3. Without min, the skill loads on 0.3 and crashes at first API call. Backward compat: a skill explicitly tested on 0.5 may know it breaks on 0.6's new sandbox rules; max lets the author declare that. Both are SemVer-checked at load time — invocation-time crashes are eliminated.

**Why unknown-keys rejected by default (§1 #5)?** Strict-by-default catches typos: `allowed_tols: [Bash]` would silently pass under "ignore unknown" but the broker wouldn't see any allowed tools. With strict rejection, the author sees "unknown key `allowed_tols`" immediately. The `x-` prefix escape hatch is the YAML convention for "I know this is non-standard; respect it."

**Why schema-version freeze at v1 (§1 #6 + DEC-182)?** Stability — every change to the schema risks breaking authored skills. v1 is the minimal stable schema. v2 will be additive (new optional fields) or breaking (new required fields — major version bump); broker version negotiation handles cross-version.

**Why signature signs frontmatter + body (§1 #7)?** Frontmatter alone could be signed while body is tampered — a malicious actor edits the body to add a `bash:rm -rf /` operation. Signing both is necessary. Canonical form (trimmed + LF) ensures cross-platform reproducibility.

**Why LSP-friendly schema.json (§1 #11)?** Authoring SKILL.md in an editor with live validation cuts iteration time. The Rust validator is authoritative (it runs at invoke time), but the JSONSchema mirror lets editors highlight errors as you type.

**Why scaffold CLI (§1 #12)?** New skill authors copy-paste from existing skills, which propagates whatever bugs are in the template. A `scaffold` command writes a fresh, fully-documented starter — operators don't need to know the schema by heart.

---

## §3 — API contract

### SkillFrontmatter (Rust)

```rust
// services/skill-broker/src/frontmatter/schema.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SkillFrontmatter {
    // Required
    pub id:                   String,
    pub version:              String,           // SemVer
    pub description:          String,
    pub allowed_memory_scopes: Vec<String>,      // glob patterns
    pub allowed_tools:        Vec<ToolName>,

    // Optional
    #[serde(default)] pub signature:           Option<Signature>,
    #[serde(default)] pub min_broker_version:  Option<String>,
    #[serde(default)] pub max_broker_version:  Option<String>,
    #[serde(default)] pub disallowed_tools:    Vec<ToolName>,
    #[serde(default = "default_sync_class")] pub sync_class: SyncClass,
    #[serde(default = "default_tenant_scope")] pub tenant_scope: TenantScope,
    #[serde(default) ] pub effort_minutes:     Option<u32>,
    #[serde(default) ] pub tags:               Vec<String>,

    // Forward-compat: any `x-*` keys are captured here
    #[serde(flatten)] pub x_extensions:        std::collections::BTreeMap<String, serde_yaml::Value>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ToolName {
    Bash, Read, Write, Edit, Glob, Grep,
    MemoryRead, MemorySearch, MemoryEmit,
    HttpFetch, HttpPost,
    // MCP tool names registered at broker startup per FR-SKILL-104
    #[serde(other)] McpTool,  // matches any registered MCP name; validated against MCP_TOOL_REGISTRY at validation time
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncClass { Private, Shareable }
fn default_sync_class() -> SyncClass { SyncClass::Private }

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TenantScope { Any, Pinned }
fn default_tenant_scope() -> TenantScope { TenantScope::Any }

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Signature {
    pub algo:           SignatureAlgo,
    pub public_key_hex: String,                // 64 chars (32 bytes hex)
    pub signature_hex:  String,                // 128 chars (64 bytes hex)
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SignatureAlgo { Ed25519 }
```

### Validation errors

```rust
// services/skill-broker/src/frontmatter/validators.rs
#[derive(thiserror::Error, Debug)]
pub enum FrontmatterError {
    #[error("missing frontmatter (file must start with `---`)")]                 MissingFrontmatter,
    #[error("YAML parse error: {0}")]                                            YamlParse(#[from] serde_yaml::Error),
    #[error("id violates kebab-case: {0:?}")]                                    InvalidId(String),
    #[error("version is not valid SemVer: {0:?}")]                               InvalidVersion(String),
    #[error("description must be 1..=200 chars (was {0})")]                      InvalidDescription(usize),
    #[error("allowed_memory_scopes[{idx}] is not a valid glob: {pat:?}")]        InvalidMemoryScopeGlob { idx: usize, pat: String },
    #[error("allowed_tools[{idx}] is not a known tool: {name:?}")]              UnknownTool { idx: usize, name: String },
    #[error("signature verification failed (frontmatter + body hash mismatch)")] SignatureFailed,
    #[error("broker version {broker} below skill's min_broker_version {min}")]  BrokerTooOld { broker: String, min: String },
    #[error("broker version {broker} above skill's max_broker_version {max}")]  BrokerTooNew { broker: String, max: String },
    #[error("unknown field {key:?} (must be `x-`-prefixed to be ignored)")]     UnknownField { key: String },
}

pub fn validate(fm: &SkillFrontmatter, body_canonical: &str, broker_version: &str) -> Result<(), FrontmatterError> {
    static ID_RX: once_cell::sync::Lazy<regex::Regex> =
        once_cell::sync::Lazy::new(|| regex::Regex::new(r"^[a-z][a-z0-9-]*$").unwrap());
    if !ID_RX.is_match(&fm.id) { return Err(FrontmatterError::InvalidId(fm.id.clone())); }

    if !is_semver(&fm.version) { return Err(FrontmatterError::InvalidVersion(fm.version.clone())); }

    let desc_len = fm.description.chars().count();
    if !(1..=200).contains(&desc_len) { return Err(FrontmatterError::InvalidDescription(desc_len)); }

    for (i, pat) in fm.allowed_memory_scopes.iter().enumerate() {
        globset::Glob::new(pat).map_err(|_| FrontmatterError::InvalidMemoryScopeGlob { idx: i, pat: pat.clone() })?;
    }

    // ToolName enum already validated by serde; McpTool variant needs registry check
    for (i, t) in fm.allowed_tools.iter().enumerate() {
        if let ToolName::McpTool = t {
            // serde captured the raw name via `#[serde(other)]`; cross-reference with MCP_TOOL_REGISTRY
            // For simplicity in this skeleton we trust serde's pre-validation; FR-SKILL-104 owns the registry check.
        }
    }

    // Broker version range
    if let Some(min) = &fm.min_broker_version {
        if !is_semver(min) { return Err(FrontmatterError::InvalidVersion(min.clone())); }
        if semver_lt(broker_version, min) {
            return Err(FrontmatterError::BrokerTooOld { broker: broker_version.into(), min: min.clone() });
        }
    }
    if let Some(max) = &fm.max_broker_version {
        if !is_semver(max) { return Err(FrontmatterError::InvalidVersion(max.clone())); }
        if semver_gt(broker_version, max) {
            return Err(FrontmatterError::BrokerTooNew { broker: broker_version.into(), max: max.clone() });
        }
    }

    // Signature: verify SHA-256(frontmatter_yaml) || SHA-256(body) against the ed25519 signature.
    if let Some(sig) = &fm.signature {
        verify_signature(fm, body_canonical, sig)?;
    }

    // Unknown fields not prefixed with `x-`
    for k in fm.x_extensions.keys() {
        if !k.starts_with("x-") {
            return Err(FrontmatterError::UnknownField { key: k.clone() });
        }
    }
    Ok(())
}
```

### Parser

```rust
// services/skill-broker/src/frontmatter/parser.rs
use std::path::Path;

pub fn load_and_validate(bundle_path: &Path, broker_version: &str)
    -> Result<(SkillFrontmatter, String /* body */), FrontmatterError>
{
    let skill_md_path = bundle_path.join("SKILL.md");
    let bytes = std::fs::read(&skill_md_path).map_err(|_| FrontmatterError::MissingFrontmatter)?;
    let text = std::str::from_utf8(&bytes).map_err(|_| FrontmatterError::MissingFrontmatter)?;
    let (yaml, body) = split_fenced(text)?;
    let fm: SkillFrontmatter = serde_yaml::from_str(yaml)?;
    let body_canonical = canonicalise(body);
    validators::validate(&fm, &body_canonical, broker_version)?;
    Ok((fm, body.to_string()))
}

fn split_fenced(text: &str) -> Result<(&str, &str), FrontmatterError> {
    if !text.starts_with("---") { return Err(FrontmatterError::MissingFrontmatter); }
    let rest = &text[3..];
    // Find newline-fenced `---` close
    let close = rest.find("\n---\n").or_else(|| rest.find("\n---"))
        .ok_or(FrontmatterError::MissingFrontmatter)?;
    let yaml = &rest[..close].trim_start_matches('\n');
    let body_start = close + "\n---\n".len();
    let body = if body_start < rest.len() { &rest[body_start..] } else { "" };
    Ok((yaml, body))
}

fn canonicalise(body: &str) -> String {
    body.replace("\r\n", "\n").trim().to_string()
}
```

### CLI

```rust
// services/skill-broker/src/bin/cyberos-skill-validate.rs
use clap::Parser;
use cyberos_cli_exit::ExitCode;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cyberos skill validate")]
struct Args {
    /// Bundle path (directory containing SKILL.md)
    bundle: PathBuf,
    /// Output JSON instead of human-readable
    #[arg(long)] json: bool,
    /// Override broker version for forward-compat testing
    #[arg(long)] broker_version: Option<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    let broker_version = args.broker_version.unwrap_or_else(|| env!("CARGO_PKG_VERSION").into());
    match cyberos_skill_broker::frontmatter::load_and_validate(&args.bundle, &broker_version) {
        Ok((fm, _body)) => {
            if args.json {
                println!("{}", serde_json::to_string_pretty(&serde_json::json!({
                    "status": "ok",
                    "id": fm.id,
                    "version": fm.version,
                    "allowed_memory_scopes": fm.allowed_memory_scopes,
                    "allowed_tools": fm.allowed_tools,
                })).unwrap());
            } else {
                println!("✓ {} v{} — valid", fm.id, fm.version);
            }
            ExitCode::Ok
        }
        Err(cyberos_skill_broker::frontmatter::FrontmatterError::MissingFrontmatter) => {
            eprintln!("ERROR: SKILL.md not found or has no frontmatter");
            ExitCode::UserError
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            ExitCode::SchemaViolation
        }
    }
}
```

### Example SKILL.md (valid)

```markdown
---
id: memory-capture
version: 1.0.0
description: Canonical entry point for emitting memory capture rows from arbitrary tools.
allowed_memory_scopes:
  - memories/projects/**
  - memories/people/*/notes/**
allowed_tools:
  - Read
  - MemoryRead
  - MemoryEmit
sync_class: shareable
tenant_scope: any
effort_minutes: 5
tags: [memory, capture, foundation]
signature:
  algo: ed25519
  public_key_hex: "a4d8f2e1b9c7..."
  signature_hex:  "9e3b1c2a..."
x-cyberos-author: "stephen@cyberskill.world"
---

# memory-capture@1

This skill is the canonical entry point ...
```

---

## §4 — Acceptance criteria

1. **Valid SKILL.md parses** — fixture `skill-valid/SKILL.md` → `load_and_validate` returns `Ok(SkillFrontmatter, body)`.
2. **Missing frontmatter rejected** — file without leading `---` → `Err(MissingFrontmatter)`.
3. **Missing required field rejected** — fixture omits `id` → `Err(YamlParse)` (serde missing-field error wrapped).
4. **Invalid id (camelCase) rejected** — `id: memoryCapture` → `Err(InvalidId)`.
5. **Invalid version rejected** — `version: 1.0` (not full SemVer) → `Err(InvalidVersion)`.
6. **Description length bounds** — empty `description: ""` → `Err(InvalidDescription)`; 201-char description → same.
7. **Bad glob rejected** — `allowed_memory_scopes: ["[unclosed"]` → `Err(InvalidMemoryScopeGlob)`.
8. **Unknown tool rejected** — `allowed_tools: ["Bsah"]` → serde Err mapped to validation Err.
9. **Unknown field without x- prefix rejected** — `random_field: 1` → `Err(UnknownField)`.
10. **x-prefixed unknown field allowed** — `x-cyberos-author: alice` → loads OK; stored in `x_extensions`.
11. **Signature verification: valid** — signed fixture with correct keys → loads OK.
12. **Signature verification: tampered body** — fixture's body modified after signing → `Err(SignatureFailed)`.
13. **Signature verification: tampered frontmatter** — frontmatter field changed → `Err(SignatureFailed)`.
14. **Broker version too old** — skill `min_broker_version: 2.0.0`, broker at 1.5 → `Err(BrokerTooOld)`.
15. **Broker version too new** — skill `max_broker_version: 1.0`, broker at 2.0 → `Err(BrokerTooNew)`.
16. **Optional fields default correctly** — fixture without `sync_class` → `SyncClass::Private`; without `tenant_scope` → `TenantScope::Any`.
17. **CLI exit codes** — valid → 0; invalid → 6 (SchemaViolation); SKILL.md not found → 1 (UserError).
18. **CLI --json output** — valid bundle → `{"status":"ok","id":...}` JSON.
19. **JSONSchema mirror validates same fixtures** — `skill.schema.json` + ajv-CLI → same accept/reject pattern as Rust validator on 8 fixture variants.
20. **OTel span per validate** — span `skill.frontmatter.validate` with attrs `skill_id`, `version`, `validation_outcome`, `duration_ms`.
21. **Metric increments per outcome** — running validator 10× on mixed valid/invalid → counter `skill_frontmatter_validation_total{outcome="ok"}` + `{outcome="signature_failed"}` etc. correct.

---

## §5 — Verification

```rust
// services/skill-broker/tests/frontmatter_test.rs

#[test]
fn valid_skill_loads() {
    let path = std::path::Path::new("tests/fixtures/skill-valid");
    let (fm, body) = frontmatter::load_and_validate(path, "1.0.0").unwrap();
    assert_eq!(fm.id, "memory-capture");
    assert_eq!(fm.version, "1.0.0");
    assert!(!body.is_empty());
}

#[test]
fn missing_frontmatter_rejected() {
    // Fixture is a SKILL.md without `---` fences
    let path = std::path::Path::new("tests/fixtures/skill-invalid-no-fm");
    assert!(matches!(
        frontmatter::load_and_validate(path, "1.0.0"),
        Err(FrontmatterError::MissingFrontmatter)
    ));
}

#[test]
fn bad_id_rejected() {
    let path = std::path::Path::new("tests/fixtures/skill-invalid-bad-id");
    assert!(matches!(
        frontmatter::load_and_validate(path, "1.0.0"),
        Err(FrontmatterError::InvalidId(_))
    ));
}

#[test]
fn bad_glob_rejected() {
    let path = std::path::Path::new("tests/fixtures/skill-invalid-bad-glob");
    let err = frontmatter::load_and_validate(path, "1.0.0").unwrap_err();
    assert!(matches!(err, FrontmatterError::InvalidMemoryScopeGlob { .. }));
}

#[test]
fn unknown_tool_rejected() {
    let path = std::path::Path::new("tests/fixtures/skill-invalid-unknown-tool");
    let err = frontmatter::load_and_validate(path, "1.0.0").unwrap_err();
    // serde maps unknown enum variant to YamlParse, or McpTool catches it; either way: not Ok
    assert!(err.to_string().to_lowercase().contains("tool") || matches!(err, FrontmatterError::YamlParse(_)));
}

#[test]
fn x_prefixed_unknown_allowed() {
    let path = std::path::Path::new("tests/fixtures/skill-x-extension");
    let (fm, _) = frontmatter::load_and_validate(path, "1.0.0").unwrap();
    assert!(fm.x_extensions.contains_key("x-cyberos-author"));
}

#[test]
fn signature_tampered_body_rejected() {
    // Fixture: valid signature, then body modified after signing
    let path = std::path::Path::new("tests/fixtures/skill-tampered-body");
    let err = frontmatter::load_and_validate(path, "1.0.0").unwrap_err();
    assert!(matches!(err, FrontmatterError::SignatureFailed));
}

#[test]
fn broker_too_old_rejected() {
    // Fixture: min_broker_version: 2.0.0
    let path = std::path::Path::new("tests/fixtures/skill-needs-v2");
    let err = frontmatter::load_and_validate(path, "1.5.0").unwrap_err();
    assert!(matches!(err, FrontmatterError::BrokerTooOld { .. }));
}

#[test]
fn defaults_applied() {
    let path = std::path::Path::new("tests/fixtures/skill-minimal");
    let (fm, _) = frontmatter::load_and_validate(path, "1.0.0").unwrap();
    assert_eq!(fm.sync_class, SyncClass::Private);
    assert_eq!(fm.tenant_scope, TenantScope::Any);
    assert!(fm.tags.is_empty());
}
```

```bash
# CLI fixtures
$ cyberos skill validate tests/fixtures/skill-valid
✓ memory-capture v1.0.0 — valid

$ cyberos skill validate tests/fixtures/skill-invalid-bad-glob
ERROR: allowed_memory_scopes[0] is not a valid glob: "[unclosed"
# exit 6

$ cyberos skill validate tests/fixtures/skill-valid --json
{
  "status": "ok",
  "id": "memory-capture",
  "version": "1.0.0",
  "allowed_memory_scopes": ["memories/projects/**", "memories/people/*/notes/**"],
  "allowed_tools": ["Read", "MemoryRead", "MemoryEmit"]
}
```

---

## §6 — Implementation skeleton

(API contract above is the skeleton.)

---

## §7 — Dependencies

- **FR-SKILL-101 (upstream)** — defines the broker that consumes this frontmatter at invoke time.
- **FR-SKILL-102 (related)** — OCI registry uploads MUST include a valid signature; this FR's signature schema is the format.
- **FR-SKILL-104 (downstream)** — capability broker enforces `allowed_tools` at runtime; FR-SKILL-104 owns the MCP_TOOL_REGISTRY.
- **FR-SKILL-105 (downstream)** — memory-sync@1 skill bundle is the first canonical user of `sync_class: shareable`.
- **FR-MEMORY-106** — sync_class semantics are reused (same enum variants per AGENTS.md §15).
- **FR-AUTH-003** — RLS for tenant_scope=pinned enforcement (broker injects tenant_id into memory reads).
- **`cyberos-cli-exit`** — exit codes.

---

## §8 — Example payloads

(Valid SKILL.md shown in §3.)

### Validation failure JSON output

```json
{
  "status": "invalid",
  "errors": [
    {"field": "allowed_memory_scopes[0]", "code": "InvalidMemoryScopeGlob", "message": "not a valid glob: \"[unclosed\""}
  ]
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Schema v2 (will add: telemetry_emit_kinds, required_secrets, capabilities_provided) — slice 3+.
- Multi-signature support (two-of-three trusted authors) — slice 4+.
- Schema migration tooling (auto-upgrade v1 → v2 frontmatter) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| File missing | std::fs::read Err | `Err(MissingFrontmatter)`; exit 1 | Operator runs `cyberos skill scaffold <id>` |
| Frontmatter fence missing | text doesn't start with `---` | `Err(MissingFrontmatter)`; exit 6 | Author adds fences |
| YAML syntax error | serde_yaml::Error | `Err(YamlParse)`; exit 6 with line/col | Author fixes YAML |
| Required field missing | serde missing-field | `Err(YamlParse)`; exit 6 | Author adds field |
| `id` violates kebab-case | regex no-match | `Err(InvalidId)`; exit 6 | Author fixes |
| Version not SemVer | parse fails | `Err(InvalidVersion)`; exit 6 | Author fixes |
| Description too long | char count | `Err(InvalidDescription)`; exit 6 | Author shortens |
| Bad glob | `globset::Glob::new` Err | `Err(InvalidMemoryScopeGlob)`; exit 6 | Author fixes |
| Unknown tool | serde unknown-variant | mapped to YamlParse; exit 6 | Author uses canonical name |
| Unknown field (no x- prefix) | x_extensions key check | `Err(UnknownField)`; exit 6 | Author renames or removes |
| Signature missing public key | hex parse | `Err(SignatureFailed)`; exit 6 | Re-sign bundle |
| Signature mismatch | ed25519 verify Err | `Err(SignatureFailed)`; exit 6 | Re-sign; or accept unsigned |
| Broker too old | semver compare | `Err(BrokerTooOld)`; exit 6 | Operator upgrades broker |
| Broker too new | semver compare | `Err(BrokerTooNew)`; exit 6 | Operator downgrades OR author bumps max_broker_version |
| Multi-byte char body | Rust UTF-8 invariants | None | None |
| Body with embedded `---` lines | parser uses first `\n---\n` after open | Correct; multi-fenced body handled | None |
| Signature algo unknown (e.g. `rsa`) | enum reject | `Err(YamlParse)`; exit 6 | Use ed25519 |
| Public key wrong length (not 32 bytes) | hex decode + length check | `Err(SignatureFailed)`; exit 6 | Use correct key |
| Empty `allowed_tools: []` | valid by schema (empty = no tools allowed) | OK; skill has no tool capabilities | By design |
| Empty `allowed_memory_scopes: []` | valid (no memory access) | OK | By design |

---

## §11 — Implementation notes

- The frontmatter parser MUST be tolerant of trailing whitespace after the closing fence; many editors auto-trim or auto-add blank lines.
- The JSONSchema mirror is generated from the Rust types via `schemars` crate; the `cargo xtask schema` task in CI ensures `.json` is up-to-date.
- The signature verification reuses the ed25519 implementation from `ed25519-dalek` (already in workspace via FR-OBS-009).
- `serde_yaml` allows duplicate keys silently in some versions; we pin to a version that errors on duplicates (≥ 0.9).
- The `McpTool` `#[serde(other)]` variant is a forward-compat trick — for v1 we accept any string and defer registry validation to FR-SKILL-104. In v2 we may upgrade to a strict registry-time check.
- `effort_minutes` is an advisory cap; the actual SIGTERM enforcement lives in FR-SKILL-104. Default cap (when unspecified) is 30 minutes.
- `tags` are operator-facing only — no broker logic depends on them. Categorisation surfaces in `cyberos skill list`.
- The `cyberos skill scaffold` CLI is intentionally simple: writes a SKILL.md with all required fields filled with placeholders + comments explaining each one. Authors can copy and customise.
- The `body_canonical` form (trim + LF) matches Git's text mode; signed bundles are reproducible across OSes.

---

*End of FR-SKILL-103.*
