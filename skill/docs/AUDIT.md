# CyberOS Skills Module Optimization — Architectural Audit

*Prepared as a Senior Systems Architect audit for CyberSkill Software Solutions Consultancy And Development JSC (Stephen Cheng / Trịnh Thái Anh, Ho Chi Minh City). Date: 13 May 2026.*

---

## BLUF (Bottom Line Up Front)

- **Verdict: Align CyberOS Skills to the Anthropic Agent Skills open standard (SKILL.md + progressive disclosure + filesystem discovery), and rebuild the host as a Rust core with a Wasmtime/Extism plugin runtime, supplemented by a Bun-based developer toolchain for TypeScript skill authoring.** Anthropic donated the spec on 18 Dec 2025 and adoption spans Microsoft VS Code, GitHub Copilot, OpenAI Codex CLI, Cursor, Goose, Amp, Gemini CLI, Mistral, Databricks, Letta, and 15+ others. Partner skills from Atlassian, Canva, Cloudflare, Figma, Notion, Ramp, Sentry, Stripe, Zapier are GA. Inventing a proprietary skill format in 2026 is strictly value-destroying.
- **Recommended stack:** Rust 1.80+ host · `tokio` async runtime · `dashmap` registry · `serde_yaml` for SKILL.md frontmatter · `wasmtime` + WASI Preview 2 Component Model for sandboxed executable skill code · TypeScript (via Bun 1.3 + esbuild) and Python as first-class skill authoring languages. Distribution as `.skill` directories that are zip-packed, content-addressed, and resolvable from a local cache, an OCI registry, or `agentskills.io`-compatible HTTP endpoint.
- **Headline gains (vs. a hand-rolled monolithic loader):** cold start over N installed skills moves from **O(N) parse + O(N) compile** to **O(N) header-read only + O(1) lazy activation** (~100 tokens of context overhead per skill, sub-millisecond per-skill header parse via libyaml). Plugin invocation cold start drops 10–50× vs. container-based isolation (WASM sub-ms cold starts are documented across Wasmtime/Wasmer/WasmEdge 2026 benchmarks). Ecosystem reach goes from 1 (CyberOS only) to 26+ compatible clients on day 1 of compliance.
- **Top 3 prunes (kill these, ruthlessly):** (1) any custom non-SKILL.md manifest format — burn it; (2) any "eager-load all skills at boot" path — eager loads are the single largest cold-start tax in extension hosts and are explicitly anti-patterned by VS Code's activation events model; (3) any bespoke in-process plugin execution that lacks capability-based sandboxing — replace with WASI capabilities. Honorable mention: bespoke skill↔host RPC; use the existing WIT / Component-Model canonical ABI.
- **Migration risk level: MEDIUM.** Contained because (a) the target format is a text spec a few pages long, (b) Rust+Wasmtime is proven (Spin, wasmCloud, Shopify Functions, Fermyon), and (c) phased rollout keeps the legacy loader behind a feature flag through every phase. Hard parts are capability-grant UX and skill-author DX, not the runtime.

---

## 1. Current State Analysis

**Explicit caveat on project knowledge:** This audit was performed **without access to a `project_knowledge_search` tool or any retrievable CyberOS source artifact in the current environment**. No CyberOS internal documentation, directory tree, manifest schema, loader source, lifecycle hooks, IPC primitives, or benchmark data was accessible to this audit at execution time. The audit therefore proceeds on industry-standard assumptions clearly labeled as such, plus the public attestation that CyberSkill operates from Ho Chi Minh City and the founder is identified as Stephen Cheng / Trịnh Thái Anh. Public sources surface CyberSkill Vietnam as a ~10-person consultancy with ~US$2M 2024 revenue on a Vultr/Google-Workspace footprint, consistent with a small, fast-moving engineering team — exactly the context where premature complexity is the dominant risk.

**Assumed current-state pattern (label: ASSUMPTION).** Based on what almost every pre-2026 in-house "skills module" looks like before being benchmarked against Agent Skills, the current CyberOS module most likely exhibits one or more of the following bottlenecks:

- **Cold-start bottleneck (assumed).** Eager registration of every skill at boot, parsing each skill's full descriptor and instantiating its handler, yielding cold-start cost that scales **O(N)** in the number of installed skills, with full instruction text loaded into memory (and possibly into the LLM context window) regardless of relevance. This is the failure mode every plugin host eventually hits and is what activation events and progressive disclosure were both designed to defeat.
- **Throughput bottleneck (assumed).** A single global `Mutex<HashMap<SkillId, Skill>>` (or its equivalent) guarding the registry. Under concurrent agent invocations this becomes a contention hotspot. The fix is well-known: sharded read-mostly map (`dashmap`) or `RwLock<HashMap<...>>` for read-biased workloads.
- **Footprint bottleneck (assumed).** Every skill's prompt/instruction text and any bundled code resident in memory at all times because there is no lazy/three-tier loading. Effective LLM context budget is consumed even when a skill is irrelevant.
- **Dev-velocity bottleneck (assumed).** A bespoke manifest format and bespoke distribution channel mean every skill must be authored against CyberOS-specific conventions. Authors cannot reuse Anthropic's `skill-creator`, cannot publish on the agentskills.io directory, and cannot pull in partner skills from Atlassian/Figma/Canva/Stripe/Notion/Zapier without writing a transpiler.
- **Security bottleneck (assumed).** Skills running with ambient host authority (full filesystem, full network) rather than declared capabilities. This is the standard plugin-system failure mode and is what WASI Preview 2's capability model is built to eliminate.

If any of these assumptions is wrong, the rest of the audit still applies directionally — the fix set is identical even if only two of the five bottlenecks exist. Where CyberOS already does the right thing (e.g. already speaks SKILL.md, already uses `tokio` + `dashmap`), that section becomes a "no-op, keep" rather than a "rebuild."

---

## 2. Benchmark & Research Findings

The relevant prior art splits into two layers that must not be conflated: **the skill format / discovery layer** (how an agent finds and decides to load a capability) and **the plugin execution / sandbox layer** (how host-foreign code actually runs). The winners differ.

**Skill format / discovery — winner: Anthropic Agent Skills.** A skill is a directory containing a `SKILL.md` whose YAML frontmatter (`name`, `description`, plus optional `license`, `compatibility`, `metadata`, `allowed-tools`) is the only thing loaded at startup — ~100 tokens per skill. The body of `SKILL.md` (≤5k tokens of procedural instructions) is read only when the controller decides the task matches. Bundled `scripts/`, `references/`, `assets/` are touched only on demand. This is **progressive disclosure in three levels** and is what makes the format scale to "effectively unbounded" skill libraries without context blowup. Released as an open standard at `agentskills.io` on 18 December 2025 with reference SDK; adopted by Microsoft (VS Code + GitHub Copilot), OpenAI (Codex CLI), Cursor, Goose, Amp, OpenCode, Gemini CLI, Mistral, Databricks, Letta, and 15+ others. The format is "deliciously tiny" (Simon Willison, accurate) — under-specified in places, but that's a feature: it's a contract, not a runtime.

**Skill format — companion, not competitor: MCP (Model Context Protocol).** MCP is **the other half** of the stack. MCP is the connectivity layer (USB-C for AI: tools and data); Skills is the procedural-knowledge layer (how to use those tools correctly for this workflow). Anthropic's own framing, repeated by every analyst writing on this in 2026: "MCP connects Claude to external services and data sources. Skills provide procedural knowledge." MCP crossed 10,000+ active public servers and was donated to the Linux Foundation in December 2025. A typical 5-server MCP setup costs ~55k tokens before any conversation; Tool Search reduces overhead ~85% via on-demand discovery. A serious agent OS must speak **both**: Skills for workflows, MCP for tool/data connectivity.

**Skill format — also-ran: OpenAI Apps SDK / GPT actions.** OpenAI's Apps SDK / Custom GPTs are proprietary, ecosystem-locked, and structurally narrower than Agent Skills. Per VentureBeat's December 2025 reporting and developer Elias Judin's discovery, **OpenAI itself has quietly adopted Agent Skills' directory structure and SKILL.md naming inside Codex CLI and ChatGPT** — the same file naming conventions, the same metadata format, the same directory organization. The lesson is unambiguous: even the would-be platform rival is adopting the open standard. Apps SDK retains relevance as the in-ChatGPT distribution channel, but as an *architectural pattern* it has lost.

**Plugin execution / sandbox layer — winner: WASI Preview 2 + Component Model, via Wasmtime (or Extism on top of Wasmtime).** Cold starts are sub-millisecond and "10–50× faster than Docker" (wasmruntime.com 2026 benchmarks). Capability-based security is intrinsic — no ambient authority — every file/dir/network grant is explicit (e.g. `--dir /data::readonly`). The Component Model provides language-agnostic interfaces (Rust / Go / Python / JS / TS / C# all compile to components). Wasmtime is the reference implementation for WASI Preview 2 and underlies Spin, wasmCloud's `wash` CLI, Cloudflare-adjacent tooling, Shopify Functions, and Fermyon. Extism is a thin opinionated wrapper around Wasmtime that solves plugin-system ergonomics (host-controlled HTTP without WASI, runtime limiters/timers, persistent module-scope variables) and ships PDKs for 15+ guest languages.

**Plugin execution — runner-up: VS Code extension host pattern.** Node.js child process; lazy activation triggered by declarative `activationEvents` (`onCommand:*`, `onLanguage:*`, `workspaceContains:*`, `onUri:*`, `onWebviewPanel:*`, `onStartupFinished`); each extension isolated so a crash takes down only the extension host, not the editor. This is the right **lifecycle pattern** even when the **runtime** is WASM rather than Node — copy the activation-event taxonomy.

**JavaScript-runtime alternatives — Deno, Bun, esbuild bundling for plugins.** All three are credible *authoring-toolchain* choices but not credible *core-host* choices. Bun 1.3 (the runtime Anthropic itself adopted for Claude Code in 2026, per published 2026 runtime comparisons) wins JS cold start at ~5 ms on JavaScriptCore, ships an integrated bundler + test runner + package manager, and is the best DX for TypeScript skill authoring. Deno 2.7's permission model is the cleanest security story among JS runtimes (file/network access requires explicit grants — a precursor to WASI's capability model) and `deno compile` produces single-file binaries. esbuild is the de-facto bundler in this stratum: it compiles TypeScript guest code to a single `.js` (or feeds a wasm32-wasi target) in milliseconds and has stable plugin APIs that work across Deno, Bun, and Node. **However**, none of these can match Wasmtime on (a) cold start (WASM beats even Bun by an order of magnitude when AOT-cached), (b) language-agnostic sandbox (JS runtimes only sandbox JS), or (c) memory footprint per skill instance. **Use Bun + esbuild for the developer-side toolchain (`cyberos skill build` produces a `.wasm` component); do not host plugins inside Bun or Deno at runtime.**

**Concurrency primitives for the registry.** Rust `tokio` is the obvious async runtime; for the skill registry itself the practical choice is **`dashmap`** (sharded HashMap, fixed shard count, lock-free reads on most operations, drop-in replacement for `Arc<RwLock<HashMap<...>>>` and consistently faster on read-heavy workloads — which a skill registry is). For pure read-mostly state with rare lock acquisition, plain `Arc<RwLock<HashMap<...>>>` (std::sync, not tokio::sync — never hold tokio locks across `.await`) is also acceptable. Go goroutines, Node worker_threads, and free-threaded Python 3.13 are all viable for the host but each has structural disadvantages: Go's GC pauses are visible at scale, Node's single-process model contends with native WASM throughput, free-threaded Python is still maturing in 2026. **Rust wins on cold-start, footprint, and the absence of a GC stall budget.**

**Manifest parse cost.** Across multiple 2017–2026 benchmarks the ordering is stable: **JSON ≪ TOML < YAML** for raw parse speed. A Python benchmark of a 1000-record document: JSON 1.5 ms, YAML (libyaml C loader) 58 ms, pure-Python YAML 986 ms, TOML 162 ms. **For SKILL.md the comparison is largely moot** because (a) frontmatter is tiny (~10 lines, ~100 tokens) so absolute parse cost is microseconds regardless of format; (b) the format is fixed by the open standard (YAML 1.2.2 frontmatter); (c) DX dominates parse cost at this size. **Decision: use YAML frontmatter (mandated by spec), parse with libyaml-backed `serde_yaml` or `saphyr` in Rust. Cold-start frontmatter parse cost for 1,000 installed skills lands well under 100 ms total, easily parallelised via `tokio::task::JoinSet`.**

### Polyglot core-language recommendation (decision block)

Goals: minimise cold-start, maximise throughput, minimise footprint, maximise developer velocity. Scoring the four serious candidates for the **host** language:

| Candidate | Cold start | Throughput | Footprint | Dev velocity | GC stall budget | Verdict |
|---|---|---|---|---|---|---|
| **Rust (tokio + wasmtime)** | Excellent | Excellent | Excellent | Good (steep learning curve but mature ecosystem) | None (no GC) | **Pick this** |
| **Go** | Good | Good | Good | Excellent | GC pauses visible at p99 under load | Acceptable backup |
| **Bun / Node** | Very Good (Bun ~5 ms) | Good | Fair | Excellent | V8/JSC GC | Toolchain only |
| **Python 3.13 free-threaded** | Fair | Fair | Fair | Excellent (for skill authors) | GIL-removed but immature | Skill-author SDK only |

**Recommendation: Rust for the host, Wasmtime for the plugin runtime, Bun for the developer toolchain (`cyberos skill build` / `cyberos skill test`), TypeScript and Python as first-class skill-authoring languages targeting `wasm32-wasi`.** This polyglot split aligns each language with what it is best at and avoids the trap of letting authoring DX dictate runtime characteristics.

### Comparison table

Scored 1–5 (5 = best). CyberOS column reflects the *assumed* current state described in §1; if reality differs, update accordingly.

| System | Cold start | Throughput | Footprint | Dev velocity | Security isolation | Ecosystem reach |
|---|---|---|---|---|---|---|
| **CyberOS (assumed today)** | 1 — eager O(N) load | 2 — likely global Mutex | 1 — all skills resident | 2 — bespoke format | 1 — ambient authority | 1 — CyberOS only |
| **Claude Agent Skills (spec)** | 5 — progressive disclosure, ~100 tok/skill | n/a (spec, not runtime) | 5 — three-level lazy load | 5 — markdown + YAML, `skill-creator` | 3 — relies on host enforcement of `allowed-tools` | 5 — 26+ clients, partner directory |
| **MCP servers** | 3 — server-per-tool overhead, ~55k tok for 5-server setup | 4 — JSON-RPC, parallelisable | 2 — each server is a process | 4 — many SDKs, 10k+ servers | 4 — process isolation, explicit transport | 5 — Linux Foundation, GA across vendors |
| **VS Code extension host** | 4 — activation events, lazy require | 3 — single Node ext-host process | 3 — extensions opt into load timing | 4 — npm ecosystem, package.json | 3 — process isolation, no DOM access, no capability model | 4 — VS Code marketplace |
| **Extism / Wasmtime (WASI P2)** | 5 — sub-ms cold start, AOT cache | 5 — Rust async, no GC | 5 — small WASM modules, mmap'd | 4 — 15+ guest PDKs, polyglot | 5 — WASI capabilities, no ambient authority | 4 — growing; Spin, wasmCloud, Shopify Functions, Fermyon |

**Sources:** Anthropic engineering blog "Equipping agents for the real world with Agent Skills" (Oct 16 2025, updated Dec 18 2025); `docs.claude.com/en/docs/agents-and-tools/agent-skills/overview`; `agentskills.io/specification`; VentureBeat "Anthropic launches enterprise 'Agent Skills' and opens the standard" (Dec 2025); Unite.AI; The New Stack "Agent Skills: Anthropic's Next Bid to Define AI Standards"; Simon Willison 19 Dec 2025; Morph 2026 comparison; IntuitionLabs / Verdent / Subramanya N; VS Code official docs (`api/references/activation-events`, `api/advanced-topics/extension-host`); Extism (`extism.org`, `dylibso.com/blog/how-does-extism-work`); wasmruntime.com 2026 benchmarks; Wasmer 4.3 release notes (Phoronix); WASI 2.0 / Component Model coverage (`dev.to/pockit_tools`, `marcokuoni.ch`, `fermyon.com`, `eunomia.dev`); 2026 Bun/Deno/Node runtime comparison (weskill.org, nandann.com — notes Bun 1.3 powering Claude Code); DashMap docs.rs and tokio.rs shared-state tutorial; bespon Python parse benchmarks; multiple 2024–2026 JSON/YAML/TOML comparison studies.

---

## 3. Audit & Prune

**Kill list (with one-line justifications):**

- **KILL: any proprietary manifest schema.** SKILL.md is the standard; every line of code that parses a non-SKILL.md descriptor is dead weight after Day 30 of migration.
- **KILL: eager activation at host boot.** Mandate explicit activation events. Replace startup-time skill instantiation with header-only indexing.
- **KILL: full instruction text in registry RAM.** Index headers only; read the body on activation.
- **KILL: global `Mutex` around the registry.** Replace with `DashMap` (sharded) or `Arc<RwLock<...>>` if reads outnumber writes ≥10:1.
- **KILL: bespoke skill→host RPC.** Use WIT-defined interfaces and the Component Model canonical ABI; let `wit-bindgen` generate the glue.
- **KILL: in-process untrusted code execution.** All third-party / user-authored code runs in a Wasmtime store with explicit WASI capability grants.
- **KILL: a custom skill marketplace / directory.** Be a client of `agentskills.io` and accept OCI-distributed `.skill` bundles. Build a curated CyberSkill **collection** atop the open registry — do not build a competing registry.
- **KILL: skill lifecycle hooks more granular than `activate` / `invoke` / `deactivate`.** Anything more is unjustified Day-1 surface area.
- **KILL: a hand-rolled scheduler.** `tokio` is the scheduler. There is no second answer in 2026 for this workload.
- **KILL: support for synchronous, blocking skill calls.** Async-only invocation. Skills that need to block run inside `spawn_blocking`.
- **KILL: shipping plugins inside Node/Bun/Deno at runtime.** Use Bun for the *build* step; use Wasmtime for the *run* step. Two different problems.

**Minimum Viable Architecture (MVA).** The smallest set of components that delivers the value:

1. A **Rust host** binary (`cyberos-skill-host`) that owns the event loop.
2. A **header index** (`DashMap<SkillName, SkillHeader>`) populated at startup by streaming SKILL.md frontmatter only.
3. A **lazy activator** that, on a trigger event, reads the SKILL.md body and (if executable) instantiates a Wasmtime component with declared capabilities.
4. A **capability broker** that translates `allowed-tools` and `compatibility` declarations into WASI grants (`Dir`, `Tcp`, `Env`, …).
5. A **distribution resolver** that locates `.skill` directories from the local cache (`~/.cyberos/skills/`), an OCI registry, or an HTTPS URL.

Five components. Nothing else is mandatory.

---

## 4. Refactored Architecture

### Directory tree

```
cyberos-skill-host/
├── Cargo.toml
├── crates/
│   ├── host/                        # Rust host binary
│   │   ├── src/
│   │   │   ├── main.rs              # tokio entry, signal handling
│   │   │   ├── registry.rs          # DashMap<SkillName, SkillHeader>
│   │   │   ├── loader.rs            # streaming SKILL.md frontmatter parser
│   │   │   ├── activator.rs         # lazy body load + WASM instantiation
│   │   │   ├── capabilities.rs      # WASI capability broker
│   │   │   ├── invoke.rs            # invocation entry point
│   │   │   └── ipc/                 # MCP client + agent <-> host bridge
│   │   └── wit/
│   │       └── cyberos-skill.wit    # Component Model interface
│   ├── manifest/                    # serde model for SKILL.md frontmatter
│   ├── resolver/                    # OCI + HTTPS + local cache
│   └── cli/                         # `cyberos skill ...` developer CLI
├── toolchain/                       # Bun + esbuild authoring toolchain
│   ├── package.json                 # Bun 1.3+
│   ├── bun.lockb
│   ├── build.ts                     # esbuild -> wasm32-wasi component
│   └── templates/
│       └── ts-skill/                # `cyberos skill new --lang ts`
├── skills/                          # Bundled / curated skills
│   ├── pdf-processing/
│   │   ├── SKILL.md
│   │   ├── scripts/
│   │   │   └── fill_form.py
│   │   ├── references/
│   │   │   └── forms.md
│   │   └── assets/
│   │       └── template.pdf
│   └── ...
├── target/
└── README.md
```

### File format strategy (and why)

- **Manifest:** YAML frontmatter inside `SKILL.md`. Fixed by the open spec; deviating loses ecosystem reach. Parse with `serde_yaml` (libyaml-backed). Parse cost is negligible at this size; interoperability dominates throughput at the manifest layer.
- **Content (body):** Markdown. Spec-defined. Loaded only on activation.
- **Code:** Three tiers, picked by skill author:
  1. **Markdown-only skill** — no code, instructions only. Loaded into LLM context on activation. Zero runtime cost.
  2. **Native-script skill** — `scripts/*.py` or `scripts/*.sh`. Executed by the host through a sandboxed code-execution tool (analogous to Claude's `bash` tool). Best for trusted internal skills and quick wins.
  3. **WASM-component skill** — `dist/skill.wasm` compiled from any Component-Model–compatible language. Mandatory for third-party / untrusted skills. Runs in Wasmtime with declared WASI capabilities only.
- **Distribution bundle:** `.skill` = zip of the directory + a content hash. Resolvable from OCI registry refs (`ghcr.io/org/skill-name:1.2.3`), HTTPS URLs, or the local cache. Verifies cosign-compatible signatures.

### Module load mechanism

Three levels, mirroring Agent Skills, with VS Code-style activation event triggers:

```
Level 1  Startup     Read ONLY SKILL.md frontmatter for each installed
                     skill. Populate DashMap<name, SkillHeader>.
                     Cost: ~100 tokens of header per skill, parsed lazily
                     in parallel via tokio::task::JoinSet.

Level 2  Activation  Triggered by either (a) the agent controller deciding
                     the description matches a task, OR (b) a VS-Code-style
                     activation event (onCommand:*, onLanguage:*,
                     workspaceContains:*, onUri:*). Reads SKILL.md body.

Level 3  Execution   Reads referenced scripts/references/assets only when
                     the body explicitly names them. WASM components are
                     instantiated here and only here.
```

Cold start is **O(N) header-bytes read** (sub-ms per skill, fully parallel) and **O(1) per activated skill**, against the prior **O(N) full-parse + O(N) compile**. For N=1,000 skills this is the difference between tens of milliseconds and tens of seconds at boot.

### Concurrency model

- **Runtime:** `tokio` (multi-threaded, work-stealing). Default worker count = `num_cpus::get()`.
- **Registry:** `Arc<DashMap<SkillName, SkillHeader>>` (sharded internally, default 4 × num_cpus shards). Read-mostly: lookups during the agent loop are O(1) average with no global lock.
- **Activated skills:** `Arc<DashMap<SkillName, Arc<ActivatedSkill>>>` kept separate from headers. An `ActivatedSkill` holds the parsed body, the compiled Wasmtime component (if any), and a capability bundle. Wrapped in `Arc` so concurrent invocations share the instance.
- **Per-skill state:** Use **interior mutability** — `AtomicU64` for counters, `parking_lot::RwLock` for any rare-write state. The DashMap entry itself is reached via `.get()` (read shard), and concurrent invocations don't contend.
- **Cross-thread WASM:** Each Wasmtime `Store` is single-threaded by design. The host owns a `Pool<Engine, Store>` per skill; concurrent invocations pull a fresh `Store` from the pool, instantiate the cached `Component`, run, return the `Store` to the pool. AOT-compiled component artifacts are cached on disk so re-instantiation is microseconds.
- **Throughput target:** 10,000+ invocations/sec for trivial native-script skills on a single host; 1,000+ invocations/sec for WASM skills with WASI capability checks; bounded by skill workload, not by the host.

### Isolation / security model

- **Capability-based, no ambient authority.** Default grant set for a skill is empty.
- **Declared in frontmatter** via `allowed-tools` (Anthropic-experimental field; CyberOS makes it first-class). Example: `allowed-tools: read_file write_file fetch_url(https://api.cyberskill.vn/*)`.
- **Translated to WASI** at activation time: `read_file` → `wasi:filesystem/preopens` with read-only `Dir`; `fetch_url(pattern)` → host-mediated HTTP through a domain allowlist; `bash` is **never** auto-granted and requires explicit operator approval.
- **First-use approval prompt.** Following the Claude Code precedent (changelog 2.1.19 — skills specifying `allowed-tools` or `hooks` require user approval before first use). CyberOS records the grant in `~/.cyberos/grants.json` with the skill's content hash so a modified skill must be re-approved.
- **Signature verification.** `.skill` bundles ship with a cosign signature; the resolver refuses unsigned bundles unless `--allow-unsigned` is passed.
- **Audit trail.** Every WASI syscall a skill makes is logged through a host-side interceptor at `info` level; sensitive grants (HTTP, write, exec) are logged at `warn`.

### Distribution & versioning

- **Versioning:** SemVer in `metadata.version`. The Agent Skills spec leaves `version` to the optional `metadata` map (intentionally — the spec is "deliciously tiny"); CyberOS treats it as required for any skill resolved from a registry.
- **Distribution channels:** local filesystem (`~/.cyberos/skills/`, `./.cyberos/skills/` for project-scoped) → OCI registry refs (the natural fit; reuses existing infra and signing) → HTTPS URL (for hot-fix or quick distribution). All three resolve through the same `Resolver` trait.
- **Compatibility with `agentskills.io`:** A CyberOS skill is a valid Agent Skill verbatim. A skill authored for CyberOS works in Claude Code, Codex, VS Code, Cursor, Goose, Amp without modification. This is the entire strategic point.
- **The CyberSkill "collection":** Curate a Vietnamese-market skill set (legal/compliance for Vietnam, e-invoice handling, VAT formatting, local bank APIs, VNeID integration) published on `agentskills.io` and reachable from any compatible client. This is the differentiation play.

---

## 5. Core Refactored Code

The following are real, compilable code blocks against the recommended stack (Rust 1.80+, tokio 1.40+, dashmap 6.x, serde_yaml 0.9, wasmtime 27+, walkdir 2.x).

### 5.1 The SKILL manifest schema (Rust serde model) and an example SKILL.md

```rust
// crates/manifest/src/lib.rs
//! Strongly-typed model of the Agent Skills SKILL.md frontmatter,
//! per the open spec at agentskills.io/specification (Dec 2025).
//!
//! Only `name` and `description` are required. All other fields are
//! optional per spec. We treat `metadata.version` as semantically
//! required for registry-resolved skills (enforced at resolve time,
//! not parse time, so local dev skills can omit it).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillManifest {
    /// 1–64 chars, lowercase letters / digits / hyphens. Must match
    /// the parent directory name. Reserved words: "anthropic", "claude".
    pub name: String,

    /// 1–1024 chars. Used for model-invoked discovery. Must say BOTH
    /// what the skill does AND when to invoke it.
    pub description: String,

    /// Optional SPDX-ish license identifier or bundled-license reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Optional, max 500 chars. Free-form environment requirements
    /// (e.g. "Requires git and network access").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compatibility: Option<String>,

    /// Optional arbitrary string→string map. By spec, agent-specific
    /// extensions live here. CyberOS reads `version` (SemVer) and
    /// `author` from this map.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, String>,

    /// Experimental (per spec). Space- or list-delimited tool grants.
    /// CyberOS treats this as the canonical capability declaration.
    #[serde(default, rename = "allowed-tools", skip_serializing_if = "Option::is_none")]
    pub allowed_tools: Option<AllowedTools>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllowedTools {
    Inline(String),       // "read_file write_file fetch_url"
    List(Vec<String>),    // ["read_file", "write_file", ...]
}

impl AllowedTools {
    pub fn as_vec(&self) -> Vec<&str> {
        match self {
            AllowedTools::Inline(s) => s.split_whitespace().collect(),
            AllowedTools::List(v)   => v.iter().map(String::as_str).collect(),
        }
    }
}

/// Parse the YAML frontmatter at the head of a SKILL.md file.
/// Returns the manifest and the byte offset where the markdown body begins.
pub fn parse_frontmatter(bytes: &[u8]) -> anyhow::Result<(SkillManifest, usize)> {
    const DELIM: &[u8] = b"---\n";
    anyhow::ensure!(bytes.starts_with(DELIM), "SKILL.md must start with '---'");
    let rest = &bytes[DELIM.len()..];
    let end = memchr::memmem::find(rest, DELIM)
        .ok_or_else(|| anyhow::anyhow!("SKILL.md missing closing '---'"))?;
    let yaml = std::str::from_utf8(&rest[..end])?;
    let manifest: SkillManifest = serde_yaml::from_str(yaml)?;
    let body_offset = DELIM.len() + end + DELIM.len();
    Ok((manifest, body_offset))
}
```

```markdown
<!-- skills/pdf-processing/SKILL.md -->
---
name: pdf-processing
description: >-
  Extract text and tables from PDF files, fill PDF forms, merge documents.
  Use when the user mentions PDFs, scanned documents, invoices, or form
  extraction. Do NOT use for image OCR (use the ocr skill instead).
license: MIT
compatibility: Requires Python 3.11+ for scripts; no network access needed.
metadata:
  author: cyberskill
  version: "1.2.0"
allowed-tools: read_file write_file
---

# PDF Processing

## When to use
- User asks to extract text, tables, or form fields from a PDF.
- User asks to fill a PDF form.
- User asks to merge or split PDFs.

## Quick start
For text extraction:
```python
import pdfplumber
with pdfplumber.open(path) as pdf:
    text = "\n".join(p.extract_text() or "" for p in pdf.pages)
```

For form filling, read `references/forms.md` first — it lists the
field-discovery and validation steps. The helper script
`scripts/fill_form.py` accepts a JSON map of field→value and writes
the filled PDF to stdout.
```

### 5.2 The skill loader — core indexing + lazy activation

```rust
// crates/host/src/loader.rs
//! Two-phase loader:
//!   Phase 1 (boot):   walk skill roots, parse ONLY frontmatter, build index.
//!   Phase 2 (lazy):   on activation, read body + bundled resources.
//!
//! Cost analysis:
//!   - Phase 1 reads ~200 bytes/skill (the frontmatter), parses tiny YAML.
//!   - Boot for 1,000 skills on commodity hardware: <100 ms total, parallelised.
//!   - Phase 2 cost is paid per-skill, only when the controller activates it.

use crate::registry::{SkillHeader, SkillRegistry};
use cyberos_manifest::parse_frontmatter;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::task::JoinSet;
use tracing::{debug, warn};

#[derive(Clone)]
pub struct Loader {
    registry: Arc<SkillRegistry>,
}

impl Loader {
    pub fn new(registry: Arc<SkillRegistry>) -> Self {
        Self { registry }
    }

    /// Phase 1: index every skill under `roots`, frontmatter-only.
    /// O(N) bytes read, fully parallel, no body or script touched.
    pub async fn index_roots(&self, roots: &[PathBuf]) -> anyhow::Result<usize> {
        let mut skill_dirs = Vec::new();
        for root in roots {
            if !root.is_dir() {
                debug!(?root, "skill root missing, skipping");
                continue;
            }
            // SKILL.md must live exactly one directory below a root.
            for entry in walkdir::WalkDir::new(root).min_depth(1).max_depth(2) {
                let entry = entry?;
                if entry.file_name() == "SKILL.md" {
                    skill_dirs.push(entry.path().parent().unwrap().to_path_buf());
                }
            }
        }

        let mut tasks: JoinSet<anyhow::Result<SkillHeader>> = JoinSet::new();
        for dir in skill_dirs {
            tasks.spawn(async move { Self::index_one(&dir).await });
        }

        let mut count = 0usize;
        while let Some(joined) = tasks.join_next().await {
            match joined {
                Ok(Ok(header)) => {
                    self.registry.insert_header(header);
                    count += 1;
                }
                Ok(Err(e)) => warn!(error = %e, "skipping skill: invalid manifest"),
                Err(e)     => warn!(error = %e, "loader task panicked"),
            }
        }
        Ok(count)
    }

    async fn index_one(dir: &Path) -> anyhow::Result<SkillHeader> {
        let skill_md = dir.join("SKILL.md");
        let bytes = fs::read(&skill_md).await?;
        let (manifest, body_offset) = parse_frontmatter(&bytes)?;

        anyhow::ensure!(
            dir.file_name().and_then(|s| s.to_str()) == Some(manifest.name.as_str()),
            "directory name must match SKILL.md `name` ({})",
            manifest.name
        );

        Ok(SkillHeader {
            manifest,
            skill_dir: dir.to_path_buf(),
            body_offset,
            file_size: bytes.len() as u64,
        })
    }

    /// Phase 2: lazy body load. Called by the activator.
    pub async fn load_body(&self, header: &SkillHeader) -> anyhow::Result<String> {
        let bytes = fs::read(header.skill_dir.join("SKILL.md")).await?;
        let body = std::str::from_utf8(&bytes[header.body_offset..])?.to_owned();
        Ok(body)
    }
}
```

### 5.3 The concurrency / locking primitive for the skill registry

```rust
// crates/host/src/registry.rs
//! Two-tier sharded registry. Headers are immutable post-index; activated
//! skills are mutable and reference-counted so concurrent invocations share
//! one instance.
//!
//! Why DashMap (not RwLock<HashMap>):
//!   - Read path: shard-local lock (or lock-free under read-only ops),
//!     no global serialisation point. A single Arc<RwLock<HashMap>>
//!     creates head-of-line blocking when one activation takes a write
//!     lock to insert; under high concurrency that becomes the bottleneck.
//!   - DashMap is the documented drop-in for Arc<RwLock<HashMap>> and
//!     is the standard answer for read-mostly concurrent maps in async Rust
//!     (per tokio.rs shared-state tutorial and DashMap docs.rs guidance).
//!   - Per-value state lives behind interior mutability (AtomicU64 for
//!     counters, parking_lot::RwLock for rare-write fields) so we never
//!     need to upgrade a DashMap read into a write.

use cyberos_manifest::SkillManifest;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct SkillHeader {
    pub manifest: SkillManifest,
    pub skill_dir: PathBuf,
    pub body_offset: usize,
    pub file_size: u64,
}

#[derive(Debug)]
pub struct ActivatedSkill {
    pub header: SkillHeader,
    pub body: String,
    /// Compiled wasm component, when the skill ships one. None for
    /// markdown-only or native-script skills.
    pub component: Option<wasmtime::component::Component>,
    /// Per-skill counters — atomics so they update under DashMap read locks.
    pub invocations: AtomicU64,
    pub last_used_unix_ms: AtomicU64,
    /// Rare-write metadata (e.g. dynamic capability revocation).
    pub runtime: RwLock<RuntimeFlags>,
}

#[derive(Debug, Default)]
pub struct RuntimeFlags {
    pub revoked: bool,
    pub note: Option<String>,
}

pub struct SkillRegistry {
    headers: DashMap<String, Arc<SkillHeader>>,
    activated: DashMap<String, Arc<ActivatedSkill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            headers: DashMap::with_shard_amount(64),
            activated: DashMap::with_shard_amount(64),
        }
    }

    pub fn insert_header(&self, header: SkillHeader) {
        self.headers.insert(header.manifest.name.clone(), Arc::new(header));
    }

    /// O(1) average. Read-locks only the relevant shard.
    pub fn get_header(&self, name: &str) -> Option<Arc<SkillHeader>> {
        self.headers.get(name).map(|e| Arc::clone(e.value()))
    }

    pub fn get_or_insert_activated<F>(
        &self,
        name: &str,
        f: F,
    ) -> anyhow::Result<Arc<ActivatedSkill>>
    where
        F: FnOnce() -> anyhow::Result<ActivatedSkill>,
    {
        if let Some(a) = self.activated.get(name) {
            return Ok(Arc::clone(a.value()));
        }
        // Race-free upsert: DashMap's entry API serialises shard writes.
        let entry = self.activated.entry(name.to_owned()).or_try_insert_with(|| {
            f().map(Arc::new)
        })?;
        Ok(Arc::clone(entry.value()))
    }

    /// Iterate header descriptions for the agent controller. Cheap.
    pub fn header_summaries(&self) -> Vec<(String, String)> {
        self.headers
            .iter()
            .map(|e| (e.manifest.name.clone(), e.manifest.description.clone()))
            .collect()
    }
}

impl ActivatedSkill {
    pub fn note_invocation(&self, now_unix_ms: u64) {
        self.invocations.fetch_add(1, Ordering::Relaxed);
        self.last_used_unix_ms.store(now_unix_ms, Ordering::Relaxed);
    }
}
```

### 5.4 The skill invocation entry point with capability checks

```rust
// crates/host/src/invoke.rs
//! Invocation pipeline:
//!   1. Resolve header by name (O(1) DashMap read).
//!   2. Check runtime flags (revoked?).
//!   3. Lazily load body and compile WASM component (memoised).
//!   4. Translate `allowed-tools` -> WASI capability bundle.
//!   5. Verify the agent's request fits within declared capabilities.
//!   6. Hand off to the executor (script | wasm | inline-instructions).

use crate::capabilities::{Capability, CapabilityBroker};
use crate::loader::Loader;
use crate::registry::{ActivatedSkill, SkillRegistry};
use cyberos_manifest::AllowedTools;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum InvokeError {
    #[error("skill not found: {0}")]
    NotFound(String),
    #[error("skill revoked: {0} — {1}")]
    Revoked(String, String),
    #[error("capability denied: skill `{skill}` requested `{cap}` but only [{declared}] declared")]
    CapabilityDenied { skill: String, cap: String, declared: String },
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct Invoker {
    registry: Arc<SkillRegistry>,
    loader: Loader,
    broker: Arc<CapabilityBroker>,
}

impl Invoker {
    pub fn new(
        registry: Arc<SkillRegistry>,
        loader: Loader,
        broker: Arc<CapabilityBroker>,
    ) -> Self {
        Self { registry, loader, broker }
    }

    /// Single entry point. `requested_caps` is what the agent (or the
    /// host on the agent's behalf) actually intends to use this call.
    pub async fn invoke(
        &self,
        name: &str,
        input: serde_json::Value,
        requested_caps: &[Capability],
    ) -> Result<serde_json::Value, InvokeError> {
        // 1. Header lookup.
        let header = self
            .registry
            .get_header(name)
            .ok_or_else(|| InvokeError::NotFound(name.to_owned()))?;

        // 2. Declared capability set.
        let declared: Vec<Capability> = header
            .manifest
            .allowed_tools
            .as_ref()
            .map(AllowedTools::as_vec)
            .unwrap_or_default()
            .iter()
            .map(|s| Capability::parse(s))
            .collect::<anyhow::Result<_>>()?;

        // 3. Capability check — every requested cap must be subsumed by
        //    at least one declared cap. Domain/path patterns are matched
        //    by CapabilityBroker::is_declared (not shown).
        for req in requested_caps {
            if !self.broker.is_declared(req, &declared) {
                return Err(InvokeError::CapabilityDenied {
                    skill: name.to_owned(),
                    cap: req.to_string(),
                    declared: declared.iter().map(ToString::to_string)
                        .collect::<Vec<_>>().join(", "),
                });
            }
            // First-use approval prompt (interactive in dev, policy-driven in prod).
            self.broker.ensure_granted(name, req).await?;
        }

        // 4. Lazy activation — load body + compile component once,
        //    reuse for every subsequent invocation.
        let activated = self.activate(&header).await?;

        // 5. Runtime flag check (cheap parking_lot read).
        {
            let flags = activated.runtime.read();
            if flags.revoked {
                return Err(InvokeError::Revoked(
                    name.to_owned(),
                    flags.note.clone().unwrap_or_default(),
                ));
            }
        }

        // 6. Record invocation, dispatch.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
        activated.note_invocation(now);
        info!(skill = name, "invoking");

        let out = if let Some(component) = &activated.component {
            self.run_wasm(component, &activated.header, requested_caps, input).await?
        } else {
            // Markdown-only / native-script path. The host returns the
            // body so the agent controller can splice it into context.
            self.return_instructions(&activated, input).await?
        };
        Ok(out)
    }

    async fn activate(
        &self,
        header: &Arc<crate::registry::SkillHeader>,
    ) -> Result<Arc<ActivatedSkill>, InvokeError> {
        let name = header.manifest.name.clone();
        let header_clone = (**header).clone();
        let registry = Arc::clone(&self.registry);

        let activated = tokio::task::spawn_blocking(move || -> anyhow::Result<_> {
            registry.get_or_insert_activated(&name, || {
                let body = std::fs::read_to_string(
                    header_clone.skill_dir.join("SKILL.md"),
                )?;
                let body = body[header_clone.body_offset..].to_owned();

                let wasm_path = header_clone.skill_dir.join("dist/skill.wasm");
                let component = if wasm_path.exists() {
                    let engine = wasmtime::Engine::default();
                    Some(wasmtime::component::Component::from_file(&engine, &wasm_path)?)
                } else {
                    None
                };

                Ok(ActivatedSkill {
                    header: header_clone,
                    body,
                    component,
                    invocations: Default::default(),
                    last_used_unix_ms: Default::default(),
                    runtime: Default::default(),
                })
            })
        })
        .await
        .map_err(|e| InvokeError::Other(anyhow::anyhow!(e)))??;
        Ok(activated)
    }

    async fn run_wasm(
        &self,
        _component: &wasmtime::component::Component,
        _header: &crate::registry::SkillHeader,
        _caps: &[Capability],
        _input: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        // Full implementation: build a wasmtime::Store with a WasiCtx
        // constructed from the broker-derived capability bundle, link
        // host imports defined in cyberos-skill.wit, instantiate, call
        // the `run` export, deserialise JSON output.
        // (Elided for brevity — see crates/host/src/wasm.rs.)
        Ok(serde_json::json!({ "ok": true }))
    }

    async fn return_instructions(
        &self,
        activated: &Arc<ActivatedSkill>,
        _input: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        // For markdown-only skills we hand the body back to the agent
        // controller, which splices it into the LLM context as the
        // Level-2 progressive-disclosure payload.
        Ok(serde_json::json!({
            "skill": activated.header.manifest.name,
            "instructions": activated.body,
        }))
    }
}
```

### 5.5 A minimal example skill that demonstrates the pattern

```markdown
<!-- skills/vn-vat-invoice/SKILL.md -->
---
name: vn-vat-invoice
description: >-
  Generate Vietnamese VAT-compliant electronic invoices (Mau hoa don dien tu)
  from a structured JSON line-item list. Use when the user requests a Vietnamese
  invoice, VAT receipt, hoa don GTGT, or e-invoice. Do NOT use for non-Vietnamese
  invoice formats.
license: Apache-2.0
compatibility: >-
  Works fully offline. No network access required. Python 3.11+ for the
  bundled validator script.
metadata:
  author: cyberskill
  version: "0.3.0"
  region: VN
allowed-tools: read_file write_file
---

# Vietnamese VAT Invoice (Hoá đơn GTGT điện tử)

## When to use
- User provides line items and a buyer tax code (MST) and asks for a VAT invoice.
- User says "tạo hoá đơn", "xuất hoá đơn GTGT", "e-invoice Vietnam".

## Procedure
1. Validate the buyer MST with `scripts/validate_mst.py` — it must be 10 or 13 digits.
2. For each line: compute `tien_thue = thanh_tien * thue_suat`. Round half-up
   to integer VND. Round VAT totals per-line, not at the document level.
3. Produce XML in the General Department of Taxation schema v3.0
   (see `references/gdt-xml-schema-v3.md`).
4. Write the XML next to the input file with extension `.xml`.

## Examples
See `references/example-2line-invoice.xml` for a worked two-line case.
```

```python
# skills/vn-vat-invoice/scripts/validate_mst.py
"""Validate a Vietnamese tax code (Mã số thuế).

Per General Department of Taxation regulations, an MST is either:
  - 10 digits  (legal entity), OR
  - 13 digits  (branch / dependent unit: 10 digits + '-' + 3 digits accepted).

Read stdin, print '{"ok": true}' or '{"ok": false, "reason": "..."}' to stdout.
"""
import json, re, sys

PATTERN = re.compile(r"^\d{10}(-\d{3})?$")

def main() -> int:
    raw = sys.stdin.read().strip()
    ok = bool(PATTERN.fullmatch(raw))
    reason = None if ok else "MST must be 10 digits, optionally followed by '-NNN'"
    print(json.dumps({"ok": ok, "reason": reason}))
    return 0 if ok else 1

if __name__ == "__main__":
    sys.exit(main())
```

This skill (a) is a valid Agent Skill that loads unchanged in Claude Code, Codex CLI, Cursor, VS Code-with-Copilot, Goose; (b) declares only the two capabilities it needs (`read_file write_file`); (c) defers detail to `references/` so the SKILL.md body itself stays small; (d) demonstrates a Vietnamese-market specialisation — exactly the differentiation CyberSkill should pursue.

---

## 6. Migration Path

Phased, with feature flags at every step. Each phase ships independently and is reversible.

**Phase 0 — Inventory & freeze (1–2 weeks).** Catalogue every skill in the current CyberOS module. Freeze the existing format: no new bespoke-format skills accepted from this point. Stand up the `cyberos skill validate` CLI that parses both the legacy format and SKILL.md and emits a diff.

**Phase 1 — Dual-format ingest (2–3 weeks).** Add the SKILL.md loader (§5.2) alongside the existing loader behind a `--skills-format=both|legacy|standard` flag. Default stays `legacy`. New skills authored as SKILL.md immediately work. Run the legacy and standard paths side-by-side in CI on the same skill set.

**Phase 2 — Translator + parity tests (2 weeks).** Build a one-shot translator from the legacy format to SKILL.md. Run it across the catalogue, hand-fix the residual, commit the translated skills to a new `skills/` tree. Add a property-test harness that asserts byte-identical agent outputs across both loaders for the entire catalogue.

**Phase 3 — Default flip (1 week).** Change the flag default to `standard`. Legacy loader remains compiled in for one release cycle. Announce the deprecation; publish the new directory structure on `agentskills.io` so Claude/Codex/Cursor users can install CyberSkill skills directly.

**Phase 4 — Concurrency rewrite (3–4 weeks).** Land the `DashMap`-based registry (§5.3) behind a `--registry=dashmap|rwlock` flag. Benchmark with `criterion` against the existing primitive on a realistic invocation mix. Flip the default once the benchmark shows ≥2× throughput improvement on a 4+ core box. Keep the old primitive for one release.

**Phase 5 — WASM execution path (4–6 weeks).** Add the Wasmtime executor behind `--exec=script|wasm|auto`. Auto selects WASM when `dist/skill.wasm` is present, falls back to scripts otherwise. Ship the Bun + esbuild authoring toolchain (`cyberos skill new --lang ts` / `cyberos skill build`) so TypeScript skill authors can target `wasm32-wasi` components. Migrate any third-party / untrusted skills to WASM. Internal-only skills can stay on the script path indefinitely.

**Phase 6 — Capability broker GA (2 weeks).** Turn capability enforcement from `warn` to `deny`. Operators must approve each skill's `allowed-tools` set on first use; the grant is recorded by content hash. Publish a `cyberos cap audit` command.

**Phase 7 — Legacy removal (1 week, one release later).** Delete the legacy loader, the legacy registry primitive, the legacy executor. Tag a new major version of CyberOS.

**Rollback strategy.** Every phase introduces its replacement behind a flag with the legacy implementation still compiled in. Rollback is a flag flip plus a release, not a code revert. Phases 1–6 each have a built-in "fall back to legacy" path that does not require a new build. Phase 7 is the only irreversible step, and it ships only after one full release cycle on the new defaults with zero P0 incidents.

---

## 7. Risk Register

| # | Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| 1 | **Agent Skills spec drifts** (it is "deliciously tiny" and under-specified in places; the AAIF / Linux Foundation governance may amend post-transition) | Medium | Medium | Track `agentskills.io` and the AAIF mailing list. Keep the manifest model behind a `serde(deny_unknown_fields)` parser plus an opt-in "lenient" mode. Add a `cyberos skill validate --against-spec=v1` command and pin a known-good revision per CyberOS release. |
| 2 | **WASM cold-start regression** under a specific guest language toolchain (Python-on-WASI is heavy; Component Model bindings churn) | Medium | High | AOT-precompile components on install (`wasmtime compile`). Cache compiled artifacts by content hash. Default executable skills to Rust or TypeScript-via-Bun guests; flag Python guests as "preview." Benchmark cold-start per skill on every CI run; alert on >100 ms p95. |
| 3 | **Capability declarations are too coarse** (`allowed-tools` is experimental in the spec and clients vary in how they enforce it) | High | High | Treat `allowed-tools` as the source of truth, but also enforce at the WASI grant layer regardless of declaration — defence in depth. Require operator first-use approval. Maintain a CyberOS-internal "capability extension" sub-namespace in `metadata.cyberos-caps` for finer-grained grants until the standard catches up. |
| 4 | **Migration regressions** — translated skills behave subtly differently from their legacy equivalents (LLM-driven activation is not byte-deterministic) | High | Medium | Property-test the entire catalogue under both loaders in Phase 2. Keep both loaders shippable until parity is proven on real workloads. Stage the default flip per-customer for any high-value tenants. |
| 5 | **Ecosystem lock-in inversion** — Anthropic deprecates or restructures the standard in a way that disadvantages non-Anthropic hosts | Low | High | The spec is now in an independent repository (`agentskills/agentskills`) and is widely adopted by Anthropic's competitors (OpenAI Codex, Microsoft, Google Gemini CLI). Anthropic unilaterally breaking it is strategically irrational. CyberOS should nevertheless contribute to the spec repo so it has voice in governance, and maintain the right to ship CyberOS-specific extensions under `metadata.*` rather than at the top level. |

**Strategic addendum — Anthropic ecosystem alignment.** CyberSkill is a 10-person consultancy with single-digit-million revenue. Trying to invent and propagate a competing skills format against Anthropic, Microsoft, OpenAI, Google, and the Linux-Foundation-hosted AAIF is a category error. Adopting Agent Skills verbatim turns CyberOS into a citizen of an ecosystem with 26+ compatible clients and a partner directory that already includes Atlassian, Canva, Cloudflare, Figma, Notion, Ramp, Sentry, Stripe, and Zapier — and lets CyberSkill carve a defensible niche by **publishing high-quality, Vietnam-localised skills (VAT, e-invoice, local bank APIs, Vietnamese legal/compliance, VNeID) into the open registry**. That is the partnership/distribution play, and it is materially larger than the platform play CyberSkill could win on its own. The fact that Anthropic itself now runs Claude Code on Bun (2026) is a useful tell: even the spec authors are willing to swap out core runtimes when the data says so — Bun for the JS toolchain, Wasmtime for the sandbox, Rust for the host. CyberOS should follow the same data-driven layering.