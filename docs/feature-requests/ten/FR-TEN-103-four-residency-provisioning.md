---
id: FR-TEN-103
title: "4-residency provisioning — sg-1 / eu-1 / us-1 / vn-1 region pinning across Postgres + S3 + NATS + Stripe + KMS with cross-residency-write trip-wire"
module: TEN
priority: MUST
status: draft
verify: T
phase: P3
milestone: P3 · residency-GA
slice: 2
owner: Stephen Cheng (CTO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_frs: [FR-TEN-001, FR-TEN-003, FR-TEN-004, FR-TEN-101, FR-TEN-102, FR-TEN-104, FR-AUTH-004, FR-AI-016, FR-AI-003, FR-MEMORY-101, FR-MEMORY-111, FR-DOC-001, FR-EMAIL-001, FR-OBS-005, FR-OBS-007, FR-OBS-008]
depends_on: [FR-AI-016, FR-TEN-001]
blocks: []

source_pages:
  - website/docs/modules/ten.html#residency
  - https://gdpr.eu/article-44-transfers-of-personal-data/
  - https://www.iso.org/standard/82875.html               # PDPL Law 91/2025
  - https://www.aicpa.org/soc                              # SOC 2 cross-border
  - https://www.imda.gov.sg/regulations-and-licensing-listing/data-protection-act

source_decisions:
  - DEC-920 2026-05-17 — Closed 4-value `residency` Postgres enum: `sg-1, eu-1, us-1, vn-1`; CI cardinality test asserts 4; new residency = schema migration + DEC entry
  - DEC-921 2026-05-17 — Each residency = one Aurora cluster + one S3 bucket + one NATS cluster + one Stripe account (per FR-TEN-003 DEC-801) + one KMS key + one OBS Loki/Tempo region; no shared infra across residencies
  - DEC-922 2026-05-17 — Residency mapping (FR-AI-016 + FR-TEN-003 DEC-785 unified): VND→vn-1, SGD→sg-1, EUR→eu-1, GBP→eu-1, USD→us-1
  - DEC-923 2026-05-17 — Cross-residency reads are FORBIDDEN at the data layer; any query that crosses residency boundaries returns 403 + emits sev-1 `ten.cross_residency_access_attempt`
  - DEC-924 2026-05-17 — Cross-residency writes are FORBIDDEN by trip-wire trigger + Postgres FDW guard + connection-pool routing; defense-in-depth (any one prevents drift)
  - DEC-925 2026-05-17 — Tenant residency immutable post-provisioning (consistent with FR-TEN-003 DEC-798 currency immutability); residency change = new tenant + manual migration
  - DEC-926 2026-05-17 — memory chain partitioned per residency (each residency has its own chain head); cross-residency memory events are FORBIDDEN; reconciliation via per-residency exports for global compliance reports
  - DEC-927 2026-05-17 — Provisioning CLI (FR-TEN-001) routes to the correct residency's Postgres/S3/NATS via the `tenants.residency` value at insert; tenant ID generation collision-free across residencies (UUIDv7 with residency-prefix nibble pattern)
  - DEC-928 2026-05-17 — Per-residency Aurora deployed in private VPC with no cross-VPC peering (per Stripe security pattern); each residency's services run in that residency's VPC only
  - DEC-929 2026-05-17 — Per-residency KMS key for envelope encryption (sg-1 → AWS KMS ap-southeast-1; eu-1 → AWS KMS eu-west-1; us-1 → AWS KMS us-east-1; vn-1 → AWS KMS ap-southeast-1 with VN data-residency contract addendum)
  - DEC-930 2026-05-17 — vn-1 residency is *physically* hosted in ap-southeast-1 (Singapore) until a Vietnam-region AWS opens; PDPL Law 91/2025 §17 contract-residency clause covers this with explicit customer disclosure at signup (FR-TEN-101 consent)
  - DEC-931 2026-05-17 — Per-residency authz issuer (FR-AUTH-004) — each residency's services accept JWTs signed by THAT residency's issuer only; cross-residency JWT presentation = 401 `wrong_residency_token`
  - DEC-932 2026-05-17 — Per-residency NATS cluster — subjects `tenant.<slug>.*` are local to the residency; no cross-region NATS routing
  - DEC-933 2026-05-17 — Per-residency S3 bucket naming: `cyberos-{residency}-tenants` (data) + `cyberos-{residency}-audit` (audit archive); cross-bucket replication DISABLED
  - DEC-934 2026-05-17 — Per-residency observability — each residency's Loki+Tempo+Prometheus stays in that residency; cross-region OBS query proxy (FR-OBS-002) federates read-only via per-residency RBAC scope
  - DEC-935 2026-05-17 — Per-residency Stripe account routing already in FR-TEN-003 DEC-801; this FR consumes that map + enforces it via api_client construction
  - DEC-936 2026-05-17 — Provisioning workflow ATOMIC at the residency level — if Aurora INSERT succeeds but S3 prefix creation fails, the Aurora row is DELETED + sev-1 alert; no half-provisioned residency state
  - DEC-937 2026-05-17 — Connection-pool routing: each service holds a `Map<Residency, PgPool>` + a `Map<Residency, S3Client>`; service-level handlers extract `residency` from `tenants.residency` lookup + select correct pool; mis-pool = sev-1 trip-wire
  - DEC-938 2026-05-17 — Per-residency Aurora cluster has its own RLS `current_setting('auth.residency')` predicate ADDED on top of tenant_id predicate (defense-in-depth: even if tenant_id collides across residencies — which shouldn't happen per DEC-927 — residency predicate prevents leak)
  - DEC-939 2026-05-17 — Cross-residency-write trip-wire: trigger on every tenant-scoped table CHECKs `NEW.residency = current_setting('auth.residency')`; mismatch raises `cross_residency_write_blocked` exception + emits memory row
  - DEC-940 2026-05-17 — Residency-failover deferred (no automatic failover from sg-1 to us-1 etc.) — DR is intra-region multi-AZ only; cross-region DR ships in P4 (FR-TEN-2xx)
  - DEC-941 2026-05-17 — Per-residency memory audit kinds (8): ten.residency_provisioned, ten.residency_pool_misroute, ten.cross_residency_access_attempt, ten.cross_residency_write_blocked, ten.cross_residency_memory_event_blocked, ten.residency_health_degraded, ten.residency_kms_unavailable, ten.tenant_residency_assigned
  - DEC-942 2026-05-17 — Residency-aware logging context — every log line carries `residency=<rid>` field via tracing instrument; missing field = OBS alarm
  - DEC-943 2026-05-17 — `cyberos-ten residency-status` CLI shows per-residency health (Aurora connection, S3 reachability, NATS heartbeat, Stripe ping, KMS responsiveness, JWT issuer responsiveness) — 6-component score per residency
  - DEC-944 2026-05-17 — Per-residency provisioning runbook (FR-OBS-007 routable) on residency_pool_misroute or kms_unavailable
  - DEC-945 2026-05-17 — Provisioning CLI requires `--residency` flag explicit at slice 2 (no auto-derive); operator MUST pass the correct residency; CCO process review checks consistency between billing_currency + residency
  - PDPL Law 91/2025 Art. 17 (cross-border transfer + contract residency disclosure)
  - GDPR Art. 44 (cross-border data transfer — eu-1 → us-1 ONLY via SCC + only with explicit customer consent at provisioning)
  - Singapore PDPA §26 (cross-border transfer comparable protection standard)
  - SOC 2 CC6.1 (residency boundary as a security boundary)
  - ISO 27001 A.13.2 (data classification + residency control)

build_envelope:
  language: rust 1.81 + sql + terraform
  service: cyberos/services/ten/ + cyberos/infra/
  new_files:
    - services/ten/migrations/0015_residency_enum.sql               # closed 4-value enum
    - services/ten/migrations/0016_residency_trip_wire.sql           # cross-residency-write trigger on every tenant table
    - services/ten/migrations/0017_residency_health_log.sql          # per-residency health snapshot log
    - services/ten/src/residency/mod.rs                              # residency orchestrator
    - services/ten/src/residency/pool_router.rs                      # Map<Residency, PgPool>
    - services/ten/src/residency/s3_router.rs                        # Map<Residency, S3Client>
    - services/ten/src/residency/nats_router.rs                      # Map<Residency, NatsClient>
    - services/ten/src/residency/kms_router.rs                       # Map<Residency, KmsClient>
    - services/ten/src/residency/health.rs                           # 6-component health check
    - services/ten/src/residency/trip_wire.rs                        # cross-residency intercept
    - services/ten/src/residency/issuer_map.rs                       # AUTH issuer per residency
    - services/ten/src/cli/residency_status.rs                       # cyberos-ten residency-status
    - services/ten/src/audit/residency_events.rs                     # 8 memory row builders
    - infra/terraform/residency/sg-1/main.tf                         # SG VPC + Aurora + S3 + NATS + KMS
    - infra/terraform/residency/eu-1/main.tf                         # EU equivalent
    - infra/terraform/residency/us-1/main.tf                         # US equivalent
    - infra/terraform/residency/vn-1/main.tf                         # VN (physically ap-southeast-1 per DEC-930)
    - infra/terraform/residency/_shared/modules/aurora-residency-cluster
    - infra/terraform/residency/_shared/modules/s3-residency-bucket
    - infra/terraform/residency/_shared/modules/nats-residency-cluster
    - infra/terraform/residency/_shared/modules/kms-residency-key
    - services/ten/tests/residency_enum_cardinality_test.rs
    - services/ten/tests/residency_currency_mapping_test.rs
    - services/ten/tests/residency_trip_wire_test.rs
    - services/ten/tests/residency_immutable_test.rs
    - services/ten/tests/residency_pool_routing_test.rs
    - services/ten/tests/residency_pool_misroute_test.rs
    - services/ten/tests/residency_jwt_cross_rejection_test.rs
    - services/ten/tests/residency_atomic_provisioning_test.rs
    - services/ten/tests/residency_health_check_test.rs
    - services/ten/tests/residency_memory_chain_partitioning_test.rs
    - services/ten/tests/residency_kms_unavailable_test.rs
    - services/ten/tests/residency_audit_emission_test.rs

  modified_files:
    - services/ten/src/lib.rs                                          # mount residency_router
    - services/ten/src/provisioning/orchestrator.rs                    # consume residency router
    - services/ten/migrations/0003_tenant_residency_map.sql            # change residency column type from TEXT to residency_enum
    - services/ten/Cargo.toml                                          # +aws-sdk-kms, +aws-sdk-s3
    - services/auth/src/handlers/login.rs                              # set auth.residency session-local on JWT validate
    - services/inv/src/lib.rs                                          # consume residency router for cross-residency-INSERT trip
    - services/metering/src/recorder.rs                                # consume residency router
    - services/email/src/sender.rs                                     # consume residency router for SES region selection
    - services/doc/src/lib.rs                                          # consume residency router for S3 region

  allowed_tools:
    - file_read: services/**
    - file_read: infra/terraform/**
    - file_write: services/ten/{src,tests,migrations}/**
    - file_write: infra/terraform/residency/**
    - file_write: services/{auth,inv,metering,email,doc}/src/**       # residency-aware handler wiring
    - bash: cd services/ten && cargo test residency
    - bash: cd infra/terraform/residency/sg-1 && terraform plan
    - bash: cd infra/terraform/residency/sg-1 && terraform apply

  disallowed_tools:
    - extend `residency` enum beyond {sg-1, eu-1, us-1, vn-1} without ADR (per DEC-920)
    - allow cross-residency read (per DEC-923)
    - allow cross-residency write (per DEC-924)
    - share KMS key across residencies (per DEC-929)
    - share Aurora cluster across residencies (per DEC-921)
    - mutate tenant.residency on existing tenant (per DEC-925)
    - hardcode residency-to-region mapping outside of `residency/mod.rs` (single source of truth)
    - skip the cross-residency-write trip-wire trigger on any tenant-scoped table (per DEC-924 + DEC-939)

effort_hours: 10
sub_tasks:
  - "0.6h: 0015_residency_enum.sql + 0017_residency_health_log.sql"
  - "0.8h: 0016_residency_trip_wire.sql — trigger function + apply to all 28 tenant-scoped tables via cursor"
  - "0.6h: residency/mod.rs + residency_enum_cardinality_test"
  - "1.0h: residency/pool_router.rs + s3_router.rs + nats_router.rs + kms_router.rs"
  - "0.5h: residency/issuer_map.rs (4 issuer URLs + per-residency JWKS fetch)"
  - "0.5h: residency/health.rs (6-component health check)"
  - "0.5h: residency/trip_wire.rs (handler-side check before any tenant-scoped write)"
  - "0.4h: cli/residency_status.rs"
  - "0.4h: audit/residency_events.rs (8 builders)"
  - "1.5h: infra/terraform — 4 residency dirs + 4 shared modules (aurora/s3/nats/kms)"
  - "1.5h: tests — 12 test files covering enum + currency mapping + trip-wire + immutable + pool routing + misroute + JWT cross-rejection + atomic provisioning + health + chain partitioning + KMS + audit"
  - "0.5h: wire-up — handlers in auth/inv/metering/email/doc consume residency router"
  - "0.7h: integration smoke — provision a tenant in each residency + verify isolation"

risk_if_skipped: "Without 4-residency provisioning, every tenant lives in one global region — non-compliant for EU (GDPR Art. 44 cross-border transfer) + SG (PDPA §26) + VN (PDPL Law 91/2025 Art. 17) regulated customers. CyberSkill cannot sell to any regulated EU/SG/VN customer without this. Without DEC-923's cross-residency read forbiddance, a bug in service A could read tenant B's data from the wrong region. Without DEC-924's trip-wire defense-in-depth (trigger + FDW guard + pool routing), one buggy handler can silently leak cross-region data — undetectable until audit. Without DEC-925's residency immutability, accidental tenant.residency UPDATE could move data to the wrong region without copying — data loss + compliance violation. Without DEC-926's per-residency memory partitioning, audit chains commingle across regions — single-tenant subpoena response leaks other tenants' audit data. Without DEC-929's per-residency KMS keys, a key compromise in one region cascades to all four. Without DEC-936's atomic provisioning, half-provisioned tenants accumulate (Aurora row but no S3 prefix) becoming silent corruption sources. Without DEC-942's residency-aware logging, post-incident attribution ('which region had the misroute?') is impossible. The 10h effort lands the residency control plane that unlocks the entire international + regulated-market commercial arc."
---

## §1 — Description (BCP-14 normative)

The TEN service **MUST** ship 4-residency provisioning (`sg-1`, `eu-1`, `us-1`, `vn-1`) with per-residency Postgres + S3 + NATS + KMS + Stripe + AUTH-issuer separation, defense-in-depth cross-residency-write trip-wires, atomic-at-residency provisioning, per-residency memory chain partitioning, residency-aware logging context, 6-component health check + CLI, and 8 memory audit kinds.

1. **MUST** define the closed `residency` Postgres enum at migration `0015`: `('sg-1','eu-1','us-1','vn-1')`. CI cardinality test asserts exactly 4 per DEC-920. Adding a fifth requires schema migration + DEC entry.

2. **MUST** change `tenant_residency_map.residency` column type from TEXT to `residency` enum via migration `0003` modification (single source of truth — no free-text residency anywhere). Existing rows are validated against the enum at migration; non-conforming rows reject migration (fail-fast).

3. **MUST** enforce closed currency → residency mapping per DEC-922 (consistent with FR-TEN-003 DEC-785 + FR-TEN-101 DEC-825):
    - `VND → vn-1`
    - `SGD → sg-1`
    - `EUR → eu-1`
    - `GBP → eu-1`
    - `USD → us-1`
   The mapping is a `const fn` in `services/ten/src/residency/mod.rs::derive_residency(billing_currency)`; no other code path may compute the mapping (single source of truth).

4. **MUST** lock `tenant.residency` post-provisioning per DEC-925. Migration `0016` includes a trigger `trg_residency_immutable()` that RAISEs on any UPDATE attempting to change `tenants.residency` (analogous to FR-TEN-003 `trg_billing_currency_immutable` per DEC-798). Mutation requires new tenant + manual data migration.

5. **MUST** provision per-residency infrastructure via Terraform modules at `infra/terraform/residency/{residency}/main.tf`. Each residency stands up:
    - One Aurora PostgreSQL cluster in a private VPC (no peering per DEC-928).
    - One S3 bucket pair: `cyberos-{residency}-tenants` (data) + `cyberos-{residency}-audit` (audit archive); cross-region replication DISABLED per DEC-933.
    - One NATS cluster (3-node JetStream); subjects local per DEC-932.
    - One AWS KMS key for envelope encryption (DEC-929: region-pinned).
    - One OBS stack subset (Loki + Tempo + Prometheus regional shards) per DEC-934.
    - One Stripe account binding per DEC-935 (FR-TEN-003 DEC-801 already provisioned; this FR consumes).
    - One AUTH issuer URL per DEC-931 (`https://auth.<residency>.cyberos.world`).

6. **MUST** provide a residency router in `services/ten/src/residency/`:
    - `pool_router.rs`: `Map<Residency, PgPool>` with per-residency connection strings loaded from KMS-encrypted secrets.
    - `s3_router.rs`: `Map<Residency, aws_sdk_s3::Client>` configured to the correct AWS region per DEC-929.
    - `nats_router.rs`: `Map<Residency, async_nats::Client>` connected to per-residency NATS clusters.
    - `kms_router.rs`: `Map<Residency, aws_sdk_kms::Client>` for envelope decrypt operations.
    - `issuer_map.rs`: `Map<Residency, IssuerConfig>` mapping residency → AUTH issuer URL + JWKS URL.

7. **MUST** route every tenant-scoped operation through the residency router. Service-level handlers MUST:
    - Receive `tenant_id` from request context.
    - Lookup `tenants.residency` via a per-residency-fanout query (one cross-residency lookup is permitted at the entry: the JWT carries `residency` claim per FR-AUTH-004 + this FR's modification, so the lookup is JWT-only in 99% of cases).
    - Select the correct pool/client from the router.
    - Issue the operation against that pool only.
    - Mis-route (e.g., handler uses sg-1 pool for an eu-1 tenant) is caught by the trip-wire trigger + emits `ten.residency_pool_misroute` sev-1.

8. **MUST** install a cross-residency-write trip-wire trigger on EVERY tenant-scoped Postgres table per DEC-924 + DEC-939 + feature-request-audit skill rule 13 derivative. The trigger function `trg_cross_residency_write_block()`:
    ```sql
    CREATE OR REPLACE FUNCTION trg_cross_residency_write_block() RETURNS trigger AS $$
    DECLARE expected_residency TEXT := current_setting('auth.residency', true);
    BEGIN
      IF expected_residency IS NULL OR expected_residency = '' THEN
        RAISE EXCEPTION 'cross_residency_write_blocked: auth.residency session var not set';
      END IF;
      IF EXISTS (SELECT 1 FROM tenants WHERE id = NEW.tenant_id AND residency::text != expected_residency) THEN
        RAISE EXCEPTION 'cross_residency_write_blocked: tenant residency != session residency';
      END IF;
      RETURN NEW;
    END $$ LANGUAGE plpgsql;
    ```
   Applied via cursor loop in migration `0016` to all 28 tenant-scoped tables identified at migration time. New tables added in future FRs MUST include this trigger.

9. **MUST** set the session-local `auth.residency` Postgres setting at handler entry from the validated JWT's `residency` claim. The FR-AUTH-004 JWT mint is extended to include this claim; FR-AUTH-004's `services/auth/src/handlers/login.rs` modification covers the wiring.

10. **MUST** enforce per-residency JWT issuer validation per DEC-931. The AUTH issuer URL embedded in the JWT (`iss` claim) MUST match the residency-derived expected issuer; mismatch returns `401 wrong_residency_token` + emits `ten.cross_residency_access_attempt` sev-1.

11. **MUST** partition the memory audit chain per residency per DEC-926. Each residency's services append to that residency's chain only; the chain head is stored in that residency's Aurora `memory.chain_state` table. Cross-residency memory events are FORBIDDEN — attempting to append a chain row for a tenant in a different residency than the current handler's residency raises `cross_residency_memory_event_blocked` + emits the corresponding memory row in the LOCAL chain.

12. **MUST** provision atomically per residency per DEC-936. The `services/ten/src/provisioning/orchestrator.rs` (modified per FR-TEN-001 build envelope) runs:
    1. Open tx in target residency's Aurora.
    2. Insert tenant row + tenant_residency_map row.
    3. Create S3 prefix in target residency's bucket (idempotent marker object).
    4. Register NATS subject in target residency's cluster.
    5. Verify AUTH bootstrap in target residency.
    6. Commit tx.
   On any step failure: ROLLBACK tx + manual cleanup of any partial S3/NATS state via `cyberos-ten residency-cleanup-orphan` (slice 3 CLI; slice 2 = operator runs cleanup manually after sev-1 alert per DEC-944).

13. **MUST** add `auth.residency` to AUTH JWT claims per #9 + DEC-931. JWT shape post-this-FR: `{ sub, tenant_id, residency, exp, iat, iss, aud, scope_grants, persona }`. Tokens issued before this FR ships (legacy) lack the claim; transitional handler accepts them only for 24h post-deploy then enforces presence (fail closed).

14. **MUST** carry `residency=<rid>` field on every log line via `tracing::instrument(fields(residency = %ctx.residency))` per DEC-942. Missing field on any log line emitted from a tenant-scoped handler raises an OBS alarm sev-3 (`ten.residency_logging_context_missing` — informational, ops sweep).

15. **MUST** ship `cyberos-ten residency-status` CLI per DEC-943 — 6-component health check per residency:
    1. Aurora connectivity (SELECT 1 + < 100ms).
    2. S3 reachability (PUT/DELETE marker object + < 500ms).
    3. NATS heartbeat (publish + subscribe round-trip + < 200ms).
    4. Stripe ping (GET /v1/account + < 1s — only stripe-rail residencies).
    5. KMS responsiveness (Encrypt 32 bytes + < 200ms).
    6. AUTH issuer responsiveness (GET JWKS + < 500ms).
    Output: per-residency score 0-6 with timing per component; exit code 0 if all 4 residencies score ≥ 5, exit code 73 otherwise.

16. **MUST** emit 8 memory audit row kinds per DEC-941 (feature-request-audit skill rule 6 namespace pattern):
    - `ten.tenant_residency_assigned` (sev-2 — material commercial event)
    - `ten.residency_provisioned` (sev-1 — infrastructure event; one per residency standup)
    - `ten.residency_pool_misroute` (sev-1 — silent-leak prevented)
    - `ten.cross_residency_access_attempt` (sev-1 — security signal)
    - `ten.cross_residency_write_blocked` (sev-1 — trip-wire fired)
    - `ten.cross_residency_memory_event_blocked` (sev-1 — chain pollution prevented)
    - `ten.residency_health_degraded` (sev-2 — one component failing)
    - `ten.residency_kms_unavailable` (sev-1 — encryption broken in one region)

17. **MUST** maintain `residency_health_log` table at migration `0017`: `(id BIGSERIAL PRIMARY KEY, residency residency NOT NULL, component TEXT NOT NULL CHECK (component IN ('aurora','s3','nats','stripe','kms','auth_issuer')), status TEXT NOT NULL CHECK (status IN ('healthy','degraded','down')), latency_ms INT, checked_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Append-only via REVOKE per feature-request-audit skill rule 12. Per-residency append + global read (no RLS — health is system-tenant scope).

18. **MUST** map vn-1 to ap-southeast-1 physical region per DEC-930. The mapping is documented in `services/ten/src/residency/mod.rs` const + reflected in `infra/terraform/residency/vn-1/main.tf` as `provider "aws" { region = "ap-southeast-1" }`. PDPL Law 91/2025 §17 disclosure is presented at FR-TEN-101 signup for VN-residency tenants.

19. **MUST** require explicit `--residency` flag on FR-TEN-001's `cyberos-ten provision` CLI per DEC-945 (no auto-derive at slice 2). CCO process review checks consistency between provided `--billing-currency` and `--residency` against the DEC-922 mapping; mismatch is rejected with exit code 64 + message naming the expected residency.

20. **MUST** PII-scrub per-residency memory rows via FR-MEMORY-111 (feature-request-audit skill rule 18). Cross-residency event payloads carry `tenant_id_hash16` not the raw tenant_id (defense-in-depth: even if a cross-residency audit row leaks across residencies due to incident, no PII flows).

21. **MUST** thread W3C `traceparent` across the residency-fanout entry (single-cross-residency lookup) + per-residency operation (feature-request-audit skill rule 22 + 23 + 24). Trace_id present on every memory row + every log line.

22. **MUST NOT** support automatic failover between residencies at slice 2 per DEC-940. Cross-region DR is out-of-scope; per-residency multi-AZ Aurora is the DR primitive at slice 2.

23. **MUST NOT** share KMS keys across residencies per DEC-929. Per-residency keys are separate AWS KMS aliases; cross-region key replication NOT enabled (consistent with DEC-921 no-shared-infra).

24. **MUST NOT** allow `tenant_id` collision across residencies per DEC-927. UUIDv7 generation is collision-free across regions via the residency-prefix nibble pattern: high nibble of byte 6 encodes residency (`0=sg-1, 1=eu-1, 2=us-1, 3=vn-1`). This is reserved encoding; FR-TEN-001 modified_files includes the generator update.

25. **SHOULD** observe per-residency p95 latency for tenant-scoped operations via OTel histogram `tenant_op_duration_seconds_by_residency`. Alarms route to FR-OBS-007 runbook router per residency.

---

## §2 — Why this design (rationale for humans)

**Why no-shared-infra per residency (§1 #5, DEC-921)?** Compliance regulators (GDPR Art. 44, PDPA §26, PDPL §17) treat "shared" as "transferred." Sharing an Aurora cluster across residencies means EU customer data is technically in scope of US-jurisdiction queries if a US handler executes against it. Zero-sharing is the only defensible architecture for regulated markets.

**Why defense-in-depth trip-wires (§1 #8, DEC-924)?** Cross-residency leaks are silent failures — the data just moves to the wrong region and we don't know until audit (typically years later). Three layers (pool router + trigger + Postgres FDW absence) means catching the bug at the first of: bad code (router catches), bad config (trigger catches), bad infra (no FDW to traverse). Each layer can fail; three together approach zero leakage probability.

**Why memory chain per-residency (§1 #11, DEC-926)?** Audit chains are subpoena-able. A single global chain means subpoena for tenant A's audit history forces production of ALL tenants' chain rows (the chain is a Merkle structure — you can't redact). Per-residency chains scope the subpoena response to one residency's tenants; cross-residency tenants are out of scope.

**Why per-residency KMS keys (§1 #5, DEC-929)?** Compromise scoping. One leaked KMS key = one residency's data decryptable. Cross-residency key sharing turns one compromise into four residencies' data exposed. Aligns with NIST SP 800-57 key-scoping guidance.

**Why vn-1 in ap-southeast-1 physical (§1 #18, DEC-930)?** AWS has no Vietnam region (Q2 2026 unchanged). Self-hosting in VN is infeasible at our scale. The PDPL §17 contract-residency clause permits this with explicit customer disclosure — the FR-TEN-101 signup consent flow handles the disclosure. When AWS opens a VN region, vn-1 migrates physically (data + Terraform); the residency identifier stays `vn-1` so customers don't see a label change.

**Why JWT-carries-residency (§1 #9, DEC-931)?** The JWT is presented at every request; reading the residency from the JWT is free (already validated). Looking up `tenants.residency` per request would require a cross-residency lookup — defeats the whole architecture. JWT-carries-residency means the per-request residency is signed by the AUTH issuer + tamper-evident.

**Why immutable residency (§1 #4, DEC-925)?** Changing residency = physically moving data across regions = a migration project (Aurora dump + restore + verify + cutover + back-out plan). Allowing UPDATE on `tenants.residency` would silently corrupt — the row would say "eu-1" but the data is still in us-1's bucket/cluster. Lock at the schema level, force migration through ops review.

**Why atomic provisioning per-residency (§1 #12, DEC-936)?** Half-provisioned tenants accumulate (tenants table row but no S3 prefix). At scale, ops alerts about "missing S3 prefix" become noise. Atomic transitions ensure tenant either exists fully or not at all in each residency. Cross-residency atomicity is intentionally NOT attempted (would require 2-phase commit across AWS regions — operational nightmare).

**Why session-local `auth.residency` Postgres setting (§1 #9)?** RLS evaluates `current_setting('auth.residency')` per row at query time. Setting it per-request via `SET LOCAL` makes it visible to RLS predicates + trip-wire triggers without polluting connection state. This is standard Postgres RLS pattern.

**Why explicit --residency flag (§1 #19, DEC-945)?** Auto-derivation hides operator intent. If billing_currency is wrong, auto-derive propagates the error. Explicit flag forces operator to commit to a residency separately from currency; CCO process review catches mismatches before provisioning.

**Why 8 audit kinds with heavy sev-1 weighting (§1 #16, DEC-941)?** Every kind in this FR is a cross-residency event — by definition unusual + forensically critical. Sev-1 routes to FR-OBS-007's CHAT/PagerDuty path for immediate response. Cross-residency leakage is a regulatory-reportable event; sev-1 catches it before any breach-notification window closes.

---

## §3 — API contract

### 3.1 Postgres schema

```sql
-- 0015_residency_enum.sql
CREATE TYPE residency AS ENUM ('sg-1','eu-1','us-1','vn-1');

-- Modification to 0003_tenant_residency_map.sql (referenced in modified_files):
ALTER TABLE tenant_residency_map ALTER COLUMN residency TYPE residency USING residency::residency;
ALTER TABLE tenants ADD COLUMN residency residency;  -- nullable until backfill complete
UPDATE tenants t SET residency = (SELECT residency FROM tenant_residency_map WHERE tenant_id=t.id);
ALTER TABLE tenants ALTER COLUMN residency SET NOT NULL;

-- Immutability trigger (DEC-925)
CREATE OR REPLACE FUNCTION trg_residency_immutable() RETURNS trigger AS $$
BEGIN
  IF OLD.residency IS DISTINCT FROM NEW.residency THEN
    RAISE EXCEPTION 'residency_immutable: cannot change residency on existing tenant (DEC-925); create new tenant + manual data migration';
  END IF;
  RETURN NEW;
END $$ LANGUAGE plpgsql;
CREATE TRIGGER tenants_residency_immutable
  BEFORE UPDATE ON tenants
  FOR EACH ROW EXECUTE FUNCTION trg_residency_immutable();

-- 0016_residency_trip_wire.sql
CREATE OR REPLACE FUNCTION trg_cross_residency_write_block() RETURNS trigger AS $$
DECLARE expected_residency TEXT := current_setting('auth.residency', true);
DECLARE row_tenant_id UUID;
DECLARE row_residency TEXT;
BEGIN
  IF expected_residency IS NULL OR expected_residency = '' THEN
    RAISE EXCEPTION 'cross_residency_write_blocked: auth.residency session var not set';
  END IF;
  -- NEW.tenant_id may be null on system-tenant rows (e.g., audit log); skip check
  IF NEW.tenant_id IS NULL THEN RETURN NEW; END IF;
  SELECT residency::text INTO row_residency FROM tenants WHERE id = NEW.tenant_id;
  IF row_residency IS NULL THEN
    RAISE EXCEPTION 'cross_residency_write_blocked: tenant % not found in this residency', NEW.tenant_id;
  END IF;
  IF row_residency != expected_residency THEN
    RAISE EXCEPTION 'cross_residency_write_blocked: tenant % residency=% but session residency=%', NEW.tenant_id, row_residency, expected_residency;
  END IF;
  RETURN NEW;
END $$ LANGUAGE plpgsql;

-- Apply to all tenant-scoped tables (cursor loop):
DO $$
DECLARE t RECORD;
BEGIN
  FOR t IN
    SELECT c.table_name
      FROM information_schema.columns c
      WHERE c.column_name='tenant_id' AND c.table_schema='public'
  LOOP
    EXECUTE format(
      'CREATE TRIGGER cross_residency_write_trigger BEFORE INSERT OR UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION trg_cross_residency_write_block()',
      t.table_name
    );
  END LOOP;
END $$;

-- 0017_residency_health_log.sql
CREATE TABLE residency_health_log (
  id BIGSERIAL PRIMARY KEY,
  residency residency NOT NULL,
  component TEXT NOT NULL CHECK (component IN ('aurora','s3','nats','stripe','kms','auth_issuer')),
  status TEXT NOT NULL CHECK (status IN ('healthy','degraded','down')),
  latency_ms INT,
  checked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  trace_id CHAR(32)
);
CREATE INDEX idx_residency_health_residency_checked ON residency_health_log(residency, checked_at DESC);
REVOKE UPDATE, DELETE ON residency_health_log FROM cyberos_app;
```

### 3.2 Rust types

```rust
// services/ten/src/residency/mod.rs
#[derive(Copy, Clone, Eq, PartialEq, Debug, sqlx::Type, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "residency", rename_all = "lowercase")]
pub enum Residency { Sg1, Eu1, Us1, Vn1 }

impl Residency {
    pub const ALL: [Residency; 4] = [Residency::Sg1, Residency::Eu1, Residency::Us1, Residency::Vn1];

    pub const fn as_str(self) -> &'static str {
        match self {
            Residency::Sg1 => "sg-1",
            Residency::Eu1 => "eu-1",
            Residency::Us1 => "us-1",
            Residency::Vn1 => "vn-1",
        }
    }

    pub const fn aws_region(self) -> &'static str {
        match self {
            Residency::Sg1 => "ap-southeast-1",
            Residency::Eu1 => "eu-west-1",
            Residency::Us1 => "us-east-1",
            Residency::Vn1 => "ap-southeast-1",  // vn-1 physically in Singapore (DEC-930)
        }
    }
}

pub const fn derive_residency(currency: BillingCurrency) -> Residency {
    match currency {
        BillingCurrency::Vnd => Residency::Vn1,
        BillingCurrency::Sgd => Residency::Sg1,
        BillingCurrency::Eur => Residency::Eu1,
        BillingCurrency::Gbp => Residency::Eu1,
        BillingCurrency::Usd => Residency::Us1,
    }
}

// services/ten/src/residency/pool_router.rs
pub struct PoolRouter {
    pools: std::collections::HashMap<Residency, sqlx::PgPool>,
}

impl PoolRouter {
    pub fn pool_for(&self, residency: Residency) -> Result<&sqlx::PgPool, ResidencyError> {
        self.pools.get(&residency).ok_or(ResidencyError::ResidencyUnavailable(residency))
    }
}
```

### 3.3 CLI

```text
cyberos-ten residency-status [--residency sg-1|eu-1|us-1|vn-1] [--json]
cyberos-ten residency-cleanup-orphan <tenant_id>     (slice 3)
```

---

## §4 — Acceptance criteria

1. **Enum cardinality** — `residency_enum_cardinality_test` asserts enum = exactly `{sg-1, eu-1, us-1, vn-1}`.
2. **Currency → residency mapping** — `derive_residency(VND)=vn-1`, `(SGD)=sg-1`, `(EUR)=eu-1`, `(GBP)=eu-1`, `(USD)=us-1`.
3. **Residency immutable trigger** — `UPDATE tenants SET residency='eu-1' WHERE id=<sg-1 tenant>` raises `residency_immutable`.
4. **Cross-residency-write trip-wire** — handler with `SET LOCAL auth.residency='sg-1'` attempting INSERT into a tenant-scoped table with tenant in eu-1 raises `cross_residency_write_blocked`.
5. **Pool router selects correct pool** — `pool_router.pool_for(Eu1)` returns the eu-1 pool; mis-call returns ResidencyError.
6. **Pool misroute detected** — handler using sg-1 pool for eu-1 tenant operation triggers trip-wire → 500 + `ten.residency_pool_misroute` sev-1 emitted.
7. **JWT cross-residency rejection** — request with JWT carrying `iss=https://auth.us-1.cyberos.world` against eu-1 endpoint returns 401 + `ten.cross_residency_access_attempt`.
8. **Atomic provisioning** — provisioning failure at S3 step rolls back Aurora insert; no half-provisioned state in any residency.
9. **6-component health check** — `cyberos-ten residency-status` reports all 6 components per residency with timing.
10. **memory chain partitioning** — appending a chain row for an eu-1 tenant from a sg-1 handler raises `cross_residency_memory_event_blocked`.
11. **KMS per-residency** — eu-1 encrypt uses eu-1 KMS key; sg-1 encrypt uses sg-1 KMS key; cross-region attempts fail.
12. **vn-1 physical region** — Terraform plan for vn-1 targets ap-southeast-1; AWS resources are tagged with `cyberos_residency=vn-1` for clarity.
13. **UUIDv7 residency-prefix nibble** — tenant_id high nibble of byte 6 = residency index; collision-free across residencies.
14. **Explicit --residency flag required** — provision CLI without `--residency` exits 64 (invalid arg).
15. **Currency-residency mismatch rejected** — `--billing-currency VND --residency us-1` exits 64 with explicit error.
16. **JWT carries residency claim** — post-FR JWT has `residency` claim; pre-FR JWTs accepted for 24h transitional then 401.
17. **Logging carries `residency=<rid>` field** — log scrape of tenant handler shows field on every line.
18. **AUTH issuer URLs per residency** — `issuer_map` has 4 entries matching `https://auth.<residency>.cyberos.world`.
19. **No cross-residency NATS subscription** — sg-1 NATS subscriber cannot subscribe to `tenant.<slug>.*` on eu-1 cluster (separate clusters per DEC-932).
20. **`residency_health_log` append-only** — UPDATE on the table raises permission-denied (REVOKE enforced).

---

## §5 — Verification

### 5.1 `residency_enum_cardinality_test.rs`

```rust
#[tokio::test]
async fn residency_enum_has_exactly_4_values() {
    let ctx = TestContext::new().await;
    let labels: Vec<String> = sqlx::query_scalar(
        "SELECT unnest(enum_range(NULL::residency))::text"
    ).fetch_all(&ctx.pool).await.unwrap();
    let mut labels = labels;
    labels.sort();
    assert_eq!(labels, vec!["eu-1","sg-1","us-1","vn-1"]);
}
```

### 5.2 `residency_trip_wire_test.rs`

```rust
#[tokio::test]
async fn cross_residency_insert_blocked() {
    let ctx = TestContext::with_residency(Residency::Eu1).await;
    let sg_tenant = ctx.provision_tenant_in(Residency::Sg1, "sg-tenant").await;

    sqlx::query("SET LOCAL auth.residency = 'eu-1'").execute(&ctx.pool).await.unwrap();
    let err = sqlx::query("INSERT INTO projects (id, tenant_id, name) VALUES ($1, $2, 'x')")
        .bind(uuid::Uuid::new_v4()).bind(sg_tenant).execute(&ctx.pool).await.unwrap_err();
    assert!(err.to_string().contains("cross_residency_write_blocked"));

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "ten.cross_residency_write_blocked"));
}

#[tokio::test]
async fn missing_session_residency_blocks_write() {
    let ctx = TestContext::new().await;
    let tenant = ctx.provision_tenant_in(Residency::Sg1, "sg-tenant").await;
    // No SET LOCAL auth.residency
    let err = sqlx::query("INSERT INTO projects (id, tenant_id, name) VALUES ($1, $2, 'x')")
        .bind(uuid::Uuid::new_v4()).bind(tenant).execute(&ctx.pool).await.unwrap_err();
    assert!(err.to_string().contains("auth.residency session var not set"));
}
```

### 5.3 `residency_immutable_test.rs`

```rust
#[tokio::test]
async fn residency_cannot_change_post_provision() {
    let ctx = TestContext::new().await;
    let tenant = ctx.provision_tenant_in(Residency::Sg1, "sg-tenant").await;
    let err = sqlx::query("UPDATE tenants SET residency='eu-1' WHERE id=$1")
        .bind(tenant).execute(&ctx.pool).await.unwrap_err();
    assert!(err.to_string().contains("residency_immutable"));
}
```

### 5.4 `residency_pool_routing_test.rs`

```rust
#[tokio::test]
async fn pool_router_selects_correct_pool() {
    let ctx = TestContext::with_all_residencies().await;
    let sg_pool = ctx.pool_router.pool_for(Residency::Sg1).unwrap();
    let eu_pool = ctx.pool_router.pool_for(Residency::Eu1).unwrap();
    assert_ne!(sg_pool as *const _, eu_pool as *const _);  // different pools
    let sg_db: String = sqlx::query_scalar("SELECT current_database()").fetch_one(sg_pool).await.unwrap();
    let eu_db: String = sqlx::query_scalar("SELECT current_database()").fetch_one(eu_pool).await.unwrap();
    assert_ne!(sg_db, eu_db);
}
```

### 5.5 `residency_jwt_cross_rejection_test.rs`

```rust
#[tokio::test]
async fn wrong_residency_jwt_rejected() {
    let ctx = TestContext::with_residency(Residency::Eu1).await;
    let us_tenant = ctx.provision_tenant_in(Residency::Us1, "us-tenant").await;
    let us_jwt = ctx.mint_jwt_in_residency(us_tenant, Residency::Us1).await;

    let r = ctx.get_in_residency(Residency::Eu1, "/v1/projects").bearer_auth(us_jwt).send().await.unwrap();
    assert_eq!(r.status(), 401);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "wrong_residency_token");
}
```

### 5.6 `residency_atomic_provisioning_test.rs`

```rust
#[tokio::test]
async fn provisioning_rollback_on_s3_failure() {
    let ctx = TestContext::with_residency(Residency::Sg1).await;
    ctx.simulate_s3_failure();
    let result = ctx.provision_tenant_in(Residency::Sg1, "atomic-test").await;
    assert!(result.is_err());

    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM tenants WHERE slug='atomic-test'")
        .fetch_one(ctx.pool_router.pool_for(Residency::Sg1).unwrap()).await.unwrap();
    assert_eq!(count, 0);
}
```

### 5.7 `residency_health_check_test.rs`

```rust
#[tokio::test]
async fn all_residencies_healthy() {
    let ctx = TestContext::with_all_residencies().await;
    let report = run_residency_status(&ctx).await;
    for residency in Residency::ALL {
        let score = report.score_for(residency);
        assert_eq!(score.aurora.status, "healthy");
        assert!(score.aurora.latency_ms < 100);
        assert_eq!(score.s3.status, "healthy");
        assert_eq!(score.nats.status, "healthy");
        assert_eq!(score.kms.status, "healthy");
        assert_eq!(score.auth_issuer.status, "healthy");
    }
}
```

### 5.8 `residency_memory_chain_partitioning_test.rs`

```rust
#[tokio::test]
async fn cross_residency_memory_append_blocked() {
    let ctx = TestContext::with_all_residencies().await;
    let eu_tenant = ctx.provision_tenant_in(Residency::Eu1, "eu-tenant").await;
    let sg_handler_ctx = ctx.handler_ctx_in(Residency::Sg1);

    let result = sg_handler_ctx.memory.append_row(
        MemoryRow::new("test.event", eu_tenant, json!({})).build()
    ).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("cross_residency_memory_event_blocked"));

    let local_audit = sg_handler_ctx.memory.recent_rows().await;
    assert!(local_audit.iter().any(|r| r.kind == "ten.cross_residency_memory_event_blocked"));
}
```

### 5.9 `residency_currency_mapping_test.rs`

```rust
#[test]
fn currency_to_residency_mapping() {
    assert_eq!(derive_residency(BillingCurrency::Vnd), Residency::Vn1);
    assert_eq!(derive_residency(BillingCurrency::Sgd), Residency::Sg1);
    assert_eq!(derive_residency(BillingCurrency::Eur), Residency::Eu1);
    assert_eq!(derive_residency(BillingCurrency::Gbp), Residency::Eu1);
    assert_eq!(derive_residency(BillingCurrency::Usd), Residency::Us1);
}
```

### 5.10 `residency_pool_misroute_test.rs`

```rust
#[tokio::test]
async fn pool_misroute_caught_by_trip_wire() {
    let ctx = TestContext::with_all_residencies().await;
    let eu_tenant = ctx.provision_tenant_in(Residency::Eu1, "eu-tenant").await;
    // Simulate a buggy handler using the sg-1 pool for an eu-1 tenant
    let sg_pool = ctx.pool_router.pool_for(Residency::Sg1).unwrap();
    sqlx::query("SET LOCAL auth.residency = 'sg-1'").execute(sg_pool).await.unwrap();
    let err = sqlx::query("INSERT INTO projects (id, tenant_id, name) VALUES ($1, $2, 'x')")
        .bind(uuid::Uuid::new_v4()).bind(eu_tenant).execute(sg_pool).await.unwrap_err();
    assert!(err.to_string().contains("cross_residency_write_blocked"));

    let audit = ctx.memory_rows_in(Residency::Sg1).await;
    assert!(audit.iter().any(|r| r.kind == "ten.residency_pool_misroute"));
}
```

---

## §6 — Implementation skeleton

(API contract in §3 is the skeleton. Additional notes below.)

### 6.1 Handler-entry residency context setting

```rust
// At every tenant-scoped handler entry:
pub async fn require_residency(ctx: &AppCtx, jwt: &Jwt) -> Result<HandlerCtx, AuthError> {
    let residency = jwt.claims.residency.parse::<Residency>()?;
    let expected_issuer = ctx.residency_issuer_map.issuer_for(residency);
    if jwt.claims.iss != expected_issuer {
        emit_audit(&ctx, "ten.cross_residency_access_attempt", json!({
            "jwt_iss": jwt.claims.iss, "expected_iss": expected_issuer,
        })).await;
        return Err(AuthError::WrongResidencyToken);
    }
    let pool = ctx.pool_router.pool_for(residency)?;
    let mut conn = pool.acquire().await?;
    sqlx::query("SET LOCAL auth.residency = $1").bind(residency.as_str()).execute(&mut *conn).await?;
    sqlx::query("SET LOCAL auth.tenant_id = $1").bind(jwt.claims.tenant_id).execute(&mut *conn).await?;
    Ok(HandlerCtx { conn, residency, tenant_id: jwt.claims.tenant_id })
}
```

### 6.2 Residency health check

```rust
pub async fn check_residency(ctx: &AppCtx, residency: Residency) -> ResidencyHealthReport {
    let mut report = ResidencyHealthReport::new(residency);
    report.aurora = check_aurora(ctx.pool_router.pool_for(residency)?).await;
    report.s3 = check_s3(ctx.s3_router.client_for(residency)?, &format!("cyberos-{}-tenants", residency.as_str())).await;
    report.nats = check_nats(ctx.nats_router.client_for(residency)?).await;
    report.stripe = check_stripe(ctx.stripe_router.client_for(residency)?).await;
    report.kms = check_kms(ctx.kms_router.client_for(residency)?).await;
    report.auth_issuer = check_auth_issuer(ctx.residency_issuer_map.issuer_for(residency)).await;
    persist_health_log(ctx, &report).await;
    report
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **FR-AI-016** Residency pinning — establishes residency concept at AI layer; TEN-103 extends to data layer.
- **FR-TEN-001** Provisioning CLI — TEN-103 modifies the orchestrator to consume residency router.

**Cross-module (related_frs):**
- **FR-TEN-003** Stripe billing — per-residency Stripe account routing (DEC-801 consumed here).
- **FR-TEN-004** 4-axis metering — emits per-residency metering events; trip-wire applies.
- **FR-TEN-101** Self-serve signup — residency derivation at signup hand-off.
- **FR-TEN-102** VND domestic rail — vn-1 residency consumes this rail.
- **FR-TEN-104** Lifecycle — tenant termination per-residency.
- **FR-AUTH-004** JWT mint — `residency` claim added.
- **FR-AI-003** memory audit-row bridge — 8 new kinds register; chain partitioned per residency.
- **FR-MEMORY-111** PII scrubbing — tenant_id hash16 in cross-residency events.
- **FR-DOC-001** Documents — S3 bucket per-residency consumed.
- **FR-EMAIL-001** Email — SES region per-residency.
- **FR-OBS-005** Trace correlation — residency tag on every span.
- **FR-OBS-007** Auto-runbook — sev-1 alerts route per-residency.
- **FR-OBS-008** Compliance views — per-residency scoping consumed.

**Downstream (blocks):** None (this FR is bottom-of-stack for residency infrastructure; other FRs consume but don't block on TEN-103 at slice 2).

---

## §8 — Example payloads

### 8.1 `ten.cross_residency_write_blocked` memory row

```json
{
  "kind": "ten.cross_residency_write_blocked",
  "severity": 1,
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "actor_id": "system.ten.residency",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "expected_residency": "sg-1",
    "blocked_tenant_residency": "eu-1",
    "blocked_tenant_id_hash16": "f8a1b2c3d4e5f607",
    "table_name": "projects",
    "operation": "INSERT",
    "service": "cyberos-projects-service",
    "stack_trace_sha256": "9c4e7a8b..."
  }
}
```

### 8.2 `cyberos-ten residency-status --json` output

```json
{
  "checked_at": "2026-05-17T09:14:32.847Z",
  "overall_score": "22/24",
  "residencies": {
    "sg-1": {
      "score": "6/6",
      "components": {
        "aurora":      {"status": "healthy",  "latency_ms": 12},
        "s3":          {"status": "healthy",  "latency_ms": 84},
        "nats":        {"status": "healthy",  "latency_ms": 7},
        "stripe":      {"status": "healthy",  "latency_ms": 412},
        "kms":         {"status": "healthy",  "latency_ms": 18},
        "auth_issuer": {"status": "healthy",  "latency_ms": 89}
      }
    },
    "eu-1": {"score": "5/6", "components": { "nats": {"status": "degraded", "latency_ms": 380}, "...": "..." }},
    "us-1": {"score": "6/6", "components": "..."},
    "vn-1": {"score": "5/6", "components": "...", "note": "vn-1 physically hosted in ap-southeast-1 per DEC-930"}
  }
}
```

### 8.3 `ten.residency_provisioned` memory row (one per residency standup)

```json
{
  "kind": "ten.residency_provisioned",
  "severity": 1,
  "tenant_id": "00000000-0000-0000-0000-000000000001",
  "actor_id": "system.ten.residency",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "residency": "sg-1",
    "aws_region": "ap-southeast-1",
    "aurora_cluster_arn": "arn:aws:rds:ap-southeast-1:...",
    "s3_bucket": "cyberos-sg-1-tenants",
    "s3_audit_bucket": "cyberos-sg-1-audit",
    "nats_endpoint": "nats.sg-1.cyberos.world:4222",
    "kms_key_arn": "arn:aws:kms:ap-southeast-1:...",
    "auth_issuer_url": "https://auth.sg-1.cyberos.world",
    "terraform_apply_id": "tf_abc123..."
  }
}
```

---

## §9 — Open questions

All resolved for slice 2. Deferred:

- **Deferred:** Cross-region DR (failover from sg-1 → us-1 etc.) — slice 3, FR-TEN-2xx (placeholder).
- **Deferred:** vn-1 physical migration to AWS Vietnam region when it opens — slice 3, ops project (no code change required; Terraform plan-only).
- **Deferred:** Per-residency self-serve via signup form — FR-TEN-101 derives residency from billing_currency; explicit residency picker in UI is slice 3.
- **Deferred:** Cross-residency tenant migration (rare admin op) — slice 3, FR-TEN-2xx.
- **Deferred:** Residency-aware backup/restore CLI (`cyberos-ten residency-backup --to s3://...`) — slice 3.
- **Deferred:** `cyberos-ten residency-cleanup-orphan` CLI — slice 3 (slice 2 = manual ops cleanup on sev-1).
- **Deferred:** Per-residency rate limiting policy (different per residency) — slice 3.
- **Deferred:** Multi-region OBS query proxy (FR-OBS-002 enhancement for residency federation) — slice 3.

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| Cross-residency write attempt | trip-wire trigger raises | INSERT/UPDATE blocked + sev-1 `ten.cross_residency_write_blocked` | Operator investigates handler bug + fixes pool routing |
| Cross-residency read attempt | RLS USING predicate `residency=current_setting('auth.residency')` rejects | 0 rows returned (silent — RLS conformant); audited at handler if it expected ≥1 | Same: handler bug investigation |
| Pool misroute (handler uses wrong pool) | trip-wire catches OR rows-returned=0 | sev-1 `ten.residency_pool_misroute` + 500 to caller | Operator audits handler; fixes pool router consumption |
| JWT residency mismatch | issuer check at handler entry | 401 + `wrong_residency_token` + `ten.cross_residency_access_attempt` sev-1 | User signs in via correct-residency portal |
| `auth.residency` session var missing | trip-wire raises | INSERT blocked + sev-1 alert; handler bug | Handler audited; missing `SET LOCAL` call added |
| Residency mutation attempt | `trg_residency_immutable` raises | UPDATE blocked + audit | Manual data migration via cross-region tooling (slice 3) |
| Atomic provisioning rolls back | orchestrator step failure | Aurora row rolled back; S3/NATS partial cleanup needed via manual ops | `cyberos-ten residency-cleanup-orphan <slug>` (slice 3) |
| KMS unavailable in one residency | health check 200ms timeout | sev-1 `ten.residency_kms_unavailable` + degraded encryption | AWS KMS incident; encrypt-fail-closed (no plaintext write) until recovery |
| NATS cluster down in one residency | health check fails | sev-1 + that residency's tenant events queue locally (FR-TEN-004 WAL queue) | NATS-side recovery; queue drains |
| Aurora primary failover within residency | RDS Multi-AZ event | < 30s outage; handlers retry; sev-2 alert | Inherent — Multi-AZ DR primitive |
| Stripe US account credentials misconfig | Stripe API 401 | sev-1 + stripe-rail temporarily fails for that residency | Operator rotates per DEC-801 / FR-TEN-003 ops |
| Per-residency Terraform drift | nightly tf plan drift detector | sev-2 alert with diff | Operator reviews + applies tf changes (with change-management) |
| Cross-residency memory append attempted | chain.append guard | sev-1 `ten.cross_residency_memory_event_blocked` + local-chain row | Inherent — chain-pollution prevented |
| Residency-aware logging missing field | OBS log-quality sweep | sev-3 informational | CI test enforces; PR rejected if missing |
| New tenant table added without trip-wire trigger | migration audit (CI) | CI fails the PR | PR adds trigger before merge |
| UUIDv7 high-nibble collision (astronomically improbable) | partial-unique constraint on `(tenant_id)` global | INSERT fails | Re-generate ID (1-in-2^60 chance) |
| Per-residency JWKS rotation mid-flight | FR-AUTH-004 JWKS cache stale | Token validation fails briefly; cache refresh triggered | Inherent |
| Health check timeout false positive | network blip | One degraded-status row in `residency_health_log`; next 5-min poll recovers | Inherent — degraded ≠ down |
| `current_setting('auth.residency')` typo in handler | dev mistake | trip-wire raises; sev-1; handler ships broken | CI integration test catches |
| Migration `0016` cursor loop misses a table | check at CI | CI test asserts every `tenant_id`-bearing table has the trigger | CI catches; manual trigger add |
| Cross-residency NATS subject subscription attempt | NATS clusters are separate; connect to wrong cluster URL → connection refused | Connection-level failure; not a runtime risk | Inherent isolation |
| Per-residency KMS key revocation | AWS KMS audit | Encrypt operations fail in that residency; sev-1 alert | Operator restores key OR rotates to new key with re-encrypt sweep |

---

## §11 — Implementation notes

**§11.1** The 28-tenant-scoped-tables count (§1 #8) is a deploy-time inventory; CI test re-counts via `information_schema.columns WHERE column_name='tenant_id' AND table_schema='public'` and asserts each has the trip-wire trigger.

**§11.2** The trip-wire trigger does ONE additional SELECT per write (`SELECT residency FROM tenants WHERE id=NEW.tenant_id`). At write-heavy workloads this is a ~5% overhead; acceptable cost for the defense.

**§11.3** The `residency_immutable` trigger uses `IS DISTINCT FROM` to handle NULL-vs-NULL correctly (residency is NOT NULL post-backfill, but the IS-DISTINCT-FROM is future-proof if NULL ever becomes valid).

**§11.4** UUIDv7 residency-nibble encoding: byte 6 high nibble = residency index (0/1/2/3). UUIDv7 already uses byte 6 for version field; we reserve the high nibble of byte 7 instead (compatible with RFC 4122 + RFC 9562 extension space). `services/ten/src/residency/uuid_gen.rs` implements + tests cross-residency collision-free.

**§11.5** AWS region mapping is `const fn` (compile-time); changing it requires recompile, which is desired (region mapping is forensic-critical).

**§11.6** Per-residency Terraform state is stored in per-residency S3 backend (each residency's tf state lives in that residency's S3) — no central tf state that becomes a cross-residency single point of failure.

**§11.7** The handler-entry `require_residency` function (§6.1) is mandatory at every tenant-scoped handler; CI rule via `clippy::custom_lint` enforces presence.

**§11.8** `pool_router` and friends are loaded at service startup from KMS-encrypted connection strings in AWS Secrets Manager. Boot fails fast if any residency's secrets unreachable; service does NOT serve partial (better: refuse to start than serve wrong residency).

**§11.9** The `cyberos-ten residency-status` CLI is operator-only; runs under `cyberos_ops` IAM role with read-only KMS/Stripe/AUTH access.

**§11.10** Health check latency thresholds are SLOs, not absolute caps; intermittent breach degrades the score but doesn't fire sev-1 (sustained breach via FR-OBS-007 alarm definition does).

**§11.11** The transitional 24h post-deploy window for residency-claim-missing JWTs uses a feature flag `auth.residency_claim_required=false` that flips to true via a scheduled job at deploy+24h.

**§11.12** Per-residency memory chain Merkle Mountain Range proof (FR-MEMORY-101 §6.4 PROPOSAL P2) is computed per-residency; cross-residency chain-of-custody verification is out-of-scope at slice 2.

**§11.13** The `Cross-Region-NATS-Sync` is intentionally absent — DEC-932 forbids; future cross-region event federation (slice 3) would require explicit consent mechanism per regulation.

**§11.14** Test fixtures for `with_all_residencies()` provision 4 in-memory Postgres instances + 4 LocalStack S3 buckets + 4 NATS clusters; integration tests against real AWS deploy nightly (not on every PR).

**§11.15** The `residency_health_log` table's `latency_ms INT` is nullable for `status='down'` (no latency observable when down); other statuses have non-null latency.

**§11.16** The 8 memory kinds (§1 #16) are all sev-1 or sev-2 because every kind by definition is unusual: routine residency operation does not emit (steady-state silent). FR-AI-003 closed-set extension adds 8.

**§11.17** Trip-wire trigger error messages include enough context to identify the buggy handler (table name, expected vs actual residency, tenant_id hash16). Operators can grep memory rows to find the offending service.

**§11.18** Per-residency Terraform modules share helper modules at `infra/terraform/residency/_shared/modules/` — DRY without sharing actual resources.

**§11.19** The vn-1 residency's PDPL-disclosure copy is at `services/ten/web/signup/consents/vi/pdpl-vn-residency-disclosure-v1.md` — FR-TEN-101 surfaces this at signup for VN-residency tenants.

**§11.20** Cross-region IAM role policies — each residency's services use IAM roles scoped to that residency's resources only; no cross-region role assumption permitted.

**§11.21** Per-residency Stripe account API keys (FR-TEN-003 DEC-801) are stored in that residency's AWS Secrets Manager only; no shared key store.

**§11.22** The `residency_currency_mapping_test` is a pure-Rust unit test (no DB); validates the const fn truth-table.

---

*End of FR-TEN-103 spec.*
