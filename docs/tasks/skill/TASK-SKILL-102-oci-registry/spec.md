---
id: TASK-SKILL-102
title: "Self-hosted OCI registry for .skill bundles — cosign signing + tenant-scoped + immutable tags + 100MB cap + audit"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-15T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: SKILL
priority: p0
status: done
verify: T
phase: P1
milestone: P1 · slice 1
slice: 1
owner: Stephen Cheng (CTO)
created: 2026-05-15
shipped: null
memory_chain_hash: null
related_tasks: [TASK-SKILL-101, TASK-AUTH-004, TASK-AUTH-003]
depends_on: [TASK-SKILL-101]
# placeholder — OCI registry deploy task (R3 stage), not yet specified
blocks: [TASK-SKILL-201]

source_pages:
  - website/docs/modules/skill.html#oci-registry
source_decisions:
  - DEC-205 (self-hosted zot; no DockerHub for skill distribution)
  - DEC-206 (cosign Ed25519 signing on every push; verify on every pull)
  - DEC-207 (immutable tags; never overwrite published version)
  - DEC-208 (tenant-scoped registry namespace; tenant A can't push to tenant B)

language: rust 1.81
service: cyberos/services/skill-registry/
new_files:
  - services/skill-registry/Cargo.toml
  - services/skill-registry/src/main.rs
  - services/skill-registry/src/oci.rs
  - services/skill-registry/src/cosign_verify.rs
  - services/skill-registry/src/auth.rs
  - services/skill-registry/src/tenant_scope.rs
  - services/skill-registry/src/audit.rs
  - services/skill-registry/src/storage.rs
  - services/skill-registry/src/bin/cyberos_skill.rs
  - services/skill-registry/tests/push_test.rs
  - services/skill-registry/tests/pull_test.rs
  - services/skill-registry/tests/cosign_test.rs
  - services/skill-registry/tests/immutability_test.rs
  - services/skill-registry/tests/cross_tenant_test.rs
  - deploy/skill-registry/docker-compose.yml
  - deploy/skill-registry/zot-config.json
modified_files: []
allowed_tools:
  - file_read: services/skill-registry/**, deploy/skill-registry/**
  - file_write: services/skill-registry/**, deploy/skill-registry/**
  - bash: cd services/skill-registry && cargo test
  - bash: cosign sign-blob bundle.tar.zst
disallowed_tools:
  #2)
  - allow push without cosign signature (per §1
  #5 immutable)
  - allow tag overwrite (per §1
  #4)
  - cross-tenant push/pull (per §1
  #3)
  - skip memory audit on push or pull (per §1
  #11)
  - bypass 100MB bundle size cap (per §1

effort_hours: 10
subtasks:
  - "0.5h: Cargo.toml + main.rs"
  - "1.0h: oci.rs — OCI Distribution v1.1 endpoints (manifest + blob upload/download)"
  - "1.0h: cosign_verify.rs — Ed25519 signature verification using sigstore library"
  - "0.5h: auth.rs — JWT verification + tenant_id extraction"
  - "0.5h: tenant_scope.rs — namespace per tenant; cross-tenant rejected"
  - "0.5h: audit.rs — skill.published + skill.pulled memory rows"
  - "0.5h: storage.rs — zot backend integration (or filesystem at slice 1)"
  - "0.5h: Immutability check (existing tag → 409)"
  - "0.5h: 100MB bundle size cap"
  - "1.0h: cyberos-skill publish/pull CLI"
  - "1.0h: docker-compose with zot + skill-registry"
  - "1.0h: zot-config.json (auth + storage)"
  - "1.5h: Tests — push + pull + cosign + immutability + cross-tenant + audit + size cap"
  - "1.0h: cosign keypair management (per-publisher)"
  - "0.5h: OTel metrics emission"
risk_if_skipped: "Skill distribution becomes ad-hoc (scp, S3, etc.). Without cosign, tampered .skill bundles install. Without tenant scope, tenant A's malicious skill could replace tenant B's good skill. Without immutable tags, version 1.0 silently changes content. Without audit, 'who published this skill when' unanswerable."
---

## §1 — Description (BCP-14 normative)

A self-hosted OCI-compliant registry **MUST** host `.skill` bundles. Each interaction:

1. **MUST** speak OCI Distribution v1.1 — standard `/v2/<name>/manifests/<reference>` + `/v2/<name>/blobs/<digest>` endpoints. Backend: zot (recommended; OCI-native + small footprint).
2. **MUST** require cosign signature on every push. Cosign uses Ed25519; per-publisher keypair (private key in publisher's secret store). Publisher signs bundle bytes; registry verifies before storing. Unsigned push → 401 with `signature_required`.
3. **MUST** emit memory rows:
    - `skill.published` on successful push: `tenant_id`, `skill_id`, `version`, `digest`, `publisher_subject_id`, `signature_pubkey_id`, `bundle_size_bytes`, `request_id`.
    - `skill.pulled` on successful pull: `tenant_id`, `skill_id`, `version`, `digest`, `puller_subject_id`, `request_id`.
4. **MUST** scope namespace by `tenant_id`: registry path is `/v2/<tenant_id>/<skill_id>/manifests/<version>`. Pulls require valid JWT (TASK-AUTH-004) AND `claims.tenant_id == namespace tenant_id`. Cross-tenant attempts → 403 with `cross_tenant_blocked`.
5. **MUST** support immutable tags — once `<skill_id>:<version>` is published, re-push same tag returns 409 with `immutable_tag`. Bug-fix releases require version bump (semver discipline).
6. **MUST** support `cyberos-skill publish <bundle.tar.zst> --version <semver>` CLI for ergonomic pushes. Auto-signs via local cosign keypair.
7. **MUST** support `cyberos-skill pull <skill_id>:<version>` CLI for ergonomic pulls. Auto-verifies signature; cached locally.
8. **MUST** verify signature on EVERY pull (not just push). Tampered storage backend → pull fails with `signature_invalid`. Defense-in-depth.
9. **MUST** authenticate via TASK-AUTH-004 JWT with `claims.scope_grants` containing `skill:publish` or `skill:pull`. Missing scope → 403.
10. **MUST** include `Idempotency-Key` header support on push (mirrors TASK-AUTH-001 §1 #5). Repeat push with same key + same content → return existing manifest; same key + different content → 409 with `idempotency_key_reuse`.
11. **MUST** enforce 100MB bundle size cap. Bundle > 100MB → 413 PAYLOAD_TOO_LARGE with `bundle_too_large`.
12. **MUST** support quota per tenant (10GB total bundle storage at slice 1; configurable). Above quota → 507 INSUFFICIENT_STORAGE.
13. **MUST** complete push p95 < 5s for 10MB bundle (typical size); pull p95 < 2s.
14. **MUST** publish manifest format matching OCI v1.1:
    ```json
    {
      "schemaVersion": 2,
      "mediaType": "application/vnd.cyberos.skill.v1+json",
      "config": { "mediaType": "application/vnd.cyberos.skill.config.v1+json", "digest": "sha256:...", "size": ... },
      "layers": [{ "mediaType": "application/vnd.cyberos.skill.bundle.v1+tar+zst", "digest": "sha256:...", "size": ... }],
      "annotations": {
        "world.cyberos.skill.signature": "<base64-cosign-sig>",
        "world.cyberos.skill.publisher": "<subject_id>",
        "world.cyberos.skill.published_at": "<iso8601>"
      }
    }
    ```
15. **SHOULD** emit OTel metrics:
    - `skill_registry_pushes_total{tenant_id, outcome}` (counter).
    - `skill_registry_pulls_total{tenant_id, outcome}` (counter).
    - `skill_registry_signature_failures_total{stage}` (counter; sev-1 alarm; stage ∈ push | pull).
    - `skill_registry_bundle_size_bytes` (histogram).
    - `skill_registry_storage_used_bytes{tenant_id}` (gauge).

---

## §2 — Why this design (rationale for humans)

**Why self-hosted (DEC-205)?** SaaS registries (DockerHub, GHCR) leak skill metadata to the registry operator. Self-hosted keeps tenant-business semantics in-region. zot is OCI-native, small, well-maintained.

**Why cosign on every push (DEC-206)?** Tampered bundles installed in tenant infrastructure = code execution opportunity. Signature on push + verify on every pull = supply-chain integrity. Per-publisher keypair limits blast radius if one is compromised.

**Why immutable tags (DEC-207)?** Mutable tags allow silent content changes. Tenant A pulls `obs.triage-alert:1.0`; later, attacker re-publishes 1.0 with malicious content; tenant A's next pull picks it up. Immutable tags prevent this.

**Why tenant-scoped namespace (DEC-208)?** Tenant A's skills shouldn't be visible/installable by tenant B. Namespace scoping at registry layer = structural isolation. Cross-tenant attempts at API level → 403.

**Why verify on EVERY pull, not just on push (§1 #8)?** Storage backend compromise (zot DB tampering) could replace bundle content. Re-verifying on pull catches this. The cost is small (Ed25519 verify is microseconds); the security benefit is large.

**Why 100MB bundle cap (§1 #11)?** Skills are typically <10MB. 100MB cap catches pathological bundles (huge embedded models, miscellaneous tarballs). Above 100MB, skill packaging probably has a bug.

**Why idempotency on push (§1 #10)?** Network retries during publish (slow upload, timeout) shouldn't produce 409 on retry. Idempotency-Key lets retry succeed AS the original.

**Why p95 budgets (§1 #13)?** CI/CD pipelines push skills frequently. 5s push budget keeps deploys fast. 2s pull keeps cold-start of skill execution fast.

**Why cyberos-skill CLI (§1 #6 + #7)?** Ergonomic UX. Without CLI, publishers use raw `oras` or `crane` commands — possible but error-prone. CLI auto-handles cosign + JWT + paths.

**Why per-publisher cosign keypair?** Publisher identity is part of audit chain. One keypair per publisher means "who signed this" is unambiguous + revocable independently.

---

## §3 — API contract

### Endpoints (OCI Distribution v1.1)

```
POST /v2/<tenant_id>/<skill_id>/blobs/uploads/         # initiate blob upload
PUT  /v2/<tenant_id>/<skill_id>/blobs/uploads/<uuid>?digest=sha256:<>
PUT  /v2/<tenant_id>/<skill_id>/manifests/<version>    # push manifest
GET  /v2/<tenant_id>/<skill_id>/manifests/<version>    # pull manifest
GET  /v2/<tenant_id>/<skill_id>/blobs/sha256:<>        # pull blob
HEAD /v2/<tenant_id>/<skill_id>/manifests/<version>    # check exists (immutability check)
```

### Push handler

```rust
// services/skill-registry/src/oci.rs
pub async fn push_manifest(
    tenant_id: Uuid, skill_id: &str, version: &str,
    body: Bytes, claims: &Claims, idempotency_key: Option<String>,
    storage: &Storage, memory: &MemoryBridge,
) -> Result<PushResponse, RegistryError> {
    // §1 #4 tenant scope
    if claims.tenant_id != tenant_id { return Err(RegistryError::CrossTenantBlocked); }
    // §1 #9 scope grant
    if !claims.scope_grants.iter().any(|g| g == "skill:publish" || g == "*") {
        return Err(RegistryError::Forbidden { needed: "skill:publish".into() });
    }

    // §1 #5 immutability
    if storage.manifest_exists(tenant_id, skill_id, version).await? {
        return Err(RegistryError::ImmutableTag { skill_id: skill_id.into(), version: version.into() });
    }

    // §1 #10 idempotency
    let body_hash = hex::encode(sha256(&body));
    if let Some(key) = &idempotency_key {
        if let Some(prior) = storage.idempotency_lookup(key).await? {
            if prior.body_hash != body_hash {
                return Err(RegistryError::IdempotencyKeyReuse);
            }
            return Ok(prior.response);
        }
    }

    let manifest: SkillManifest = serde_json::from_slice(&body)?;

    // §1 #11 size cap
    let total_size: u64 = manifest.layers.iter().map(|l| l.size).sum();
    if total_size > 100 * 1024 * 1024 {
        return Err(RegistryError::BundleTooLarge { actual_bytes: total_size });
    }

    // §1 #2 cosign signature verify
    let signature = manifest.annotations.get("world.cyberos.skill.signature")
        .ok_or(RegistryError::SignatureRequired)?;
    let pubkey_id = claims.subject_id.to_string();
    cosign_verify::verify_signature(&body, signature, &pubkey_id).await
        .map_err(|e| RegistryError::SignatureInvalid(e.to_string()))?;

    // §1 #12 quota
    let used = storage.tenant_used_bytes(tenant_id).await?;
    if used + total_size > tenant_quota(tenant_id) {
        return Err(RegistryError::QuotaExceeded);
    }

    storage.put_manifest(tenant_id, skill_id, version, &body).await?;

    let request_id = format!("registry_{}", ulid::Ulid::new());
    memory.emit(canonical::skill_published(
        tenant_id, skill_id, version, &body_hash, claims.subject_id,
        &pubkey_id, total_size, &request_id,
    )).await?;

    if let Some(key) = idempotency_key {
        storage.idempotency_insert(&key, &body_hash, &response).await?;
    }

    metrics::push(tenant_id, "ok");
    Ok(PushResponse { manifest_digest: body_hash, status: 201 })
}
```

### Pull handler

```rust
pub async fn pull_manifest(
    tenant_id: Uuid, skill_id: &str, version: &str, claims: &Claims,
    storage: &Storage, memory: &MemoryBridge,
) -> Result<Bytes, RegistryError> {
    if claims.tenant_id != tenant_id { return Err(RegistryError::CrossTenantBlocked); }
    if !claims.scope_grants.iter().any(|g| g == "skill:pull" || g == "*") {
        return Err(RegistryError::Forbidden { needed: "skill:pull".into() });
    }

    let body = storage.get_manifest(tenant_id, skill_id, version).await?
        .ok_or(RegistryError::NotFound)?;

    // §1 #8 verify on pull (defense in depth)
    let manifest: SkillManifest = serde_json::from_slice(&body)?;
    let signature = manifest.annotations.get("world.cyberos.skill.signature")
        .ok_or(RegistryError::SignatureRequired)?;
    let publisher = manifest.annotations.get("world.cyberos.skill.publisher").cloned().unwrap_or_default();
    cosign_verify::verify_signature(&body, signature, &publisher).await
        .map_err(|e| { metrics::signature_failure("pull"); RegistryError::SignatureInvalid(e.to_string()) })?;

    let request_id = format!("registry_{}", ulid::Ulid::new());
    memory.emit(canonical::skill_pulled(
        tenant_id, skill_id, version, hex::encode(sha256(&body)), claims.subject_id, &request_id,
    )).await?;

    metrics::pull(tenant_id, "ok");
    Ok(body)
}
```

### CLI

```rust
// services/skill-registry/src/bin/cyberos_skill.rs
#[derive(clap::Parser)]
struct Cli { #[command(subcommand)] cmd: Cmd }

#[derive(clap::Subcommand)]
enum Cmd {
    Publish { #[arg(long)] bundle: PathBuf, #[arg(long)] version: String },
    Pull    { skill_ref: String },   // "obs.triage-alert:1.0.0"
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Publish { bundle, version } => {
            let bytes = std::fs::read(&bundle)?;
            let signature = cosign::sign_local(&bytes, "~/.cyberos/cosign.key")?;
            let manifest = build_manifest(&bytes, &signature, &version)?;
            let resp = http::push(&manifest, &load_jwt()).await?;
            println!("✅ Published: {}@{}", resp.skill_id, version);
        }
        Cmd::Pull { skill_ref } => {
            let (skill_id, version) = parse_ref(&skill_ref)?;
            let bytes = http::pull(&skill_id, &version, &load_jwt()).await?;
            std::fs::write(format!("./{skill_id}-{version}.tar.zst"), &bytes)?;
            println!("✅ Pulled: {skill_id}@{version}");
        }
    }
    Ok(())
}
```

---

## §4 — Acceptance criteria

1. Push `.skill` bundle with valid cosign sig → 201; manifest stored.
2. Push without signature → 401 `signature_required`.
3. Push with invalid signature → 401 `signature_invalid`.
4. Pull → verifies signature; emits skill.pulled audit row.
5. Re-push same tag → 409 `immutable_tag`.
6. Cross-tenant pull (tenant B pulling from tenant A namespace) → 403 `cross_tenant_blocked`.
7. Cross-tenant push → 403.
8. JWT lacks `skill:publish` → 403.
9. JWT lacks `skill:pull` → 403.
10. memory rows emitted on push (skill.published) and pull (skill.pulled).
11. Bundle > 100MB → 413 `bundle_too_large`.
12. Tenant over 10GB quota → 507 `quota_exceeded`.
13. Idempotent push (same key + same content) → returns prior manifest.
14. Idempotent push (same key + different content) → 409 `idempotency_key_reuse`.
15. p95 push < 5s for 10MB bundle.
16. p95 pull < 2s.
17. CLI `cyberos-skill publish` works end-to-end.
18. CLI `cyberos-skill pull` works end-to-end.
19. Pull with tampered storage (manual byte change) → 401 `signature_invalid`.

---

## §5 — Verification

```rust
#[tokio::test]
async fn push_with_valid_signature_succeeds() {
    let bundle = test_helper::build_bundle("obs.triage-alert");
    let sig = cosign::sign(&bundle, &test_keypair());
    let resp = push_manifest(test_tenant(), "obs.triage-alert", "1.0.0",
                              build_manifest(&bundle, &sig), &test_publisher_claims(), None,
                              &test_storage(), &test_memory()).await.unwrap();
    assert_eq!(resp.status, 201);
    assert!(memory_test_helper::has_row("skill.published", "1.0.0").is_some());
}

#[tokio::test]
async fn push_without_signature_returns_401() {
    let bundle = test_helper::build_bundle("x");
    let manifest = build_manifest_unsigned(&bundle);
    let err = push_manifest(test_tenant(), "x", "1.0.0", manifest, &test_publisher_claims(), None, &test_storage(), &test_memory()).await.expect_err("expected SignatureRequired");
    assert!(matches!(err, RegistryError::SignatureRequired));
}

#[tokio::test]
async fn push_with_invalid_signature_returns_401() {
    let bundle = test_helper::build_bundle("x");
    let sig = "TAMPERED_SIGNATURE_BASE64";
    let manifest = build_manifest(&bundle, sig);
    let err = push_manifest(test_tenant(), "x", "1.0.0", manifest, &test_publisher_claims(), None, &test_storage(), &test_memory()).await.expect_err("expected SignatureInvalid");
    assert!(matches!(err, RegistryError::SignatureInvalid(_)));
}

#[tokio::test]
async fn re_push_same_tag_returns_409_immutable() {
    let _ = push_test_manifest("obs.triage-alert", "1.0.0").await.unwrap();
    let err = push_test_manifest("obs.triage-alert", "1.0.0").await.expect_err("expected ImmutableTag");
    assert!(matches!(err, RegistryError::ImmutableTag { .. }));
}

#[tokio::test]
async fn cross_tenant_pull_returns_403() {
    let tenant_a = test_helper::create_tenant().await;
    let tenant_b = test_helper::create_tenant().await;
    let _ = push_with_tenant(tenant_a, "x", "1.0.0").await.unwrap();

    let claims_b = claims_for(tenant_b);
    let err = pull_manifest(tenant_a, "x", "1.0.0", &claims_b, &test_storage(), &test_memory()).await.expect_err("expected CrossTenantBlocked");
    assert!(matches!(err, RegistryError::CrossTenantBlocked));
}

#[tokio::test]
async fn pull_emits_skill_pulled_audit_row() {
    let _ = push_test_manifest("x", "1.0.0").await.unwrap();
    let _ = pull_manifest(test_tenant(), "x", "1.0.0", &test_puller_claims(), &test_storage(), &test_memory()).await.unwrap();
    let row = memory_test_helper::find_latest("skill.pulled").unwrap();
    assert_eq!(row.payload["skill_id"], "x");
    assert_eq!(row.payload["version"], "1.0.0");
}

#[tokio::test]
async fn bundle_over_100mb_returns_413() {
    let huge = vec![0u8; 110 * 1024 * 1024];
    let manifest = build_manifest_with_layer_size(&huge, 110 * 1024 * 1024);
    let err = push_manifest(test_tenant(), "x", "1.0.0", manifest, &test_publisher_claims(), None, &test_storage(), &test_memory()).await.expect_err("expected BundleTooLarge");
    assert!(matches!(err, RegistryError::BundleTooLarge { .. }));
}

#[tokio::test]
async fn tampered_storage_pull_fails_signature_verify() {
    let _ = push_test_manifest("x", "1.0.0").await.unwrap();
    test_helper::tamper_storage_byte(test_tenant(), "x", "1.0.0", 100, 0xff).await;
    let err = pull_manifest(test_tenant(), "x", "1.0.0", &test_puller_claims(), &test_storage(), &test_memory()).await.expect_err("expected SignatureInvalid");
    assert!(matches!(err, RegistryError::SignatureInvalid(_)));
}

#[tokio::test]
async fn idempotent_push_returns_prior_manifest() {
    let key = "idem-001".to_string();
    let manifest = test_manifest("x", "1.0.0");
    let r1 = push_manifest(test_tenant(), "x", "1.0.0", manifest.clone(), &test_publisher_claims(), Some(key.clone()), &test_storage(), &test_memory()).await.unwrap();
    let r2 = push_manifest(test_tenant(), "x", "1.0.0", manifest, &test_publisher_claims(), Some(key), &test_storage(), &test_memory()).await.unwrap();
    assert_eq!(r1.manifest_digest, r2.manifest_digest);
}
```

---

## §6 — Implementation skeleton

See §3.

```yaml
# deploy/skill-registry/docker-compose.yml
services:
  zot:
    image: ghcr.io/project-zot/zot:v2.1.0
    ports: ["5000:5000"]
    volumes:
      - ./zot-config.json:/etc/zot/config.json:ro
      - zot-data:/var/lib/registry
  skill-registry:
    build: ../../services/skill-registry
    ports: ["7878:7878"]
    environment: { ZOT_URL: http://zot:5000, MEMORY_URL: http://memory:8080 }
    depends_on: [zot]
volumes: { zot-data: }
```

```json
{
  "storage": { "rootDirectory": "/var/lib/registry" },
  "http": { "address": "0.0.0.0", "port": "5000",
            "auth": { "htpasswd": { "path": "/etc/zot/htpasswd" }}},
  "extensions": { "search": { "enable": true }}
}
```

---

## §7 — Dependencies

- **TASK-SKILL-101** — audit row pattern.
- **TASK-AUTH-004** — JWT with scope_grants.
- **TASK-AUTH-003** — RLS pattern (tenant scoping).
- **TASK-AI-003** — memory_writer for audit emission.
- Crates: `axum`, `reqwest`, `tonic` (zot client), `sigstore@0.10` (cosign), `clap@4`, `serde`, `tokio`.
- zot OCI registry binary.
- cosign CLI for publisher-side signing.

---

## §8 — Example payloads

### Push request (CLI)

```bash
$ cyberos-skill publish ./obs-triage-alert.tar.zst --version 1.0.0
✅ Signed with cosign (key: ~/.cyberos/cosign.key)
✅ Published: obs.triage-alert@1.0.0 (digest: sha256:abc123...)
```

### Push manifest

```http
PUT /v2/550e.../obs.triage-alert/manifests/1.0.0 HTTP/1.1
Authorization: Bearer <jwt>
Content-Type: application/vnd.cyberos.skill.v1+json
Idempotency-Key: pub-001

{
  "schemaVersion": 2,
  "mediaType": "application/vnd.cyberos.skill.v1+json",
  "config": { "mediaType": "application/vnd.cyberos.skill.config.v1+json", "digest": "sha256:abc...", "size": 512 },
  "layers": [{ "mediaType": "application/vnd.cyberos.skill.bundle.v1+tar+zst", "digest": "sha256:def...", "size": 8192345 }],
  "annotations": {
    "world.cyberos.skill.signature": "MEQCI...",
    "world.cyberos.skill.publisher": "subject-stephen-...",
    "world.cyberos.skill.published_at": "2026-05-15T14:00:00Z"
  }
}
```

### Audit rows

```json
{
  "kind": "skill.published",
  "payload": {
    "tenant_id": "550e...", "skill_id": "obs.triage-alert", "version": "1.0.0",
    "digest": "sha256:abc...", "publisher_subject_id": "...",
    "signature_pubkey_id": "subject-stephen-...",
    "bundle_size_bytes": 8192345, "request_id": "registry_..."
  }
}

{
  "kind": "skill.pulled",
  "payload": {
    "tenant_id": "550e...", "skill_id": "obs.triage-alert", "version": "1.0.0",
    "digest": "sha256:abc...", "puller_subject_id": "...", "request_id": "registry_..."
  }
}
```

---

## §9 — Open questions

All resolved. Deferred:
- Skill marketplace UI (browse cross-tenant skills) — slice 5+; current scope is intra-tenant.
- Skill versioning policies (semver enforcement, deprecation) — slice 4+.
- Skill dependencies (one skill requires another) — slice 5+.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| cosign signature fails | sigstore verify | 401 signature_invalid | Publisher fixes signing |
| Bundle too large (>100MB) | size check | 413 bundle_too_large | Reduce bundle |
| Immutable tag exists | manifest_exists check | 409 immutable_tag | Bump version |
| Cross-tenant push | claims check | 403 cross_tenant_blocked | Caller uses correct tenant JWT |
| Cross-tenant pull | claims check | 403 | Same |
| JWT lacks skill:publish | scope check | 403 forbidden | Grant scope |
| Quota exceeded (>10GB) | tenant_used check | 507 quota_exceeded | Operator extends quota OR delete old |
| Storage backend error | zot 5xx | 503 | Operator investigates |
| Idempotent replay (same key + same content) | lookup | 201 with prior digest | By design |
| Idempotency reuse (same key + different content) | hash mismatch | 409 idempotency_key_reuse | Caller uses different key |
| Tampered storage (post-push) | pull-time verify catches | 401 signature_invalid | Investigate storage integrity |
| memory audit emit fails | memory_writer error | Push succeeds; sev-1 log | Operator investigates |
| zot down | http error | 503 | Restart zot |
| Publisher key revoked | signature still verifies (key cached) | Future pushes fail | Update key allow-list |
| Bundle integrity corruption (during transfer) | digest mismatch | 400 | Caller retries |
| Unknown manifest reference (HEAD nonexistent) | 404 | Caller proceeds with PUT | By design |
| Concurrent push same version | DB unique constraint | One succeeds; other 409 | By design |
| CLI cosign key missing | file not found | Exit 1 with clear message | User generates keypair |

---

## §11 — Notes

- zot is the reference OCI implementation chosen for its small footprint + active maintenance + full v1.1 support. Alternative: Harbor (heavier; richer UI).
- cosign signing on publisher side via `cosign sign-blob`; verification at registry uses sigstore-rs library.
- Per-publisher cosign keypair stored in publisher's local secret store (`~/.cyberos/cosign.key`); rotation per TASK-AUTH-006-style sweeper.
- Immutable tags enforce semver discipline — bug fixes go to new patch version, not silent overwrite.
- Tenant scope at registry layer prevents accidental cross-tenant skill installation.
- 100MB cap catches bundle bloat (typical skills are <10MB; bundles approaching 100MB usually have a packaging bug).
- 10GB quota per tenant is slice-1 default; configurable via tenant policy.
- Pull-side verify catches storage tampering — defense in depth.
- Idempotency-Key matches TASK-AUTH-001 pattern; standard retry-safety.
- Audit rows on push + pull = compliance answer to "who published/installed what when."

---

*End of TASK-SKILL-102. Status: draft (10/10 target).*

## As built (2026-07-02)

skill-registry was consolidated into services/skill-broker (src/oci.rs).
