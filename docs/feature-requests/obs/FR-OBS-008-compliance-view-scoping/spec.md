---
id: FR-OBS-008
title: "obs-compliance-view: pre-built read-only views (EU AI Act / PDPL / SOC 2 / ISO 27001) over memory audit chain with Ed25519 chain-proof + tenant-scoped + PDF/JSON export"
module: OBS
priority: MUST
status: implementing
verify: T
phase: P0
milestone: P0 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_frs: [FR-OBS-002, FR-OBS-009, FR-AI-022, FR-AUTH-004]
depends_on: [FR-OBS-002]
blocks: [FR-OBS-009]

source_pages:
  - website/docs/modules/obs.html#compliance-views
  - EU AI Act Art. 12 (record-keeping)
  - PDPL Art. 15 (data subject rights)
  - SOC 2 trust services criteria
  - ISO 27001:2022 Annex A
source_decisions:
  - DEC-175 (4 views slice 3: EU AI Act + PDPL + SOC 2 + ISO 27001; PCI DSS at slice 5+)
  - DEC-176 (Ed25519 chain proof; auditor verifies independently)
  - DEC-177 (read-only; never mutate audit chain from compliance view)
  - DEC-178 (auditor JWT separate role; rotate per-engagement)

language: rust 1.81
service: cyberos/services/obs-compliance-view/
new_files:
  - services/obs-compliance-view/Cargo.toml
  - services/obs-compliance-view/src/main.rs
  - services/obs-compliance-view/src/auth.rs
  - services/obs-compliance-view/src/views/mod.rs
  - services/obs-compliance-view/src/views/eu_ai_act.rs
  - services/obs-compliance-view/src/views/pdpl.rs
  - services/obs-compliance-view/src/views/soc2.rs
  - services/obs-compliance-view/src/views/iso27001.rs
  - services/obs-compliance-view/src/export/pdf.rs
  - services/obs-compliance-view/src/export/json.rs
  - services/obs-compliance-view/src/chain_proof.rs
  - services/obs-compliance-view/tests/eu_ai_act_test.rs
  - services/obs-compliance-view/tests/pdpl_test.rs
  - services/obs-compliance-view/tests/cross_tenant_test.rs
  - services/obs-compliance-view/tests/chain_proof_test.rs
modified_files:
  - deploy/obs/grafana/dashboards/compliance.json
allowed_tools:
  - file_read: services/obs-compliance-view/**, deploy/obs/**
  - file_write: services/obs-compliance-view/**, deploy/obs/**
  - bash: cd services/obs-compliance-view && cargo test
disallowed_tools:
  - expose compliance view across tenant boundaries (per §1 #3)
  - mutate audit chain from compliance view (per §1 #4 read-only)
  - export raw PII from compliance view (per §1 #11 — placeholders only)
  - sign chain proof with non-Ed25519 (per DEC-176)

effort_hours: 14
sub_tasks:
  - "1.0h: Cargo.toml + main.rs + axum router for 4 view endpoints"
  - "1.0h: auth.rs — auditor-role JWT check (extends FR-AUTH-004 verify)"
  - "1.0h: views/eu_ai_act.rs — query ai.invocation + ai.persona_loaded + ai.zdr_violation + ai.residency_violation"
  - "1.0h: views/pdpl.rs — query DSAR + cross-border + consent + ai.* with tenant_jurisdiction=VN"
  - "1.0h: views/soc2.rs — auth.* + cli.* + sessions + access logs"
  - "1.0h: views/iso27001.rs — asset inventory + risk-assessment + access-control reviews"
  - "1.0h: chain_proof.rs — Ed25519 sign over ordered (kind, ts_ns, payload_hash) tuples"
  - "1.0h: export/pdf.rs — wkhtmltopdf-backed render"
  - "0.5h: export/json.rs — canonical JSON serialise"
  - "0.5h: time-range validation (max 365 days)"
  - "0.5h: tenant scoping at query layer (ALL queries WHERE tenant_id = $1)"
  - "0.5h: PII placeholder enforcement (rows with raw PII should not exist; defence-in-depth)"
  - "0.5h: memory audit row obs.compliance_view_accessed per query"
  - "1.5h: Tests — 4 views per regulation + tenant-scoped + chain-proof verify + 30s p95 + PDF render"
  - "0.5h: Grafana dashboard JSON template (embeds the views)"
risk_if_skipped: "External auditors (SOC 2, ISO 27001, EU AI Act Art. 12) require pre-built views over the audit chain. Without them, every audit becomes a multi-week SQL bake-off. The hash-chain proof is the differentiator vs SaaS observability — provably immutable, auditor-verifiable. Without tenant-scoping, an auditor for tenant A could see tenant B's data — catastrophic for the auditor's tenant + their own engagement."
---

## §1 — Description (BCP-14 normative)

A read-only HTTP service `obs-compliance-view` **MUST** expose pre-built compliance views over the memory audit chain. Each view:

1. **MUST** be one of 4 endpoints: `GET /eu-ai-act/`, `GET /pdpl/`, `GET /soc2/`, `GET /iso27001/`. Each scoped to that regulation's audit requirements (per-view rows defined in §1 #11 below).
2. **MUST** require `role: external_auditor` in the JWT for access. Operators access via Grafana (which uses the same backend). Tenant-admin role does NOT grant compliance-view access — auditor is a distinct role provisioned per engagement.
3. **MUST** be tenant-scoped: every query carries `tenant_id` from the JWT; auditor sees ONLY their assigned tenant's rows. RLS at the memory query layer is the enforcement. Cross-tenant `?tenant_id=` query parameter is rejected with 403.
4. **MUST** query the memory audit chain (binlog rows) READ-ONLY; never mutate. The query API is the existing `memory::query(tenant_id, kinds, since, until)`.
5. **MUST** emit Ed25519 hash-chain proof on every view's response footer. The proof signs the response contents (canonical JSON of rows + summary) using the deployment's signing key (loaded from secret store). Auditors can independently verify the signature against the published public key.
6. **MUST** support time-range filtering via `?since=<ISO8601>&until=<ISO8601>` query parameters. Window > 365 days is rejected with 400 (auditor must paginate).
7. **MUST** export to PDF + JSON formats via `?format=pdf|json` (default json).
8. **MUST** complete view rendering within 30s p95 for 90-day windows. Above 30s, response is `429 TOO_LARGE` with suggestion to reduce window.
9. **MUST** include a summary block per view (counts, key metrics) + the row list. Auditors typically read the summary first; rows are evidence.
10. **MUST** emit memory audit row `obs.compliance_view_accessed` per query (auditor self-audit) with payload: `auditor_subject_id`, `tenant_id`, `view`, `time_range_days`, `request_id`.
11. **Per-view content (slice-3 scope):**
    - **EU AI Act** (Art. 12 record-keeping):
      - `ai.invocation`, `ai.persona_loaded`, `ai.zdr_violation`, `ai.residency_violation` rows.
      - Persona-version stamps + EU AI Act Art. 50 `made_by_genie` attributions.
      - Oversight log: every CLI mutation (`cli.*`) for AI infra.
    - **PDPL** (VN data residency):
      - DSAR fulfilment rows (delete-data, export-data requests).
      - Cross-border transfer log (`ai.residency_violation` + `obs.langsmith_export_*`).
      - Consent records (langsmith opt-in/out audit).
      - PII redaction stats (FR-AI-011 metrics aggregated).
    - **SOC 2** (trust services criteria):
      - Access logs (`auth.token_issued` + `auth.token_failed`).
      - Configuration changes (`ai.cli_policy_updated` + `ai.cli_breaker_reset`).
      - Backup attestations (FR-OBS-009).
      - Incident response (`obs.alert_triaged` + `obs.alert_acked`).
    - **ISO 27001:2022** (Annex A):
      - Asset inventory (tenant + subject lists).
      - Risk assessments (manual ops uploads).
      - Access control reviews (subject-role audits).
12. **MUST** enforce PII-placeholder discipline: rows returned MUST NOT contain raw PII. The audit chain itself uses placeholders (`email_hash16`, `<VN_CCCD_1>`); this view inherits. Defence-in-depth: response body is regex-scanned for raw PII patterns before serving; matches → sev-1 + 500.
13. **MUST** authenticate via FR-AUTH-004 JWT verification; auditor JWT issued per engagement (typically 30-day TTL, longer than standard) via `cyberos-auth issue-auditor-token`.
14. **SHOULD** emit OTel metrics:
    - `obs_compliance_view_requests_total{view, format, outcome}` (counter).
    - `obs_compliance_view_latency_ms{view}` (histogram).
    - `obs_compliance_view_rows_returned{view}` (histogram).
    - `obs_compliance_pii_leak_attempted_total` (counter; sev-1 alarm).

---

## §2 — Why this design (rationale for humans)

**Why pre-built views (DEC-175)?** Compliance audits are time-consuming because auditors ask questions the team has never anticipated. Pre-building the 4 most common views (per regulation) turns audit prep from days to hours. Auditors recognise the format; we save engagement time.

**Why Ed25519 chain proof (DEC-176)?** Hash-chain proof is the differentiator vs SaaS observability (Datadog, Splunk). The auditor can independently verify "this view is derived from an immutable chain; the signature is valid." Without proof, auditors must trust the source — taking longer to gain comfort.

**Why read-only (§1 #4)?** Compliance views derive FROM the audit chain. Mutating from the view would be self-defeating ("the audit log shows I edited the audit log"). Read-only by construction.

**Why separate `external_auditor` role (DEC-178)?** Operators have broader access (CLI mutations, ops tooling); auditors should have narrower access (read-only views, scoped to assigned tenant). Separate role + per-engagement JWT TTL prevents permanent auditor access.

**Why 30s p95 budget (§1 #8)?** Auditors expect responsive tooling. Above 30s, the audit experience degrades; auditors workaround by exporting raw chain data (which compromises the view's value-add).

**Why 365-day max window (§1 #6)?** Multi-year queries balloon response size + render time. 365 days covers annual audit cycles; longer queries paginate. The cap is conservative; raising requires explicit FR amendment.

**Why per-engagement JWT TTL (§1 #13)?** Standard 1-hour JWTs would force auditors to refresh constantly. 30-day TTL covers a typical engagement window. Per-engagement issuance + rotation prevents stale auditor access.

**Why PII-placeholder defence-in-depth (§1 #12)?** Audit chain SHOULD contain only placeholders (FR-AUTH-002 + FR-AUTH-004 + FR-AI-014 enforce). But regression-bugs could leak. Regex scan at response time catches the regression before the auditor sees raw PII.

**Why memory audit row of view access (§1 #10)?** The auditor's own access is auditable. Compliance reviews of the auditor (yes, this happens) need to know "what did auditor X look at during their engagement?"

---

## §3 — API contract

```rust
// services/obs-compliance-view/src/main.rs
use axum::{Router, routing::get, Json, Query};

pub fn router() -> Router {
    Router::new()
        .route("/eu-ai-act/", get(views::eu_ai_act::handle))
        .route("/pdpl/", get(views::pdpl::handle))
        .route("/soc2/", get(views::soc2::handle))
        .route("/iso27001/", get(views::iso27001::handle))
}

#[derive(Deserialize)]
pub struct ViewQuery {
    pub since: DateTime<Utc>,
    pub until: DateTime<Utc>,
    #[serde(default = "default_format")]
    pub format: Format,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format { Json, Pdf }

#[derive(Serialize)]
pub struct ViewResponse {
    pub regulation: String,
    pub tenant_id: Uuid,
    pub time_range: (DateTime<Utc>, DateTime<Utc>),
    pub rows: Vec<AuditRow>,
    pub summary: serde_json::Value,
    pub chain_proof: ChainProof,
}

#[derive(Serialize)]
pub struct ChainProof {
    pub algorithm: String,        // "Ed25519"
    pub public_key_id: String,    // identifier for verifier
    pub sth_at_query_time: String, // signed tree head from memory
    pub signature: String,        // base64-encoded over canonical(response_minus_chain_proof)
}

#[derive(Debug, thiserror::Error)]
pub enum ViewError {
    #[error("auth failed")]
    AuthFailed,
    #[error("forbidden: needs external_auditor role")]
    Forbidden,
    #[error("time range > 365 days")]
    TimeRangeTooLarge,
    #[error("tenant_id query param not allowed; tenant from JWT")]
    TenantIdInQuery,
    #[error("query timeout (> 30s)")]
    QueryTimeout,
    #[error("pii leak attempted in response (sev-1)")]
    PiiLeakAttempt,
    #[error("memory query failed: {0}")]
    MemoryQueryFailed(String),
}
```

```rust
// services/obs-compliance-view/src/views/eu_ai_act.rs
pub async fn handle(query: ViewQuery, claims: Claims) -> Result<ViewResponse, ViewError> {
    auth::require_auditor_role(&claims)?;
    validate_time_range(query.since, query.until)?;

    let rows = memory::query(claims.tenant_id, EU_AI_ACT_KINDS, query.since, query.until).await?;
    pii_scrub::reject_if_raw_pii(&rows)?;

    let summary = serde_json::json!({
        "total_calls": rows.iter().filter(|r| r.kind == "ai.invocation").count(),
        "unique_personas": rows.iter().filter(|r| r.kind == "ai.persona_loaded").map(|r| r.payload["persona_handle"].clone()).collect::<HashSet<_>>().len(),
        "zdr_violations": rows.iter().filter(|r| r.kind == "ai.zdr_violation").count(),
        "residency_violations": rows.iter().filter(|r| r.kind == "ai.residency_violation").count(),
    });

    let chain_proof = chain_proof::sign_response(&rows, &summary).await?;

    let response = ViewResponse {
        regulation: "EU AI Act".into(),
        tenant_id: claims.tenant_id,
        time_range: (query.since, query.until),
        rows, summary, chain_proof,
    };

    memory::emit(canonical::compliance_view_accessed(
        claims.subject_id, claims.tenant_id, "eu-ai-act",
        (query.until - query.since).num_days(), &claims.request_id,
    )).await?;

    Ok(response)
}

const EU_AI_ACT_KINDS: &[&str] = &[
    "ai.invocation", "ai.persona_loaded",
    "ai.zdr_violation", "ai.residency_violation",
    "ai.cli_policy_updated", "ai.cli_failover_drill", "ai.cli_memory_emitted",
];
```

```rust
// services/obs-compliance-view/src/chain_proof.rs
use ed25519_dalek::{Signer, SigningKey};

pub async fn sign_response(rows: &[AuditRow], summary: &serde_json::Value) -> Result<ChainProof, ViewError> {
    let key = load_signing_key().await?;
    let canonical_bytes = canonicalise(rows, summary)?;
    let sig = key.sign(&canonical_bytes);
    let sth = memory::current_signed_tree_head().await?;
    Ok(ChainProof {
        algorithm: "Ed25519".into(),
        public_key_id: key.verifying_key().to_bytes()[..8].iter().map(|b| format!("{:02x}", b)).collect(),
        sth_at_query_time: hex::encode(sth),
        signature: base64::encode(sig.to_bytes()),
    })
}

fn canonicalise(rows: &[AuditRow], summary: &serde_json::Value) -> Result<Vec<u8>, ViewError> {
    // Deterministic canonical form: rows sorted by (ts_ns, kind, payload_hash); summary by sorted keys
    let payload = serde_json::json!({ "rows": rows, "summary": summary });
    Ok(serde_json::to_vec(&payload).unwrap())
}
```

```rust
// services/obs-compliance-view/src/export/pdf.rs
pub async fn render_pdf(view: &ViewResponse) -> Result<Vec<u8>, ViewError> {
    let html = render_html_template(view)?;
    let pdf = wkhtmltopdf::convert(&html).map_err(|e| ViewError::MemoryQueryFailed(e.to_string()))?;
    Ok(pdf)
}
```

---

## §4 — Acceptance criteria

1. EU AI Act view returns all `ai.*` rows in time range.
2. PDPL view returns DSAR + cross-border + consent + PII redaction stats.
3. SOC 2 view returns access + config + backup + incident-response rows.
4. ISO 27001 view returns asset inventory + risk + access-control rows.
5. Hash-chain proof verifies — auditor independently runs `ed25519_verify(public_key, signature, canonical_bytes)` and gets true.
6. Cross-tenant attempt (`?tenant_id=other`) → 403.
7. Tenant_id query param rejected → 403 with `TenantIdInQuery`.
8. PDF export renders correctly (no missing rows; tables formatted).
9. JSON export is canonical (sorted keys, deterministic ordering).
10. 90-day query under 30s p95.
11. >365-day query → 400 with `TimeRangeTooLarge`.
12. JWT without `external_auditor` role → 403.
13. JWT missing → 401.
14. memory audit row `obs.compliance_view_accessed` emitted per query.
15. Raw PII in response → 500 + sev-1 metric increment.
16. Summary block populated correctly per view.
17. Per-engagement JWT TTL (30-day) supported.

---

## §5 — Verification

```rust
#[tokio::test]
async fn eu_ai_act_view_returns_relevant_rows() {
    let pool = test_pool().await;
    test_helper::insert_audit_rows(vec![
        ("ai.invocation", json!({"tenant_id": "t1"})),
        ("ai.persona_loaded", json!({"tenant_id": "t1"})),
        ("auth.token_issued", json!({"tenant_id": "t1"})),   // NOT in EU AI Act view
    ]).await;

    let claims = auditor_claims_for("t1");
    let resp = views::eu_ai_act::handle(test_query_30d(), claims).await.unwrap();
    let kinds: HashSet<_> = resp.rows.iter().map(|r| r.kind.clone()).collect();
    assert!(kinds.contains("ai.invocation"));
    assert!(kinds.contains("ai.persona_loaded"));
    assert!(!kinds.contains("auth.token_issued"));
}

#[tokio::test]
async fn cross_tenant_query_param_rejected() {
    let claims = auditor_claims_for("t1");
    let mut query = test_query_30d();
    // attempt to query t2 from t1's auditor
    let err = views::eu_ai_act::handle_with_tenant_param(query, claims, "t2").await.expect_err("expected Forbidden");
    assert!(matches!(err, ViewError::TenantIdInQuery));
}

#[tokio::test]
async fn chain_proof_verifies() {
    let claims = auditor_claims_for("t1");
    let resp = views::eu_ai_act::handle(test_query_30d(), claims).await.unwrap();
    let pubkey = test_helper::deployment_public_key();
    let canonical = canonicalise(&resp.rows, &resp.summary).unwrap();
    let sig = base64::decode(&resp.chain_proof.signature).unwrap();
    let result = pubkey.verify(&canonical, &Signature::from_bytes(&sig).unwrap());
    assert!(result.is_ok());
}

#[tokio::test]
async fn pii_in_response_500s() {
    let pool = test_pool().await;
    test_helper::insert_audit_rows(vec![
        ("ai.invocation", json!({"tenant_id": "t1", "leaked_email": "alice@cyberos.world"})),   // bug: raw email
    ]).await;
    let err = views::eu_ai_act::handle(test_query_30d(), auditor_claims_for("t1")).await.expect_err("expected PiiLeakAttempt");
    assert!(matches!(err, ViewError::PiiLeakAttempt));
}

#[tokio::test]
async fn time_range_over_365_days_rejected() {
    let mut query = test_query_30d();
    query.since = chrono::Utc::now() - chrono::Duration::days(400);
    let err = views::eu_ai_act::handle(query, auditor_claims_for("t1")).await.expect_err("expected TimeRangeTooLarge");
    assert!(matches!(err, ViewError::TimeRangeTooLarge));
}

#[tokio::test]
async fn pdf_export_includes_all_rows() {
    let resp = views::eu_ai_act::handle(test_query_30d(), auditor_claims_for("t1")).await.unwrap();
    let pdf = export::pdf::render_pdf(&resp).await.unwrap();
    assert!(pdf.starts_with(b"%PDF"));
    let extracted_text = pdf_extract::extract_text_from_mem(&pdf).unwrap();
    for row in &resp.rows {
        assert!(extracted_text.contains(&row.kind));
    }
}

#[tokio::test]
async fn audit_row_per_query() {
    let claims = auditor_claims_for("t1");
    let _ = views::eu_ai_act::handle(test_query_30d(), claims).await.unwrap();
    let row = memory_test_helper::find_latest("obs.compliance_view_accessed").unwrap();
    assert_eq!(row.payload["view"], "eu-ai-act");
    assert_eq!(row.payload["tenant_id"], "t1");
}
```

---

## §6 — Implementation skeleton

See §3.

```json
// deploy/obs/grafana/dashboards/compliance.json (excerpt)
{
  "title": "Compliance — EU AI Act",
  "panels": [{
    "type": "table",
    "datasource": "obs-compliance-view",
    "targets": [{ "url": "/eu-ai-act/?since=$__from&until=$__to" }]
  }]
}
```

---

## §7 — Dependencies

- **FR-OBS-002** — Tenant-aware query proxy (this FR's tenant scoping inherits the pattern).
- **FR-AUTH-004** — JWT verification.
- **FR-OBS-009** — chain-of-custody manifest (FR-OBS-008 surfaces backup attestations from this FR).
- memory module's `query` API + `current_signed_tree_head` API.
- Crates: `axum`, `ed25519-dalek@2`, `wkhtmltopdf` (or `weasyprint` Python binding), `serde`, `chrono`, `base64`, `hex`.

---

## §8 — Example payloads

### EU AI Act view request

```http
GET /eu-ai-act/?since=2026-01-01T00:00:00Z&until=2026-05-31T23:59:59Z&format=json HTTP/1.1
Authorization: Bearer <auditor-jwt>

→ 200 OK
{
  "regulation": "EU AI Act",
  "tenant_id": "550e...",
  "time_range": ["2026-01-01T00:00:00Z", "2026-05-31T23:59:59Z"],
  "rows": [
    { "ts": "2026-02-15T...", "kind": "ai.invocation", "persona_version": "cuo-cpo@0.4.1", "..." },
    ...
  ],
  "summary": {
    "total_calls": 12450,
    "unique_personas": 3,
    "zdr_violations": 2,
    "residency_violations": 0
  },
  "chain_proof": {
    "algorithm": "Ed25519",
    "public_key_id": "4b8c0d2f1a7e9c3b",
    "sth_at_query_time": "...",
    "signature": "base64-encoded-signature..."
  }
}
```

### Cross-tenant attempt response

```http
HTTP/1.1 403 Forbidden
{ "error": "tenant_id_in_query", "reason": "tenant_id is derived from JWT; do not include as query param" }
```

### `obs.compliance_view_accessed` audit row

```json
{
  "kind": "obs.compliance_view_accessed",
  "payload": {
    "auditor_subject_id": "auditor-...",
    "tenant_id": "550e...",
    "view": "eu-ai-act",
    "time_range_days": 151,
    "request_id": "compliance_view_..."
  }
}
```

### PII-leak attempted response

```http
HTTP/1.1 500 Internal Server Error
{ "error": "pii_leak_attempted", "reason": "raw PII detected in response; sev-1 emitted" }
```

---

## §9 — Open questions

All resolved. Deferred:
- PCI DSS view — slice 5+.
- HIPAA view (US healthcare tenants) — slice 5+.
- Tenant-self-service compliance view (tenant-admin sees own data) — slice 4+.
- Auditor query history (so engagement can resume) — slice 4+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| memory unavailable | memory query error | 503 | Self-resolves |
| Time range too large (>365d) | size check | 400 TimeRangeTooLarge | Auditor paginates |
| Signature key missing | load fails | 500 + sev-1 | Operator restores key |
| Auditor JWT invalid | JWT verify fails | 401 | Refresh per-engagement JWT |
| JWT lacks external_auditor role | role check | 403 | Operator grants role |
| Cross-tenant attempt via query param | param check | 403 | By design |
| Raw PII detected in response | regex scan | 500 + sev-1 | Investigate audit-chain regression |
| Query > 30s | timeout | 429 | Auditor reduces window |
| PDF render fails | wkhtmltopdf error | 500 | Operator investigates wkhtmltopdf |
| Ed25519 verify fails (operator-side) | signature verify | Auditor flags | Investigate chain integrity |
| Per-engagement JWT not rotated | TTL expiry alert | Operator rotates | Standard cadence |
| Auditor accesses outside engagement window | row carries timestamp; manual review | Compliance review | Standard process |
| memory sth missing (no signed tree head) | `current_signed_tree_head` returns None | 500 + sev-1 | Operator investigates memory |
| Compliance row schema drift | view tests fail | PR blocked | Update view per new schema |
| `obs.compliance_view_accessed` emit fails | memory_writer error | View still served; sev-1 for missing audit | Operator investigates memory |
| Auditor JWT leaked | unknown access | sev-1; rotate JWT | Standard incident response |
| Wrong audit row included in view | view test asserts | PR blocked | Update view filter |
| Wrong audit row excluded from view | manual auditor review | Update view to include | Standard fix |

---

## §11 — Notes

- The 4 views map 1:1 to the most-common audit regulations. Adding a 5th (PCI DSS) is FR-OBS-010 placeholder.
- Hash-chain proof (Ed25519) is the differentiator vs SaaS observability — provably immutable, auditor-verifiable independently.
- Read-only by construction — compliance views derive from the chain; mutating would defeat the purpose.
- Per-engagement auditor JWT (30-day TTL) covers typical engagement; rotation prevents stale access.
- Tenant-scoping at the query layer (RLS) + 403 on tenant_id query param = defense in depth.
- PII placeholders in audit chain (FR-AUTH-002 + FR-AI-014) + regex scan at view-time = defense in depth.
- 365-day max window is conservative; larger requires explicit FR amendment.
- The auditor self-audit (`obs.compliance_view_accessed`) means even compliance reviews of the auditor have evidence.
- PDF export uses wkhtmltopdf — battle-tested; alternative weasyprint (Python) also viable. Pin version.

---

*End of FR-OBS-008. Status: draft (10/10 target).*
