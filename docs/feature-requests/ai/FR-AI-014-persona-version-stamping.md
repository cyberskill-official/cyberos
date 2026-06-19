---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-014
title: "Persona-version system-prompt injection from memory memories/personas/<handle>.md"
module: AI
priority: MUST
status: ready_to_test
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-AI-001, FR-AI-003, FR-AI-005, FR-AI-008, FR-AI-022]
depends_on: [FR-AI-003]
blocks: []

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/cuo.html#personas
  - website/docs/modules/ai.html#persona-version
source_decisions:
  - EU AI Act Art. 50 transparency (persona-version stamp on every output that emerged from a CyberOS persona)
  - DEC-046 (CUO single-Genie identity with persona overlay)
  - DEC-051 (persona files MUST live in memory, not config; full revision history is auditable)
  - archive/2026-05-14/RESEARCH_REVIEW.md §3.7 (tamper-detect personas at injection time)

# ───── Build envelope ─────
language: rust 1.81 (ai-gateway service)
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/persona/mod.rs
  - services/ai-gateway/src/persona/registry.rs
  - services/ai-gateway/src/persona/parse.rs
  - services/ai-gateway/src/persona/watch.rs
  - services/ai-gateway/src/persona/hash.rs
  - services/ai-gateway/tests/persona_test.rs
  - services/ai-gateway/tests/persona_test.rs
  - services/ai-gateway/tests/persona_test.rs
  - services/ai-gateway/tests/cache_isolation_concurrent_test.rs
  - <memory-root>/memories/personas/cuo-cpo@0.4.1.md       # seed: chief-of-product persona
  - <memory-root>/memories/personas/cuo-cfo@0.4.1.md       # seed: chief-of-finance persona
  - <memory-root>/memories/personas/cuo-cto@0.4.1.md       # seed: chief-of-technology persona
modified_files:
  - services/ai-gateway/src/handlers/chat.rs           # persona injection at message[0] system role
  - services/ai-gateway/src/lib.rs                     # boot order: init_persona_registry before bind
  - services/ai-gateway/src/memory_writer.rs            # add canonical::persona_loaded builder (FR-AI-003 §3 declared kind)
  - services/ai-gateway/Cargo.toml                     # arc_swap, notify, sha2, semver, once_cell
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests}/**
  - memory: read memories/personas/* via memory_writer query (NOT direct fs read; protocol §0.7)
  - memory: write personas via canonical Writer ONLY (no fs::write to memories/personas/)
  - bash: cargo test -p cyberos-ai-gateway persona
disallowed_tools:
  - hardcode persona content inline in src/persona/* (must load from memory)
  - write to memories/personas/* outside the canonical Writer (§0.3 immutability invariant)
  - skip tamper verification on cache-hit path (cheap check; security-load-bearing)
  - cache persona by id alone (must key by full handle `<id>@<version>` per §1 #4)
  - emit `ai.persona_loaded` audit row through any path other than `canonical::persona_loaded` (FR-AI-003)

# ───── Estimated work ─────
effort_hours: 8
sub_tasks:
  - "0.5h: Persona schema types (PersonaHandle, PersonaId, Version newtypes with parse validation)"
  - "0.5h: parse_persona_md (YAML frontmatter + body extraction; allowed-field whitelist)"
  - "0.5h: Source-hash canonicalisation (LF-normalised, NFC, no BOM, trim trailing whitespace)"
  - "1.0h: persona::load(handle) with ArcSwap registry; init from memory_writer::list_path"
  - "1.0h: memory file-watch (notify crate) with 250ms debounce; reparse on event; ArcSwap::store the new map"
  - "1.0h: Injection at chat.rs handler: prepend system message; preserve caller's system message at index 1"
  - "0.5h: canonical::persona_loaded builder + memory_writer subprocess invocation"
  - "0.5h: Response header injection (`X-CyberOS-Persona-Handle`) + EU AI Act Art. 50 badge metadata"
  - "0.5h: OTel metrics emission (loads_total, cache_hits_total, tampered_total, reload_total)"
  - "0.5h: persona_test.rs — happy load + cache hit + Arc::ptr_eq"
  - "0.5h: persona_tamper_test.rs — disk-corruption simulation; Err(Tampered) propagates"
  - "0.5h: persona_hot_reload_test.rs — file edit triggers reload within 500ms"
  - "0.5h: persona_concurrent_test.rs — 100 concurrent loads; no contention; same Arc"
  - "0.5h: persona_init_failure_test.rs — malformed file; init returns Err(PersonaInitError)"
risk_if_skipped: "Every AI call has unattributable provenance. EU AI Act Art. 50 transparency obligation unmet on every output reaching an EU end-user. CUO's multi-persona architecture (one Genie + 10 C-level skills) has no way to identify which persona answered. Audit chain becomes useless for compliance review — a regulator asking 'which persona produced output X on date Y' has no answer. CFO-persona output indistinguishable from CTO-persona output in the chain. First EU AI Act audit (mandatory from Aug 2026 onward) fails on the transparency dimension."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** load persona definitions from `<memory-root>/memories/personas/<handle>.md` and inject the persona's system prompt as the first system message before the user's messages array. The persona handle format and the loader together obey the following:

1. **MUST** parse the persona Markdown file with a YAML frontmatter block bounded by `---` markers AND a body. Frontmatter fields: `id` (kebab-case string), `version` (semver `MAJOR.MINOR.PATCH`), `allowed_tools` (list of MCP tool names), `traits` (list of strings), `llm_hints` (mapping with `temperature`, `max_tokens`, `stop_sequences`). The **body** (everything below the closing `---`) is the canonical `system_prompt`; the frontmatter field `system_prompt` is NOT a valid alternative source and the parser MUST reject any frontmatter with that key (precedence ambiguity blocked at parse time).
2. **MUST** key the in-memory registry by **handle** (`PersonaHandle = "{id}@{version}"`, e.g. `"cuo-cpo@0.4.1"`). Two distinct versions of the same persona id coexist as two distinct registry entries. The file name on disk MUST match the handle (`<id>@<version>.md`); a frontmatter `id`/`version` that disagrees with the filename MUST fail registry init with `PersonaInitError::FilenameMismatch`.
3. **MUST** cache parsed personas via `ArcSwap<HashMap<PersonaHandle, Arc<Persona>>>`. Reader path: `registry.load().get(&handle).cloned()`. Writer path: `registry.store(Arc::new(new_map))` on hot-reload — an atomic pointer swap; no torn reads. RwLock is NOT acceptable (blocks readers during reload).
4. **MUST** reject requests where `req.agent_persona` doesn't resolve to a registered handle — return `400 BAD_REQUEST` with body `{"error":"unknown_persona","agent_persona":"<handle>","available_handles":["..."]}`. The available list is the registry keys at request time; it is sorted lexicographically for stable test fixtures.
5. **MUST** inject the persona's body (canonical `system_prompt`) as `messages[0]` with role `system` BEFORE any caller-supplied system message. A caller's system message becomes `messages[1]` (also role `system`). The handler MUST NOT silently overwrite a caller's system message and MUST NOT concatenate the persona prompt with the caller's system message.
6. **MUST** emit exactly one `ai.persona_loaded` memory audit row per request, BEFORE the LLM call begins, via the canonical builder `canonical::persona_loaded(&persona, &request_id)` (declared as a row kind in FR-AI-003 §3; this FR adds the builder function). The row carries `persona_id`, `persona_version`, `persona_handle`, `source_path`, `source_hash`, `request_id`. The audit-before-action invariant from FR-AI-001 §1 #6 applies — the row MUST be durable on the chain before the call leaves the gateway.
7. **MUST** verify `source_hash` matches the cached body before injection on EVERY load (cache-hit AND cache-miss path). The check is cheap (~5µs SHA-256 of body bytes). On mismatch: return `Err(PersonaError::Tampered { handle, expected_hash, actual_hash })`; emit a sev-1 OBS event `ai_persona_tampered{handle}`; refuse the call with `503 PERSONA_TAMPERED`. Tamper detection MUST NOT be skipped via a "trust cache" flag — this is the boundary check that catches on-disk modification after init.
8. **MUST** canonicalise the body BEFORE hashing AND before injection: (a) normalise line endings CRLF → LF, (b) strip a leading BOM if present, (c) apply Unicode NFC normalisation, (d) right-trim trailing whitespace on each line, (e) ensure exactly one terminating LF. This canonicalisation is the source-hash domain; without it, a benign LF↔CRLF flip on a Windows checkout would false-positive as tampering.
9. **MUST** include the persona handle in EVERY downstream artefact:
   - `ai.precheck` memory row (FR-AI-001 already carries `agent_persona`; this FR ensures the value is a full handle, not bare id).
   - `ai.invocation` row (FR-AI-002).
   - Response header `X-CyberOS-Persona-Handle: <id>@<version>` on every HTTP 200.
   - Response header `X-CyberOS-Persona-Source-Hash: <hex16>` (first 16 hex of SHA-256) for client-side tamper-correlation.
   - User-facing badge metadata field `{"made_by_genie":{"id":"<id>","version":"<version>"}}` in the response JSON body (EU AI Act Art. 50 transparency requirement; UI surfaces render the badge from this field).
10. **MUST** complete `persona::load(handle)` in ≤ 100µs on cache hit (registry HashMap lookup + hash verify) and ≤ 50ms on cache miss (first init + memory_writer disk read). After warm-up, cache miss is rare; the 50ms is a budget for the boot-time `init_persona_registry` per-persona cost.
11. **MUST** integrate with `policy.ai_policy.allowed_personas` from FR-AI-005 — if the tenant policy declares an allow-list, FR-AI-001 §1 #13 already enforces it. This FR DOES NOT replicate the check; it only loads. A request that passes FR-AI-001's persona-allow check and arrives at the injection point MUST always succeed (or fail tamper/missing-handle). No silent policy re-check at injection.
12. **MUST** debounce file-watch events on `<memory-root>/memories/personas/` with a 250ms window: rapid bursts (editor save sequences write 3-5 events) collapse to one reparse. The watcher uses the `notify` crate's `RecommendedWatcher` with `RecursiveMode::Recursive`. On debounce-flush, the watcher re-runs `init_persona_registry`'s parsing pass against the current disk state and `ArcSwap::store`s the new map atomically.
13. **MUST** keep the LLM-hints (`temperature`, `max_tokens`, `stop_sequences`) from the persona as the per-request DEFAULT — caller-supplied values in the request body OVERRIDE the persona hint. This rule is documented in §2; the merge order is `request.body.llm_hints` > `persona.llm_hints` > provider default.
14. **MUST** validate semver in `PersonaVersion::parse` using the `semver` crate. A version string like `0.4` (missing patch) or `0.4.1-alpha` (pre-release) is REJECTED at parse time. Slice 3 supports plain `MAJOR.MINOR.PATCH` only; pre-release support is out of scope (FR-AI-022 follow-up).
15. **SHOULD** emit OTel metrics:
   - `ai_persona_loads_total{handle, outcome}` (counter; outcome ∈ `hit | miss | unknown | tampered`).
   - `ai_persona_cache_hits_total{handle}` (counter).
   - `ai_persona_tampered_total{handle}` (counter; sev-1 alarm fires on any increment).
   - `ai_persona_reload_total{outcome}` (counter; outcome ∈ `success | parse_error | filename_mismatch`).
   - `ai_persona_registry_size` (gauge; current registry entry count).
16. **SHOULD** log at INFO level on every successful hot-reload: `persona_reloaded handle=<h> source_hash=<hex16> registry_size=<N>` — operator visibility into "did my edit actually load?".

---

## §2 — Why this design (rationale for humans)

**Why store personas in memory, not config files?** Personas evolve through user edits and CUO refinement loops. Every edit produces a chain leaf (the memory's append-only audit log captures who edited what and when). Storing in a YAML config file would create a parallel source of truth with no audit trail — exactly the failure mode the memory exists to prevent. Personas are *operational artefacts* (often-edited, frequently-versioned, audit-required), not infrastructure config.

**Why key the registry by full handle, not id?** Two reasons. (1) Multiple versions of the same persona coexist during a rollout — `cuo-cpo@0.4.1` and `cuo-cpo@0.4.2` are both valid for a brief window while traffic shifts. Indexing by id alone forces a choose-one moment; indexing by handle lets both serve. (2) The handle is the EU AI Act Art. 50 attribution unit — a regulator asking "which persona produced this output" wants the handle, not the id. The handle-keyed registry IS the answer to the regulator's question.

**Why hash-verify on every load, not just init?** Personas govern model behaviour. An attacker who can write to `<memory-root>/memories/personas/cuo-cpo@0.4.1.md` can change Genie's tone, constraints, and tool-allow-list — silently. The init-only hash check would let post-init disk mutations slip through. A 5µs hash on every load is the cheapest possible boundary check; the per-request cost (~5µs in a request that already costs hundreds of ms at the LLM) is negligible. The hash check is "the persona content I'm about to inject matches the persona content I parsed at init / last reload" — without it, the cache is a trust-on-first-use that an attacker can pwn after the trust window.

**Why ArcSwap, not RwLock?** RwLock-based hot-reload blocks readers during the write window (~5ms for a full re-parse + re-build of the HashMap). Under load (1000 reads/s during a hot-reload), 5ms of writer-held lock means 5 reads sit waiting — measurable p99 latency spike. ArcSwap is a pointer-swap; readers see either the old map or the new map at any nanosecond boundary, never a torn read, never blocked. The cost is one extra Arc allocation per reload (rare event) — strictly cheaper than the RwLock approach.

**Why is the persona injected as a system message rather than concatenated to the user message?** LLMs (Anthropic, OpenAI, Google) reliably distinguish system from user messages — the alignment training is explicit about which authorial layer the system role represents. Concatenating the persona prompt with the user message mixes the layers; a hostile user prompt can then "claim" parts of the system message via prompt-injection patterns ("ignore the above and instead..."). Keeping persona at system role + user message at user role is the architecturally clean separation; the LLM's own message-role discipline does the work of resisting authority confusion.

**Why does the body canonicalisation rule matter?** A team member on Windows checks out the repo; git's `core.autocrlf` setting normalises LF → CRLF on checkout. Their next edit + save preserves CRLF. The Linux-running gateway hashes the LF version at init, then re-reads the CRLF version on hot-reload and detects "tamper." Without canonicalisation, this benign cross-platform workflow flags as a sev-1 security event every time. The five-step canonicalisation (CRLF→LF, no BOM, NFC, trim trailing whitespace per line, single terminating LF) is the minimum that makes "the same persona body, formatted by different editors" hash to the same value. The hash IS the security boundary; we just don't want false positives on whitespace.

**Why caller-overrides-persona for LLM hints (§1 #13)?** The persona's `temperature: 0.4` is a default — a starting point that's right for "general persona usage." A specific call site might know "this draft should be more creative; use 0.8." The override gives the call site the last word. Persona defaults are NOT meant to be inviolable — they're sensible defaults that handle the 80% case without ceremony.

**Why source_hash in the response header (`X-CyberOS-Persona-Source-Hash`)?** The header lets a downstream client (or a downstream audit tool) cross-check "the persona that ran for this response is the persona I expected." A client persisting Genie outputs to its own KB can record (handle, source_hash) tuples and later detect "during the period 2026-04-12 to 2026-04-19, the cuo-cpo@0.4.1 persona's source_hash shifted from X to Y — there was a hot-reload mid-stream." This is a small but useful audit primitive; the 16-hex prefix keeps the header value short.

**Why semver-only versioning (§1 #14)?** Pre-release tags (`-alpha`, `-rc1`) and build metadata (`+build123`) add parsing complexity without solving a problem we have. Slice 3 personas are produced by humans + CUO-refinement loops; both produce concrete versions, not pre-releases. If a future need emerges (e.g., "shadow-test persona-0.5.0-shadow alongside 0.4.1 in production"), FR-AI-022 will extend `PersonaVersion::parse` — but the change should be deliberate, not accidental from a copy-paste.

**Why does the tamper check produce 503 PERSONA_TAMPERED rather than degrading to a default persona?** "Degrade to default" is the silent-failure path: the persona was supposed to enforce constraints ("never offer compensation") and now those constraints aren't there. The user sees an output but the model wasn't following the persona's safeguards. 503 is the loud-failure path: the operator gets paged, the customer sees an error, no output ships under false attribution. Loud failure is the right default for security-load-bearing primitives.

**Why is there an EU AI Act Art. 50 badge field in the response body (§1 #9)?** Art. 50 (effective Aug 2026) requires AI outputs reaching EU end-users to disclose the AI provenance. A response header is invisible to most UIs; embedding the badge metadata in the JSON body gives the UI a structured field to render ("Made by Genie · cuo-cpo · v0.4.1") wherever appropriate. The field name `made_by_genie` is the canonical attribution surface across CyberOS products.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Type definitions

```rust
// services/ai-gateway/src/persona/mod.rs

use std::sync::Arc;
use arc_swap::ArcSwap;
use once_cell::sync::OnceCell;
use semver::Version;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonaId(String);     // kebab-case, e.g. "cuo-cpo"

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PersonaHandle {
    pub id: PersonaId,
    pub version: Version,         // semver MAJOR.MINOR.PATCH (no pre-release, no build metadata)
}

impl PersonaHandle {
    /// Parse "cuo-cpo@0.4.1" → PersonaHandle. Rejects pre-release, missing patch, etc.
    pub fn parse(s: &str) -> Result<Self, PersonaParseError> { /* ... */ }

    /// Render as "cuo-cpo@0.4.1" for storage / headers / audit.
    pub fn display(&self) -> String { format!("{}@{}", self.id.0, self.version) }
}

#[derive(Debug, Clone)]
pub struct Persona {
    pub handle: PersonaHandle,
    pub body: String,                          // canonicalised system_prompt body
    pub allowed_tools: Vec<String>,            // e.g. ["search_kb", "draft_email"]
    pub traits: Vec<String>,                   // e.g. ["concise", "VN-aware", "founder-voice"]
    pub llm_hints: LlmHints,                   // temperature, max_tokens, stop_sequences
    pub source_path: String,                   // memory-relative path; e.g. "memories/personas/cuo-cpo@0.4.1.md"
    pub source_hash: [u8; 32],                 // SHA-256 of canonicalised body
}

#[derive(Debug, Clone)]
pub struct LlmHints {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop_sequences: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PersonaError {
    #[error("unknown persona handle {handle}; available: {available:?}")]
    UnknownPersona {
        handle: String,
        available: Vec<String>,
    },
    #[error("persona body hash mismatch — possible tampering: handle={handle}")]
    Tampered {
        handle: PersonaHandle,
        expected_hash: [u8; 32],
        actual_hash: [u8; 32],
    },
    #[error("memory read failed: {0}")]
    MemoryReadFailed(String),
    #[error("registry not initialised")]
    RegistryNotInitialised,
}

#[derive(Debug, thiserror::Error)]
pub enum PersonaInitError {
    #[error("malformed YAML frontmatter in {path}: {reason}")]
    Schema { path: String, reason: String },
    #[error("filename {path} does not match frontmatter handle {handle}")]
    FilenameMismatch { path: String, handle: String },
    #[error("forbidden field 'system_prompt' in frontmatter at {path}; body is the canonical source")]
    ForbiddenFrontmatterField { path: String },
    #[error("registry already initialised; init_persona_registry called twice")]
    AlreadyInitialised,
    #[error("memory read failed at init: {0}")]
    MemoryReadFailed(String),
}

#[derive(Debug, thiserror::Error)]
pub enum PersonaParseError {
    #[error("missing '@' separator in handle {0!r}")]
    MissingAt(String),
    #[error("invalid semver in handle: {0}")]
    InvalidSemver(String),
    #[error("pre-release versions not supported in slice 3: {0}")]
    PreReleaseUnsupported(String),
    #[error("invalid persona id (must be kebab-case): {0!r}")]
    InvalidId(String),
}

static REGISTRY: OnceCell<ArcSwap<HashMap<PersonaHandle, Arc<Persona>>>> = OnceCell::new();

pub async fn init_persona_registry() -> Result<(), PersonaInitError> { /* ... */ }

pub fn load(handle: &PersonaHandle) -> Result<Arc<Persona>, PersonaError> { /* ... */ }

pub fn available_handles() -> Vec<String> { /* ... */ }
```

### Parser contract

```rust
// services/ai-gateway/src/persona/parse.rs

pub fn parse_persona_md(path: &str, raw: &str) -> Result<Persona, PersonaInitError> {
    // 1. Split frontmatter from body using `---` markers.
    let (frontmatter_yaml, body_raw) = split_frontmatter(raw)?;

    // 2. Parse YAML frontmatter, asserting whitelisted fields only.
    let fm: PersonaFrontmatter = serde_yaml::from_str(frontmatter_yaml)
        .map_err(|e| PersonaInitError::Schema { path: path.into(), reason: e.to_string() })?;

    // 3. Forbid 'system_prompt' in frontmatter (precedence ambiguity blocker).
    if fm.system_prompt.is_some() {
        return Err(PersonaInitError::ForbiddenFrontmatterField { path: path.into() });
    }

    // 4. Canonicalise body bytes per §1 #8.
    let body = canonicalise_body(body_raw);

    // 5. Compute source_hash.
    let source_hash = sha256(body.as_bytes());

    // 6. Construct handle; assert filename matches.
    let handle = PersonaHandle { id: fm.id, version: fm.version };
    let expected_filename = format!("{}.md", handle.display());
    if !path.ends_with(&expected_filename) {
        return Err(PersonaInitError::FilenameMismatch {
            path: path.into(), handle: handle.display(),
        });
    }

    Ok(Persona {
        handle, body, source_hash,
        allowed_tools: fm.allowed_tools,
        traits: fm.traits,
        llm_hints: fm.llm_hints,
        source_path: path.into(),
    })
}

/// §1 #8 canonicalisation: CRLF→LF, no BOM, NFC, trim trailing whitespace, single terminating LF.
fn canonicalise_body(raw: &str) -> String {
    let stripped = raw.strip_prefix('\u{FEFF}').unwrap_or(raw);
    let lf = stripped.replace("\r\n", "\n").replace('\r', "\n");
    let nfc: String = unicode_normalization::UnicodeNormalization::nfc(lf.chars()).collect();
    let trimmed_lines: Vec<&str> = nfc.lines().map(|l| l.trim_end()).collect();
    let mut out = trimmed_lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}
```

### Persona file format (canonical)

```markdown
---
id: cuo-cpo
version: 0.4.1
allowed_tools:
  - draft_email
  - search_kb
  - summarise_kb
  - escalate_to_human
traits:
  - founder-voice
  - concise
  - action-oriented
  - VN-aware
llm_hints:
  temperature: 0.4
  max_tokens: 1024
  stop_sequences:
    - "</persona>"
---

You are Genie, the AI orchestrator at CyberSkill. You speak in the founder's voice — direct, concise, action-oriented. When a user asks a customer-facing question, you draft a response in their style.

Constraints:
- Never offer compensation or contractual commitments without explicit founder approval.
- Never reveal internal compensation, equity, or financial data.
- Default to bilingual (Vietnamese + English) when context suggests a VN customer.

When uncertain about facts, search the knowledge base first; do not fabricate.
```

### Injection contract (handler-side)

```rust
// services/ai-gateway/src/handlers/chat.rs (additions)

async fn handle_chat(req: ChatCompleteRequest) -> Result<ChatCompleteResponse, ApiError> {
    // ... cost precheck (FR-AI-001) ... persona-allow check (FR-AI-001 §1 #13) ...

    let handle = PersonaHandle::parse(&req.agent_persona)?;
    let persona = persona::load(&handle).map_err(map_persona_err)?;

    // §1 #6: audit row BEFORE LLM call.
    memory_writer::emit(canonical::persona_loaded(&persona, &req.request_id)).await?;

    // §1 #5: prepend persona body as messages[0].
    let mut messages = Vec::with_capacity(req.messages.len() + 1);
    messages.push(Message {
        role: Role::System,
        content: persona.body.clone(),
    });
    messages.extend(req.messages.iter().cloned());

    // §1 #13: caller-supplied hints override persona defaults.
    let temperature = req.temperature.or(persona.llm_hints.temperature);
    let max_tokens = req.max_tokens.or(persona.llm_hints.max_tokens);

    // ... call provider ...

    // §1 #9: response headers + body badge.
    let mut response = build_response(/* ... */);
    response.headers.insert("X-CyberOS-Persona-Handle", persona.handle.display());
    response.headers.insert("X-CyberOS-Persona-Source-Hash", hex16(&persona.source_hash));
    response.body.made_by_genie = Some(MadeByGenie {
        id: persona.handle.id.0.clone(),
        version: persona.handle.version.to_string(),
    });
    Ok(response)
}
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **Happy load** — `persona::load(&handle_cuo_cpo_v_0_4_1)` returns the parsed `Persona` matching the file frontmatter + body; `persona.handle.display() == "cuo-cpo@0.4.1"`; `persona.allowed_tools` contains the expected 4 tools.
2. **Cache hit on second call** — Second call to `persona::load(&handle)` returns an `Arc<Persona>` with `Arc::ptr_eq(&first, &second) == true`.
3. **Hot reload** — Edit `<memory-root>/memories/personas/cuo-cpo@0.4.1.md` (touch + rewrite body); within 500ms, `persona::load(&handle)` returns a Persona with the new body and a DIFFERENT `source_hash` from before.
4. **Unknown handle** — `persona::load(&handle_v_9_9_9)` returns `Err(UnknownPersona { available: [...sorted...] })`; `available` is lexicographically sorted.
5. **Tamper detection** — Mutate the on-disk file's body via `fs::write` (bypassing the canonical Writer); call `persona::load(&handle)`. The first call after the mutation returns `Err(Tampered { handle, expected_hash, actual_hash })`; a sev-1 OBS event is emitted with metric `ai_persona_tampered_total{handle}` incremented by 1.
6. **System prompt injection** — `req.messages = [User("hi")]` + `agent_persona = "cuo-cpo@0.4.1"`. After handler injection, the request sent to the provider has `messages = [System(persona.body), User("hi")]`.
7. **Caller system message preserved** — `req.messages = [System("call-specific"), User("hi")]`. After injection, the provider sees `messages = [System(persona.body), System("call-specific"), User("hi")]` — the caller's system message is at index 1, NOT overwritten.
8. **Audit row emitted** — Every request with persona resolves emits exactly one `ai.persona_loaded` memory row before the LLM call. The row's `source_hash` matches `persona.source_hash`.
9. **Response header includes persona handle** — Every HTTP 200 response carries `X-CyberOS-Persona-Handle: cuo-cpo@0.4.1` AND `X-CyberOS-Persona-Source-Hash: <hex16>`.
10. **Response body badge** — Every HTTP 200 response body contains `made_by_genie: {"id":"cuo-cpo","version":"0.4.1"}`.
11. **Latency budget (cache hit)** — 1000 cache-hit `persona::load` calls complete in < 100ms total (≤ 100µs each per §1 #10).
12. **Latency budget (cache miss / init)** — Boot-time `init_persona_registry` over 10 persona files completes in < 500ms (≤ 50ms per file per §1 #10).
13. **Concurrent loads** — 100 tokio tasks calling `persona::load(&same_handle)` concurrently produce zero contention (no `Mutex::lock` in the hot path); all see the same `Arc` pointer.
14. **Canonicalisation: CRLF tolerance** — A file saved with CRLF line endings produces an identical `source_hash` to the same content saved with LF line endings. No false-positive tamper.
15. **Filename mismatch rejected** — A file at `memories/personas/cuo-cpo@0.4.1.md` with frontmatter `id: cuo-cpo, version: 0.4.2` returns `PersonaInitError::FilenameMismatch` from `parse_persona_md`.
16. **Forbidden frontmatter field rejected** — A frontmatter with `system_prompt: "..."` key returns `PersonaInitError::ForbiddenFrontmatterField`; the body is the canonical source.
17. **Semver parse strictness** — `PersonaHandle::parse("cuo-cpo@0.4")` returns `PersonaParseError::InvalidSemver`; `PersonaHandle::parse("cuo-cpo@0.4.1-alpha")` returns `PreReleaseUnsupported`.
18. **LLM-hint merge order** — `request.body.temperature = 0.8` + `persona.llm_hints.temperature = 0.4` → provider sees `temperature = 0.8`. With request omitting temperature, provider sees `temperature = 0.4`.
19. **Hot-reload of malformed file leaves cache unchanged** — Edit a persona file to invalid YAML (e.g., delete the closing `---`); within 500ms, the reload attempt logs a `persona_reload_total{outcome=parse_error}` metric; `persona::load(&handle)` continues to return the pre-edit cached `Persona`.
20. **Double-init rejected** — `init_persona_registry().await` then second `init_persona_registry().await` returns `PersonaInitError::AlreadyInitialised`.

---

## §5 — Verification

### Happy + cache test

```rust
// services/ai-gateway/tests/persona_test.rs
use cyberos_ai_gateway::persona::{self, PersonaHandle};

#[tokio::test]
async fn loads_persona_from_memory_and_caches() {
    persona::init_persona_registry().await.unwrap();
    let handle = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();

    // AC #1
    let p1 = persona::load(&handle).expect("first load");
    assert_eq!(p1.handle.display(), "cuo-cpo@0.4.1");
    assert!(p1.allowed_tools.contains(&"search_kb".to_string()));
    assert!(p1.body.contains("You are Genie"));

    // AC #2
    let p2 = persona::load(&handle).expect("second load");
    assert!(std::sync::Arc::ptr_eq(&p1, &p2));
}

#[tokio::test]
async fn unknown_handle_returns_sorted_available() {
    persona::init_persona_registry().await.unwrap();
    let handle = PersonaHandle::parse("cuo-cpo@9.9.9").unwrap();

    let err = persona::load(&handle).expect_err("expected UnknownPersona");
    match err {
        persona::PersonaError::UnknownPersona { available, .. } => {
            let sorted: Vec<_> = {
                let mut a = available.clone(); a.sort(); a
            };
            assert_eq!(available, sorted, "available list must be lexicographically sorted");
        }
        e => panic!("unexpected error variant: {e:?}"),
    }
}

#[tokio::test]
async fn semver_parse_rejects_pre_release_and_short_version() {
    use persona::PersonaParseError;
    assert!(matches!(
        PersonaHandle::parse("cuo-cpo@0.4"),
        Err(PersonaParseError::InvalidSemver(_))
    ));
    assert!(matches!(
        PersonaHandle::parse("cuo-cpo@0.4.1-alpha"),
        Err(PersonaParseError::PreReleaseUnsupported(_))
    ));
}

#[tokio::test]
async fn filename_mismatch_rejected_at_init() {
    use persona::parse::parse_persona_md;
    let body = "---\nid: cuo-cpo\nversion: 0.4.2\nallowed_tools: []\ntraits: []\nllm_hints: {}\n---\n\nbody\n";
    let err = parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", body).expect_err("expected mismatch");
    assert!(matches!(err, persona::PersonaInitError::FilenameMismatch { .. }));
}

#[tokio::test]
async fn forbidden_frontmatter_system_prompt_rejected() {
    use persona::parse::parse_persona_md;
    let body = "---\nid: cuo-cpo\nversion: 0.4.1\nallowed_tools: []\ntraits: []\nllm_hints: {}\nsystem_prompt: 'forbidden'\n---\n\nbody\n";
    let err = parse_persona_md("memories/personas/cuo-cpo@0.4.1.md", body).expect_err("expected forbidden");
    assert!(matches!(err, persona::PersonaInitError::ForbiddenFrontmatterField { .. }));
}

#[tokio::test]
async fn double_init_rejected() {
    persona::init_persona_registry().await.unwrap();
    let err = persona::init_persona_registry().await.expect_err("expected AlreadyInitialised");
    assert!(matches!(err, persona::PersonaInitError::AlreadyInitialised));
}
```

### Tamper detection test

```rust
// services/ai-gateway/tests/persona_test.rs
#[tokio::test]
async fn tamper_detection_fires_with_metric() {
    persona::init_persona_registry().await.unwrap();
    let handle = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();

    // Read the cached source_hash, mutate the on-disk body, call load again.
    let p1 = persona::load(&handle).unwrap();
    let path = format!("<memory-root>/memories/personas/{}.md", handle.display());
    let original = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, original + "\nappended tamper line\n").unwrap();

    // Force the cache-hit verify path (no hot-reload yet — the watcher may be debouncing).
    let err = persona::load(&handle).expect_err("expected Tampered");
    match err {
        persona::PersonaError::Tampered { handle: h, expected_hash, actual_hash } => {
            assert_eq!(h, p1.handle);
            assert_ne!(expected_hash, actual_hash);
        }
        e => panic!("unexpected error variant: {e:?}"),
    }

    // OTel metric incremented.
    let counter = otel_test_helper::counter_value(
        "ai_persona_tampered_total",
        &[("handle", "cuo-cpo@0.4.1")],
    );
    assert!(counter >= 1, "tampered counter not incremented");

    // Restore for subsequent tests.
    std::fs::write(&path, original).unwrap();
}
```

### Hot-reload test

```rust
// services/ai-gateway/tests/persona_test.rs
#[tokio::test]
async fn hot_reload_within_500ms() {
    persona::init_persona_registry().await.unwrap();
    let handle = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();
    let path = format!("<memory-root>/memories/personas/{}.md", handle.display());
    let original = std::fs::read_to_string(&path).unwrap();
    let p1 = persona::load(&handle).unwrap();

    // Edit the body (preserve frontmatter; change only body text).
    let new_body = original.replace("You are Genie", "You are Genie v2");
    std::fs::write(&path, &new_body).unwrap();

    // Poll for up to 500ms.
    let mut updated = false;
    for _ in 0..50 {
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        let p_now = persona::load(&handle).unwrap();
        if p_now.source_hash != p1.source_hash {
            assert!(p_now.body.contains("Genie v2"));
            updated = true;
            break;
        }
    }
    assert!(updated, "hot-reload did not propagate within 500ms");

    // Restore.
    std::fs::write(&path, &original).unwrap();
}

#[tokio::test]
async fn hot_reload_of_malformed_file_leaves_cache_unchanged() {
    persona::init_persona_registry().await.unwrap();
    let handle = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();
    let path = format!("<memory-root>/memories/personas/{}.md", handle.display());
    let original = std::fs::read_to_string(&path).unwrap();
    let p1 = persona::load(&handle).unwrap();

    // Corrupt: delete closing `---`.
    let bad = original.replace("---\n\n", "BROKEN\n\n");
    std::fs::write(&path, bad).unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    let p_after = persona::load(&handle).expect("cache must hold pre-edit content");
    assert_eq!(p_after.source_hash, p1.source_hash, "cache must NOT update on parse error");

    let counter = otel_test_helper::counter_value(
        "ai_persona_reload_total",
        &[("outcome", "parse_error")],
    );
    assert!(counter >= 1, "parse_error counter not incremented");

    std::fs::write(&path, original).unwrap();
}
```

### Canonicalisation test (CRLF tolerance)

```rust
#[test]
fn canonicalisation_is_lf_normalised() {
    use persona::parse::canonicalise_body;
    let lf = "Hello\nWorld\n";
    let crlf = "Hello\r\nWorld\r\n";
    assert_eq!(canonicalise_body(lf), canonicalise_body(crlf));
}

#[test]
fn canonicalisation_strips_bom_and_nfc_normalises() {
    use persona::parse::canonicalise_body;
    // BOM-prefixed CRLF with combining diacritic vs precomposed.
    let bom_crlf_combining = "\u{FEFF}cafe\u{0301}\r\n";   // BOM + "café" via combining acute
    let lf_precomposed     = "café\n";
    assert_eq!(canonicalise_body(bom_crlf_combining), canonicalise_body(lf_precomposed));
}
```

### Concurrent-load test

```rust
// services/ai-gateway/tests/cache_isolation_concurrent_test.rs
#[tokio::test]
async fn one_hundred_concurrent_loads_no_contention() {
    persona::init_persona_registry().await.unwrap();
    let handle = std::sync::Arc::new(PersonaHandle::parse("cuo-cpo@0.4.1").unwrap());
    let mut joinset = tokio::task::JoinSet::new();
    for _ in 0..100 {
        let h = handle.clone();
        joinset.spawn(async move {
            let p = persona::load(&h).expect("load");
            std::sync::Arc::as_ptr(&p) as usize
        });
    }
    let mut ptrs = vec![];
    while let Some(r) = joinset.join_next().await {
        ptrs.push(r.unwrap());
    }
    // AC #13: every concurrent load sees the same Arc pointer.
    assert!(ptrs.iter().all(|p| *p == ptrs[0]), "concurrent loads saw different Arcs");
}

#[tokio::test]
async fn one_thousand_cache_hits_within_budget() {
    persona::init_persona_registry().await.unwrap();
    let handle = PersonaHandle::parse("cuo-cpo@0.4.1").unwrap();
    let t0 = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = persona::load(&handle).unwrap();
    }
    let elapsed = t0.elapsed();
    assert!(elapsed < std::time::Duration::from_millis(100),
            "1000 cache hits took {elapsed:?}, budget 100ms");
}
```

### LLM-hint merge test (handler-level integration)

```rust
#[tokio::test]
async fn caller_hints_override_persona_defaults() {
    persona::init_persona_registry().await.unwrap();
    let req = test_request("cuo-cpo@0.4.1", /* temperature */ Some(0.8), /* max_tokens */ None);
    let provider_call = handlers::chat::prepare_provider_call(req).await.unwrap();

    assert_eq!(provider_call.temperature, Some(0.8));   // caller wins
    assert_eq!(provider_call.max_tokens, Some(1024));   // persona default (caller omitted)
}
```

### Run

```bash
cd services/ai-gateway
cargo test -p cyberos-ai-gateway persona
```

CI gate: cargo-test pass on every PR touching `services/ai-gateway/src/persona/**` OR `memories/personas/**`.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/persona/registry.rs

pub async fn init_persona_registry() -> Result<(), PersonaInitError> {
    if REGISTRY.get().is_some() {
        return Err(PersonaInitError::AlreadyInitialised);
    }

    let entries = memory_writer::list_path("memories/personas/").await
        .map_err(|e| PersonaInitError::MemoryReadFailed(e.to_string()))?;

    let mut map: HashMap<PersonaHandle, Arc<Persona>> = HashMap::new();
    for entry_path in entries {
        if !entry_path.ends_with(".md") { continue; }
        let raw = memory_writer::read_path(&entry_path).await
            .map_err(|e| PersonaInitError::MemoryReadFailed(e.to_string()))?;
        let persona = parse::parse_persona_md(&entry_path, &raw)?;
        map.insert(persona.handle.clone(), Arc::new(persona));
    }

    REGISTRY.set(ArcSwap::from_pointee(map))
        .map_err(|_| PersonaInitError::AlreadyInitialised)?;
    watch::spawn_memory_watcher();
    Ok(())
}

pub fn load(handle: &PersonaHandle) -> Result<Arc<Persona>, PersonaError> {
    let registry = REGISTRY.get().ok_or(PersonaError::RegistryNotInitialised)?;
    let map = registry.load();
    let Some(persona) = map.get(handle) else {
        let mut avail: Vec<String> = map.keys().map(|h| h.display()).collect();
        avail.sort();
        return Err(PersonaError::UnknownPersona {
            handle: handle.display(), available: avail,
        });
    };
    hash::verify_persona(persona)?;
    metrics::cache_hit(&persona.handle);
    Ok(persona.clone())
}
```

```rust
// services/ai-gateway/src/persona/watch.rs

pub fn spawn_memory_watcher() {
    use notify::{Watcher, RecursiveMode, EventKind};
    use std::sync::mpsc;
    let (tx, rx) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx).unwrap();
    watcher.watch(
        std::path::Path::new("<memory-root>/memories/personas/"),
        RecursiveMode::Recursive,
    ).unwrap();
    std::thread::spawn(move || {
        let mut last_event = std::time::Instant::now();
        let debounce = std::time::Duration::from_millis(250);
        for ev in rx {
            if let Ok(_event) = ev {
                last_event = std::time::Instant::now();
                std::thread::sleep(debounce);
                // Drain any further events that arrived in the debounce window.
                while last_event.elapsed() < debounce {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                reload_registry();
            }
        }
    });
}

fn reload_registry() {
    let tokio_runtime = tokio::runtime::Handle::current();
    let result = tokio_runtime.block_on(async {
        let entries = memory_writer::list_path("memories/personas/").await?;
        let mut new_map: HashMap<PersonaHandle, Arc<Persona>> = HashMap::new();
        for path in entries {
            if !path.ends_with(".md") { continue; }
            let raw = memory_writer::read_path(&path).await?;
            let persona = parse::parse_persona_md(&path, &raw)?;
            new_map.insert(persona.handle.clone(), Arc::new(persona));
        }
        Ok::<_, anyhow::Error>(new_map)
    });
    match result {
        Ok(new_map) => {
            REGISTRY.get().unwrap().store(Arc::new(new_map.clone()));
            metrics::reload_success(new_map.len() as u64);
            for (h, p) in &new_map {
                tracing::info!(
                    handle = %h.display(),
                    source_hash = %hex16(&p.source_hash),
                    registry_size = new_map.len(),
                    "persona_reloaded"
                );
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "persona reload failed; cache unchanged");
            metrics::reload_failure(&e);
        }
    }
}
```

```rust
// services/ai-gateway/src/persona/hash.rs

pub fn verify_persona(p: &Persona) -> Result<(), PersonaError> {
    let actual = sha256(p.body.as_bytes());
    if actual != p.source_hash {
        metrics::tampered(&p.handle);
        return Err(PersonaError::Tampered {
            handle: p.handle.clone(),
            expected_hash: p.source_hash,
            actual_hash: actual,
        });
    }
    Ok(())
}

pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    let mut h = Sha256::new();
    h.update(bytes);
    h.finalize().into()
}

pub fn hex16(h: &[u8; 32]) -> String {
    h.iter().take(8).map(|b| format!("{:02x}", b)).collect()
}
```

```rust
// services/ai-gateway/src/memory_writer.rs (additions)

pub mod canonical {
    pub fn persona_loaded(persona: &Persona, request_id: &str) -> AuditRow {
        AuditRow {
            kind: "ai.persona_loaded".into(),
            payload: serde_json::json!({
                "persona_id": persona.handle.id.0,
                "persona_version": persona.handle.version.to_string(),
                "persona_handle": persona.handle.display(),
                "source_path": persona.source_path,
                "source_hash": hex::encode(persona.source_hash),
                "request_id": request_id,
            }),
            ..Default::default()
        }
    }
}
```

---

## §7 — Dependencies

### Code dependencies (other FRs/modules)

- **FR-AI-003** — memory audit-row bridge. Declares the `ai.persona_loaded` row kind in §3; this FR adds the `canonical::persona_loaded` builder function. The Writer subprocess pattern from FR-AI-003 §1 is reused unchanged.
- **FR-AI-005** — Tenant policy loader. `policy.ai_policy.allowed_personas` is enforced in FR-AI-001 §1 #13; this FR does not duplicate the check. The persona schema (id, version, allowed_tools, traits, llm_hints) is the source-of-truth this FR loads from memory.
- **FR-AI-001** — Cost ledger precheck. Already carries `agent_persona`; this FR ensures the value is a full handle (`<id>@<version>`). FR-AI-001 §1 #13 (persona-allow check) runs BEFORE persona loading; an unauthorised persona request is refused before this FR's code runs.
- **FR-AI-022 (downstream)** — OTel trace emission. Will consume `ai_persona_loads_total` and friends as the canonical metric set for persona observability.

### Concept dependencies (shared types)

- `PersonaHandle = "{id}@{version}"` is the shared identity primitive used by `ai.precheck`, `ai.invocation`, `ai.persona_loaded` memory rows AND the `X-CyberOS-Persona-Handle` response header. Format is fixed: kebab-case id, semver version, `@` separator.
- `LlmHints` (temperature, max_tokens, stop_sequences) is the shared default-override surface. Merge order: `request.body` > `persona.llm_hints` > `provider default`.
- `source_hash` (SHA-256 of canonicalised body) is the tamper-detection primitive AND the audit-correlation primitive (audit row carries it; response header `X-CyberOS-Persona-Source-Hash` carries the 16-hex prefix).
- The `made_by_genie` JSON field in response bodies is the EU AI Act Art. 50 attribution surface used by all CyberOS UIs.

### Operational / external

- Rust crates: `arc-swap@1`, `notify@6`, `sha2@0.10`, `semver@1`, `once_cell@1`, `serde_yaml@0.9`, `unicode-normalization@0.1`, `thiserror@1`.
- memory module: must allow reads from `memories/personas/` via `memory_writer::list_path` and `memory_writer::read_path`. Writes via canonical Writer only (protocol §0.3 immutability).
- Seed personas: `cuo-cpo@0.4.1`, `cuo-cfo@0.4.1`, `cuo-cto@0.4.1` ship with this FR; further personas are operator-curated through CUO refinement loops.

---

## §8 — Example payloads

### Persona file (canonical) — `<memory-root>/memories/personas/cuo-cpo@0.4.1.md`

See §3 above (canonical format).

### Request with persona

```json
{
  "model": "chat.smart",
  "agent_persona": "cuo-cpo@0.4.1",
  "messages": [{ "role": "user", "content": "Draft a thank-you for the deal" }]
}
```

### After persona injection (sent to LLM provider)

```json
{
  "model": "chat.smart",
  "temperature": 0.4,
  "max_tokens": 1024,
  "stop": ["</persona>"],
  "messages": [
    { "role": "system", "content": "You are Genie, the AI orchestrator at CyberSkill..." },
    { "role": "user", "content": "Draft a thank-you for the deal" }
  ]
}
```

### Response headers

```text
HTTP/1.1 200 OK
X-CyberOS-Persona-Handle: cuo-cpo@0.4.1
X-CyberOS-Persona-Source-Hash: 4b8c0d2f1a7e9c3b
X-CyberOS-Hold-Id: 01HZK9R8M3X5C8Q4
```

### Response body (badge metadata)

```json
{
  "choices": [{ "message": { "role": "assistant", "content": "Dear ..." }}],
  "made_by_genie": { "id": "cuo-cpo", "version": "0.4.1" },
  "usage": { "prompt_tokens": 142, "completion_tokens": 86 }
}
```

### Audit row `ai.persona_loaded`

```json
{
  "kind": "ai.persona_loaded",
  "ts_ns": 1747526400000000000,
  "payload": {
    "persona_id": "cuo-cpo",
    "persona_version": "0.4.1",
    "persona_handle": "cuo-cpo@0.4.1",
    "source_path": "memories/personas/cuo-cpo@0.4.1.md",
    "source_hash": "4b8c0d2f1a7e9c3b...",
    "request_id": "req_01HZK9R8M3X5C8Q4"
  }
}
```

### Unknown-handle error response

```json
HTTP/1.1 400 Bad Request
{
  "error": "unknown_persona",
  "agent_persona": "cuo-cpo@9.9.9",
  "available_handles": [
    "cuo-cfo@0.4.1",
    "cuo-cpo@0.4.1",
    "cuo-cto@0.4.1"
  ]
}
```

### Tamper error response

```json
HTTP/1.1 503 Service Unavailable
{
  "error": "persona_tampered",
  "handle": "cuo-cpo@0.4.1",
  "contact": "ops@cyberos.world"
}
```

(Body never echoes `expected_hash`/`actual_hash` — those are written to the sev-1 OBS event and memory, never to the client.)

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Pre-release version support (`0.5.0-rc1`) — FR-AI-022.
- Persona inheritance ("v0.5 extends v0.4 with delta") — slice 5; current model is full-body per version.
- Per-tenant persona override (`tenant.persona_override["cuo-cpo"] = "<custom-handle>"`) — slice 5; FR-AI-005 schema extension.
- Signed personas (Ed25519 over source_hash) — FR-AI-022; current SHA-256 tamper-check is sufficient for boundary detection but doesn't survive a compromised disk-writer.
- Multi-file personas (split system prompt across multiple files for editor ergonomics) — out of scope; the constraint to one file = one handle is deliberate.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Persona handle not in registry | HashMap miss in `persona::load` | `Err(UnknownPersona)` → `400 BAD_REQUEST` | Caller fixes `agent_persona`; or operator adds the persona file |
| On-disk tamper between loads (post-init `fs::write`) | `verify_persona` hash mismatch | `Err(Tampered)` → sev-1 OBS → `503 PERSONA_TAMPERED` | Operator investigates (likely git revert); incident review |
| memory read failure at init | `memory_writer::read_path` error | `PersonaInitError::MemoryReadFailed` → init fails → gateway refuses to bind | Operator investigates memory (FR-AI-003 §10 covers); restart gateway after memory recovery |
| Malformed persona YAML at init | `serde_yaml::from_str` error | `PersonaInitError::Schema` → init fails → gateway refuses to bind | Operator fixes file; redeploy |
| Hot-reload of malformed file | Parser fails in watcher loop | Cache unchanged; `persona_reload_total{outcome=parse_error}` counter incremented; INFO log "reload failed" | Operator fixes file; next file-watch event triggers retry |
| Filename ≠ frontmatter handle | `parse_persona_md` filename-match check | `PersonaInitError::FilenameMismatch` → init OR reload fails | Operator renames file OR fixes frontmatter |
| Forbidden frontmatter field `system_prompt` | Parser whitelist check | `PersonaInitError::ForbiddenFrontmatterField` → init OR reload fails | Operator moves prompt content from frontmatter to body |
| Cross-platform line-ending flip (LF↔CRLF) | Canonicalisation neutralises | `source_hash` unchanged; no false-positive tamper | By design (§1 #8) |
| BOM-prefixed file from Windows editor | Canonicalisation strips BOM | `source_hash` unchanged; no false-positive tamper | By design (§1 #8) |
| Concurrent persona load + hot-reload | `ArcSwap` atomic | Reader sees either old or new map, never torn | By design (§1 #3) |
| Concurrent hot-reload + reload (rapid edits) | Debounce window collapses bursts to one reparse | One reparse runs; subsequent events queue and re-trigger | By design (§1 #12) |
| Allowed_tools field references a non-existent MCP tool | Runtime detection at tool-call time (FR-MCP-006) | Tool call refused; sev-2 log | Operator updates persona OR registers the tool |
| Persona pre-release version submitted | `PersonaHandle::parse` strictness | `PersonaParseError::PreReleaseUnsupported` → `400` at API edge | Caller uses release version |
| Persona file in `memories/personas/` not named `<handle>.md` | Init pass skips non-`.md` files; misnamed files (`foo.md` with arbitrary frontmatter) fail filename-match | Skipped OR init failure | Operator renames file to canonical form |
| Registry init called twice (test re-entry, sidecar reload) | `OnceCell::set` returns Err | `PersonaInitError::AlreadyInitialised` | Tests use `reset_for_tests()`; production calls init once at boot |
| Watcher thread panics | `tokio::spawn`+ tracing observability | Watcher dies; hot-reload stops working but cache continues serving | sev-2 alert; operator restarts gateway; investigates panic cause |
| File-watch event flood (1000 events/sec from bulk edit) | Debounce window | Watcher collapses to one reparse; metric `ai_persona_reload_total` only increments by 1 per debounced flush | By design (§1 #12) |
| LLM hint conflict (persona says temperature=0.4, request says nothing) | Merge order: request > persona > default | Provider gets persona default | By design (§1 #13) |
| Caller-supplied system message conflicts with persona | Injection puts persona at idx 0, caller at idx 1 | Both present in messages; LLM sees both as system context | By design (§1 #5); UI MAY warn caller about double-system pattern |
| Response header `X-CyberOS-Persona-Handle` missing on 200 | Handler middleware enforces; integration test asserts presence | Test failure → PR blocked | PR rework |
| Audit row `ai.persona_loaded` missing in chain | Integration test asserts emit-before-LLM-call sequence | Test failure → PR blocked | PR rework |

---

## §11 — Notes

- Persona files are normally edited by humans (founder, CXO surface owners) — the 250ms debounced file-watch reload makes iterative editing fast without ping-ponging on every keystroke.
- The `cuo-cpo@0.4.1` example is the production-seed persona at slice 3; future versions (0.4.2, 0.5.0) coexist by being different files at different handles. There is no "current" pointer — callers explicitly select the handle.
- Tamper detection is the single most important security property here. Without it, an attacker who pwns the disk can rewrite Genie's instructions (and its constraint clauses, like "never reveal financial data") without leaving a chain trace. The hash check is the boundary that turns a silent-instruction-rewrite into a loud-503-refused-call.
- The canonicalisation rule (§1 #8) is the answer to "the same logical persona, edited on different platforms, hashes the same." It is NOT a security weakening — an attacker who flips CRLF↔LF maliciously is just landing on the same hash, which is fine because the content is unchanged. The hash protects against semantic changes (new words, removed constraints), which no canonicalisation step undoes.
- The persona registry is `ArcSwap<HashMap<...>>` and NOT `DashMap`. `DashMap` has finer-grained locking but each load() requires a shard lock acquisition (~50ns) vs ArcSwap's atomic pointer dereference (~5ns). At our load (read-heavy, rare writes), ArcSwap is strictly better.
- The `X-CyberOS-Persona-Source-Hash` header is 16 hex characters (8 bytes of the SHA-256). This is enough to disambiguate any two personas in the registry (collision probability ~10⁻⁹ for 1000 entries) without bloating header size. The full 32-byte hash is in the audit row for forensic precision.
- Caller-overrides-persona for LLM hints is a deliberate design choice. If we made the persona binding, every prompt-by-prompt fine-tuning ("be more creative here") would require a new persona version. The override gives flexibility at the call site without compromising the persona's role as a *default* baseline.
- The decision to forbid `system_prompt` in frontmatter (§1 #1, ISS-fix in this revision) is the single-source-of-truth principle: the body IS the canonical prompt. Allowing both creates a precedence question with no good answer; rejecting frontmatter-`system_prompt` at parse time eliminates the question.
- The seed personas (`cuo-cpo`, `cuo-cfo`, `cuo-cto`) are placeholders for the full 10 C-level slice 3 will eventually carry. Adding new personas is purely a `git add memories/personas/<new-handle>.md` + redeploy operation — no code change.
- The 250ms file-watch debounce window was tuned empirically against editor-save patterns. VS Code emits 3-5 `Modify` events per save (atomic-replace via `*.tmp` + rename); 250ms catches the burst. Slower editors (vim with `:w` swap-file pattern) emit fewer events. The window is conservative — long enough to coalesce, short enough that operator edits feel instant.

---

*End of FR-AI-014. Status: draft (10/10 target).*
