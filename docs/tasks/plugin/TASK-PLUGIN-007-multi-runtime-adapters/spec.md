---
id: TASK-PLUGIN-007
title: "Multi-runtime adapters — cyberos-plugin pack --target {claude-code,cursor,cowork,codex-cli} emitters; deferred targets in P2"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-19T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: PLUGIN
priority: p0
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PLUGIN-001, TASK-PLUGIN-002, TASK-PLUGIN-003, TASK-PLUGIN-004, TASK-PLUGIN-005, TASK-PLUGIN-006, TASK-PLUGIN-008]
depends_on: [TASK-PLUGIN-002, TASK-PLUGIN-004, TASK-PLUGIN-005]
blocks: [TASK-PLUGIN-008]

source_pages:
  - modules/plugin/README.md §3 (adapters/ folder)
  - modules/plugin/INTEROP.md target runtimes matrix

source_decisions:
  - DEC-2460 2026-05-19 — Four P1 targets ship in v1: claude-code, cursor, cowork, codex-cli. P2 targets (goose, amp, continue-dev) deferred to successor task
  - DEC-2461 2026-05-19 — Each adapter is its own Rust module at services/plugin-host/src/adapters/<target>/ exposing a single pack(canonical_manifest, out_dir) -> Bundle function
  - DEC-2462 2026-05-19 — Adapters MUST consume the SAME canonical manifest from modules/plugin/manifests/ — no per-target manifest files; differences expressed at adapter pack time only
  - DEC-2463 2026-05-19 — Adapter outputs are reproducible per TASK-PLUGIN-001 clause 10 — same canonical manifest → same target bundle hash for a given target
  - DEC-2464 2026-05-19 — Adapter MUST include MCP bridge binary (cyberos-mcp-bridge) inside the bundle for targets that consume MCP locally (cursor, codex-cli, cowork); claude-code references the binary via cwd-relative path
  - DEC-2465 2026-05-19 — Adapter MUST omit playbooks (skills/) from Cursor bundle per TASK-PLUGIN-004 clause 8 — Cursor's MCP integration does not render Skills
  - DEC-2466 2026-05-19 — Adapter MUST emit a per-target manifest format conforming to that target's published spec — adapters do NOT extend or proprietarily modify target formats

build_envelope:
  language: rust 1.81
  service: services/plugin-host/
  new_files:
    - services/plugin-host/src/adapters/mod.rs
    - services/plugin-host/src/adapters/claude_code.rs
    - services/plugin-host/src/adapters/cursor.rs
    - services/plugin-host/src/adapters/cowork.rs
    - services/plugin-host/src/adapters/codex_cli.rs
    - services/plugin-host/src/adapters/common.rs
    - services/plugin-host/src/bin/cyberos-plugin.rs (extends Python CLI from TASK-PLUGIN-001 with Rust pack subcommand for multi-target)
    - services/plugin-host/tests/adapter_claude_code_test.rs
    - services/plugin-host/tests/adapter_cursor_test.rs
    - services/plugin-host/tests/adapter_cowork_test.rs
    - services/plugin-host/tests/adapter_codex_cli_test.rs
    - services/plugin-host/tests/adapter_reproducibility_test.rs

  modified_files:
    - services/plugin-host/Cargo.toml (add binary entry for cyberos-plugin)
    - modules/plugin/manifests/cyberos@1.0.0.plugin.json (targets array)

  allowed_tools:
    - file_read: services/plugin-host/**, modules/plugin/**
    - file_write: services/plugin-host/**
    - bash: cd services && cargo test -p cyberos-plugin-host adapter

  disallowed_tools:
    - per-target canonical manifests (per DEC-2462)
    - extend target formats with proprietary fields (per DEC-2466)
    - leak runtime context into bundle (per DEC-2463)

effort_hours: 10
subtasks:
  - "0.5h: adapters/mod.rs trait + types"
  - "0.5h: adapters/common.rs reproducibility helpers"
  - "1.5h: adapters/claude_code.rs (plugin.json + commands + skills + bridge binary ref)"
  - "1.5h: adapters/cursor.rs (.mcp.json with stdio transport; skip commands + skills)"
  - "1.5h: adapters/cowork.rs (Customize manifest.json + commands + skills + bridge over HTTP)"
  - "1.0h: adapters/codex_cli.rs (top-level SKILL.md + env var binding)"
  - "0.5h: bin/cyberos-plugin.rs (Rust CLI dispatching adapters)"
  - "3.0h: 5 test files (per-target + reproducibility)"

risk_if_skipped: "Without multi-runtime adapters, the plugin only works in one runtime (likely Claude Code) — Strategy §4 Level 1 ecosystem-distribution stalls. Without DEC-2462 single canonical manifest, authors maintain N manifests in parallel and they drift. Without DEC-2463 reproducibility, Sigstore signing breaks per-target. Without DEC-2464 bridge-bundled, runtime users must install the bridge separately — adoption friction kills it. Without DEC-2465 Cursor playbook omission, Cursor renders broken skill UI. Without DEC-2466 target-format conformance, target hosts reject the bundle on install."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** ship multi-runtime adapters at `services/plugin-host/src/adapters/<target>/`. Each adapter transforms the canonical CyberOS manifest (per TASK-PLUGIN-001) into a target-runtime-native bundle. P1 targets: claude-code, cursor, cowork, codex-cli.

1. **MUST** ship adapters for all four P1 targets per DEC-2460:
   - **`claude-code`** — produces `.plugin` zip with `plugin.json` at root + `commands/*.md` + `skills/*` + `bin/cyberos-mcp-bridge`
   - **`cursor`** — produces `.mcp.json` single-file config pointing at `cyberos-mcp-bridge` stdio transport; omits commands + skills per DEC-2465
   - **`cowork`** — produces Customize-slot zip with `manifest.json` + `commands/*.md` + `skills/*` + HTTP-transport bridge config
   - **`codex-cli`** — produces top-level `SKILL.md` + nested `skills/` subfolders + env var binding for MCP

2. **MUST** consume the SAME canonical manifest per DEC-2462. The CLI `cyberos-plugin pack --target <name>` reads `modules/plugin/manifests/<id>@<version>.plugin.json` once and dispatches to the target adapter. Authors do NOT write per-target manifests.

3. **MUST** be reproducible per DEC-2463. For a given canonical manifest + target, two pack invocations produce identical SHA-256 bundle hashes. Adapters MUST use the reproducibility helpers in `adapters/common.rs` (epoch mtimes, sorted entries, fixed permissions) — same as TASK-PLUGIN-001 clause 10.

4. **MUST** include the MCP bridge binary `cyberos-mcp-bridge` inside the bundle for cursor + cowork + codex-cli per DEC-2464. Claude Code references the binary via cwd-relative path because Claude Code manages binary lifecycle separately. Each adapter MUST select the right architecture binary (`x86_64-linux-musl`, `aarch64-darwin`, `x86_64-windows-msvc`) from a CDN-published artefact set.

5. **MUST** omit `skills/` directory from the Cursor bundle per DEC-2465. Cursor's MCP integration surfaces tools but does NOT render Skills. Shipping skills/ bloats the bundle without benefit and may confuse users.

6. **MUST** emit bundles conforming to each target's published format per DEC-2466:
   - claude-code: `.plugin` format per Anthropic Claude Code plugin spec (zip with manifest at root)
   - cursor: `.mcp.json` format per Cursor MCP integration docs
   - cowork: Customize slot format per Anthropic Cowork docs
   - codex-cli: Anthropic Agent Skills SKILL.md format (single SKILL.md at root + nested skills/)

7. **MUST** translate the canonical manifest's `tools[]` to target-appropriate shape:
   - claude-code: tools nested under `mcp_servers[0].tools[]`
   - cursor: tools live in `command` config; Cursor introspects via MCP `tools/list` at runtime
   - cowork: tools under `connectors[0].tools[]`
   - codex-cli: tools surface via MCP server env-var; SKILL.md describes via prose

8. **MUST** translate `commands[]` and `skills[]` per target capability:
   - claude-code: copy `commands/*.md` into `.claude/commands/`; copy `skills/*` into `.claude/skills/`
   - cursor: omit both
   - cowork: copy commands into `commands/`; copy skills into `skills/`
   - codex-cli: convert commands into supplementary SKILL.md files (one per command); copy skills as-is

9. **MUST** validate target name against the manifest's `targets[]` array — if author packs `--target cursor` but manifest doesn't list `cursor` in `targets[]`, the pack fails with a clear error. This prevents shipping bundles for targets the author didn't intend.

10. **MUST** emit a bundle-level Sigstore signature distinct per target per TASK-PLUGIN-001 clause 8. The same canonical manifest produces 4 distinct signatures (one per target bundle).

11. **MUST** include adapter-specific test coverage at `services/plugin-host/tests/adapter_<target>_test.rs` validating: bundle structure, manifest translation, tools-list translation, reproducibility, and absence of disallowed content (e.g. skills in cursor bundle).

12. **MUST NOT** introduce proprietary extensions to target formats per DEC-2466. Adapter output MUST be loadable in the standard host without custom patches.

13. **MUST NOT** require authors to maintain per-target manifests per DEC-2462.

14. **MUST NOT** leak machine/runtime context (cwd, env vars, mtimes) into bundles per DEC-2463.

---

## §2 — Why this design

**Why exactly 4 P1 targets (DEC-2460)?** Each P1 target reflects a real 2026 user base: Claude Code (Anthropic users), Cursor (developer base), Cowork (Anthropic non-developer base), Codex CLI (OpenAI users adopting SKILL.md format). Goose / Amp / Continue.dev are smaller, more niche; ship in P2 once core targets validate the adapter pattern.

**Why single canonical manifest (DEC-2462, clause 2)?** Drift between per-target manifests is the #1 source of bug reports in multi-runtime ecosystems. Single source of truth + adapter-time translation eliminates the drift class entirely. Authors edit one file; adapters compute the rest.

**Why reproducibility per target (DEC-2463, clause 3)?** Sigstore Rekor anchors the bundle hash. If the same source produces different bundles per build, the signature is unverifiable. Reproducibility makes "build it yourself, verify the hash" work — a credibility-builder for OSS distribution (Strategy §4 Level 1).

**Why bundle the bridge for most targets (DEC-2464, clause 4)?** Cursor + Codex CLI + Cowork users expect a one-step install. If they have to "also install the bridge separately," adoption drops. Claude Code has its own binary lifecycle (`claude-code plugin install` handles dependencies), so bundling there would be wasteful.

**Why omit playbooks from Cursor (DEC-2465, clause 5)?** Cursor's MCP integration as of 2026 surfaces tools to its built-in agent (which has its own routing). Skills are an Anthropic-spec concept; Cursor doesn't read them. Shipping ~12 SKILL.md files in the Cursor bundle is bloat. The canonical manifest still has `skills[]`; the Cursor adapter drops them.

**Why per-target signatures (clause 10)?** Each bundle has a different byte stream → different hash → different signature. A single signature on the canonical manifest doesn't bind to the per-target bundle. Per-target signing is mandatory for verifier round-trip.

**Why architecture-specific binaries (clause 4)?** CyberOS users run macOS (Intel + ARM), Linux (x86_64), Windows (x86_64). Adapter picks the right binary at pack time based on the target architecture metadata. CDN-published artefacts let the adapter not bake-in the binary at compile time.

**Why test per-target (clause 11)?** Each target's format is the conformance bar. Generic tests can't catch "Cursor rejected this because port field name was wrong." Per-target tests embed the target's expectations.

**Why no proprietary extensions (DEC-2466, clause 12)?** A bundle that needs a CyberOS-only patch is not really portable. Strategy §4 hinges on "open-standard" positioning. Extensions undermine the claim.

**Why adapter-target-list cross-check (clause 9)?** A bundle packed for a target not in the manifest's `targets[]` array is a manifest bug — author forgot to update. Catching at pack time saves a publish-side rejection.

---

## §3 — API contract

### Adapter trait

```rust
// services/plugin-host/src/adapters/mod.rs
#[async_trait]
pub trait Adapter: Send + Sync {
    fn target_name(&self) -> &'static str;
    fn pack(&self, manifest: &CanonicalManifest, out_dir: &Path) -> Result<PackedBundle>;
}

pub struct PackedBundle {
    pub target: &'static str,
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
    pub included_tools: u32,
    pub included_commands: u32,
    pub included_skills: u32,
    pub binary_included: bool,
}
```

### Adapter selection

```rust
// services/plugin-host/src/bin/cyberos-plugin.rs
fn select_adapter(target: &str) -> Result<Box<dyn Adapter>> {
    match target {
        "claude-code" => Ok(Box::new(ClaudeCodeAdapter)),
        "cursor"      => Ok(Box::new(CursorAdapter)),
        "cowork"      => Ok(Box::new(CoworkAdapter)),
        "codex-cli"   => Ok(Box::new(CodexCliAdapter)),
        "goose"|"amp"|"continue-dev" => Err(Error::DeferredTarget(target.to_string())),
        _ => Err(Error::UnknownTarget(target.to_string())),
    }
}
```

### Claude Code bundle layout

```
cyberos-1.0.0.claude-code.plugin (zip)
├── plugin.json                    (target-specific manifest)
├── README.md                      (rendered in Claude Code's plugin detail view)
├── .claude/
│   ├── commands/                  (from manifest commands[])
│   │   ├── cyberos-run.md
│   │   ├── cyberos-memory.md
│   │   ├── cyberos-skill-list.md
│   │   └── cyberos-route.md
│   └── skills/                    (from manifest skills[])
│       ├── run-cuo-workflow/SKILL.md
│       └── ... (11 more)
└── (no bridge binary — claude-code manages separately)
```

### Cursor bundle layout

```
cyberos-1.0.0.cursor.mcp.json (single file)
{
  "mcpServers": {
    "cyberos": {
      "command": "/path/to/cyberos-mcp-bridge",
      "args": ["--transport", "stdio"],
      "env": {
        "CYBEROS_AUTH_ENDPOINT": "https://auth.cyberskill.world",
        "CYBEROS_MEMORY_ENDPOINT": "https://memory.cyberskill.world"
      }
    }
  }
}
```
(Bridge binary shipped alongside as `cyberos-mcp-bridge.<arch>`; user moves to PATH.)

### Cowork bundle layout

```
cyberos-1.0.0.cowork.zip
├── manifest.json                   (Customize slot manifest)
├── README.md
├── commands/
│   └── ... (4 commands)
├── skills/
│   └── ... (12 skills)
└── bin/
    ├── cyberos-mcp-bridge.x86_64-linux-musl
    ├── cyberos-mcp-bridge.aarch64-darwin
    └── cyberos-mcp-bridge.x86_64-windows-msvc
```

### Codex CLI bundle layout

```
cyberos-1.0.0.codex-cli/  (folder, not zipped)
├── SKILL.md                        (top-level: routes to cyberos.* tools via MCP env var)
├── README.md
├── skills/
│   ├── run-cuo-workflow/SKILL.md
│   └── ... (11 more)
└── bin/cyberos-mcp-bridge.<arch>
```

---

## §4 — Acceptance criteria

1. **`pack --target claude-code` emits .plugin zip** — output exists, valid zip.
2. **claude-code bundle has plugin.json at root** — zip listing shows `plugin.json` entry.
3. **claude-code bundle has .claude/commands/ with 4 entries** — zip listing.
4. **claude-code bundle has .claude/skills/ with 12 entries** — zip listing.
5. **claude-code bundle does NOT include bridge binary** — no `bin/cyberos-mcp-bridge*` entries.
6. **`pack --target cursor` emits .mcp.json** — output is single file, JSON.
7. **cursor .mcp.json validates against Cursor spec** — schema check.
8. **cursor bundle does NOT include commands or skills** — bundle is single file.
9. **`pack --target cowork` emits zip with HTTP transport** — manifest.json has `mcp_url: "..."` not stdio path.
10. **cowork bundle includes bridge binary** — zip listing has `bin/cyberos-mcp-bridge.*`.
11. **`pack --target codex-cli` emits folder with SKILL.md root** — folder structure correct.
12. **codex-cli SKILL.md has frontmatter with `name: cyberos`** — frontmatter check.
13. **Reproducibility: two packs same target = same SHA-256** — test runs pack twice, asserts equal.
14. **Cross-target SHA-256 differs** — pack target A and target B; hashes differ (different content).
15. **`pack --target goose` fails with DeferredTarget error** — adapter selector explicit.
16. **`pack --target bogus` fails with UnknownTarget error** — adapter selector explicit.
17. **`pack --target cursor` on manifest without cursor in targets[]` fails** — clause 9 check.
18. **claude-code adapter copies commands verbatim** — content hash equal to source.
19. **codex-cli adapter converts commands to supplementary SKILL.md files** — count matches commands count.
20. **cowork adapter uses HTTP transport not stdio** — manifest.json has HTTP URL.
21. **All bundles' Sigstore signatures verify** — `cosign verify` on each bundle passes.
22. **Cursor bundle works in Cursor (smoke)** — install Cursor adapter output → MCP tool list shows 8 tools.

---

## §5 — Verification

```rust
// services/plugin-host/tests/adapter_claude_code_test.rs
#[test]
fn claude_code_bundle_structure() {
    let manifest = load_canonical_manifest("cyberos@1.0.0");
    let result = ClaudeCodeAdapter.pack(&manifest, &tmp_out_dir()).unwrap();
    let zip = zip::ZipArchive::new(File::open(&result.path).unwrap()).unwrap();
    let names: Vec<String> = zip.file_names().map(String::from).collect();
    assert!(names.contains(&"plugin.json".to_string()));
    let claude_cmds = names.iter().filter(|n| n.starts_with(".claude/commands/")).count();
    assert_eq!(claude_cmds, 4);
    let claude_skills = names.iter().filter(|n| n.starts_with(".claude/skills/")).count();
    assert!(claude_skills >= 12); // 12 SKILL.md + 12 TRIGGER_TESTS.md
    let bin_entries = names.iter().filter(|n| n.starts_with("bin/")).count();
    assert_eq!(bin_entries, 0, "claude-code should NOT include bridge binary");
}
```

```rust
// services/plugin-host/tests/adapter_cursor_test.rs
#[test]
fn cursor_bundle_is_single_mcp_json() {
    let manifest = load_canonical_manifest("cyberos@1.0.0");
    let result = CursorAdapter.pack(&manifest, &tmp_out_dir()).unwrap();
    assert!(result.path.ends_with("cyberos-1.0.0.mcp.json"));
    assert_eq!(result.included_commands, 0);
    assert_eq!(result.included_skills, 0);
    let content: serde_json::Value = serde_json::from_str(&fs::read_to_string(&result.path).unwrap()).unwrap();
    assert!(content["mcpServers"]["cyberos"]["command"].is_string());
    assert!(content["mcpServers"]["cyberos"]["args"].as_array().unwrap().contains(&json!("--transport")));
}
```

```rust
// services/plugin-host/tests/adapter_cowork_test.rs
#[test]
fn cowork_bundle_uses_http_transport() {
    let manifest = load_canonical_manifest("cyberos@1.0.0");
    let result = CoworkAdapter.pack(&manifest, &tmp_out_dir()).unwrap();
    let zip = zip::ZipArchive::new(File::open(&result.path).unwrap()).unwrap();
    let manifest_bytes = read_zip_file(&zip, "manifest.json");
    let m: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
    assert!(m["connectors"][0]["mcp_url"].as_str().unwrap().starts_with("http"));
}

#[test]
fn cowork_bundle_includes_bridge_binary() {
    let result = CoworkAdapter.pack(&load_canonical_manifest("cyberos@1.0.0"), &tmp_out_dir()).unwrap();
    let zip = zip::ZipArchive::new(File::open(&result.path).unwrap()).unwrap();
    let names: Vec<String> = zip.file_names().map(String::from).collect();
    assert!(names.iter().any(|n| n.starts_with("bin/cyberos-mcp-bridge")));
}
```

```rust
// services/plugin-host/tests/adapter_codex_cli_test.rs
#[test]
fn codex_cli_emits_top_level_skill_md() {
    let result = CodexCliAdapter.pack(&load_canonical_manifest("cyberos@1.0.0"), &tmp_out_dir()).unwrap();
    let skill_md = result.path.join("SKILL.md");
    assert!(skill_md.exists());
    let raw = fs::read_to_string(&skill_md).unwrap();
    assert!(raw.starts_with("---\n"));
    let fm: serde_yaml::Value = serde_yaml::from_str(extract_frontmatter(&raw)).unwrap();
    assert_eq!(fm["name"].as_str().unwrap(), "cyberos");
}
```

```rust
// services/plugin-host/tests/adapter_reproducibility_test.rs
#[test]
fn pack_is_reproducible_per_target() {
    let manifest = load_canonical_manifest("cyberos@1.0.0");
    for adapter in [Box::new(ClaudeCodeAdapter) as Box<dyn Adapter>,
                    Box::new(CursorAdapter), Box::new(CoworkAdapter), Box::new(CodexCliAdapter)] {
        let r1 = adapter.pack(&manifest, &tmp_out_dir()).unwrap();
        let r2 = adapter.pack(&manifest, &tmp_out_dir()).unwrap();
        assert_eq!(r1.sha256, r2.sha256, "target {} not reproducible", adapter.target_name());
    }
}

#[test]
fn cross_target_hashes_differ() {
    let manifest = load_canonical_manifest("cyberos@1.0.0");
    let cc = ClaudeCodeAdapter.pack(&manifest, &tmp_out_dir()).unwrap();
    let cu = CursorAdapter.pack(&manifest, &tmp_out_dir()).unwrap();
    assert_ne!(cc.sha256, cu.sha256);
}
```

---

## §6 — Implementation skeleton

(Bundle layouts in §3 + adapter trait are the skeleton. Each adapter is ~150-300 Rust lines.)

---

## §7 — Dependencies

- **Upstream:** TASK-PLUGIN-002 (bridge binary referenced in 3 of 4 adapter bundles); TASK-PLUGIN-004 (skills folder layout); TASK-PLUGIN-005 (auth manifest section translation).
- **Downstream:** TASK-PLUGIN-008 (marketplace publish reads target list per bundle).
- **Cross-module:** TASK-PLUGIN-003 (commands surface per adapter); TASK-PLUGIN-006 (audit endpoint per adapter target).

---

## §8 — Example payloads

### Claude Code `plugin.json` (after translation from canonical)

```json
{
  "name": "cyberos",
  "version": "1.0.0",
  "description": "Persona-aware orchestration + memory + skills for any agentic IDE.",
  "mcp_servers": [{
    "name": "cyberos",
    "command": "./bin/cyberos-mcp-bridge",
    "args": ["--transport", "stdio"],
    "tools": [/* 8 tools verbatim */]
  }]
}
```

### Cowork Customize `manifest.json` (after translation)

```json
{
  "schema_version": "1.0.0",
  "id": "cyberos",
  "version": "1.0.0",
  "name": "CyberOS",
  "connectors": [{
    "type": "mcp",
    "mcp_url": "http://127.0.0.1:8082/mcp",
    "transport": "http"
  }],
  "commands": [/* 4 entries */],
  "skills": [/* 12 entries */]
}
```

### Codex CLI top-level `SKILL.md`

```markdown
---
name: cyberos
description: >
  CyberOS persona-aware orchestration + memory + skills. Use when the user wants to "execute a CUO workflow",
  "look up the audit trail", "find a CyberOS skill", or "route a query to the right persona".
license: Apache-2.0
---

This skill provides 8 CyberOS tools via MCP. Refer to skills/ for granular routing playbooks.

## Setup

Ensure `CYBEROS_MCP_BRIDGE_PATH` points at the bundled `bin/cyberos-mcp-bridge` binary.
Authenticate once via `cyberos-plugin auth login` (browser-based OAuth-PKCE).

## Tools available

(see skills/*/SKILL.md for the playbook map)
```

---

## §9 — Open questions

All resolved.

- ~~Should the adapter chain produce a single multi-target bundle (one zip with all targets)?~~ → No, one bundle per target per clause 1. Multi-target zip complicates install and signature verification.
- ~~Should we ship adapter for VS Code's MCP integration?~~ → VS Code uses Continue.dev for MCP; deferred to P2 successor task.
- ~~Should the codex-cli adapter convert each command into a separate skill (1:1)?~~ → Yes for now (per clause 8); revisit when Codex CLI's command surface ships.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Unknown target name | adapter selector | UnknownTarget error | Author uses one of 4 P1 targets |
| Deferred target name | adapter selector | DeferredTarget error | Wait for P2 successor task or contribute |
| Target not in manifest.targets[] | pack-time cross-check | error: "manifest doesn't include target" | Author updates manifest |
| Bridge binary not found for target arch | filesystem check | error: "binary for x86_64-linux-musl not in CDN cache" | Run pre-fetch step OR fall back to download-at-pack-time |
| Reproducibility broken | adapter test | CI fails | Investigate which adapter helper leaks state |
| Bundle exceeds target size limit | adapter size guard | error | Trim large skills body content; image assets |
| Sigstore signing fails | sign step | adapter error | Operator inspects Sigstore credentials |
| Target adapter has bug | per-target test | CI fails | Author fixes adapter |
| Canonical manifest changes mid-pack | file modtime check | error: "manifest changed during pack" | Re-run pack |
| Cursor bundle includes skills | adapter test | CI fails | Adapter logic bug |
| Codex-cli bundle missing SKILL.md root | adapter test | CI fails | Adapter logic bug |
| Cowork bundle uses stdio | adapter test | CI fails | Adapter logic bug |
| Adapter omits commands when target supports them | adapter test | CI fails | Adapter logic bug |
| Adapter introduces proprietary field | manual code review at PR | reviewer rejects | Strip the extension |
| Per-target bundles have same hash | adapter test | CI fails | One of the adapters is broken |

---

## §11 — Implementation notes

- §11.1 **Adapter implementation pattern.** Each adapter has 3 phases: (a) `prepare()` — read canonical manifest, validate target compatibility; (b) `emit()` — produce target-specific files in scratch dir; (c) `seal()` — zip + sign + compute hash. Common helpers in `adapters/common.rs::seal_reproducible_zip`.

- §11.2 **Binary selection.** Adapters that bundle binary use a `bin_cache` directory pre-populated by CI from CDN URLs `https://cdn.cyberskill.world/binaries/cyberos-mcp-bridge/{version}/{target}`. Pack failure if any required arch missing.

- §11.3 **CDN binary cache.** CI build job uploads built binaries to CDN at every release. Adapter downloads on demand if cache miss. SHA-256 of binary is pinned in canonical manifest's `binary_pins` map (added in task-PLUGIN-007a if needed).

- §11.4 **Claude Code bundle.** `.plugin` files are zips per Anthropic Claude Code spec. Bundle root has `plugin.json`. Nested `.claude/commands/` and `.claude/skills/` are read by Claude Code at install.

- §11.5 **Cursor `.mcp.json`.** Single file written to user's `.cursor/mcp.json` location at install. Cursor reads at startup. The file has no commands or skills — Cursor doesn't render those. Tool list is discovered via MCP `tools/list` at runtime.

- §11.6 **Cowork manifest.** Customize slots accept zip uploads. Manifest at root has `connectors[]` array; each connector entry declares MCP transport URL. HTTP transport is used because Cowork runs in-cloud (no local stdio).

- §11.7 **Codex CLI.** As of late 2025, OpenAI's Codex CLI adopted Anthropic Agent Skills SKILL.md format. Bundle is a folder (not zip) per spec. Top-level SKILL.md is the entrypoint; nested skills/ are routing playbooks. Codex CLI auto-discovers via filesystem scan.

- §11.8 **Why no zip for codex-cli.** Codex CLI's discovery is filesystem-based — users symlink the bundle folder into their skills directory. Zipping would force an unzip step. Folder is the right unit.

- §11.9 **Multi-arch single bundle.** Cursor + Cowork + Codex bundles include 3 binaries (Linux x86_64, macOS arm64, Windows x86_64) by default. Adapter has a `--arch` flag to ship single-arch bundles for size-sensitive distribution. Default is multi-arch.

- §11.10 **Why no goose / amp / continue-dev in v1.** Each adds ~200 LoC + tests + format learning cost. Validating the 4-adapter pattern in v1 lets us evolve confidently. Successor task (task-PLUGIN-007a) covers P2 targets.

---

*End of TASK-PLUGIN-007 spec.*
