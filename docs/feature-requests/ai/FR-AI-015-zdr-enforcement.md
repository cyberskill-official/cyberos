---
# ───── Machine-readable frontmatter (parsed by feature-request-audit + future fr-catalog renderer) ─────
id: FR-AI-015
title: "ZDR (Zero Data Retention) attestation table + enforcement when tenant policy requires"
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
related_frs: [FR-AI-001, FR-AI-005, FR-AI-006, FR-AI-008, FR-AI-016]
depends_on: [FR-AI-006]
blocks: []

# ───── Source contracts ─────
source_pages:
  - website/docs/modules/ai.html#zdr
  - website/docs/legal/eu-ai-act-data-governance.html
source_decisions:
  - PDPL Art. 7 (Vietnam personal-data-sale ban; no cross-border PII without explicit consent)
  - GDPR Art. 5(1)(f) integrity/confidentiality principle
  - EU AI Act Annex IV (data governance — providers MUST attest retention practices)
  - DEC-061 (ZDR is a tenant-policy-driven enforcement, not a global default)
  - archive/2026-05-14/RESEARCH_REVIEW.md §4.4 (multi-provider ZDR table needs source-URL provenance)

# ───── Build envelope ─────
language: rust 1.81
service: cyberos/services/ai-gateway/
new_files:
  - services/ai-gateway/src/zdr/mod.rs
  - services/ai-gateway/src/zdr/parse.rs
  - services/ai-gateway/src/zdr/watch.rs
  - services/ai-gateway/src/zdr/staleness.rs
  - services/ai-gateway/config/zdr_attestations.yaml          # provider × model ZDR status
  - services/ai-gateway/tests/zdr_test.rs
  - services/ai-gateway/tests/alias_resolution_test.rs
  - services/ai-gateway/tests/zdr_test.rs
  - services/ai-gateway/tests/otel_propagation_test.rs
  - .github/workflows/zdr-staleness-check.yml                 # weekly cron: warn if attestations >90d old
modified_files:
  - services/ai-gateway/src/alias.rs                          # FR-AI-006 §1 #6 invokes zdr::is_zdr
  - services/ai-gateway/src/handlers/chat.rs                  # emit ai.zdr_violation memory row on refusal
  - services/ai-gateway/src/memory_writer.rs                   # add canonical::zdr_violation builder
  - services/ai-gateway/Cargo.toml                            # url crate for source_url validation
allowed_tools:
  - file_read: services/ai-gateway/**
  - file_write: services/ai-gateway/{src,tests,config}/**
  - file_write: .github/workflows/zdr-staleness-check.yml
  - bash: cargo test -p cyberos-ai-gateway zdr
disallowed_tools:
  - self-attest ZDR for any provider (every entry MUST cite a published source URL per §1 #4)
  - bypass `policy.ai_policy.zdr_required` from any code path
  - inline ZDR booleans in alias map or cost table (single source of truth: zdr_attestations.yaml)
  - ship an entry without `attested_by` (attribution is the audit primitive per §1 #4)

# ───── Estimated work ─────
effort_hours: 6
sub_tasks:
  - "0.5h: zdr_attestations.yaml schema + initial entries (Bedrock × Claude × 4 + Anthropic + OpenAI + Vertex Gemini)"
  - "0.5h: parse_attestations + URL validation (HTTPS-only) + attested_by-required check"
  - "1.0h: zdr::is_zdr() + zdr::attestation_for() + ArcSwap registry (same pattern as FR-AI-005, FR-AI-014)"
  - "0.5h: Hot-reload via notify with revocation detection (true → false logs WARN; emits sev-2 metric)"
  - "0.5h: Staleness check (verified_at > 90 days → CI warn; > 365 days → fail-closed override)"
  - "0.5h: alias::resolve integration (FR-AI-006 §1 #6 wiring)"
  - "0.5h: ai.zdr_violation memory audit row builder + handler emission on refusal"
  - "0.5h: OTel metrics (lookups_total, violations_total, attestations_revoked_total, attestations_stale_total)"
  - "0.5h: tests — happy lookup, fail-closed, alias integration, revocation warn, staleness, URL validation"
  - "0.5h: zdr-staleness-check.yml weekly cron workflow"
  - "0.5h: Enterprise-plan note enforcement (Anthropic direct API requires Enterprise; runtime check stubbed for slice-5 follow-up)"
risk_if_skipped: "Tenant policies declaring `zdr_required: true` are unenforced. Calls leak to non-ZDR providers (OpenAI default with 30-day retention, Vertex default region-dependent). GDPR Art. 5(1)(f) data-confidentiality + PDPL Art. 6 minimisation violated for every PDPL-pinned tenant. EU AI Act Annex IV data-governance evidence missing — first regulator audit fails on the data-retention dimension, with no defence (we promised ZDR in the DPA and didn't enforce it). Worst case: a single mis-routed call to a non-ZDR provider with VN-PII triggers PDPL Art. 7 cross-border-data action; remediation cost dwarfs the 6 hours of gate work."
---

## §1 — Description (BCP-14 normative)

The AI Gateway service **MUST** maintain an authoritative ZDR (Zero Data Retention) attestation table and enforce it at alias-resolution time when tenant policy demands ZDR. The table and enforcement together obey the following:

1. **MUST** load `services/ai-gateway/config/zdr_attestations.yaml` at gateway startup (boot order: after FR-AI-007 cost-table loader, before binding the HTTP server). Every (provider, model) combination MUST be explicitly enumerated; the file is the single source of truth.
2. **MUST** expose `zdr::is_zdr(provider: &ProviderKind, model: &str) -> bool` and `zdr::attestation_for(provider: &ProviderKind, model: &str) -> Option<ZdrAttestation>`. The boolean is the gate; the attestation struct is the audit-trail primitive.
3. **MUST** default `is_zdr` to `false` for any (provider, model) NOT in the table — fail closed. The lookup must never return `true` based on a missing entry, a parse fallback, or a "trust the provider" heuristic. A non-attested call when policy requires ZDR is a refusal, not an unknown.
4. **MUST** require every YAML entry to carry `is_zdr` (bool), `verified_at` (ISO date), `source_url` (HTTPS URL — HTTP rejected at parse), and `attested_by` (kebab-case email or username). `notes` is optional. An entry missing any required field MUST fail `init_zdr_table` with `LoaderInitError::Schema { reason }`. ZDR claims without source = audit failure; the parser blocks them at boot.
5. **MUST** be consulted by `alias::resolve` (FR-AI-006 §1 #6 invokes this FR) BEFORE returning a resolved (provider, model). When `policy.ai_policy.zdr_required == true` AND `zdr::is_zdr(&resolved_provider, &resolved_model) == false`, `alias::resolve` MUST return `Err(AliasError::ZdrViolation { resolved_provider, resolved_model, attestation: <option> })`. The `attestation` field surfaces the documented status (e.g., a known-non-ZDR provider returns the entry with `is_zdr: false`; a missing entry returns `None`) so the operator can distinguish "we know this isn't ZDR" from "we have no record."
6. **MUST** emit an `ai.zdr_violation` memory audit row when a request is refused due to ZDR. The row carries `tenant_id`, `agent_persona`, `requested_alias`, `resolved_provider`, `resolved_model`, `policy_requires_zdr: true`, `attestation_present: bool`, `request_id`. The audit-before-refusal invariant from FR-AI-001 §1 #6 applies.
7. **MUST** be hot-reloadable via the `notify` crate's file-watch with a 250ms debounce (same machinery as FR-AI-005, FR-AI-007, FR-AI-014). On reload-success, the new attestations replace the cache via `ArcSwap::store`.
8. **MUST** detect ZDR-status revocations on hot-reload: for every (provider, model) where the previous cache entry had `is_zdr == true` AND the new entry has `is_zdr == false`, emit `tracing::warn!` AND increment `ai_zdr_attestations_revoked_total{provider, model}`. Operators MUST be alerted (the metric is a sev-2 alarm trigger).
9. **MUST** implement staleness handling at two thresholds. (a) **Soft stale** at `verified_at + 90 days`: weekly CI cron (`.github/workflows/zdr-staleness-check.yml`) flags the entry; PR-time validation logs WARN; metric `ai_zdr_attestations_stale_total{provider, model}` increments. (b) **Hard stale** at `verified_at + 365 days`: `is_zdr` MUST return `false` regardless of the table's recorded value (defensive override; an attestation no one has reverified for a year is no longer trustworthy). The hard-stale path also logs `tracing::error!` and increments `ai_zdr_attestations_expired_total`.
10. **MUST** validate `source_url` at parse time: scheme MUST be `https`, host MUST be a valid DNS name (or known provider domain — `aws.amazon.com`, `anthropic.com`, `platform.openai.com`, `cloud.google.com`, etc.). HTTP URLs and bare paths are rejected with `LoaderInitError::InvalidSourceUrl`. The validator does NOT fetch the URL (no network call at parse time); it only validates syntax + scheme.
11. **MUST** validate `attested_by` at parse time as either `<localpart>@cyberos.world` (CyberSkill staff) OR `<auditor-id>@<approved-auditor-domain>` (third-party auditor on a maintained allow-list). Random strings are rejected with `LoaderInitError::InvalidAttestor`. The allow-list lives in `parse.rs` constants for slice 3; FR-AI-022 will move it to a separate config.
12. **MUST** integrate with `policy.ai_policy.zdr_required` from FR-AI-005's tenant policy schema. The flag is read once per request via FR-AI-001's policy load; this FR reads it through the request context, never directly from disk.
13. **MUST** propagate `ZdrViolation` errors as HTTP `403 ZDR_VIOLATION` with body `{"error":"zdr_violation","resolved_provider":"<p>","resolved_model":"<m>","policy_requires_zdr":true,"contact":"ops@cyberos.world"}`. The body MUST NOT echo the attestation's `notes` field (operator-facing explanation, not customer-facing).
14. **SHOULD** emit OTel metrics:
    - `ai_zdr_lookups_total{provider, model, outcome}` (counter; outcome ∈ `attested | missing | revoked | expired`).
    - `ai_zdr_violations_total{tenant_id}` (counter; alarm threshold > 0 over a 5-minute window).
    - `ai_zdr_attestations_revoked_total{provider, model}` (counter; sev-2 alert on increment).
    - `ai_zdr_attestations_stale_total{provider, model}` (counter; weekly evaluated).
    - `ai_zdr_attestations_expired_total{provider, model}` (counter; sev-1 alert).
    - `ai_zdr_table_size` (gauge; current attestation count).
15. **SHOULD** log at INFO level on every successful hot-reload: `zdr_table_reloaded count=<N> sources={hash16}` so operators can verify "did my edit actually load?".

---

## §2 — Why this design (rationale for humans)

**Why a separate YAML, not inline in the cost-table or alias map?** ZDR is a legal-compliance assertion that drifts independently from cost. Bedrock might be ZDR today but a new model launched on Bedrock might not initially carry the same attestation — even though the cost-table entry exists for billing. Coupling ZDR to cost or alias creates two failure modes: (a) updating cost flips ZDR by accident; (b) reviewing ZDR requires reading three files. Keeping ZDR in its own file with its own owner means the audit conversation ("show me the ZDR attestations for the period under review") is one file, one history.

**Why fail closed (default false, §1 #3)?** Same reasoning as FR-AI-001 §1 #9 (policy-missing fails closed): silent defaults bury compliance failures. Missing entry = treat as non-ZDR = refuse if policy demands ZDR. The alternative — defaulting to `true` for "providers we trust" — is exactly the failure mode that produces a "we promised ZDR and didn't enforce it" headline.

**Why does each row carry `verified_at` + `source_url` + `attested_by` (§1 #4)?** When an auditor asks "prove you only sent PDPL-protected data to ZDR-attested providers," the chain of evidence is:
1. Tenant policy says `zdr_required: true` (memory row + YAML).
2. Request was routed to (provider, model).
3. `zdr_attestations.yaml` shows that (provider, model) had `is_zdr: true` at the time of the request, citing a published source URL.
4. The attestor (`attested_by`) is identifiable; if questioned, they can produce the source they read.

Each piece of provenance is load-bearing; an attestation without `source_url` is a claim, not evidence. The parse-time enforcement of all four required fields is the discipline that keeps the table audit-grade.

**Why HTTPS-only source URLs (§1 #10)?** A source URL on HTTP can be MITMed — an attacker between the auditor and the documentation could serve a forged "we have ZDR" page. HTTPS isn't perfect but it's the floor. Rejecting HTTP at parse time is free and prevents accidentally-shipped HTTP citations from a copy-paste error.

**Why two-tier staleness (90d soft, 365d hard, §1 #9)?** Vendor policies change. SOC 2 cadence is annual reassessment for vendor controls; we mirror that as the hard floor. Soft stale at 90d (matching standard quarterly vendor-review cycles) gives operators a reminder before things go critical. Hard stale at 365d is the defensive override: an attestation no human has reverified for a year cannot be trusted to still describe reality. The enforcement is automatic — not "we should reverify" but "the bool returns false until reverified" — so the failure mode is loud (a request refusal) not silent (a stale attestation continuing to gate calls open).

**Why does revocation produce a WARN log + metric, not an automatic refusal (§1 #8)?** Revocation is an operational signal: "the attestor changed `is_zdr: true → false` deliberately, probably because the vendor changed their policy." The flip itself is the new authoritative state — calls to that (provider, model) start being refused immediately. The WARN + metric exist so operators *know* it happened (so they can email affected tenants) — not to undo the flip.

**Why a separate `attested_by` field rather than git blame?** Git blame works for slice 3 (small team, all attestations from `stephen@cyberos.world`). At scale (multi-attestor org, third-party auditors), the attestor is operationally relevant: "who do I email to question this attestation?" An auditor's id (e.g., a third-party SOC 2 firm) is not in our git history. Surfacing `attested_by` in the YAML AND in the audit row makes the attribution explicit.

**Why is the enforcement at `alias::resolve` time, not at the precheck (FR-AI-001) time?** The precheck has the alias name (`chat.smart`), not the resolved (provider, model). Resolution happens later; that's where the (provider, model) is known. Enforcing at resolution means the gate fires at the right point with the right data, AND we get exactly one enforcement site (no duplication, no risk of one path checking and another not). The cost is one extra error class (`ZdrViolation`) propagating from `alias::resolve` to the handler — small.

**Why a separate memory audit row (`ai.zdr_violation`, §1 #6)?** A regulator's audit of "did we ever send PDPL data to a non-ZDR provider" needs a search target. Without the dedicated row, the evidence is "absence of evidence" (no request went through to the non-ZDR provider, but how do we prove we didn't try?). The dedicated row converts the question to "show me all `ai.zdr_violation` rows for tenant X" — a positive answer, not an absence. The row is the proof we *did* refuse.

**Why doesn't `LoaderInitError::Schema` fail-closed-on-init?** It does (`Result<(), LoaderInitError>` propagates up through the boot path; the gateway refuses to bind). The §10 inventory makes this explicit. Inability to load the ZDR table is treated as severe as inability to load the cost table — both are compliance-load-bearing primitives.

---

## §3 — API contract (formal spec for AI-agent implementers)

### Type definitions

```rust
// services/ai-gateway/src/zdr/mod.rs

use std::sync::Arc;
use std::path::Path;
use arc_swap::ArcSwap;
use chrono::NaiveDate;
use once_cell::sync::OnceCell;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ZdrAttestation {
    pub is_zdr: bool,
    pub verified_at: NaiveDate,
    pub source_url: String,                // HTTPS-validated at parse
    pub attested_by: String,               // <localpart>@cyberos.world OR approved-auditor
    pub notes: Option<String>,
}

pub fn is_zdr(provider: &ProviderKind, model: &str) -> bool;
pub fn attestation_for(provider: &ProviderKind, model: &str) -> Option<ZdrAttestation>;
pub async fn init_zdr_table(config_path: &Path) -> Result<(), LoaderInitError>;

#[derive(Debug, thiserror::Error)]
pub enum LoaderInitError {
    #[error("zdr_attestations.yaml malformed: {reason}")]
    Schema { reason: String },
    #[error("invalid source_url at {provider}/{model}: must be https://, got {url}")]
    InvalidSourceUrl { provider: String, model: String, url: String },
    #[error("invalid attested_by at {provider}/{model}: {value}")]
    InvalidAttestor { provider: String, model: String, value: String },
    #[error("zdr table already initialised; init_zdr_table called twice")]
    AlreadyInitialised,
    #[error("io error reading config: {0}")]
    Io(#[from] std::io::Error),
}

// In FR-AI-006 alias.rs (modified_files):
pub enum AliasError {
    // ... existing variants ...
    ZdrViolation {
        resolved_provider: ProviderKind,
        resolved_model: String,
        attestation: Option<ZdrAttestation>,   // None ⇒ no entry; Some(a) ⇒ entry shows is_zdr=false
    },
}

static TABLE: OnceCell<ArcSwap<HashMap<(ProviderKind, String), ZdrAttestation>>> = OnceCell::new();
```

### YAML schema (with all required fields enforced at parse)

```yaml
# services/ai-gateway/config/zdr_attestations.yaml
version: 1
last_updated: 2026-05-15

# Each entry MUST cite a source. ZDR claims without source = audit failure.
# Entries lacking is_zdr / verified_at / source_url / attested_by FAIL init.
attestations:
  bedrock:
    "anthropic.claude-3-5-sonnet-20241022-v2:0":
      is_zdr: true
      verified_at: 2026-05-15
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"
      notes: "Bedrock guarantees no-data-retention per their data privacy whitepaper"

    "anthropic.claude-3-haiku-20240307-v1:0":
      is_zdr: true
      verified_at: 2026-05-15
      source_url: "https://aws.amazon.com/bedrock/data-privacy/"
      attested_by: "stephen@cyberos.world"

  anthropic:
    "claude-3-5-sonnet-20241022":
      is_zdr: true
      verified_at: 2026-05-15
      source_url: "https://www.anthropic.com/legal/zero-data-retention"
      attested_by: "stephen@cyberos.world"
      notes: "ZDR available on Enterprise plan only; runtime tenant-tier check is FR-AI-022 follow-up"

  openai:
    "gpt-4o":
      is_zdr: false
      verified_at: 2026-05-15
      source_url: "https://platform.openai.com/docs/models#data-policy"
      attested_by: "stephen@cyberos.world"
      notes: |
        Standard OpenAI retains data 30 days by default. ZDR requires the
        zero-data-retention org policy enabled per OpenAI's documentation;
        CyberOS has not validated this. Treat as non-ZDR until policy enabled.

  vertex:
    "gemini-2.0-pro":
      is_zdr: false
      verified_at: 2026-05-15
      source_url: "https://cloud.google.com/vertex-ai/docs/general/data-governance"
      attested_by: "stephen@cyberos.world"
      notes: "Vertex retention is region-dependent; ZDR not validated for VN tenants"
```

### Parser contract

```rust
// services/ai-gateway/src/zdr/parse.rs

use url::Url;

const APPROVED_AUDITOR_DOMAINS: &[&str] = &[
    "cyberos.world",
    // Third-party auditor allow-list:
    "kpmg.com.vn",
    "ey.com",
    "deloitte.com",
    // ... maintained by ops; FR-AI-022 moves to separate config
];

pub fn parse_attestations(yaml: &str) -> Result<HashMap<(ProviderKind, String), ZdrAttestation>, LoaderInitError> {
    let raw: serde_yaml::Value = serde_yaml::from_str(yaml)
        .map_err(|e| LoaderInitError::Schema { reason: e.to_string() })?;

    let attestations = raw.get("attestations")
        .ok_or_else(|| LoaderInitError::Schema { reason: "missing 'attestations' root key".into() })?;

    let mut out = HashMap::new();
    for (provider_str, models) in attestations.as_mapping().unwrap() {
        let provider = ProviderKind::from_str(provider_str.as_str().unwrap())
            .map_err(|e| LoaderInitError::Schema { reason: format!("unknown provider {provider_str:?}: {e}") })?;
        for (model_str, fields) in models.as_mapping().unwrap() {
            let model = model_str.as_str().unwrap().to_string();
            let att = parse_one_attestation(&provider, &model, fields)?;
            out.insert((provider, model), att);
        }
    }
    Ok(out)
}

fn parse_one_attestation(
    provider: &ProviderKind, model: &str, fields: &serde_yaml::Value,
) -> Result<ZdrAttestation, LoaderInitError> {
    let map = fields.as_mapping().ok_or_else(|| LoaderInitError::Schema {
        reason: format!("{provider:?}/{model}: not a mapping"),
    })?;

    let is_zdr = map.get(&"is_zdr".into()).and_then(|v| v.as_bool())
        .ok_or_else(|| LoaderInitError::Schema { reason: format!("{provider:?}/{model}: missing is_zdr") })?;

    let verified_at_s = map.get(&"verified_at".into()).and_then(|v| v.as_str())
        .ok_or_else(|| LoaderInitError::Schema { reason: format!("{provider:?}/{model}: missing verified_at") })?;
    let verified_at = NaiveDate::parse_from_str(verified_at_s, "%Y-%m-%d")
        .map_err(|e| LoaderInitError::Schema { reason: format!("{provider:?}/{model}: bad verified_at: {e}") })?;

    let source_url = map.get(&"source_url".into()).and_then(|v| v.as_str())
        .ok_or_else(|| LoaderInitError::Schema { reason: format!("{provider:?}/{model}: missing source_url") })?
        .to_string();
    validate_source_url(provider, model, &source_url)?;

    let attested_by = map.get(&"attested_by".into()).and_then(|v| v.as_str())
        .ok_or_else(|| LoaderInitError::Schema { reason: format!("{provider:?}/{model}: missing attested_by") })?
        .to_string();
    validate_attested_by(provider, model, &attested_by)?;

    let notes = map.get(&"notes".into()).and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(ZdrAttestation { is_zdr, verified_at, source_url, attested_by, notes })
}

fn validate_source_url(provider: &ProviderKind, model: &str, url: &str) -> Result<(), LoaderInitError> {
    let parsed = Url::parse(url).map_err(|_| LoaderInitError::InvalidSourceUrl {
        provider: format!("{provider:?}"), model: model.into(), url: url.into(),
    })?;
    if parsed.scheme() != "https" {
        return Err(LoaderInitError::InvalidSourceUrl {
            provider: format!("{provider:?}"), model: model.into(), url: url.into(),
        });
    }
    Ok(())
}

fn validate_attested_by(provider: &ProviderKind, model: &str, value: &str) -> Result<(), LoaderInitError> {
    let Some((_local, domain)) = value.split_once('@') else {
        return Err(LoaderInitError::InvalidAttestor {
            provider: format!("{provider:?}"), model: model.into(), value: value.into(),
        });
    };
    if !APPROVED_AUDITOR_DOMAINS.contains(&domain) {
        return Err(LoaderInitError::InvalidAttestor {
            provider: format!("{provider:?}"), model: model.into(), value: value.into(),
        });
    }
    Ok(())
}
```

### Staleness check

```rust
// services/ai-gateway/src/zdr/staleness.rs

use chrono::{Duration, Utc};

pub const SOFT_STALE_DAYS: i64 = 90;
pub const HARD_STALE_DAYS: i64 = 365;

pub fn is_soft_stale(att: &ZdrAttestation) -> bool {
    Utc::now().date_naive() - att.verified_at > Duration::days(SOFT_STALE_DAYS)
}

pub fn is_hard_stale(att: &ZdrAttestation) -> bool {
    Utc::now().date_naive() - att.verified_at > Duration::days(HARD_STALE_DAYS)
}
```

```rust
// In zdr/mod.rs::is_zdr — apply hard-stale override per §1 #9.
pub fn is_zdr(provider: &ProviderKind, model: &str) -> bool {
    let table = match TABLE.get() {
        Some(t) => t.load(),
        None => return false,
    };
    let key = (*provider, model.to_string());
    match table.get(&key) {
        None => {
            metrics::lookup(&key, "missing");
            false
        }
        Some(att) if staleness::is_hard_stale(att) => {
            metrics::lookup(&key, "expired");
            tracing::error!(provider=?provider, model=%model, verified_at=%att.verified_at,
                            "zdr attestation HARD-stale (>365d); forcing is_zdr=false");
            false
        }
        Some(att) => {
            if staleness::is_soft_stale(att) {
                metrics::soft_stale(&key);
            }
            metrics::lookup(&key, if att.is_zdr { "attested" } else { "missing" });
            att.is_zdr
        }
    }
}
```

### CI staleness workflow

```yaml
# .github/workflows/zdr-staleness-check.yml
name: ZDR Attestation Staleness Check
on:
  schedule:
    - cron: '0 0 * * 1'   # every Monday 00:00 UTC
  workflow_dispatch: {}

jobs:
  check-staleness:
    runs-on: ubuntu-22.04
    permissions:
      issues: write
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/setup-rust-toolchain@v1
      - name: Run staleness check
        working-directory: services/ai-gateway
        run: cargo run --bin zdr-staleness-check -- config/zdr_attestations.yaml
      - name: Open issue if soft-stale entries found
        if: failure()
        uses: actions/github-script@v7
        with:
          script: |
            github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `ZDR attestations soft-stale (>90 days) — refresh due`,
              labels: ['compliance', 'zdr', 'staleness'],
              body: `Per FR-AI-015 §1 #9, one or more ZDR attestations exceed 90 days.\n\nRefresh by visiting each provider's published policy and bumping verified_at.`
            });
```

---

## §4 — Acceptance criteria (testable, ordered, numbered)

1. **ZDR-attested provider passes** — `is_zdr(&Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0")` returns `true`.
2. **Documented non-ZDR fails** — `is_zdr(&OpenAI, "gpt-4o")` returns `false`.
3. **Missing entry fails closed** — `is_zdr(&Vertex, "gemini-9.9.9")` (not in table) returns `false`; metric `ai_zdr_lookups_total{outcome="missing"}` increments.
4. **FR-AI-006 integration: refusal when policy requires** — Tenant policy `zdr_required: true`; alias resolves to `openai:gpt-4o`; `alias::resolve` returns `Err(AliasError::ZdrViolation { attestation: Some(att_with_is_zdr_false), .. })`.
5. **FR-AI-006 integration: refusal when entry missing** — Tenant policy `zdr_required: true`; alias resolves to a (provider, model) not in the table; `alias::resolve` returns `Err(AliasError::ZdrViolation { attestation: None, .. })`.
6. **HTTP 403 ZDR_VIOLATION on refusal** — Handler converts the AliasError to a `403` response with the documented body shape; `notes` field NOT echoed in response body.
7. **Audit row emitted** — Every `ZdrViolation` refusal emits exactly one `ai.zdr_violation` memory row carrying tenant_id, resolved_provider, resolved_model, policy_requires_zdr, attestation_present, request_id.
8. **Hot reload picks up new attestation** — Adding a new entry to YAML; within 500ms `is_zdr` returns the new value.
9. **Revocation warns and metricises** — Changing an entry from `is_zdr: true` to `is_zdr: false` triggers `tracing::warn!` AND increments `ai_zdr_attestations_revoked_total{provider, model}`.
10. **Soft staleness flagged in CI** — `verified_at = today - 91 days`; `cargo run --bin zdr-staleness-check` exits non-zero with the entry listed; weekly cron opens an issue.
11. **Hard staleness forces is_zdr=false** — `verified_at = today - 366 days`; `is_zdr` returns `false` regardless of the YAML's recorded value; `tracing::error!` fired; metric `ai_zdr_attestations_expired_total` incremented.
12. **HTTP source_url rejected at parse** — A YAML entry with `source_url: "http://..."` fails `init_zdr_table` with `InvalidSourceUrl`.
13. **Bare-string attestor rejected at parse** — A YAML entry with `attested_by: "alice"` (no `@`) fails with `InvalidAttestor`.
14. **Out-of-domain attestor rejected at parse** — A YAML entry with `attested_by: "alice@gmail.com"` fails with `InvalidAttestor` (domain not in approved list).
15. **Required-field validation: missing source_url** — A YAML entry without `source_url` fails `init_zdr_table` with `Schema`.
16. **Required-field validation: missing attested_by** — A YAML entry without `attested_by` fails `init_zdr_table` with `Schema`.
17. **Attestation provenance retrievable** — `attestation_for(&Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0")` returns a `ZdrAttestation` with all five fields populated.
18. **Concurrent lookups + hot-reload safe** — 100 tokio tasks calling `is_zdr` concurrently while a hot-reload runs see either old or new state, never torn or panicking.
19. **Double-init rejected** — `init_zdr_table` then second `init_zdr_table` returns `LoaderInitError::AlreadyInitialised`.

---

## §5 — Verification

```rust
// services/ai-gateway/tests/zdr_test.rs
use cyberos_ai_gateway::zdr::{self, init_zdr_table};
use cyberos_ai_gateway::providers::ProviderKind;
use std::path::Path;

#[tokio::test]
async fn zdr_attested_passes_and_non_attested_fails_closed() {
    init_zdr_table(Path::new("config/zdr_attestations.yaml")).await.unwrap();

    // AC #1
    assert!(zdr::is_zdr(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0"));
    // AC #2
    assert!(!zdr::is_zdr(&ProviderKind::OpenAI, "gpt-4o"));
    // AC #3
    assert!(!zdr::is_zdr(&ProviderKind::Vertex, "gemini-9.9.9"));
}

#[tokio::test]
async fn alias_resolve_refuses_when_policy_requires_zdr() {
    init_zdr_table(Path::new("config/zdr_attestations.yaml")).await.unwrap();
    let policy = test_policy_with_zdr_required(true);
    let result = alias::resolve("chat.smart-non-zdr", &policy);   // alias maps to openai:gpt-4o
    match result {
        Err(AliasError::ZdrViolation { resolved_provider, attestation, .. }) => {
            assert_eq!(resolved_provider, ProviderKind::OpenAI);
            assert!(matches!(attestation, Some(a) if !a.is_zdr));
        }
        _ => panic!("expected ZdrViolation"),
    }
}

#[tokio::test]
async fn audit_row_emitted_on_zdr_refusal() {
    let request_id = "req_test_zdr_001";
    let tenant_id = "tenant_alpha";
    let _ = handlers::chat::handle(test_request(tenant_id, request_id, "chat.smart-non-zdr")).await;
    let rows = memory_test_helper::find_rows("ai.zdr_violation", request_id);
    assert_eq!(rows.len(), 1);
    let p = &rows[0].payload;
    assert_eq!(p["tenant_id"], tenant_id);
    assert_eq!(p["resolved_provider"], "openai");
    assert_eq!(p["resolved_model"], "gpt-4o");
    assert_eq!(p["policy_requires_zdr"], true);
    assert_eq!(p["attestation_present"], true);
}

#[tokio::test]
async fn http_source_url_rejected() {
    let yaml = r#"
        version: 1
        attestations:
          openai:
            "gpt-4o":
              is_zdr: false
              verified_at: "2026-05-15"
              source_url: "http://platform.openai.com/policy"
              attested_by: "stephen@cyberos.world"
    "#;
    let err = zdr::parse::parse_attestations(yaml).expect_err("expected InvalidSourceUrl");
    assert!(matches!(err, LoaderInitError::InvalidSourceUrl { .. }));
}

#[tokio::test]
async fn bare_string_attestor_rejected() {
    let yaml = r#"
        version: 1
        attestations:
          openai:
            "gpt-4o":
              is_zdr: false
              verified_at: "2026-05-15"
              source_url: "https://platform.openai.com/policy"
              attested_by: "alice"
    "#;
    let err = zdr::parse::parse_attestations(yaml).expect_err("expected InvalidAttestor");
    assert!(matches!(err, LoaderInitError::InvalidAttestor { .. }));
}

#[tokio::test]
async fn missing_source_url_rejected() {
    let yaml = r#"
        version: 1
        attestations:
          openai:
            "gpt-4o":
              is_zdr: false
              verified_at: "2026-05-15"
              attested_by: "stephen@cyberos.world"
    "#;
    let err = zdr::parse::parse_attestations(yaml).expect_err("expected Schema");
    match err {
        LoaderInitError::Schema { reason } => assert!(reason.contains("source_url")),
        e => panic!("wrong variant: {e:?}"),
    }
}
```

```rust
// services/ai-gateway/tests/alias_resolution_test.rs
#[tokio::test]
async fn revocation_warns_and_metricises() {
    init_zdr_table(Path::new("config/zdr_attestations.yaml")).await.unwrap();
    let path = "config/zdr_attestations.yaml";
    let original = std::fs::read_to_string(path).unwrap();
    let revoked = original.replace(
        "is_zdr: true\n      verified_at: 2026-05-15\n      source_url: \"https://aws.amazon.com/bedrock/data-privacy/\"",
        "is_zdr: false\n      verified_at: 2026-05-15\n      source_url: \"https://aws.amazon.com/bedrock/data-privacy/\"",
    );
    std::fs::write(path, &revoked).unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    assert!(!zdr::is_zdr(&ProviderKind::Bedrock, "anthropic.claude-3-5-sonnet-20241022-v2:0"));
    let counter = otel_test_helper::counter_value(
        "ai_zdr_attestations_revoked_total",
        &[("provider", "bedrock"), ("model", "anthropic.claude-3-5-sonnet-20241022-v2:0")],
    );
    assert!(counter >= 1, "revocation counter not incremented");

    std::fs::write(path, &original).unwrap();
}
```

```rust
// services/ai-gateway/tests/zdr_test.rs
#[test]
fn soft_stale_at_91_days() {
    let att = ZdrAttestation {
        is_zdr: true,
        verified_at: chrono::Utc::now().date_naive() - chrono::Duration::days(91),
        source_url: "https://x".into(), attested_by: "stephen@cyberos.world".into(), notes: None,
    };
    assert!(zdr::staleness::is_soft_stale(&att));
    assert!(!zdr::staleness::is_hard_stale(&att));
}

#[test]
fn hard_stale_at_366_days_overrides_is_zdr() {
    let att = ZdrAttestation {
        is_zdr: true,
        verified_at: chrono::Utc::now().date_naive() - chrono::Duration::days(366),
        source_url: "https://x".into(), attested_by: "stephen@cyberos.world".into(), notes: None,
    };
    assert!(zdr::staleness::is_hard_stale(&att));

    // Force-inject att into table; is_zdr MUST return false.
    test_helper::inject_attestation(&ProviderKind::Bedrock, "test-model", att);
    assert!(!zdr::is_zdr(&ProviderKind::Bedrock, "test-model"));
}
```

```bash
cd services/ai-gateway
cargo test -p cyberos-ai-gateway zdr
```

CI gate: cargo-test runs on every PR touching `services/ai-gateway/src/zdr/**`, `services/ai-gateway/config/zdr_attestations.yaml`, or `services/ai-gateway/src/alias.rs` (FR-AI-006 wiring).

---

## §6 — Implementation skeleton

See §3 for type defs + parser + staleness module. Hot-reload follows FR-AI-014's pattern (250ms debounce, ArcSwap pointer-swap, parse-error keeps cache):

```rust
// services/ai-gateway/src/zdr/watch.rs
pub fn spawn_watcher_with_revocation_detection(config_path: &Path) {
    let path = config_path.to_path_buf();
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher = notify::recommended_watcher(tx).unwrap();
        watcher.watch(&path, notify::RecursiveMode::NonRecursive).unwrap();

        for ev in rx {
            if ev.is_err() { continue; }
            std::thread::sleep(std::time::Duration::from_millis(250));
            // Drain debounce window.
            while rx.try_recv().is_ok() {}

            match reload_with_diff(&path) {
                Ok((new_size, revocations)) => {
                    tracing::info!(count = new_size, "zdr_table_reloaded");
                    for (key, prev_is_zdr) in revocations {
                        if prev_is_zdr {
                            tracing::warn!(provider = ?key.0, model = %key.1,
                                          "zdr attestation REVOKED (was true, now false or missing)");
                            metrics::revocation(&key);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, "zdr table reload failed; cache unchanged");
                    metrics::reload_failure(&e);
                }
            }
        }
    });
}

fn reload_with_diff(path: &Path) -> Result<(usize, Vec<((ProviderKind, String), bool)>), LoaderInitError> {
    let yaml = std::fs::read_to_string(path)?;
    let new = parse::parse_attestations(&yaml)?;
    let old = TABLE.get().unwrap().load();
    let mut revocations = vec![];
    for (key, old_att) in old.iter() {
        let new_is_zdr = new.get(key).map(|a| a.is_zdr).unwrap_or(false);
        if old_att.is_zdr && !new_is_zdr {
            revocations.push((key.clone(), old_att.is_zdr));
        }
    }
    TABLE.get().unwrap().store(Arc::new(new.clone()));
    Ok((new.len(), revocations))
}
```

`canonical::zdr_violation` builder (added to FR-AI-003's memory bridge):

```rust
pub mod canonical {
    pub fn zdr_violation(
        tenant_id: &str, agent_persona: &str, requested_alias: &str,
        resolved_provider: &ProviderKind, resolved_model: &str,
        attestation_present: bool, request_id: &str,
    ) -> AuditRow {
        AuditRow {
            kind: "ai.zdr_violation".into(),
            payload: serde_json::json!({
                "tenant_id": tenant_id,
                "agent_persona": agent_persona,
                "requested_alias": requested_alias,
                "resolved_provider": format!("{resolved_provider:?}").to_lowercase(),
                "resolved_model": resolved_model,
                "policy_requires_zdr": true,
                "attestation_present": attestation_present,
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

- **FR-AI-006** — `alias::resolve` invokes `zdr::is_zdr` (declared in FR-AI-006 §1 #6); this FR provides the implementation. The `AliasError::ZdrViolation` variant is added to FR-AI-006's enum.
- **FR-AI-001** — Cost ledger precheck routes the request; `ZdrViolation` errors propagate through the precheck path to the handler.
- **FR-AI-005** — Tenant policy schema declares `policy.ai_policy.zdr_required`; this FR consumes it via the request context.
- **FR-AI-003** — memory audit-row bridge. This FR adds the `canonical::zdr_violation` builder for the `ai.zdr_violation` row kind (declared in FR-AI-003 §3).
- **FR-AI-016 (downstream)** — Residency pinning is the natural pair of ZDR; both gates live behind tenant policy. FR-AI-016 may extend this FR's table with regional-attestation columns.
- **FR-AI-022 (downstream)** — OTel trace emission consumes `ai_zdr_*` metrics.

### Concept dependencies (shared types)

- `ZdrAttestation` is the single attestation primitive used in lookups, audit rows, and operator queries. The five required fields (is_zdr, verified_at, source_url, attested_by, notes) are immutable across slice 3.
- `(ProviderKind, String)` is the attestation key. ProviderKind enum is declared in FR-AI-008's multi-provider router; this FR uses the same enum.
- `APPROVED_AUDITOR_DOMAINS` is the attestor allow-list (cyberos.world + third-party SOC 2 firms). Maintained in `parse.rs` constants for slice 3; FR-AI-022 moves to a separate config.
- Soft/hard staleness thresholds (90d / 365d) are baked-in constants matching SOC 2 cadence; changes require explicit FR amendment.

### Operational / external

- Rust crates: `arc-swap@1`, `notify@6`, `chrono@0.4`, `serde_yaml@0.9`, `url@2`, `once_cell@1`, `thiserror@1`.
- The `url` crate parses + validates `source_url` syntax (HTTPS-only enforced by us).
- GitHub Actions for the weekly staleness cron.
- Initial entries cite Bedrock, Anthropic, OpenAI, Vertex documentation pages; these are evergreen URLs maintained by each vendor.

---

## §8 — Example payloads

### Caller in FR-AI-006 alias.rs

```rust
// FR-AI-006 §1 #6 (modified to call zdr::is_zdr)
pub fn resolve(alias: &str, policy: &TenantPolicy) -> Result<(ProviderKind, String), AliasError> {
    let (provider, model) = ALIAS_MAP.get().unwrap().load().get(alias)
        .ok_or_else(|| AliasError::UnknownAlias(alias.into()))?;

    if policy.ai_policy.zdr_required && !zdr::is_zdr(&provider, &model) {
        let attestation = zdr::attestation_for(&provider, &model);
        return Err(AliasError::ZdrViolation {
            resolved_provider: provider, resolved_model: model.clone(), attestation,
        });
    }

    Ok((provider, model.clone()))
}
```

### Audit row `ai.zdr_violation`

```json
{
  "kind": "ai.zdr_violation",
  "ts_ns": 1747526400000000000,
  "payload": {
    "tenant_id": "tenant_alpha",
    "agent_persona": "cuo-cpo@0.4.1",
    "requested_alias": "chat.smart",
    "resolved_provider": "openai",
    "resolved_model": "gpt-4o",
    "policy_requires_zdr": true,
    "attestation_present": true,
    "request_id": "req_01HZK9R8M3X5C8Q4"
  }
}
```

### HTTP refusal

```text
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": "zdr_violation",
  "resolved_provider": "openai",
  "resolved_model": "gpt-4o",
  "policy_requires_zdr": true,
  "contact": "ops@cyberos.world"
}
```

### Attestation lookup for audit reporting

```rust
let att = zdr::attestation_for(&ProviderKind::Bedrock,
                              "anthropic.claude-3-5-sonnet-20241022-v2:0").unwrap();
println!("ZDR: {} | source: {} | attested: {} on {}",
    att.is_zdr, att.source_url, att.attested_by, att.verified_at);
// => ZDR: true | source: https://aws.amazon.com/bedrock/data-privacy/
//    attested: stephen@cyberos.world on 2026-05-15
```

### Hot-reload INFO log (success)

```text
INFO  zdr_table_reloaded count=12
```

### Hot-reload WARN log (revocation)

```text
WARN  provider=Bedrock model=anthropic.claude-3-5-sonnet-20241022-v2:0
      zdr attestation REVOKED (was true, now false or missing)
```

### Hard-stale ERROR log

```text
ERROR provider=Vertex model=gemini-2.0-pro verified_at=2025-04-15
      zdr attestation HARD-stale (>365d); forcing is_zdr=false
```

### Weekly staleness CI output

```text
$ cargo run --bin zdr-staleness-check -- config/zdr_attestations.yaml
Soft-stale entries (>90 days; refresh due):
  vertex/gemini-2.0-pro          verified_at=2026-02-10 (95 days old)
  openai/gpt-4o                  verified_at=2026-02-13 (92 days old)
2 entries flagged. Exiting non-zero.
```

---

## §9 — Open questions

All resolved at authoring time. Items deferred to later FRs:

- Enterprise-plan tenant-tier check (Anthropic direct API requires Enterprise plan for ZDR; current FR ships the note-as-documentation but doesn't runtime-validate the tenant tier) — FR-AI-022 follow-up.
- Per-region ZDR attestation (Vertex retention is region-dependent; current table is provider+model granularity) — FR-AI-016 (residency pinning) area.
- Auto-refresh attestations from provider APIs (programmatic scrape of provider policy pages) — out of scope; the human-in-the-loop verification step is the audit primitive.
- Signed attestations (cryptographic signature from the attestor) — out of scope; `attested_by` + git-blame is sufficient evidence.
- Multi-attestor rows (e.g., both `stephen@cyberos.world` AND `auditor@kpmg.com.vn` co-attest) — slice 5; current schema is one attestor per entry.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Missing ZDR attestation for (provider, model) | HashMap miss in `is_zdr` | Returns false (fail closed); metric `lookups_total{outcome=missing}` | Operator adds entry to YAML |
| Provider revokes ZDR (true→false on hot-reload) | Diff-detect in `reload_with_diff` | `tracing::warn!` + metric `attestations_revoked_total` increment; subsequent lookups return false | Operator notifies affected tenants |
| Provider entry deleted from YAML on reload | Diff sees old entry true, new entry absent | Counts as revocation; same WARN+metric path | Operator confirms intent |
| YAML parse error at init | `LoaderInitError::Schema` | Gateway exits 1 (refuses to bind) | Operator fixes YAML; redeploy |
| YAML parse error at hot-reload | Reload fails; cache unchanged | INFO log "reload failed"; metric `reload_failure_total` | Operator fixes YAML; next file-watch event triggers retry |
| Concurrent lookup + hot-reload | ArcSwap atomic | Reader sees old or new state, never torn | By design (§1 #7) |
| Concurrent hot-reload + reload (rapid edits) | 250ms debounce | One reparse runs; events queue | By design (§1 #7) |
| `source_url` is HTTP not HTTPS | `validate_source_url` parser check | `LoaderInitError::InvalidSourceUrl` → init fails | Operator changes URL to HTTPS |
| `attested_by` is bare string (no `@`) | `validate_attested_by` parser check | `LoaderInitError::InvalidAttestor` → init fails | Operator uses `<localpart>@cyberos.world` |
| `attested_by` domain not in approved list | `validate_attested_by` allow-list check | `LoaderInitError::InvalidAttestor` → init fails | Operator uses approved domain or extends allow-list (PR review) |
| Required field missing (is_zdr, verified_at, source_url, attested_by) | `parse_one_attestation` field checks | `LoaderInitError::Schema` → init fails | Operator fills missing field |
| Soft-stale entry (>90d) | Weekly CI cron check; `is_soft_stale` | CI fails non-zero; GitHub issue opened; metric `attestations_stale_total` increments | Operator reviews provider policy; bumps verified_at |
| Hard-stale entry (>365d) | `is_hard_stale` check in `is_zdr` | `is_zdr` forced to false regardless of recorded value; ERROR log + metric `attestations_expired_total` | Operator reverifies attestation immediately; if confirmed still ZDR, bumps verified_at |
| Tenant policy `zdr_required` missing | FR-AI-005 schema default | Defaults to false (no enforcement for that tenant) | Operator updates tenant policy YAML if ZDR is required |
| Alias resolves to provider not in table | `is_zdr` returns false; `attestation_for` returns None | `ZdrViolation { attestation: None }` | Operator adds attestation OR removes the alias mapping |
| Audit row emit fails (memory bridge down) | `memory_writer::emit` returns Err | Refusal still proceeds; sev-1 log ("ZDR refused but audit row failed") | Operator investigates memory; FR-AI-003 §10 covers |
| Revocation notification missed (operator absent) | OTel alarm on `attestations_revoked_total` | Alarm pages on-call | Standard incident response |
| `notes` field accidentally echoed in 403 response body | Integration test asserts `notes` absent | Test fails → PR blocked | Handler MUST scrub `notes` before serialising response |
| New provider added without entry | `is_zdr` returns false → calls refused if policy requires ZDR | New-provider rollout PR includes attestation entry | Standard PR process |
| `notify` watcher thread panics | tokio observability | Watcher dies; hot-reload stops; cache continues serving old state | sev-2 alert; restart gateway |
| Double init (test re-entry) | `OnceCell::set` returns Err | `LoaderInitError::AlreadyInitialised` | Tests use `reset_for_tests()` |

---

## §11 — Notes

- The OpenAI `gpt-4o` entry intentionally records `is_zdr: false` with notes — many readers assume OpenAI defaults to ZDR; they don't (30-day retention by default; ZDR requires the org-level zero-data-retention policy). Documenting it inline prevents copy-paste errors.
- The 90-day soft-stale + 365-day hard-stale cadence matches industry SOC 2 expectations for vendor-control reassessment. The hard-stale auto-override is the defence-in-depth: even if the operator misses every weekly reminder, the 365th day automatically force-fails the gate.
- The `attested_by` allow-list is small and PR-reviewed. Adding an auditor domain (e.g., a new SOC 2 firm) requires a code change with reviewer approval — keeping the attestation surface small and known.
- HTTPS-only source URLs is a small but real defence against documentation tampering. A copy-pasted `http://` URL might survive operator review by accident; the parser-level rejection makes accidents impossible.
- The hard-stale override (§1 #9) is the single most important defensive property here. Without it, a benign "we forgot to refresh the attestation" becomes a silent compliance failure that an auditor might catch a year later. With it, the failure is loud (calls refused) the day after the 365-day mark — operators get pinged immediately.
- The audit row (`ai.zdr_violation`) is the proof-of-refusal primitive. Regulators investigating "did you ever route PDPL data to a non-ZDR provider for tenant X" can search the chain for `ai.zdr_violation` rows scoped to that tenant; positive results prove the gate fired.
- The Anthropic "Enterprise plan only" caveat in the YAML is documentation-only for slice 3. A tenant on a non-Enterprise Anthropic plan whose calls route to Anthropic direct will get ZDR-attested results from this gate but might NOT actually have ZDR coverage (Anthropic's enforcement is at the API key tier). FR-AI-022 will add the runtime tenant-tier check; until then, ops manually validates Enterprise tier per tenant during onboarding.
- The OTel metric set (`ai_zdr_*`) is designed for direct dashboarding. The `attestations_expired_total` metric should pager-alert (sev-1) on any non-zero increment because it indicates the hard-stale safety net engaged — operators should reverify within hours, not days.
- Future expansion (FR-AI-016): regional ZDR attestation. Vertex's "is_zdr" depends on which region the call routes to; current schema collapses all regions into one boolean. The residency-pinning FR will likely extend this schema with `regions: { vn: true, eu: true, us: false }` per attestation. Until then, the conservative approach is to mark Vertex as `is_zdr: false` (current state) until per-region attestation lands.

---

*End of FR-AI-015. Status: draft (10/10 target).*
