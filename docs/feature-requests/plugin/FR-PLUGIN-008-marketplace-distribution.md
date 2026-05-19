---
id: FR-PLUGIN-008
title: "Marketplace distribution — cyberos-plugin publish pushes signed bundle to plugins.cyberskill.world + mirrors to agentskills.io; revenue-share + vetted badge"
module: PLUGIN
priority: SHOULD
status: draft
verify: T
phase: P1
milestone: P1 · slice 3
slice: 3
owner: Stephen Cheng (CTO)
created: 2026-05-19
shipped: null
memory_chain_hash: null
related_frs: [FR-PLUGIN-001, FR-PLUGIN-005, FR-PLUGIN-006, FR-PLUGIN-007, FR-TEN-005, FR-PORTAL-005]
depends_on: [FR-PLUGIN-006, FR-PLUGIN-007]
blocks: []

source_pages:
  - strategy/CYBEROS_STRATEGY.md §4 Level 3 (marketplace) + Level 4 (vertical packs) + Level 5 (private marketplace)
  - modules/plugin/manifest.schema.json (marketplace section)
  - modules/plugin/INTEROP.md universal constraint 2

source_decisions:
  - DEC-2470 2026-05-19 — Marketplace lives at plugins.cyberskill.world with OCI-compatible API; private/enterprise marketplaces share API but use distinct origins
  - DEC-2471 2026-05-19 — Visibility = public (anyone can install) | private (tenant-scoped) | enterprise (white-label marketplace per Strategy Level 5)
  - DEC-2472 2026-05-19 — Public plugins MUST mirror to agentskills.io for ecosystem visibility per Strategy Level 1 OSS distribution
  - DEC-2473 2026-05-19 — Revenue share: 70% to plugin author / 30% to CyberSkill on paid plugins; free plugins exempt
  - DEC-2474 2026-05-19 — Vetted-by-CyberSkill badge awarded after security review (manual at first, automated where possible); badge persists per (plugin_id, version)
  - DEC-2475 2026-05-19 — Publish validates 8 INTEROP invariants per modules/plugin/INTEROP.md + re-runs FR-PLUGIN-001 validation; rejects on any failure
  - DEC-2476 2026-05-19 — Every publish emits memory audit row plugin.published with body containing version, sha256, signature anchor, visibility

build_envelope:
  language: rust 1.81
  service: services/plugin-host/ (CLI side) + new services/plugin-marketplace/ (server side, scaffolded in this FR; full server implementation deferred to FR-PLUGIN-008a)
  new_files:
    - services/plugin-host/src/marketplace/mod.rs
    - services/plugin-host/src/marketplace/publish.rs
    - services/plugin-host/src/marketplace/mirror_agentskills.rs
    - services/plugin-host/src/marketplace/badge_verify.rs
    - services/plugin-host/src/bin/cyberos-plugin-publish.rs
    - services/plugin-marketplace/Cargo.toml (scaffold only)
    - services/plugin-marketplace/src/lib.rs (scaffold only)
    - services/plugin-marketplace/migrations/0001_plugin_registry.sql
    - services/plugin-host/tests/publish_invariant_check_test.rs
    - services/plugin-host/tests/publish_mirrors_to_agentskills_test.rs
    - services/plugin-host/tests/publish_emits_audit_test.rs
    - services/plugin-host/tests/private_visibility_scoped_to_tenant_test.rs

  modified_files:
    - services/Cargo.toml (workspace member plugin-marketplace)
    - modules/plugin/manifest.schema.json (marketplace section finalised)

  allowed_tools:
    - file_read: services/plugin-host/**, services/plugin-marketplace/**
    - file_write: services/plugin-host/src/marketplace/**, services/plugin-marketplace/**
    - bash: cd services && cargo test -p cyberos-plugin-host publish

  disallowed_tools:
    - publish unsigned bundle (per DEC-2475)
    - publish without re-validation (per DEC-2475)
    - mirror private plugin to agentskills.io (per DEC-2472 — public only)

effort_hours: 6
sub_tasks:
  - "0.4h: marketplace/mod.rs types + publish flow trait"
  - "1.2h: marketplace/publish.rs (OCI push to plugins.cyberskill.world)"
  - "0.8h: marketplace/mirror_agentskills.rs (HTTP mirror for public plugins)"
  - "0.4h: marketplace/badge_verify.rs (check vetted-by badge)"
  - "0.5h: bin/cyberos-plugin-publish.rs"
  - "0.3h: services/plugin-marketplace/ scaffold (Cargo + migrations)"
  - "2.4h: 4 test files"

risk_if_skipped: "Without marketplace, plugins distribute only via direct download or git clone — Strategy §4 Level 3 marketplace + Level 4 vertical packs both stall. Without DEC-2472 agentskills.io mirror, public plugins miss the Anthropic ecosystem visibility that Strategy Level 1 depends on. Without DEC-2473 revenue share, paid plugin economy never bootstraps. Without DEC-2475 publish-time re-validation, bundles that pass local validation but fail invariants land in users' hands. Without DEC-2476 audit emission, marketplace operations are invisible — Strategy §2 'audit-chained' collapses at the distribution layer."
---

## §1 — Description (BCP-14 normative)

The PLUGIN module **MUST** ship a marketplace publish surface at `services/plugin-host/src/marketplace/`. CLI `cyberos-plugin publish <bundle>` uploads a packed bundle to `plugins.cyberskill.world` (OCI-compatible registry); public plugins additionally mirror to `agentskills.io`. The marketplace server itself (registry API + UI) is scaffolded here and fully implemented in FR-PLUGIN-008a.

1. **MUST** implement `cyberos-plugin publish <bundle>` CLI with flags per DEC-2470 + DEC-2471:
   - `--visibility {public,private,enterprise}` — default reads from manifest's `marketplace.visibility` (per FR-PLUGIN-001 schema)
   - `--registry <url>` — defaults to `https://plugins.cyberskill.world`
   - `--mirror-agentskills` — boolean, default true for public visibility; force-false otherwise
   - `--vetted-badge-token <token>` — optional, supplied by CyberSkill after manual review (clause 5)

2. **MUST** validate bundle integrity before upload per DEC-2475:
   - Re-run `cyberos-plugin doctor <bundle>` and require all 8 INTEROP invariants pass
   - Re-validate canonical manifest against `manifest.schema.json` per FR-PLUGIN-001
   - Verify Sigstore Rekor anchor matches the bundle bytes
   - Verify bundle's target adapter output matches the per-target reproducibility check

3. **MUST** push to OCI-compatible registry at `plugins.cyberskill.world/v2/<id>/blobs` per OCI Distribution Spec v1.1:
   - Bundle bytes pushed as `application/vnd.cyberskill.plugin.v1+zip` (or `+folder` for codex-cli)
   - Manifest pushed as `application/vnd.cyberskill.plugin.manifest.v1+json`
   - Tag = `<version>` (e.g. `1.0.0`)
   - Authentication via OAuth-PKCE bearer token (per FR-PLUGIN-005)

4. **MUST** mirror public-visibility plugins to `agentskills.io` per DEC-2472:
   - Only `marketplace.visibility == "public"` qualifies
   - Mirror uploads the canonical manifest + skills/ folder (NOT the binary, NOT the OAuth-PKCE-bound runtime fields)
   - agentskills.io receives an "anthropic-skills" subset of the manifest
   - Failure to mirror is a soft failure (publish to plugins.cyberskill.world succeeds, mirror retried via FR-PLUGIN-006 audit_outbox pattern)

5. **MUST** support the vetted-by-CyberSkill badge per DEC-2474:
   - Badge token issued via out-of-band manual review process (initially) at `https://plugins.cyberskill.world/admin/vet`
   - Token is JWT signed by CyberSkill with `aud: "plugin:<id>:<version>"`
   - Publish embeds token in marketplace metadata; registry verifies signature; badge persists across reads
   - Token-less publish still succeeds; just no badge

6. **MUST** support three visibility modes per DEC-2471:
   - `public` — discoverable by any user; appears in marketplace search; mirrored to agentskills.io
   - `private` — discoverable only within the publishing tenant; mirror disabled; install requires tenant membership
   - `enterprise` — like private, but the registry is a separate tenant-isolated origin (Strategy Level 5 white-label); URL pattern `plugins.<enterprise-domain>.cyberskill.world`

7. **MUST** validate revenue-share rules per DEC-2473:
   - Free plugins (`marketplace.price_usd_per_month` absent or 0): no revenue share, badge irrelevant
   - Paid plugins (`marketplace.price_usd_per_month > 0`): 70/30 author/CyberSkill split enforced server-side at billing time
   - `marketplace.revenue_share_percent` defaults to 70 per manifest schema; values < 70 trigger publish-time warning (author getting less than fair share)

8. **MUST** emit memory audit row `plugin.published` per DEC-2476 with body containing:
   - `plugin_id`, `version`, `sha256` of bundle, `signature.rekor_uuid`, `visibility`, `vetted_badge: boolean`, `mirror_targets: [...]`, `trace_id`
   Audit emission follows FR-PLUGIN-006 retry semantics.

9. **MUST** enforce version monotonicity at registry — uploading version `1.0.0` after `1.0.1` MUST fail. SemVer 2.0 semantic ordering per FR-PLUGIN-001 clause 4.

10. **MUST** support `cyberos-plugin yank <id>@<version>` for emergency removal — yanked versions are hidden from search but remain installable by users with the SHA-256 (so existing installs don't break). Yank emits memory audit `plugin.yanked`.

11. **MUST NOT** publish a bundle that fails any of the 8 INTEROP invariants per DEC-2475 + clause 2.

12. **MUST NOT** mirror private or enterprise plugins to agentskills.io per DEC-2472 + clause 4. Private bundles MUST stay on CyberSkill's infrastructure.

13. **MUST NOT** allow a paid plugin to set revenue_share_percent > 100 — schema enforces 0-100 range; publish double-checks.

14. **MUST NOT** publish without a verified Sigstore signature anchor per clause 2.

---

## §2 — Why this design

**Why OCI-compatible registry (DEC-2470, clause 3)?** OCI registries are the de-facto standard for bundle distribution. Docker Hub, GitHub Container Registry, AWS ECR all speak OCI. Using OCI gives us free integration with existing tooling (cosign signing, opens up multi-cloud mirroring). Marketplace UI sits on top of the OCI API.

**Why three visibility modes (DEC-2471, clause 6)?** Strategy Level 3 needs public (anyone installs); Strategy Level 1 OSS amplifies via public mirror; Strategy Level 4 (vertical packs) is mostly public but may be tenant-specific in early stages → private; Strategy Level 5 (enterprise white-label) is enterprise. Three covers all four levels cleanly.

**Why mirror to agentskills.io (DEC-2472, clause 4)?** Strategy §2 lists agentskills.io as the open Anthropic registry; CyberSkill is a citizen. Public plugins benefit from being discoverable via Anthropic's ecosystem search, not just CyberSkill's. Mirror gives reach without ceding ownership (registry of record remains plugins.cyberskill.world).

**Why 70/30 split (DEC-2473, clause 7)?** Industry standard (Apple App Store, Shopify post-Shopify Capital, Salesforce AppExchange). 70% is the "fair share" headline; deviation triggers a warning because plugin authors deserve protection from accidental low splits.

**Why JWT-signed vetted badge (DEC-2474, clause 5)?** Badge claim must be unforgeable. JWT signed by CyberSkill's marketplace key is verifiable client-side (`cyberos-plugin doctor` rechecks). Manual review at first; automated for security-only plugins later.

**Why publish-time re-validation (DEC-2475, clause 2)?** Local validation can be tampered with (modified `cyberos-plugin` binary). Server-side re-validation is the trust anchor. Same reason GitHub re-verifies signed commits.

**Why version monotonicity (clause 9)?** Out-of-order publish (1.0.1 then 1.0.0) confuses dependency resolvers. Either version is "latest"? Monotonic publish-by-version means `latest` tag always points at the highest version.

**Why yank-not-delete (clause 10)?** Permanent deletion breaks existing installs that have the SHA-256. Yank hides from discovery but preserves install ability. Same model as crates.io, npm.

**Why audit publish (DEC-2476, clause 8)?** Strategy §2 demands every action be audit-chained. Publish is a high-stakes action (introduces code into users' environments). Audit row is forensic record + DSAR-exportable.

**Why no mirror for private/enterprise (clause 12)?** Private plugins may contain tenant-confidential content (vertical packs for specific clients). Mirroring to a public registry leaks. Hard separation.

---

## §3 — API contract

### CLI surface

```text
cyberos-plugin publish <bundle> [--visibility VIS] [--registry URL] [--mirror-agentskills] [--vetted-badge-token TOKEN]
cyberos-plugin yank <id>@<version> [--reason TEXT]
cyberos-plugin list [--visibility VIS] [--vetted]
cyberos-plugin info <id>@<version>
```

### Plugin registry Postgres schema (scaffold for FR-PLUGIN-008a server)

```sql
-- services/plugin-marketplace/migrations/0001_plugin_registry.sql
CREATE TABLE plugin_marketplace.plugins (
  plugin_id TEXT NOT NULL,
  version TEXT NOT NULL,
  owner_tenant_id UUID NOT NULL,
  visibility TEXT NOT NULL CHECK (visibility IN ('public','private','enterprise')),
  bundle_sha256 BYTEA NOT NULL,
  bundle_size_bytes BIGINT NOT NULL,
  rekor_uuid TEXT NOT NULL,
  vetted_badge_token TEXT,
  vetted_at TIMESTAMPTZ,
  price_usd_per_month NUMERIC(10,2),
  revenue_share_percent SMALLINT NOT NULL DEFAULT 70,
  yanked_at TIMESTAMPTZ,
  yanked_reason TEXT,
  published_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_by_subject UUID NOT NULL,
  trace_id CHAR(32),
  PRIMARY KEY (plugin_id, version)
);

CREATE INDEX ON plugin_marketplace.plugins (visibility, published_at DESC)
  WHERE yanked_at IS NULL;
CREATE INDEX ON plugin_marketplace.plugins (owner_tenant_id, visibility);
CREATE INDEX ON plugin_marketplace.plugins (vetted_at) WHERE vetted_at IS NOT NULL;
```

### Publish flow

```rust
// services/plugin-host/src/marketplace/publish.rs
pub async fn publish(
    bundle_path: &Path,
    visibility: Visibility,
    registry: &Url,
    mirror_agentskills: bool,
    vetted_badge_token: Option<&str>,
) -> Result<PublishResult> {
    // Step 1: validate bundle (re-run doctor + manifest schema)
    let invariants = doctor(bundle_path)?;
    if !invariants.all_pass() {
        return Err(PublishError::InvariantViolation(invariants.failures));
    }

    // Step 2: parse manifest
    let manifest = read_canonical_manifest_from_bundle(bundle_path)?;

    // Step 3: verify Sigstore anchor matches bundle bytes
    sigstore::verify(bundle_path, &manifest.signature.rekor_uuid).await?;

    // Step 4: push to plugins.cyberskill.world OCI
    let push_result = oci_push(bundle_path, &manifest, registry).await?;

    // Step 5: mirror to agentskills.io if public
    let mirror_status = if visibility == Visibility::Public && mirror_agentskills {
        mirror_agentskills::push(&manifest, bundle_path).await.into()
    } else {
        MirrorStatus::Skipped
    };

    // Step 6: emit memory audit
    audit::emit_published(
        &manifest, &push_result.sha256, visibility, mirror_status.clone(),
    ).await?;

    Ok(PublishResult {
        plugin_id: manifest.id,
        version: manifest.version,
        sha256: push_result.sha256,
        rekor_uuid: manifest.signature.rekor_uuid,
        mirror_status,
        vetted_badge: vetted_badge_token.is_some(),
    })
}
```

### Vetted-badge token format

```text
JWT (RS256) signed by CyberSkill marketplace key:
  iss = "https://plugins.cyberskill.world"
  aud = "plugin:<id>:<version>"
  sub = <reviewer_subject_id>
  iat, exp (typically 90-day validity)
  body:
    review_id = "REV-2026-...",
    review_date = "...",
    findings_summary = "..."
```

### Mirror request to agentskills.io

```http
POST https://agentskills.io/v1/skills/publish HTTP/1.1
Content-Type: application/json
Authorization: Bearer <agentskills.io API key for CyberSkill org>

{
  "publisher": "cyberskill",
  "skill_id": "cyberos",
  "version": "1.0.0",
  "skill_md_url": "https://plugins.cyberskill.world/cyberos/1.0.0/SKILL.md",
  "license": "Apache-2.0",
  "description": "..."
}
```

---

## §4 — Acceptance criteria

1. **`publish` rejects bundle failing doctor invariants** — mock a bundle with broken signature; publish exits 1.
2. **`publish` rejects unsigned bundle** — bundle missing Sigstore anchor; publish exits 1.
3. **`publish` pushes to OCI registry** — mock registry receives PUT to `/v2/cyberos/blobs/uploads/`.
4. **`publish` for public mirrors to agentskills.io** — mock agentskills.io receives mirror call.
5. **`publish` for private does NOT mirror** — mock agentskills.io receives 0 calls.
6. **`publish` emits plugin.published audit** — memory row exists with kind='plugin.published'.
7. **plugin.published body has version + sha256 + rekor_uuid + visibility** — body field check.
8. **Out-of-order version rejected** — publish 1.0.1; publish 1.0.0 next; second fails.
9. **Yank hides from default search** — publish; yank; `cyberos-plugin list` does not show yanked.
10. **Yanked plugin still installable by SHA-256** — install by hash succeeds.
11. **Vetted badge persists on info** — `cyberos-plugin info <id>@<version>` shows vetted: true when token verified.
12. **Bad vetted badge token rejected** — JWT signed by wrong key; publish stores but `info` shows vetted: false with reason.
13. **Private plugin cross-tenant install denied** — tenant B trying to install tenant A's private plugin fails.
14. **Enterprise plugin appears only at enterprise origin** — `plugins.cyberskill.world` does NOT list it; `plugins.acme.cyberskill.world` does.
15. **revenue_share_percent < 70 triggers warning** — publish-time stderr warning; publish still succeeds.
16. **revenue_share_percent > 100 rejected** — schema + publish double-check fails.
17. **Mirror failure is soft** — agentskills.io 500s; publish still succeeds; mirror row queued in retry table.
18. **Publish audit row scrubbed** — body MUST NOT contain bundle bytes (only sha256).
19. **`info` returns visibility, vetted status, version list** — RPC works.
20. **`list --vetted` filters to vetted-only** — query semantics.
21. **OCI blob has correct media type** — `application/vnd.cyberskill.plugin.v1+zip`.
22. **Publish requires OAuth-PKCE bearer token** — anonymous publish fails 401.

---

## §5 — Verification

```rust
// services/plugin-host/tests/publish_invariant_check_test.rs
#[tokio::test]
async fn publish_rejects_unsigned_bundle() {
    let bundle = pack_bundle_without_signature().await;
    let result = publish(&bundle, Visibility::Public, &mock_registry_url(), false, None).await;
    assert!(matches!(result, Err(PublishError::InvariantViolation(_))));
}

#[tokio::test]
async fn publish_rejects_broken_doctor() {
    let bundle = pack_bundle_with_wrong_tool_naming().await;
    let result = publish(&bundle, Visibility::Public, &mock_registry_url(), false, None).await;
    assert!(matches!(result, Err(PublishError::InvariantViolation(_))));
}
```

```rust
// services/plugin-host/tests/publish_mirrors_to_agentskills_test.rs
#[tokio::test]
async fn public_publish_mirrors() {
    let agentskills_mock = MockAgentSkillsServer::start().await;
    let bundle = pack_valid_bundle().await;
    publish(&bundle, Visibility::Public, &mock_registry_url(), true,
            Some("https://localhost/agentskills")).await.unwrap();
    let calls = agentskills_mock.recorded_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].path, "/v1/skills/publish");
}

#[tokio::test]
async fn private_publish_does_not_mirror() {
    let agentskills_mock = MockAgentSkillsServer::start().await;
    let bundle = pack_valid_bundle().await;
    publish(&bundle, Visibility::Private, &mock_registry_url(), true, None).await.unwrap();
    assert_eq!(agentskills_mock.recorded_calls().len(), 0);
}
```

```rust
// services/plugin-host/tests/publish_emits_audit_test.rs
#[tokio::test]
async fn publish_emits_audit_row() {
    let ctx = TestContext::with_memory_mock().await;
    let bundle = pack_valid_bundle().await;
    publish(&bundle, Visibility::Public, &ctx.registry, false, None).await.unwrap();
    let rows = ctx.memory.fetch_rows(kind="plugin.published").await;
    assert_eq!(rows.len(), 1);
    let body = &rows[0]["body"];
    assert_eq!(body["plugin_id"], "cyberos");
    assert_eq!(body["version"], "1.0.0");
    assert_eq!(body["visibility"], "public");
    assert!(body["sha256"].as_str().unwrap().len() == 64);
    assert!(body["rekor_uuid"].is_string());
    // body MUST NOT contain bundle bytes
    let body_str = serde_json::to_string(body).unwrap();
    assert!(!body_str.contains("PK\x03\x04")); // zip magic
}
```

```rust
// services/plugin-host/tests/private_visibility_scoped_to_tenant_test.rs
#[tokio::test]
async fn cross_tenant_private_install_denied() {
    let ctx_a = TestContext::for_tenant("a").await;
    let bundle = ctx_a.pack_and_publish(Visibility::Private).await;

    let ctx_b = TestContext::for_tenant("b").await;
    let result = ctx_b.install_by_id(&bundle.plugin_id, &bundle.version).await;
    assert!(matches!(result, Err(InstallError::NotFound)));
}
```

---

## §6 — Implementation skeleton

(API contract + Postgres schema in §3 are the skeleton. Full marketplace server implementation deferred to FR-PLUGIN-008a, which fleshes out search, browse UI, billing integration with FR-TEN-005, and admin review tooling.)

---

## §7 — Dependencies

- **Upstream:** FR-PLUGIN-006 (audit emission for plugin.published / plugin.yanked); FR-PLUGIN-007 (per-target adapters produce the bundles being published).
- **Downstream:** FR-TEN-005 (vertical pack pricing — paid plugins billed via TEN service); FR-PORTAL-005 (branded Genie chat — enterprise marketplaces appear in branded portal); FR-PLUGIN-008a (full marketplace server: search, UI, billing).
- **Cross-module:** Strategy §4 Levels 1, 3, 4, 5 — every distribution-facing milestone in the strategy depends on this surface shipping at least at the publish-CLI level.

---

## §8 — Example payloads

### Publish result

```json
{
  "plugin_id": "cyberos",
  "version": "1.0.0",
  "sha256": "a1b2c3d4...",
  "rekor_uuid": "24296fb24b8ad77a...",
  "mirror_status": "mirrored",
  "vetted_badge": true,
  "registry_url": "https://plugins.cyberskill.world/cyberos/1.0.0"
}
```

### `plugin.published` audit body

```json
{
  "plugin_id": "cyberos",
  "version": "1.0.0",
  "sha256": "a1b2c3d4...",
  "rekor_uuid": "24296fb...",
  "visibility": "public",
  "vetted_badge": true,
  "mirror_targets": ["agentskills.io"],
  "size_bytes": 184320,
  "trace_id": "01HX..."
}
```

### Vetted badge JWT (decoded)

```json
{
  "iss": "https://plugins.cyberskill.world",
  "aud": "plugin:cyberos:1.0.0",
  "sub": "reviewer-uuid-...",
  "iat": 1748000000,
  "exp": 1755800000,
  "review_id": "REV-2026-042",
  "review_date": "2026-05-19",
  "findings_summary": "All FR-PLUGIN-001..007 invariants pass; manual review passed."
}
```

---

## §9 — Open questions

All resolved.

- ~~Should the marketplace server ship in this FR?~~ → No. CLI publish + registry scaffolding in this FR; full server (search, UI, billing) in FR-PLUGIN-008a.
- ~~Should agentskills.io mirror be optional (off-by-default for public)?~~ → No, on by default per clause 1. Strategy §4 Level 1 OSS-distribution depends on agentskills.io presence.
- ~~Should revenue_share allow > 70 for author (more generous)?~~ → Yes (schema 0-100); only < 70 triggers warning. Author getting more than 70 is fine.
- ~~Should we support arbitrary review chains (multiple vetters)?~~ → Deferred to FR-PLUGIN-008b; v1 is single CyberSkill reviewer.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Bundle doctor invariants fail | publish.rs step 1 | exit 1 with details | Re-pack bundle correctly |
| Sigstore anchor mismatch | sigstore::verify | exit 1 | Re-sign bundle |
| Registry unreachable | OCI client | exit 1 | Retry with backoff (manual) |
| agentskills.io unreachable | mirror call | soft fail; outbox queue | Mirror retries via FR-PLUGIN-006 outbox |
| Out-of-order version | registry monotonicity check | exit 1 | Use higher version number |
| Yank without permission | registry RBAC | exit 1 | Owner-or-admin only |
| Vetted badge token wrong sig | badge_verify | publish proceeds; badge: false; warning | Get badge re-issued |
| Vetted badge token expired | badge_verify exp check | badge: false | Renew via review process |
| Private plugin published to public origin | visibility check | exit 1 | Use visibility=private + correct registry |
| Enterprise plugin to wrong enterprise origin | origin check | exit 1 | Use the correct enterprise registry URL |
| Bundle byte change after sign | sigstore re-verify | exit 1 | Re-sign |
| Cross-tenant private install | registry RLS | 404 | inherent |
| revenue_share > 100 | schema + double-check | exit 1 | Author fixes |
| Audit emission fails | FR-PLUGIN-006 outbox path | publish succeeds; audit queued | Inherent retry |
| Publisher token lacks scope | OAuth scope check | 401 | Re-authenticate with publish scope |

---

## §11 — Implementation notes

- §11.1 **Why scaffold the marketplace server here.** Full server is FR-PLUGIN-008a (registry API, search UI, browse, install metrics, billing). This FR ships the CLI side + Postgres schema so the publish path is operational. Server implementation can land independently because schema is locked.

- §11.2 **OCI push library.** Rust `oci-distribution` crate (4.x) handles the OCI v1.1 spec including blobs, manifests, tags. Adapter wraps it for the CyberSkill-specific media types.

- §11.3 **agentskills.io API key.** CyberSkill-org-level API key stored in AWS Secrets Manager at `plugins.cyberskill.world/agentskills_api_key`. Fetched once at process start; rotated quarterly.

- §11.4 **Vetted badge token issuance flow.** Out of scope of this FR; manual at `https://plugins.cyberskill.world/admin/vet` for v1. Future automation: scan bundle for known-bad patterns (FR-PLUGIN-008b).

- §11.5 **Enterprise origin pattern.** `plugins.<enterprise-name>.cyberskill.world` — DNS managed by CyberSkill; routed to the same OCI backend with tenant_id filter. Enterprise tenant_id derived from origin.

- §11.6 **Mirror retry semantics.** Mirror failures populate the FR-PLUGIN-006 outbox with kind=`plugin.mirror_retry` (special variant). Retry worker checks every 5 minutes. Mirror is best-effort, not blocking.

- §11.7 **Why `yank` instead of `delete`.** Yank model from crates.io. Permanent deletion breaks reproducibility of historical installs. Yank hides from discovery but preserves bytes. The bundle bytes can be force-deleted by admin (DSAR or court order); that emits a separate `plugin.deleted` audit kind.

- §11.8 **Revenue billing integration.** FR-TEN-005 receives memory audit `plugin.invoked` rows from paid plugins; bills the user tenant; pays the author tenant per `revenue_share_percent`. This FR ships the data; TEN does the money.

- §11.9 **Why CyberSkill 30% (not lower).** Below 20% the marketplace cannot fund: hosting, security review, fraud detection, OAuth issuance, audit chain storage. Above 30% authors balk. 30% is the equilibrium.

- §11.10 **CLI ergonomics.** `cyberos-plugin publish dist/cyberos-1.0.0.plugin --visibility public` is the canonical command. Defaults are aggressive (mirror on, OAuth-PKCE handshake automatic) so authors don't need to manage flags.

---

*End of FR-PLUGIN-008 spec.*
