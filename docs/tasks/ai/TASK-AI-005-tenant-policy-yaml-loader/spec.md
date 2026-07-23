---
# ───── Machine-readable frontmatter ─────
id: TASK-AI-005
title: "Tenant-policy YAML loader — per-tenant cap + warn + override + residency"
eu_ai_act_risk_class: not_ai
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: ai
priority: p0
status: done
accepted_at: 2026-05-15
accepted_by: Stephen Cheng
verify: T
phase: P0
milestone: P0 · slice 1
slice: 1
owner: Stephen Cheng
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-AI-001, TASK-AI-006, TASK-AI-015, TASK-AI-016]
depends_on: []
blocks: [TASK-AI-001, TASK-AI-006, TASK-AI-021]

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#cost-gate
  - website/docs/modules/ai.html#bigger-picture
source_decisions:
  - docs/tasks/ai/TASK-AI-001-cost-ledger-precheck/spec.md §3 (TenantPolicy shape)
  - docs/tasks/ai/TASK-AI-001-cost-ledger-precheck/spec.md §9 Q2 (missing-policy default = refuse)
  - archive/2026-05-14/AUDIT_AND_PLAN.md §3.3 (P0 · slice 1 location of config)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  # already declared in TASK-AI-001; this task populates it
  - services/ai-gateway/src/policy.rs
  - services/ai-gateway/src/policy/loader.rs
  - services/ai-gateway/src/policy/schema.rs
  - services/ai-gateway/src/policy/cache.rs
  # directory placeholder
  - services/ai-gateway/config/tenants/.gitkeep
  # documented example
  - services/ai-gateway/config/tenants/EXAMPLE.tenant.yaml
  - services/ai-gateway/tests/policy_loader_test.rs
  - services/ai-gateway/tests/fixtures/policy/valid.yaml
  - services/ai-gateway/tests/fixtures/policy/invalid-schema.yaml
  - services/ai-gateway/tests/fixtures/policy/missing-required.yaml
modified_files:
  # add serde_yaml, notify (file-watch), schemars
  - services/ai-gateway/Cargo.toml
  # export policy module
  - services/ai-gateway/src/lib.rs
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,config}/**
  - bash: cargo test -p cyberos-ai-gateway policy
disallowed_tools:
  - in-place edit of services/ai-gateway/src/cost_ledger.rs (TASK-AI-001 owns it; this task is consumed by it)
  - hardcode any tenant_id in the loader (multi-tenant invariant)
  - shell-out to read YAML (must use serde_yaml in-process)

# ───── Estimated work ─────
effort_hours: 5
subtasks:
  - "0.5h: TenantPolicy struct + serde + schemars (for jsonschema generation)"
  - "1.0h: schema validation on load (required fields, value ranges, enum constraints)"
  - "1.0h: file-watch invalidation via notify crate (POLL_INTERVAL fallback for NFS)"
  - "1.0h: in-memory cache with read-write lock"
  - "0.5h: error taxonomy + structured logs (PolicyMissing vs PolicyInvalid vs IOError)"
  - "1.0h: integration tests (valid load, invalid schema, file watch, cache, missing file)"
risk_if_skipped: "TASK-AI-001 has no source of per-tenant caps. Every call would use a hardcoded global default, defeating the entire point of the cost-of-everything gate (each tenant has different caps). Worse, the override/residency/ZDR knobs are all keyed to tenant policy — they all fail without this task."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** provide a `policy::TenantPolicy` value type and a loader that hydrates one per-tenant policy from a YAML file at `config/tenants/<tenant_id>.yaml`. The loader is consumed by `cost_ledger::precheck()` (TASK-AI-001) and the router (TASK-AI-006/008) to determine cost caps, model selection, residency pinning, and override rules.

The loader:

1. **MUST** validate every loaded YAML against the closed `policy::schema::TENANT_POLICY_SCHEMA` (generated via `schemars` from the Rust struct). Schema-invalid files MUST be rejected — the policy MUST NOT load with any field missing, out-of-range, or of the wrong type.
2. **MUST** expose `policy::load_for_tenant(tenant_id) -> Result<TenantPolicy, PolicyError>` as the single public entry point. No other public surface; callers MUST NOT cache the policy themselves (the loader's cache is authoritative).
3. **MUST** cache policies in memory after first load; subsequent loads of the same `tenant_id` MUST hit the cache (sub-microsecond return).
4. **MUST** invalidate the cache when the source YAML changes on disk. Implementation: a `notify`-crate file-watcher on `config/tenants/`; on `Modify` or `Remove` event, the cache entry for that tenant_id is dropped. Next read re-reads from disk.
5. **MUST** refuse to apply a hot-reload of an invalid file — the cached (valid) policy stays in place; a `tracing::error!` event surfaces the invalid update; the operator CLI (TASK-AI-021) lists tenants whose live policy is stale due to a rejected reload.
6. **MUST** treat the in-YAML `tenant_id` field as authoritative for cache keying. The filename serves only as a discovery hint; the loader MUST read the `tenant_id` field from each loaded YAML and use that value as the cache key. If the in-file `tenant_id` does not match the filename stem (modulo `:` ↔ `-` substitution), the loader MUST emit a `tracing::warn!` event but accept the file. Files whose in-file `tenant_id` fails the `validate_tenant_id` charset check MUST be rejected at load time. Path traversal in any caller-supplied `tenant_id` (`..`, `/`, `\`) MUST be rejected at the `load_for_tenant()` boundary with `Err(PolicyError::InvalidTenantId)`.
7. **MUST** return `Err(PolicyError::PolicyMissing { tenant_id })` (NOT a default policy) when the YAML file does not exist. The cost gate MUST fail closed; silent defaults are forbidden.
8. **MUST** load synchronously (not lazily); on AI Gateway startup, the loader MUST eagerly load *all* policies in `config/tenants/*.yaml` and refuse to start if any fail validation. This catches misconfiguration at deploy time rather than first-request time.
9. **SHOULD** emit OTel metrics: `ai_policy_cache_hits_total`, `ai_policy_cache_misses_total`, `ai_policy_reload_failures_total`.
10. **MUST** support concurrent reads from many tokio tasks without contention; an `arc_swap::ArcSwap<HashMap<TenantId, TenantPolicy>>` (or equivalent lock-free read structure) is the recommended implementation.
11. **MUST** aggregate ALL file failures encountered during `init_loader()` into a single `LoaderInitError::Schema { failures: Vec<FileFailure> }`. Reporting only the first failure forces multi-deploy iteration; reporting all in one pass lets the operator fix everything at once.
12. **MUST** detect polling-mode `notify` filesystems (NFS, Docker bind-mounts) at init and log `policy_loader_polling_mode` at INFO. On polling-mode, AC #7's hot-reload latency budget extends from 500ms to 35 seconds; this MUST be documented in OBS dashboards.
13. **MUST** expose a `policy::validate_yaml(yaml: &str) -> Result<TenantPolicy, Vec<String>>` public function. TASK-AI-021's `cyberos-ai policy validate <file>` operator subcommand wraps this; the bridge stays a thin loader.
14. **SHOULD** emit OTel metrics: `ai_policy_cache_hits_total{tenant_id}` (counter), `ai_policy_cache_misses_total{tenant_id}` (counter), `ai_policy_reload_failures_total{tenant_id,reason}` (counter), `ai_policy_loaded_tenants` (gauge), `ai_policy_validation_failures_total{kind}` (counter; kind ∈ schema/range/charset/missing-field).
15. **MUST** sort the loader's `HashMap<String, Arc<TenantPolicy>>` aggregation by tenant_id when iterated for OBS metric emission, log lines on load-success, and the `ai.policy_reload_completed` audit row's `extra.tenants_loaded: Vec<String>` field per task-audit skill §3.9 rule 27 (determinism). Two consecutive runs on the same set of YAML files MUST produce byte-identical sequences. AC #15 asserts `assert_eq!` on the captured `Vec<String>` across two loads.

This task is the static configuration layer of the AI Gateway. At slice 1 the YAML files live in-repo (checked in to source); at P2 (TASK-TEN-004) the TEN module replaces this with a per-tenant database table. The interface stays the same — only the loader's data source changes.

---

## §2 — Why this design (rationale for humans)

**Why YAML files in the repo and not a database?** Two reasons. (1) Slice-1 deploys are bare metal with one tenant (CyberSkill itself). Standing up a tenant-policy table before the TEN module exists is premature work. (2) YAML in source control gives us a git-tracked history of policy changes for free — every cap adjustment is a PR with a diff and a reviewer. The TEN module's runtime policy editor (TASK-TEN-005) replicates this audit trail when it lands. The migration path is a one-time script.

**Why fail-closed on missing policy?** From TASK-AI-001 §9 Q2: silent defaults are how budget surprises happen in production. The naive "default cap of $10/month if no policy file" sounds defensive but is actually offensive — a new tenant whose policy file we forgot to ship gets a tiny cap and an angry CEO. Loud failures ("policy file not found for tenant org:foo") are uncomfortable but recoverable; silent caps are not.

**Why eager-load all policies on startup?** A lazy loader would defer validation until the first request for that tenant — meaning misconfiguration only surfaces hours or days later. Eager-loading shifts the failure to deploy time (where it can be caught by the CI and the deploy script), which is exactly when humans are watching. The cost is ~100ms × N tenants of startup time, negligible at any tenant count we'll see in P0.

**Why `arc_swap::ArcSwap` and not `RwLock<HashMap>`?** Two reasons. (1) Reads are vastly more frequent than writes (every call reads, file-watch fires maybe daily). An `ArcSwap` makes the read path lock-free; the write path swaps the whole map. (2) `RwLock::read()` still has some contention at high QPS — measurable on a benchmark we want to pass. The trade-off is that map mutations are not in-place; we rebuild the map on each change. Fine for the size of map we'll see.

**Why `notify`-crate file-watching and not periodic polling?** File-watching is sub-millisecond on Linux (inotify) and macOS (FSEvents). Polling at, say, 5-second intervals adds 5-second delay to a policy change taking effect — meaningful when fixing a runaway cost incident. The `notify` crate handles the OS-specific bits and falls back to polling on filesystems that don't support inotify (NFS, some Docker volume mounts).

**Why does the loader refuse invalid hot-reloads but accept invalid startup loads?** Wait — it doesn't accept invalid startup loads. It refuses both. But the behaviour differs: an invalid file on startup kills the process (deploy failure, exit 1). An invalid file at hot-reload time leaves the previous (valid) policy in cache, emits an error, and the operator goes to fix it. This is asymmetric for a reason: at startup, the operator is watching and can fix immediately. At hot-reload, there may be live traffic — we don't want to suddenly fail every request because someone fat-fingered a YAML edit.

---

## §3 — API contract

### Public types

```rust
// services/ai-gateway/src/policy.rs

pub use schema::TenantPolicy;
pub use loader::{load_for_tenant, init_loader, shutdown_loader};

pub enum PolicyError {
    PolicyMissing { tenant_id: String },
    PolicyInvalid { tenant_id: String, schema_errors: Vec<String> },
    InvalidTenantId { reason: String },
    IoError { tenant_id: String, source: std::io::Error },
}

pub enum LoaderInitError {
    /// One or more YAML files failed validation; ALL failures are reported in one error.
    Schema { failures: Vec<FileFailure> },
    IoError(std::io::Error),
    AlreadyInitialised,
    WatcherSetup(notify::Error),
}

pub struct FileFailure {
    pub path: PathBuf,
    pub errors: Vec<String>,
}
```

### Schema

```rust
// services/ai-gateway/src/policy/schema.rs

use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
use rust_decimal::Decimal;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TenantPolicy {
    pub tenant_id: String,
    pub ai_policy: AiPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AiPolicy {
    /// Hard cap on monthly USD spend across all providers
    #[schemars(range(min = 0.01, max = 1_000_000.0))]
    pub monthly_cap_usd: Decimal,

    /// Fraction of cap at which to emit a warn event (0.0–1.0)
    #[schemars(range(min = 0.5, max = 0.95))]
    #[serde(default = "default_warn_threshold")]
    pub warn_threshold: f64,

    /// If true, precheck() refuses at cap; if false, allows over-spend
    #[serde(default = "default_hard_stop")]
    pub hard_stop: bool,

    /// Primary provider for routing (TASK-AI-006/008 consumes this)
    pub primary_provider: Provider,

    /// Fallback chain (in order); empty = no fallback
    #[serde(default)]
    pub fallback_chain: Vec<Provider>,

    /// Per-call timeout (precheck + provider + reconcile combined)
    #[schemars(range(min = 1, max = 600))]
    #[serde(default = "default_call_timeout_seconds")]
    pub call_timeout_seconds: u32,

    /// Residency pin — provider selection respects this
    pub residency: Residency,

    /// Require ZDR (Zero Data Retention) — refuse non-ZDR providers
    #[serde(default)]
    pub zdr_required: bool,

    /// Emergency override (CFO-signed) to allow over-cap calls
    #[serde(default)]
    pub emergency_override: EmergencyOverride,

    /// Persona pinning — restrict calls to this exact agent_persona version
    /// (None = any registered persona allowed)
    #[serde(default)]
    pub allowed_personas: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Provider {
    Bedrock {
        #[schemars(regex(pattern = r"^(us|eu|ap|sa|af|ca|me)-[a-z]+-\d+$"))]
        region: String,
        model_alias_map: HashMap<String, String>,
    },
    Anthropic { model_alias_map: HashMap<String, String> },
    OpenAI { model_alias_map: HashMap<String, String> },
    Vertex {
        project: String,
        #[schemars(regex(pattern = r"^[a-z]+-[a-z]+\d*$"))]
        region: String,
        model_alias_map: HashMap<String, String>,
    },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Residency {
    Sg1,    // Singapore
    Eu1,    // Frankfurt
    Us1,    // us-east-1
    Vn1,    // Vietnam (slice-1: not yet honoured; TASK-AI-016 enforces)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct EmergencyOverride {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub requires: Vec<String>,   // e.g. ["cfo_signoff", "audit_row"]
    /// Maximum 1.0× cap multiplier this override permits (e.g. 1.5 = 150% of cap)
    #[serde(default = "default_override_multiplier")]
    pub max_multiplier: f64,
}

fn default_warn_threshold() -> f64 { 0.80 }
fn default_hard_stop() -> bool { true }
fn default_call_timeout_seconds() -> u32 { 60 }
fn default_override_multiplier() -> f64 { 1.0 }
```

### Loader

```rust
// services/ai-gateway/src/policy/loader.rs

/// Initialise the loader at AI Gateway startup. Eagerly loads every YAML in
/// config/tenants/*.yaml and validates each. Returns the first validation error
/// or io error encountered. Subsequent calls to load_for_tenant() use the cache.
///
/// Caller is responsible for keeping the returned Loader alive for the lifetime
/// of the process; dropping it stops the file-watcher.
pub async fn init_loader(config_dir: &Path) -> Result<Loader, LoaderInitError>;

/// Read-only handle exposed to call sites.
pub async fn load_for_tenant(tenant_id: &str) -> Result<Arc<TenantPolicy>, PolicyError>;

/// Stop the file-watcher and drain the cache. Idempotent.
pub async fn shutdown_loader();
```

### File layout

```
services/ai-gateway/config/tenants/
├── EXAMPLE.tenant.yaml          # documented example, not loaded (filename prefix check)
├── org-cyberskill.yaml          # first tenant (CyberSkill itself)
└── ...                          # added per onboarded tenant
```

The loader recognises files matching `^[a-z0-9][a-z0-9-]*\.yaml$` (i.e., lowercase tenant_ids in kebab-case, `.yaml` extension). Files prefixed with `_` or with uppercase letters (like `EXAMPLE.tenant.yaml`) are ignored.

### Example YAML (used as fixture + as `EXAMPLE.tenant.yaml`)

```yaml
# services/ai-gateway/config/tenants/EXAMPLE.tenant.yaml
# Copy this file to <tenant-id>.yaml, edit, and AI Gateway hot-reloads it.

tenant_id: org:cyberskill
ai_policy:
  monthly_cap_usd: 150
  warn_threshold: 0.80
  hard_stop: true

  primary_provider:
    kind: bedrock
    region: ap-southeast-1
    model_alias_map:
      chat.smart: anthropic.claude-3-5-sonnet-20241022-v2:0
      chat.fast:  anthropic.claude-3-haiku-20240307-v1:0

  fallback_chain:
    - kind: anthropic
      model_alias_map:
        chat.smart: claude-3-5-sonnet-20241022
        chat.fast:  claude-3-haiku-20240307

  call_timeout_seconds: 60
  residency: sg-1
  zdr_required: true

  emergency_override:
    enabled: true
    requires: ["cfo_signoff", "audit_row"]
    max_multiplier: 1.5

  allowed_personas: null   # any registered persona OK
```

---

## §4 — Acceptance criteria

1. **Valid YAML loads** — Given `config/tenants/test-a.yaml` containing the EXAMPLE.tenant.yaml content (with `tenant_id: org:test-a`), `init_loader()` MUST succeed; `load_for_tenant("org:test-a")` MUST return `Ok(Arc<TenantPolicy>)` matching the YAML 1:1.
2. **Missing YAML returns PolicyMissing** — `load_for_tenant("org:nobody")` (no file at `config/tenants/org-nobody.yaml`) MUST return `Err(PolicyError::PolicyMissing { tenant_id: "org:nobody" })`. MUST NOT return a default policy.
3. **Invalid schema rejected on init** — Given `config/tenants/bad.yaml` with `monthly_cap_usd: "not-a-number"`, `init_loader()` MUST return `Err(LoaderInitError::Schema)` with a structured `schema_errors` list. AI Gateway MUST exit 1 (caller responsibility) on init failure.
4. **Out-of-range values rejected** — `monthly_cap_usd: -5` MUST be rejected ("range: min 0.01"). `warn_threshold: 1.5` MUST be rejected ("range: max 0.95"). `call_timeout_seconds: 0` MUST be rejected.
5. **Path traversal in tenant_id rejected** — `load_for_tenant("../escape")` MUST return `Err(PolicyError::InvalidTenantId { reason })` without touching the filesystem.
6. **Cache hit on second call** — After `load_for_tenant("org:test-a")` succeeds once, a second call MUST return the same `Arc<TenantPolicy>` pointer. Verify via `Arc::ptr_eq`. The `ai_policy_cache_hits_total` counter MUST increment by 1.
7. **Hot reload on file modify** — Modify `config/tenants/test-a.yaml` to change `monthly_cap_usd: 150 → 200`. Within 500ms (file-watch + reload + cache swap), `load_for_tenant("org:test-a")` MUST return a policy with `monthly_cap_usd: 200`. The previous `Arc` MUST still be valid for any in-flight reader (no torn reads).
8. **Hot reload of invalid file preserves cache** — Modify `config/tenants/test-a.yaml` to invalid YAML (`monthly_cap_usd: "broken"`). The cached `Arc<TenantPolicy>` from before MUST remain returnable; `ai_policy_reload_failures_total` MUST increment by 1; a `tracing::error!` event MUST fire with the validation error.
9. **Concurrent reads have no lock contention** — Spawn 1000 tokio tasks each calling `load_for_tenant("org:test-a")` 100 times in a tight loop. Measure: total wall time. MUST be ≤ 1 second on a 4-core dev machine (effectively unbounded throughput on the read path).
10. **File deletion clears cache entry** — `rm config/tenants/test-a.yaml`. Within 500ms, `load_for_tenant("org:test-a")` MUST return `Err(PolicyMissing)` (the cache entry was removed by the file-watch).

---

## §5 — Verification method

**Integration test:** `services/ai-gateway/tests/policy_loader_test.rs`

```rust
#[tokio::test]
async fn valid_yaml_loads() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test-a.yaml"), VALID_FIXTURE).unwrap();

    let loader = policy::init_loader(dir.path()).await.unwrap();
    let p = policy::load_for_tenant("org:test-a").await.unwrap();

    assert_eq!(p.tenant_id, "org:test-a");
    assert_eq!(p.ai_policy.monthly_cap_usd, dec!(150));
    assert_eq!(p.ai_policy.warn_threshold, 0.80);
    assert_eq!(p.ai_policy.residency, Residency::Sg1);
    assert!(p.ai_policy.hard_stop);

    loader.shutdown().await;
}

#[tokio::test]
async fn missing_yaml_returns_policy_missing() {
    let dir = TempDir::new().unwrap();
    let _loader = policy::init_loader(dir.path()).await.unwrap();
    let err = policy::load_for_tenant("org:nobody").await.unwrap_err();
    assert!(matches!(err, PolicyError::PolicyMissing { tenant_id } if tenant_id == "org:nobody"));
}

#[tokio::test]
async fn invalid_schema_rejected_on_init() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("bad.yaml"), "ai_policy:\n  monthly_cap_usd: not-a-number\n").unwrap();

    let init = policy::init_loader(dir.path()).await;
    assert!(matches!(init, Err(LoaderInitError::Schema { .. })));
}

#[tokio::test]
async fn hot_reload_applies_within_500ms() {
    let dir = TempDir::new().unwrap();
    let yaml_path = dir.path().join("test-a.yaml");
    fs::write(&yaml_path, VALID_FIXTURE.replace("monthly_cap_usd: 150", "monthly_cap_usd: 150")).unwrap();

    let _loader = policy::init_loader(dir.path()).await.unwrap();
    let before = policy::load_for_tenant("org:test-a").await.unwrap();
    assert_eq!(before.ai_policy.monthly_cap_usd, dec!(150));

    fs::write(&yaml_path, VALID_FIXTURE.replace("monthly_cap_usd: 150", "monthly_cap_usd: 200")).unwrap();

    let start = Instant::now();
    loop {
        let p = policy::load_for_tenant("org:test-a").await.unwrap();
        if p.ai_policy.monthly_cap_usd == dec!(200) {
            assert!(start.elapsed() < Duration::from_millis(500));
            break;
        }
        if start.elapsed() > Duration::from_millis(500) {
            panic!("hot reload did not apply within 500ms");
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn path_traversal_rejected() {
    let dir = TempDir::new().unwrap();
    let _loader = policy::init_loader(dir.path()).await.unwrap();
    let err = policy::load_for_tenant("../etc/passwd").await.unwrap_err();
    assert!(matches!(err, PolicyError::InvalidTenantId { .. }));
}

#[tokio::test]
async fn concurrent_1000_reads_under_1s() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("test-a.yaml"), VALID_FIXTURE).unwrap();
    let _loader = policy::init_loader(dir.path()).await.unwrap();

    let start = Instant::now();
    let handles: Vec<_> = (0..1000).map(|_| {
        tokio::spawn(async {
            for _ in 0..100 { let _ = policy::load_for_tenant("org:test-a").await.unwrap(); }
        })
    }).collect();
    futures::future::join_all(handles).await;
    assert!(start.elapsed() < Duration::from_secs(1),
        "concurrent reads took {:?}, expected < 1s", start.elapsed());
}
```

Run via:

```bash
cd /Users/stephencheng/Projects/CyberSkill/cyberos
cargo test -p cyberos-ai-gateway policy
```

**Schema regeneration check (CI gate):**

```bash
cargo run -p cyberos-ai-gateway --bin gen-schema -- --out config/tenants/SCHEMA.json
git diff --exit-code config/tenants/SCHEMA.json
```

If the schema regenerates differently than what's checked in, the CI fails. This prevents silent schema drift between the Rust struct and the documented schema.

---

## §6 — Implementation skeleton

```rust
// services/ai-gateway/src/policy/cache.rs

use arc_swap::ArcSwap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct PolicyCache {
    inner: ArcSwap<HashMap<String, Arc<TenantPolicy>>>,
}

impl PolicyCache {
    pub fn new() -> Self {
        Self { inner: ArcSwap::from_pointee(HashMap::new()) }
    }

    pub fn get(&self, tenant_id: &str) -> Option<Arc<TenantPolicy>> {
        self.inner.load().get(tenant_id).cloned()
    }

    pub fn insert(&self, tenant_id: String, policy: Arc<TenantPolicy>) {
        let mut new_map = (**self.inner.load()).clone();
        new_map.insert(tenant_id, policy);
        self.inner.store(Arc::new(new_map));
    }

    pub fn remove(&self, tenant_id: &str) {
        let mut new_map = (**self.inner.load()).clone();
        new_map.remove(tenant_id);
        self.inner.store(Arc::new(new_map));
    }
}

// services/ai-gateway/src/policy/loader.rs

use notify::{RecommendedWatcher, Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use once_cell::sync::OnceCell;

static CACHE: OnceCell<PolicyCache> = OnceCell::new();
static CONFIG_DIR: OnceCell<PathBuf> = OnceCell::new();

pub async fn init_loader(config_dir: &Path) -> Result<Loader, LoaderInitError> {
    let cache = PolicyCache::new();

    // Eager load all YAMLs
    for entry in std::fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if !is_loadable_filename(name) { continue; }

        let yaml = std::fs::read_to_string(&path)?;
        let policy: TenantPolicy = serde_yaml::from_str(&yaml)
            .map_err(|e| LoaderInitError::Schema {
                file: path.clone(),
                errors: vec![e.to_string()],
            })?;
        validate_policy(&policy)?;
        cache.insert(policy.tenant_id.clone(), Arc::new(policy));
    }

    CACHE.set(cache).map_err(|_| LoaderInitError::AlreadyInitialised)?;
    CONFIG_DIR.set(config_dir.to_path_buf()).map_err(|_| LoaderInitError::AlreadyInitialised)?;

    // Spawn file-watcher
    let watcher = spawn_watcher(config_dir).await?;

    Ok(Loader { _watcher: watcher })
}

pub async fn load_for_tenant(tenant_id: &str) -> Result<Arc<TenantPolicy>, PolicyError> {
    validate_tenant_id(tenant_id)?;

    let cache = CACHE.get().ok_or(PolicyError::IoError {
        tenant_id: tenant_id.to_string(),
        source: std::io::Error::new(std::io::ErrorKind::Other, "loader not initialised"),
    })?;

    if let Some(p) = cache.get(tenant_id) {
        metrics::POLICY_CACHE_HITS.inc();
        return Ok(p);
    }

    metrics::POLICY_CACHE_MISSES.inc();
    // Fall through to disk read (rare path; only happens if a new file appears
    // between init and the read but before the file-watch event fires)
    let dir = CONFIG_DIR.get().ok_or(PolicyError::IoError {
        tenant_id: tenant_id.to_string(),
        source: std::io::Error::new(std::io::ErrorKind::Other, "config dir unknown"),
    })?;

    let file_name = format!("{}.yaml", tenant_id.replace(':', "-"));
    let path = dir.join(&file_name);
    if !path.exists() {
        return Err(PolicyError::PolicyMissing { tenant_id: tenant_id.to_string() });
    }

    let yaml = std::fs::read_to_string(&path).map_err(|e| PolicyError::IoError {
        tenant_id: tenant_id.to_string(), source: e,
    })?;
    let policy: TenantPolicy = serde_yaml::from_str(&yaml).map_err(|e| PolicyError::PolicyInvalid {
        tenant_id: tenant_id.to_string(), schema_errors: vec![e.to_string()],
    })?;
    validate_policy(&policy).map_err(|init_err| PolicyError::PolicyInvalid {
        tenant_id: tenant_id.to_string(),
        schema_errors: match init_err {
            LoaderInitError::Schema { errors, .. } => errors,
            _ => vec!["unknown validation error".into()],
        },
    })?;

    let arc = Arc::new(policy);
    cache.insert(tenant_id.to_string(), arc.clone());
    Ok(arc)
}

fn validate_tenant_id(tenant_id: &str) -> Result<(), PolicyError> {
    if tenant_id.contains("..") || tenant_id.contains('/') || tenant_id.contains('\\') {
        return Err(PolicyError::InvalidTenantId { reason: "traversal".into() });
    }
    if tenant_id.is_empty() || tenant_id.len() > 128 {
        return Err(PolicyError::InvalidTenantId { reason: "length".into() });
    }
    if !tenant_id.chars().all(|c| c.is_ascii_alphanumeric() || c == ':' || c == '-' || c == '_') {
        return Err(PolicyError::InvalidTenantId { reason: "charset".into() });
    }
    Ok(())
}

fn is_loadable_filename(name: &str) -> bool {
    name.ends_with(".yaml")
        && !name.starts_with('_')
        && name.chars().take_while(|c| *c != '.').all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

async fn spawn_watcher(config_dir: &Path) -> Result<RecommendedWatcher, notify::Error> {
    let (tx, mut rx) = mpsc::channel(64);
    let mut watcher = notify::recommended_watcher(move |res| {
        if let Ok(event) = res { let _ = tx.blocking_send(event); }
    })?;
    watcher.watch(config_dir, RecursiveMode::NonRecursive)?;

    let config_dir = config_dir.to_path_buf();
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            handle_fs_event(event, &config_dir).await;
        }
    });

    Ok(watcher)
}

async fn handle_fs_event(event: Event, config_dir: &Path) {
    let Some(path) = event.paths.first() else { return };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else { return };
    if !is_loadable_filename(name) { return; }

    let tenant_id = name.trim_end_matches(".yaml").replace('-', ":");

    match event.kind {
        EventKind::Modify(_) | EventKind::Create(_) => {
            match load_from_disk(path).await {
                Ok(policy) => {
                    if let Some(cache) = CACHE.get() {
                        cache.insert(tenant_id, Arc::new(policy));
                    }
                }
                Err(e) => {
                    metrics::POLICY_RELOAD_FAILURES.inc();
                    tracing::error!(tenant_id, ?e, "policy_reload_failed");
                }
            }
        }
        EventKind::Remove(_) => {
            if let Some(cache) = CACHE.get() {
                cache.remove(&tenant_id);
            }
        }
        _ => {}
    }
}
```

*Scaffold above is suggestive. AC §4 is the contract.*

---

## §7 — Dependencies

**Code dependencies:**
- None — this task is foundational. TASK-AI-001 consumes its output; no upstream code dependency.

**Crate dependencies (Cargo.toml additions):**
- `serde_yaml = "0.9"` — YAML parsing.
- `notify = "6"` — cross-platform file-watching.
- `schemars = "0.8"` — JSON schema generation from Rust structs.
- `arc-swap = "1"` — lock-free atomic Arc swap for the cache.
- `once_cell = "1"` — for the OnceCell statics.

**Operational dependencies:**
- A writable `config/tenants/` directory at the path passed to `init_loader()`.
- On NFS or similar filesystems where inotify doesn't fire, `notify` falls back to polling at a default 30s interval. This is acceptable for slice 1; if it becomes a problem, TASK-AI-021 surfaces the issue.

---

## §8 — Example payloads

### Loader init at gateway startup

```rust
// services/ai-gateway/src/main.rs
let _loader = policy::init_loader(Path::new("config/tenants/")).await
    .map_err(|e| anyhow::anyhow!("policy loader init failed: {:?}", e))?;
tracing::info!("policy_loader_initialised");
```

### Consumer in TASK-AI-001's precheck

```rust
let policy = policy::load_for_tenant(&req.tenant_id).await
    .map_err(|e| match e {
        PolicyError::PolicyMissing { .. } => PrecheckError::PolicyLoadFailed,
        _ => PrecheckError::PolicyLoadFailed,
    })?;

if policy.ai_policy.zdr_required && !provider_is_zdr(&policy.ai_policy.primary_provider) {
    return Ok(PrecheckOutcome::Refuse {
        reason: RefuseReason::ProviderUnavailable,
        current_spent_usd: dec!(0),
        cap_usd: policy.ai_policy.monthly_cap_usd,
    });
}
```

### YAML fixture — valid

```yaml
# services/ai-gateway/tests/fixtures/policy/valid.yaml
tenant_id: org:test-a
ai_policy:
  monthly_cap_usd: 150
  warn_threshold: 0.80
  hard_stop: true
  primary_provider:
    kind: bedrock
    region: ap-southeast-1
    model_alias_map:
      chat.smart: anthropic.claude-3-5-sonnet-20241022-v2:0
      chat.fast:  anthropic.claude-3-haiku-20240307-v1:0
  fallback_chain: []
  call_timeout_seconds: 60
  residency: sg-1
  zdr_required: true
  emergency_override:
    enabled: true
    requires: ["cfo_signoff", "audit_row"]
    max_multiplier: 1.5
```

### YAML fixture — invalid schema (wrong type)

```yaml
# services/ai-gateway/tests/fixtures/policy/invalid-schema.yaml
tenant_id: org:test-a
ai_policy:
  monthly_cap_usd: "not-a-number"   # FAILS: should be Decimal
  warn_threshold: 0.80
  hard_stop: true
  primary_provider:
    kind: bedrock
    region: ap-southeast-1
    model_alias_map: {}
  residency: sg-1
```

Error returned:

```rust
LoaderInitError::Schema {
    file: PathBuf::from("config/tenants/invalid-schema.yaml"),
    errors: vec![
        "monthly_cap_usd: invalid type: string \"not-a-number\", expected a decimal at line 3 column 19".into(),
    ],
}
```

### YAML fixture — missing required field

```yaml
# services/ai-gateway/tests/fixtures/policy/missing-required.yaml
tenant_id: org:test-a
ai_policy:
  monthly_cap_usd: 150
  # primary_provider missing — required
  residency: sg-1
```

Error returned:

```rust
LoaderInitError::Schema {
    file: ...,
    errors: vec!["missing field `primary_provider` at line 2 column 1".into()],
}
```

### OBS metric snapshot after running

```
# HELP ai_policy_cache_hits_total Number of policy reads served from cache
# TYPE ai_policy_cache_hits_total counter
ai_policy_cache_hits_total{tenant_id="org:cyberskill"} 8421
ai_policy_cache_hits_total{tenant_id="org:test-a"} 12

# HELP ai_policy_cache_misses_total Number of policy reads served from disk
# TYPE ai_policy_cache_misses_total counter
ai_policy_cache_misses_total{tenant_id="org:cyberskill"} 1
ai_policy_cache_misses_total{tenant_id="org:test-a"} 1

# HELP ai_policy_reload_failures_total Number of file-watch reloads that failed validation
# TYPE ai_policy_reload_failures_total counter
ai_policy_reload_failures_total{tenant_id="org:test-a"} 0
```

---

## §9 — Open questions

All resolved 2026-05-15 (round 2). Promoted to §1 normative clauses:

1. **~~Filename → tenant_id mapping~~** → §1 #6 (in-YAML tenant_id authoritative). Was Q1.
2. **~~Hot-reload latency on polling filesystems~~** → §1 #12 (detect + log; 35s budget on polling). Was Q2.
3. **~~Provider model_alias_map overlap with cost table~~** → deferred to slice 4 (TASK-AI-007 + alias_profile reference). Was Q3.
4. **~~allowed_personas enforcement~~** → TASK-AI-001 §1 #13 (enforced in `precheck()`). Was Q4.
5. **~~policy validate CLI subcommand~~** → §1 #13 of this task exposes `validate_yaml()`; TASK-AI-021 wraps it. Was Q5.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| No `config/tenants/` directory at init | `fs::read_dir` returns `NotFound` | `LoaderInitError::IoError`; gateway exits 1 | Operator creates the directory + at least one tenant YAML |
| Empty `config/tenants/` directory | Walked but no `.yaml` files | `init_loader()` returns Ok with empty cache; gateway starts | Tenants added later via file-watch — but first request will return `PolicyMissing` (TASK-AI-001 #9) |
| Invalid YAML in one file | `serde_yaml::from_str` returns Err | Collected in `LoaderInitError::Schema { failures }`; ALL failures reported at once | Operator fixes; redeploy |
| Schema violation (out-of-range value, missing required field) | `schemars`-derived validation | Same as above | Same as above |
| `tenant_id` mismatch with filename | Loader detects in-YAML vs filename divergence | `tracing::warn!`; file still loaded under in-YAML `tenant_id` | Optional: operator renames file to match (cosmetic only) |
| Path traversal in `load_for_tenant(tenant_id)` | `validate_tenant_id` charset check | `Err(PolicyError::InvalidTenantId)` | Caller fixes; programmer error |
| Polling-mode filesystem (NFS/Docker bind-mount) | `notify` introspection at init | `tracing::info!` `policy_loader_polling_mode`; AC #7 latency budget = 35s | Documented; OBS dashboard surfaces |
| Hot-reload of invalid YAML | `notify` event + validation failure | Cached policy preserved; `ai_policy_reload_failures_total++`; sev-2 log | Operator fixes; next file event re-validates |
| File deletion via `rm` | `notify` `Remove` event | Cache entry removed; subsequent `load_for_tenant` returns `PolicyMissing` | Operator restores file or accepts the tenant's removal |
| Race: file modify + concurrent reader | `ArcSwap` atomic swap | Reader gets either old or new policy; never torn read | No-op; ArcSwap guarantees |
| `notify` watcher dies (rare) | No file events received for >60s after a known change | Cache becomes stale; OBS detects via lag metric | Operator restarts gateway (notify reinit) |

---

## §11 — Notes

- This task is named "tenant-policy loader" but it loads the entire policy surface, not just the AI-specific knobs. As more AI Gateway concerns surface (caching, residency, persona pinning), they all hang off `TenantPolicy.ai_policy` rather than scattering across separate YAMLs. The `ai_policy` namespacing leaves room for `auth_policy`, `obs_policy`, etc., at P1+.
- At slice 1 there is exactly one tenant: `org:cyberskill`. The loader is over-engineered for a single tenant by design — building the multi-tenant story now means we don't have to retrofit it when the second tenant lands.
- The schema-regeneration CI gate (§5) is light but powerful: it prevents the kind of silent schema drift that bites multi-month projects when a developer adds a field to the Rust struct but forgets to update the YAML schema docs. The `gen-schema` binary is ~15 lines of `schemars` output.
- The `notify` crate's `RecommendedWatcher` chooses inotify (Linux), FSEvents (macOS), or ReadDirectoryChangesW (Windows). All three are sub-millisecond on local filesystems. Don't be tempted to swap for a polling-only crate "for simplicity" — the latency difference matters for cost-runaway incidents.
- TASK-AI-021 (operator CLI) will surface a `policy list` subcommand that walks the cache and prints active policies; useful for "what is my live cap for tenant X" debugging without grepping YAMLs on disk.
- Long-term migration to TASK-TEN-004's per-tenant database table: the YAML loader stays as a fallback for ops scenarios where the DB is unreachable (degraded mode). The Postgres backing store becomes primary; YAML becomes the bootstrap + recovery path.

---

*End of TASK-AI-005. Run `task-audit` next: `cargo run -p cyberos-skill-cli -- run task-audit --input '{"artefact_paths": ["docs/tasks/ai/TASK-AI-005-tenant-policy-yaml-loader/spec.md"]}'`*

**Slice-1 status after TASK-AI-005:** all 5 slice-1 tasks (TASK-AI-001 .. TASK-AI-005) are now `status: draft`. Next step per workflow §4: run `task-audit` on the batch, then the user reviews and accepts. After accept, slice 1 is implementable as 5 PRs (one per task) in dependency order: TASK-AI-005 (loader) → TASK-AI-003 (audit bridge) → TASK-AI-001 (precheck) → TASK-AI-002 (reconcile) → TASK-AI-004 (cleanup).
