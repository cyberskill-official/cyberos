---
id: TASK-PORTAL-002
title: "PORTAL per-tenant brand pack — logo + colour palette + custom CNAME + email template overrides + ACME-issued TLS cert + brand-asset versioning"
module: PORTAL
priority: MUST
status: draft
verify: T
phase: P4
milestone: P4 · slice 1
slice: 1
owner: Stephen Cheng (CCO)
created: 2026-05-17
shipped: null
memory_chain_hash: null
related_tasks: [TASK-PORTAL-001, TASK-PORTAL-003, TASK-PORTAL-005, TASK-TEN-101, TASK-TEN-103, TASK-AUTH-101, TASK-EMAIL-001, TASK-DOC-001, TASK-AI-003, TASK-MEMORY-111, TASK-OBS-005]
depends_on: [TASK-TEN-101]
blocks: []

source_pages:
  - website/docs/modules/portal.html#brand-pack
  - https://datatracker.ietf.org/doc/html/rfc8555    # ACME (TLS cert issuance)
  - https://www.w3.org/TR/WCAG21/#contrast-minimum   # contrast accessibility
  - https://html.spec.whatwg.org/multipage/links.html#sec-link-types

source_decisions:
  - DEC-1000 2026-05-17 — Per-tenant brand pack stored in `portal_brand_packs` table, KMS-encrypted at rest, served via CDN-cached endpoint with ETag invalidation on update
  - DEC-1001 2026-05-17 — Logo: PNG / SVG only (no JPEG — alpha-channel needed); max 1 MiB per file; 3 size variants (favicon 32×32, header 200×60, splash 800×240) auto-generated from canonical upload via vips
  - DEC-1002 2026-05-17 — Colour palette: 6 named slots (primary, secondary, accent, background, surface, error) — closed enum; tenant uploads either hex codes or picks from preset palettes (12 presets)
  - DEC-1003 2026-05-17 — WCAG 2.1 AA contrast enforced at save-time: primary-on-background ≥ 4.5:1; error-on-background ≥ 4.5:1; non-conformant submissions rejected with explicit contrast-ratio feedback
  - DEC-1004 2026-05-17 — Custom CNAME (e.g., `portal.acme.com → <slug>.cyberos.world`) — DNS verification via TXT record before CNAME activation; ACME (Let's Encrypt) TLS cert auto-issued + auto-renewed at 30-day pre-expiry
  - DEC-1005 2026-05-17 — Per-tenant CNAME requires `tenant_admin` role at TENANT level (not engagement_admin) — DNS misconfig has cross-engagement blast radius (mirrors TASK-PORTAL-003 DEC-882 pattern)
  - DEC-1006 2026-05-17 — Email template overrides: per-tenant Tera template fragments for TASK-EMAIL-001 templates (welcome, magic-link, password-reset, invoice-receipt); fallback chain tenant → CyberSkill default
  - DEC-1007 2026-05-17 — Brand-pack versioning: every save creates a new immutable version row; activation pointer in `portal_brand_pack_active` table; rollback = re-point active to prior version
  - DEC-1008 2026-05-17 — CDN caching: 5-min edge TTL + ETag invalidation on activation pointer change; serves at `cdn.cyberos.world/brand/<tenant_slug>/<asset>.<ext>?v=<sha16>` with content-hash query param
  - DEC-1009 2026-05-17 — Brand assets served public-no-auth from CDN (logos + colour CSS) per industry convention; PII-scrubbed (no PII inside brand assets per validation)
  - DEC-1010 2026-05-17 — Per-tenant CNAME limited to ONE custom CNAME at slice 1 (multi-CNAME for multi-region landing pages = slice 2)
  - DEC-1011 2026-05-17 — ACME TLS cert auto-renewal job runs daily; certs renewed at T-30 days; renewal failure → sev-2 alert + 24h escalation window before sev-1
  - DEC-1012 2026-05-17 — Brand-pack export: tenant_admin can export current brand-pack as JSON + assets bundle (.zip) for migration / backup; ALWAYS deterministic per task-audit skill rule 27-28
  - DEC-1013 2026-05-17 — Closed enum `brand_asset_kind` = {favicon, header_logo, splash_logo, email_logo}; CI cardinality test asserts 4
  - DEC-1014 2026-05-17 — RLS scoped to `tenant_id = current_setting('auth.tenant_id')::uuid` on all 3 PORTAL brand tables
  - DEC-1015 2026-05-17 — Asset upload size cap: 1 MiB per asset; total brand pack < 5 MiB (5 assets × 1 MiB); rejected uploads return 413 PAYLOAD_TOO_LARGE
  - DEC-1016 2026-05-17 — Email template override syntax: Tera fragments with a fixed allowed-tag list (no script execution, no file_read, no env access) — sandboxed Tera context per CyberOS template guidelines
  - DEC-1017 2026-05-17 — memory audit kinds: portal.brand_pack_created, portal.brand_pack_activated, portal.brand_pack_rolled_back, portal.cname_dns_verified, portal.cname_tls_issued, portal.cname_tls_renewed, portal.cname_tls_renewal_failed, portal.brand_pack_validation_rejected
  - DEC-1018 2026-05-17 — Per-tenant brand-pack edit limited to 10 saves per day (rate limit) to prevent storage exhaustion via rapid re-saves
  - DEC-1019 2026-05-17 — Image content-type validation via magic-bytes (not Content-Type header) — defends against spoofed uploads (e.g. executable disguised as PNG)
  - DEC-1020 2026-05-17 — Tenant-set custom favicon overrides browser default for `<slug>.cyberos.world` AND custom CNAME if active
  - eIDAS QES (out-of-scope at slice 1 — brand pack does not include signing certs; QES integration is FR-DOC-2xx)
  - WCAG 2.1 AA (contrast minima for accessibility — DEC-1003)
  - GDPR Art. 25 (data protection by design — brand pack is no-PII per DEC-1009 validation)

build_envelope:
  language: rust 1.81 + typescript 5.5 (UI)
  service: cyberos/services/portal/
  new_files:
    - services/portal/migrations/0005_portal_brand_packs.sql           # versioned brand pack store
    - services/portal/migrations/0006_portal_brand_pack_active.sql      # activation pointer (one per tenant)
    - services/portal/migrations/0007_portal_brand_assets.sql           # KMS-wrapped asset binaries + variants
    - services/portal/migrations/0008_portal_cname_configs.sql          # custom CNAME + DNS verify + TLS cert lifecycle
    - services/portal/src/brand/mod.rs                                  # brand orchestrator
    - services/portal/src/brand/validate.rs                             # WCAG contrast + magic-bytes + size checks
    - services/portal/src/brand/image_pipeline.rs                       # logo → 3 variants via vips
    - services/portal/src/brand/email_overrides.rs                      # Tera fragment loader + sandbox
    - services/portal/src/brand/version.rs                              # version + activate + rollback
    - services/portal/src/brand/cdn.rs                                  # CDN URL builder + ETag
    - services/portal/src/cname/mod.rs                                  # CNAME orchestrator
    - services/portal/src/cname/dns_verify.rs                           # TXT-record verification
    - services/portal/src/cname/acme.rs                                 # ACME TLS issuance + renewal
    - services/portal/src/handlers/brand_pack_routes.rs                 # POST/PATCH/GET tenant brand pack
    - services/portal/src/handlers/brand_asset_serve.rs                 # public CDN-origin asset serve
    - services/portal/src/handlers/cname_admin.rs                       # POST CNAME / verify / activate
    - services/portal/src/cli/cname_renewal_job.rs                      # daily ACME renewal
    - services/portal/src/audit/brand_events.rs                         # 8 memory row builders
    - services/portal/tests/brand_pack_create_test.rs
    - services/portal/tests/brand_pack_version_test.rs
    - services/portal/tests/brand_pack_rollback_test.rs
    - services/portal/tests/wcag_contrast_enforcement_test.rs
    - services/portal/tests/magic_bytes_validation_test.rs
    - services/portal/tests/image_pipeline_variants_test.rs
    - services/portal/tests/cname_dns_verify_test.rs
    - services/portal/tests/cname_acme_issuance_test.rs
    - services/portal/tests/cname_acme_renewal_test.rs
    - services/portal/tests/email_override_sandbox_test.rs
    - services/portal/tests/brand_asset_etag_test.rs
    - services/portal/tests/brand_pack_export_test.rs
    - services/portal/tests/brand_pack_size_cap_test.rs
    - services/portal/tests/brand_asset_enum_cardinality_test.rs
    - services/portal/tests/brand_pack_rls_isolation_test.rs

  modified_files:
    - services/portal/src/lib.rs                                          # mount brand routes
    - services/portal/Cargo.toml                                          # +libvips, +instant-acme, +csscolorparser
    - services/email/src/templates/                                       # fallback-chain consumer for tenant overrides

  allowed_tools:
    - file_read: services/portal/**
    - file_read: services/email/src/templates/**
    - file_write: services/portal/{src,tests,migrations}/**
    - bash: cd services/portal && cargo test brand

  disallowed_tools:
    - skip WCAG contrast validation on save (per DEC-1003)
    - trust uploaded Content-Type header for asset type (per DEC-1019 — magic-bytes only)
    - allow non-tenant_admin to set CNAME (per DEC-1005)
    - enable email-override Tera with file_read or env access (per DEC-1016 sandbox)
    - serve brand assets with auth (per DEC-1009 — public-no-auth)
    - allow multi-CNAME per tenant at slice 1 (per DEC-1010)

effort_hours: 8
subtasks:
  - "0.5h: 0005..0008 migrations (brand_packs + active pointer + assets + cname_configs) + RLS"
  - "0.4h: brand/validate.rs — WCAG contrast (W3C formula) + magic-bytes + size checks"
  - "0.5h: brand/image_pipeline.rs — vips 3-variant resize"
  - "0.4h: brand/email_overrides.rs — Tera fragment loader + sandbox config"
  - "0.4h: brand/version.rs — create new version + activate + rollback"
  - "0.3h: brand/cdn.rs — URL + ETag builder"
  - "0.4h: cname/dns_verify.rs — TXT record poll + 5-min retry"
  - "0.7h: cname/acme.rs — instant-acme order + HTTP-01 challenge + cert persist"
  - "0.4h: handlers/brand_pack_routes.rs (CRUD)"
  - "0.3h: handlers/brand_asset_serve.rs (public)"
  - "0.4h: handlers/cname_admin.rs (tenant_admin gated)"
  - "0.4h: cli/cname_renewal_job.rs (daily ACME refresh)"
  - "0.4h: audit/brand_events.rs (8 builders)"
  - "1.5h: tests — 15 test files covering CRUD + version + rollback + WCAG + magic-bytes + image pipeline + DNS verify + ACME + email sandbox + ETag + export + size cap + RLS"
  - "0.4h: wire-up — lib.rs mounting + email-template fallback-chain consumption"

risk_if_skipped: "Without per-tenant brand pack, every PORTAL surface looks like CyberSkill — fatal for white-label B2B2C use cases (consulting firms wanting to surface 'their' branded portal to end-clients). Without DEC-1003's WCAG contrast enforcement, tenants ship inaccessible portals → AODA/ADA risk for tenants AND brand-quality dilution for CyberOS. Without DEC-1004's ACME-issued TLS, custom CNAMEs require operator-mediated cert issuance (~24h per cert at scale, ops blocker). Without DEC-1019's magic-bytes validation, an uploaded 'PNG' could be a payload that breaks downstream image processors. Without DEC-1011's auto-renewal, certs expire mid-quarter + tenant portals 500-error. Without DEC-1007's versioning, an oops on logo upload requires re-upload (no rollback safety). The 8h effort lands the white-label primitive that unlocks B2B2C + consultant-vertical commercial motion."
---

## §1 — Description (BCP-14 normative)

The PORTAL service **MUST** ship per-tenant brand pack (logo + colour palette + email overrides) and custom CNAME with ACME-issued TLS at `services/portal/src/brand/` + `services/portal/src/cname/`, with versioning + rollback, WCAG 2.1 AA contrast enforcement, magic-bytes asset validation, daily auto-renewal, and 8 memory audit kinds.

1. **MUST** define `portal_brand_packs` (versioned, immutable rows) at migration `0005`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, version INT NOT NULL, palette JSONB NOT NULL, email_overrides JSONB NOT NULL DEFAULT '{}'::jsonb, created_at TIMESTAMPTZ NOT NULL DEFAULT now(), created_by_subject_id UUID NOT NULL, asset_set_id BIGINT NOT NULL REFERENCES portal_brand_assets(asset_set_id))`. Per-tenant version is monotonic. REVOKE UPDATE, DELETE per task-audit skill rule 12 (rollback = pointer change, not row mutation).

2. **MUST** define `portal_brand_pack_active` at migration `0006`: `(tenant_id UUID PRIMARY KEY, active_pack_id BIGINT NOT NULL REFERENCES portal_brand_packs(id), activated_at TIMESTAMPTZ NOT NULL DEFAULT now(), activated_by_subject_id UUID NOT NULL)`. One-row-per-tenant activation pointer.

3. **MUST** define `portal_brand_assets` at migration `0007`: `(asset_set_id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, kind brand_asset_kind NOT NULL, mime_type TEXT NOT NULL CHECK (mime_type IN ('image/png','image/svg+xml')), content_kms_blob BYTEA NOT NULL, content_sha256 CHAR(64) NOT NULL, kms_key_id TEXT NOT NULL, content_length_bytes INT NOT NULL CHECK (content_length_bytes <= 1048576), created_at TIMESTAMPTZ NOT NULL DEFAULT now())`. Closed `brand_asset_kind` enum per DEC-1013.

4. **MUST** define the closed `brand_asset_kind` enum at migration `0007`: `('favicon','header_logo','splash_logo','email_logo')`. CI cardinality test asserts 4.

5. **MUST** define `portal_cname_configs` at migration `0008`: `(id BIGSERIAL PRIMARY KEY, tenant_id UUID NOT NULL, cname TEXT NOT NULL, dns_verify_token TEXT NOT NULL, dns_verified_at TIMESTAMPTZ, tls_cert_pem_kms_blob BYTEA, tls_cert_chain_pem_kms_blob BYTEA, tls_key_pem_kms_blob BYTEA, tls_kms_key_id TEXT, tls_issued_at TIMESTAMPTZ, tls_expires_at TIMESTAMPTZ, tls_renewal_failures INT NOT NULL DEFAULT 0, status TEXT NOT NULL CHECK (status IN ('pending_dns','dns_verified','active','revoked')) DEFAULT 'pending_dns', last_renewal_attempt_at TIMESTAMPTZ)`. Partial unique `(tenant_id) WHERE status IN ('pending_dns','dns_verified','active')` — one active CNAME per tenant per DEC-1010.

6. **MUST** enforce RLS with both USING and WITH CHECK on all 4 PORTAL brand tables (per DEC-1014 + task-audit skill rule 13). Policy: `tenant_id = current_setting('auth.tenant_id')::uuid`.

7. **MUST** expose `POST /v1/admin/tenants/{tenant_id}/brand-pack` for brand-pack creation. Caller has `tenant_admin` role. Body: `{ palette: {primary, secondary, accent, background, surface, error}, email_overrides?: {welcome?, magic_link?, password_reset?, invoice_receipt?}, assets: [{kind, base64_content}] }`. Handler:
    - Validates palette: 6 named slots, each `#RRGGBB` hex.
    - Runs WCAG 2.1 AA contrast check (per §1 #11) — reject if non-conformant.
    - For each asset: magic-bytes validate (PNG or SVG); size ≤ 1 MiB; generates 3 size variants via image_pipeline.
    - Validates email overrides (sandboxed Tera per §1 #14).
    - INSERTs new `portal_brand_assets` row(s) + new `portal_brand_packs` row at version=max(version)+1.
    - DOES NOT activate (separate step per §1 #8).
    - Emits `portal.brand_pack_created`.

8. **MUST** expose `POST /v1/admin/tenants/{tenant_id}/brand-pack/{pack_id}/activate`. Caller has `tenant_admin` role. Handler:
    - UPSERT `portal_brand_pack_active` with the new pack_id.
    - Invalidate CDN cache for `<tenant_slug>` (publish to NATS `cyberos.portal.brand_cdn.invalidate.<tenant_slug>`).
    - Emit `portal.brand_pack_activated`.

9. **MUST** expose `POST /v1/admin/tenants/{tenant_id}/brand-pack/rollback` per DEC-1007. Body: `{ target_pack_id }`. Handler re-points `portal_brand_pack_active.active_pack_id` to the named historic pack_id. Emits `portal.brand_pack_rolled_back`. Idempotent on `target_pack_id`.

10. **MUST** enforce per-tenant rate limit of 10 saves per day on `POST /brand-pack` per DEC-1018. Excess returns `429 + Retry-After: 86400` + emits `portal.brand_pack_validation_rejected` with reason='rate_limited'.

11. **MUST** enforce WCAG 2.1 AA contrast per DEC-1003 + W3C contrast formula:
    - `contrast_ratio(primary, background) ≥ 4.5`
    - `contrast_ratio(error, background) ≥ 4.5`
    - `contrast_ratio(primary, surface) ≥ 3.0` (text-on-card UX)
    Non-conformant submission returns `400 + { error: "wcag_contrast_violation", offending_pair: ["primary","background"], actual_ratio: 3.2, required_ratio: 4.5 }` + emits `portal.brand_pack_validation_rejected`.

12. **MUST** validate uploaded asset content-type via magic-bytes per DEC-1019. PNG header = `89 50 4E 47 0D 0A 1A 0A` (8 bytes); SVG header = `<?xml` OR `<svg`. Mismatched magic-bytes vs claimed mime_type → 400 + `invalid_asset_content`. Never trust the Content-Type request header.

13. **MUST** generate 3 size variants from each uploaded logo per DEC-1001 via libvips:
    - `favicon`: 32×32 px (from any uploaded logo).
    - `header_logo`: 200×60 px (preserve aspect; pad transparent).
    - `splash_logo`: 800×240 px (preserve aspect; pad transparent).
    Variants stored as separate `portal_brand_assets` rows with `kind` differentiation; canonical upload stored at upload-supplied size.

14. **MUST** apply Tera sandboxing for email overrides per DEC-1016. Allowed Tera tags: `{{ var }}`, `{% if %}`, `{% for %}`, `{% include %}` (restricted to a per-tenant include path). DISALLOWED: `{% set %}` with file/env reads, `{% raw %}` with HTML injection, any `tera_*` registered functions that access I/O. Validation: parse Tera template + walk AST + reject if any disallowed tag detected.

15. **MUST** expose CNAME setup at `POST /v1/admin/tenants/{tenant_id}/cname` per DEC-1004 + DEC-1005. Caller has `tenant_admin` role at tenant level. Body: `{ cname: "portal.acme.com" }`. Handler:
    - Generates a random 32-char `dns_verify_token`.
    - INSERTs `portal_cname_configs` row with `status='pending_dns'`.
    - Returns `201 + { cname, dns_verify_record: "_cyberos-portal-verify TXT \"<token>\"" }`.
    Tenant adds the TXT record at their DNS provider.

16. **MUST** expose `POST /v1/admin/tenants/{tenant_id}/cname/{id}/verify` per DEC-1004. Handler:
    - Resolves the TXT record at `_cyberos-portal-verify.<cname>` via DNS lookup.
    - If TXT value matches `dns_verify_token`: UPDATE status='dns_verified' + `dns_verified_at=now()` + invoke ACME issuance per §1 #17 + emit `portal.cname_dns_verified`.
    - On miss: returns `424 + { error: "dns_record_not_found_or_mismatched" }`; client retries (DNS propagation can take 5min-1h).

17. **MUST** issue ACME TLS cert via `instant-acme` crate per DEC-1004 + RFC 8555:
    - Use Let's Encrypt production endpoint (sandbox env uses staging).
    - HTTP-01 challenge served from `https://<cname>/.well-known/acme-challenge/<token>`.
    - On successful cert issuance: KMS-encrypt cert + key + chain into `portal_cname_configs.tls_*_kms_blob` columns + UPDATE status='active' + `tls_issued_at=now()` + `tls_expires_at=now()+90 days`.
    - Emit `portal.cname_tls_issued` sev-2.

18. **MUST** run daily ACME renewal job per DEC-1011 + DEC-1004. The `cname_renewal_job.rs` scheduled job:
    - Queries `portal_cname_configs WHERE status='active' AND tls_expires_at < now() + interval '30 days' AND last_renewal_attempt_at < now() - interval '4 hours'`.
    - For each: invoke ACME renewal (same flow as issuance but reusing the DNS-verified domain).
    - On success: UPDATE cert blob + `tls_expires_at` + reset `tls_renewal_failures=0` + emit `portal.cname_tls_renewed` sev-2.
    - On failure: increment `tls_renewal_failures` + emit `portal.cname_tls_renewal_failed` sev-2; after 3 consecutive failures → sev-1 alert; after 7 days at < 30d expiry → page on-call.

19. **MUST** serve brand assets publicly at `GET https://cdn.cyberos.world/brand/{tenant_slug}/{kind}.{ext}?v={sha16}` per DEC-1008 + DEC-1009. Handler:
    - Resolves tenant_slug → tenant_id.
    - Looks up active brand pack + asset by kind.
    - Returns asset binary with `Content-Type` from row + `ETag: "<sha16>"` + `Cache-Control: public, max-age=300` (5-min edge TTL per DEC-1008).
    - On `If-None-Match` match: 304.
    - Unknown slug or kind: 404.

20. **MUST** publish CDN cache-invalidation event on brand-pack activation per DEC-1008. NATS subject `cyberos.portal.brand_cdn.invalidate.<tenant_slug>`; downstream CloudFront/CDN edge consumer (slice 1 ops-managed) invalidates the cache. ETag in URL query (`?v=<sha16>`) means CDN cache hit ratio remains high after invalidation (only the changed URL invalidates).

21. **MUST** support brand-pack export per DEC-1012. `GET /v1/admin/tenants/{tenant_id}/brand-pack/{pack_id}/export` returns a deterministic .zip containing:
    - `pack.json` — canonical-JSON of the pack metadata (palette + email overrides + asset filenames).
    - `assets/<kind>.<ext>` files for each asset.
    Deterministic per task-audit skill rule 27-28: file order alphabetic; zip mtime = `2000-01-01T00:00:00Z`; mode 0o644.

22. **MUST** apply standard fallback chain for email overrides per DEC-1006. TASK-EMAIL-001's template loader tries (in order):
    1. `services/email/templates/_overrides/<tenant_slug>/<template>.tera` (per-tenant override mounted from `portal_brand_packs.email_overrides`).
    2. `services/email/templates/<template>.tera` (CyberSkill default).
    Missing override silently falls through to default; no error.

23. **MUST** emit 8 memory audit row kinds per DEC-1017 (task-audit skill rule 6 namespace):
    - `portal.brand_pack_created` (sev-2)
    - `portal.brand_pack_activated` (sev-2)
    - `portal.brand_pack_rolled_back` (sev-2)
    - `portal.cname_dns_verified` (sev-2)
    - `portal.cname_tls_issued` (sev-2)
    - `portal.cname_tls_renewed` (sev-3 — routine)
    - `portal.cname_tls_renewal_failed` (sev-2 → sev-1 after 3 consecutive)
    - `portal.brand_pack_validation_rejected` (sev-3 — informational; high volume during onboarding)

24. **MUST** thread W3C `traceparent` across upload → validate → image_pipeline → INSERT → activate → CDN invalidate (task-audit skill rule 22-24). Single trace_id per save operation.

25. **MUST NOT** persist plaintext TLS keys or plaintext assets — every blob in `portal_brand_assets.content_kms_blob` and `portal_cname_configs.tls_*_kms_blob` is KMS-encrypted at rest. Asset serve handler decrypts on-demand into a per-request buffer (no plaintext on disk).

26. **MUST NOT** allow >1 active CNAME per tenant at slice 1 per DEC-1010. Adding a second active CNAME returns `409 + cname_already_configured`.

27. **MUST** be idempotent on brand-pack activation by `pack_id` — re-activating the currently-active pack returns 200 OK (no-op).

---

## §2 — Why this design (rationale for humans)

**Why immutable versioned rows + activation pointer (§1 #1, DEC-1007)?** Brand-pack edits are commercial decisions — a CMO who wants to A/B test logos or roll back a bad change needs the prior version intact. Versioning + activation pointer is the standard pattern (Drupal/WordPress revisions, AWS launch templates). Mutating in-place forfeits rollback safety.

**Why WCAG 2.1 AA enforcement at save (§1 #11, DEC-1003)?** Inaccessible portals harm tenants' end-users + create AODA/ADA legal exposure for the tenants. Save-time enforcement (rather than warn-only) catches the problem before it reaches users; the explicit contrast-ratio feedback in the rejection lets the tenant pick conformant colours immediately.

**Why magic-bytes validation (§1 #12, DEC-1019)?** Content-Type headers are user-controlled — an attacker can claim `image/png` while uploading an executable. Magic-bytes (first 8 bytes for PNG, `<?xml`/`<svg` for SVG) are the actual file-format signature; trust them, not the header. Defends against downstream image-processor exploits (libvips CVE history is substantial).

**Why ACME-issued TLS rather than operator-mediated (§1 #17, DEC-1004)?** Operator-mediated cert issuance is ~24h-to-72h SLA at scale. For self-serve white-label, that latency kills the onboarding UX ("set up your portal in <1 hour"). ACME via Let's Encrypt is automatic, free, RFC-conformant, and renewable — the operational cost is one daily job vs. one ops ticket per cert.

**Why daily renewal vs continuous (§1 #18, DEC-1011)?** Let's Encrypt certs are 90 days; renewing at T-30 days gives a 30-day buffer for renewal failures. Daily polling at T-30 vs continuous monitoring means the renewal job is one batch query + per-cert workflow — operationally simple. 4-hour cooldown between attempts (per row) prevents tight retry loops if ACME-side has transient issues.

**Why public-no-auth CDN serve (§1 #19, DEC-1009)?** Brand assets are PUBLIC by definition (logos appear on the tenant's landing page; anyone can scrape them with curl). Requiring auth on a CDN endpoint adds latency + complicates browser caching + doesn't add security. The PII-scrubbing validation at save-time ensures no PII enters the assets, so public serve is safe.

**Why per-tenant Tera fragment sandboxing (§1 #14, DEC-1016)?** Email templates with full Tera power are a remote-code-execution vector — a tenant who can `{% set x = read_file("/etc/passwd") %}` reads the host. Whitelisting allowed tags + walking the AST is the standard mitigation. Allows useful customisation (variable interpolation, conditional sections) without the foot-gun.

**Why 5-min CDN TTL + ETag (§1 #19, DEC-1008)?** 5-min TTL means a brand-pack change propagates within 5 min worst-case (without manual cache invalidation). ETag means stale cached responses still serve correctly while the cache updates — eventually-consistent. The ?v=<sha16> URL query means each pack version has its own URL, so cache invalidation is automatic on activation (the URL changes; new fetches; no purge needed).

**Why tenant_admin only for CNAME (§1 #15, DEC-1005)?** DNS misconfig affects every user landing on `portal.<tenant>.com`; engagement_admin role's scope is one engagement, not the tenant-wide DNS. Mirrors TASK-PORTAL-003 DEC-882 pattern. Reduces blast-radius surface.

**Why 6-slot fixed palette vs free-form (§1 #11, DEC-1002)?** Fixed slots map to the design-system tokens used across the PORTAL UI components. Free-form palette would force every UI component to invent its own colour-picking logic. 6 slots is the standard tier (Material Design uses 5; Apple HIG uses 6; we picked 6 for parity with the most common design-system patterns).

**Why 1 MiB asset cap + 5 MiB total (§1 #3, DEC-1015)?** 1 MiB PNG at 800×240 splash size is generous; > 1 MiB usually indicates unoptimised export. 5 MiB total cap prevents storage exhaustion via 100s of large uploads. The image_pipeline produces 3 variants per upload, so the total stored ~3x cap.

---

## §3 — API contract

### 3.1 Postgres schema (key migrations)

```sql
-- 0005_portal_brand_packs.sql
CREATE TABLE portal_brand_packs (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  version INT NOT NULL,
  palette JSONB NOT NULL,
  email_overrides JSONB NOT NULL DEFAULT '{}'::jsonb,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  created_by_subject_id UUID NOT NULL,
  asset_set_id BIGINT NOT NULL,
  UNIQUE (tenant_id, version)
);
ALTER TABLE portal_brand_packs ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_brand_packs_rls ON portal_brand_packs
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_brand_packs FROM cyberos_app;

-- 0006_portal_brand_pack_active.sql
CREATE TABLE portal_brand_pack_active (
  tenant_id UUID PRIMARY KEY,
  active_pack_id BIGINT NOT NULL REFERENCES portal_brand_packs(id),
  activated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  activated_by_subject_id UUID NOT NULL
);
ALTER TABLE portal_brand_pack_active ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_brand_pack_active_rls ON portal_brand_pack_active
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE DELETE ON portal_brand_pack_active FROM cyberos_app;
GRANT UPDATE (active_pack_id, activated_at, activated_by_subject_id) ON portal_brand_pack_active TO cyberos_app;

-- 0007_portal_brand_assets.sql
CREATE TYPE brand_asset_kind AS ENUM ('favicon','header_logo','splash_logo','email_logo');
CREATE TABLE portal_brand_assets (
  asset_set_id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  kind brand_asset_kind NOT NULL,
  mime_type TEXT NOT NULL CHECK (mime_type IN ('image/png','image/svg+xml')),
  content_kms_blob BYTEA NOT NULL,
  content_sha256 CHAR(64) NOT NULL,
  kms_key_id TEXT NOT NULL,
  content_length_bytes INT NOT NULL CHECK (content_length_bytes <= 1048576),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
ALTER TABLE portal_brand_assets ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_brand_assets_rls ON portal_brand_assets
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_brand_assets FROM cyberos_app;

-- 0008_portal_cname_configs.sql
CREATE TABLE portal_cname_configs (
  id BIGSERIAL PRIMARY KEY,
  tenant_id UUID NOT NULL,
  cname TEXT NOT NULL,
  dns_verify_token TEXT NOT NULL,
  dns_verified_at TIMESTAMPTZ,
  tls_cert_pem_kms_blob BYTEA,
  tls_cert_chain_pem_kms_blob BYTEA,
  tls_key_pem_kms_blob BYTEA,
  tls_kms_key_id TEXT,
  tls_issued_at TIMESTAMPTZ,
  tls_expires_at TIMESTAMPTZ,
  tls_renewal_failures INT NOT NULL DEFAULT 0,
  last_renewal_attempt_at TIMESTAMPTZ,
  status TEXT NOT NULL DEFAULT 'pending_dns'
    CHECK (status IN ('pending_dns','dns_verified','active','revoked'))
);
CREATE UNIQUE INDEX uniq_cname_active_per_tenant ON portal_cname_configs(tenant_id)
  WHERE status IN ('pending_dns','dns_verified','active');
CREATE UNIQUE INDEX uniq_cname_global ON portal_cname_configs(cname)
  WHERE status != 'revoked';
ALTER TABLE portal_cname_configs ENABLE ROW LEVEL SECURITY;
CREATE POLICY portal_cname_configs_rls ON portal_cname_configs
  USING (tenant_id = current_setting('auth.tenant_id')::uuid)
  WITH CHECK (tenant_id = current_setting('auth.tenant_id')::uuid);
REVOKE UPDATE, DELETE ON portal_cname_configs FROM cyberos_app;
GRANT UPDATE (dns_verified_at, tls_cert_pem_kms_blob, tls_cert_chain_pem_kms_blob, tls_key_pem_kms_blob,
              tls_kms_key_id, tls_issued_at, tls_expires_at, tls_renewal_failures,
              last_renewal_attempt_at, status) ON portal_cname_configs TO cyberos_app;
```

### 3.2 Rust types

```rust
// services/portal/src/brand/mod.rs
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct Palette {
    pub primary:    String,  // "#RRGGBB"
    pub secondary:  String,
    pub accent:     String,
    pub background: String,
    pub surface:    String,
    pub error:      String,
}

#[derive(Copy, Clone, Debug, sqlx::Type)]
#[sqlx(type_name = "brand_asset_kind", rename_all = "snake_case")]
pub enum BrandAssetKind { Favicon, HeaderLogo, SplashLogo, EmailLogo }

#[derive(serde::Deserialize, Debug)]
pub struct BrandPackCreateReq {
    pub palette: Palette,
    pub email_overrides: Option<EmailOverrides>,
    pub assets: Vec<AssetUpload>,
}

#[derive(serde::Deserialize, Debug)]
pub struct EmailOverrides {
    pub welcome: Option<String>,
    pub magic_link: Option<String>,
    pub password_reset: Option<String>,
    pub invoice_receipt: Option<String>,
}
```

### 3.3 REST endpoints

```text
POST   /v1/admin/tenants/{tenant_id}/brand-pack                    (tenant_admin)
POST   /v1/admin/tenants/{tenant_id}/brand-pack/{pack_id}/activate (tenant_admin)
POST   /v1/admin/tenants/{tenant_id}/brand-pack/rollback            (tenant_admin)
GET    /v1/admin/tenants/{tenant_id}/brand-pack/{pack_id}/export    (tenant_admin)
POST   /v1/admin/tenants/{tenant_id}/cname                          (tenant_admin)
POST   /v1/admin/tenants/{tenant_id}/cname/{id}/verify              (tenant_admin)
GET    https://cdn.cyberos.world/brand/{tenant_slug}/{kind}.{ext}?v={sha16}  (public)
```

---

## §4 — Acceptance criteria

1. **Brand pack CRUD** — POST creates version 1; second POST creates version 2; both immutable.
2. **Activation pointer update** — POST activate sets `portal_brand_pack_active.active_pack_id`; subsequent GET serves the new pack.
3. **Rollback** — POST rollback to v1 re-points the active pointer; v2 still exists as historic.
4. **WCAG contrast enforcement** — palette with `primary=#888 background=#999` (contrast 1.1) → 400 + `wcag_contrast_violation` + offending_pair body.
5. **Magic-bytes validation** — asset uploaded as `image/png` but with `<html>` content → 400 + `invalid_asset_content`.
6. **Image pipeline 3 variants** — uploaded 800×600 PNG produces 32×32 favicon + 200×60 header + 800×240 splash rows in `portal_brand_assets`.
7. **CNAME DNS verify** — POST CNAME returns verify token; manual TXT record set; POST verify resolves token → status='dns_verified'.
8. **ACME issuance** — post-verify, ACME job runs HTTP-01 → cert issued + persisted KMS-encrypted + status='active'.
9. **Daily renewal** — cert with `tls_expires_at = now() + 25 days` triggered by renewal job → cert re-issued + `tls_expires_at = now() + 90 days`.
10. **Email override sandbox** — override template with `{% set x = read_file("/etc/passwd") %}` → 400 + `tera_disallowed_tag`.
11. **CDN asset ETag** — first GET returns 200 + ETag; second GET with If-None-Match → 304.
12. **CDN ETag invalidation on activation** — activate new pack → URL ?v= changes → fresh fetch.
13. **Export deterministic** — two exports of same pack produce byte-identical .zip.
14. **Size cap rejection** — 2 MiB PNG upload → 413 + `asset_too_large`.
15. **brand_asset_kind cardinality** — enum = exactly `{favicon, header_logo, splash_logo, email_logo}`.
16. **RLS isolation** — tenant A's session cannot read tenant B's brand pack (RLS returns 0 rows).
17. **CNAME global uniqueness** — second tenant claiming the same `cname` → 409 + `cname_taken`.
18. **Rate-limit 10/day** — 11th brand-pack save in 24h → 429.
19. **Renewal failure escalation** — 3 consecutive failures → sev-1 alert; row's `tls_renewal_failures=3`.
20. **8 memory audit kinds emitted** — full lifecycle (create + activate + rollback + cname_verify + tls_issued + tls_renewed + tls_renewal_failed + validation_rejected) covered.

---

## §5 — Verification

### 5.1 `brand_pack_create_test.rs`

```rust
#[tokio::test]
async fn brand_pack_create_versions_monotonic() {
    let ctx = TestContext::new().await;
    let v1 = ctx.create_brand_pack(default_palette(), valid_asset_png()).await.unwrap();
    let v2 = ctx.create_brand_pack(default_palette(), valid_asset_png()).await.unwrap();
    assert_eq!(v1.version, 1);
    assert_eq!(v2.version, 2);
}
```

### 5.2 `wcag_contrast_enforcement_test.rs`

```rust
#[tokio::test]
async fn low_contrast_palette_rejected() {
    let ctx = TestContext::new().await;
    let palette = Palette {
        primary: "#888888".into(), background: "#999999".into(),
        secondary: "#000".into(), accent: "#000".into(),
        surface: "#fff".into(), error: "#f00".into(),
    };
    let r = ctx.post_brand_pack(palette, valid_asset_png()).await;
    assert_eq!(r.status(), 400);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "wcag_contrast_violation");
    assert_eq!(body["offending_pair"][0], "primary");
    assert_eq!(body["offending_pair"][1], "background");
}
```

### 5.3 `magic_bytes_validation_test.rs`

```rust
#[tokio::test]
async fn html_uploaded_as_png_rejected() {
    let ctx = TestContext::new().await;
    let fake_png_body = b"<html><body>not a png</body></html>".to_vec();
    let r = ctx.upload_asset(BrandAssetKind::HeaderLogo, "image/png", fake_png_body).await;
    assert_eq!(r.status(), 400);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "invalid_asset_content");
}
```

### 5.4 `image_pipeline_variants_test.rs`

```rust
#[tokio::test]
async fn upload_generates_three_variants() {
    let ctx = TestContext::new().await;
    let png_800x600 = ctx.fixture_png(800, 600);
    let _ = ctx.upload_asset(BrandAssetKind::HeaderLogo, "image/png", png_800x600).await;

    let variants: Vec<(String, i32)> = sqlx::query_as(
        "SELECT kind::text, content_length_bytes FROM portal_brand_assets WHERE tenant_id=$1 ORDER BY kind"
    ).bind(ctx.tenant_id).fetch_all(&ctx.pool).await.unwrap();
    assert!(variants.iter().any(|(k,_)| k == "favicon"));
    assert!(variants.iter().any(|(k,_)| k == "header_logo"));
    assert!(variants.iter().any(|(k,_)| k == "splash_logo"));
}
```

### 5.5 `cname_acme_issuance_test.rs`

```rust
#[tokio::test]
async fn dns_verified_triggers_acme_issuance() {
    let ctx = TestContext::with_acme_sandbox().await;
    let cname_id = ctx.post_cname("portal-test.example.com").await;
    ctx.simulate_dns_txt_record_set(cname_id).await;
    ctx.post_cname_verify(cname_id).await.expect_status(200);
    ctx.wait_for_acme_complete(cname_id, Duration::from_secs(30)).await;

    let row: (String, Option<DateTime<Utc>>) = sqlx::query_as(
        "SELECT status, tls_issued_at FROM portal_cname_configs WHERE id=$1"
    ).bind(cname_id).fetch_one(&ctx.pool).await.unwrap();
    assert_eq!(row.0, "active");
    assert!(row.1.is_some());

    let audit = ctx.memory_rows().await;
    assert!(audit.iter().any(|r| r.kind == "portal.cname_dns_verified"));
    assert!(audit.iter().any(|r| r.kind == "portal.cname_tls_issued"));
}
```

### 5.6 `cname_acme_renewal_test.rs`

```rust
#[tokio::test]
async fn renewal_job_renews_cert_under_30d() {
    let ctx = TestContext::with_acme_sandbox().await;
    let cname_id = ctx.seed_cname_with_cert_expiring_in(Duration::from_days(25)).await;
    ctx.run_renewal_job().await;

    let row: (Option<DateTime<Utc>>, i32) = sqlx::query_as(
        "SELECT tls_expires_at, tls_renewal_failures FROM portal_cname_configs WHERE id=$1"
    ).bind(cname_id).fetch_one(&ctx.pool).await.unwrap();
    assert!(row.0.unwrap() > Utc::now() + Duration::days(85));
    assert_eq!(row.1, 0);
}
```

### 5.7 `email_override_sandbox_test.rs`

```rust
#[tokio::test]
async fn dangerous_tera_tag_rejected() {
    let ctx = TestContext::new().await;
    let bad_template = r#"{% set leaked = read_file("/etc/passwd") %}Hello {{ name }}"#;
    let r = ctx.post_brand_pack_with_overrides(EmailOverrides {
        welcome: Some(bad_template.into()), ..Default::default()
    }).await;
    assert_eq!(r.status(), 400);
    let body: serde_json::Value = r.json().await.unwrap();
    assert_eq!(body["error"], "tera_disallowed_tag");
}
```

### 5.8 `brand_asset_etag_test.rs`

```rust
#[tokio::test]
async fn cdn_asset_etag_304_on_match() {
    let ctx = TestContext::new().await;
    ctx.activate_brand_pack().await;
    let r1 = ctx.get_cdn_asset("acme", "header_logo", "png").send().await.unwrap();
    let etag = r1.headers()["etag"].to_str().unwrap().to_owned();
    let r2 = ctx.get_cdn_asset("acme", "header_logo", "png")
        .header("if-none-match", &etag).send().await.unwrap();
    assert_eq!(r2.status(), 304);
}
```

### 5.9 `brand_pack_export_test.rs`

```rust
#[tokio::test]
async fn export_is_deterministic() {
    let ctx = TestContext::new().await;
    let pack_id = ctx.create_brand_pack(default_palette(), valid_asset_png()).await.unwrap().id;
    let z1 = ctx.export_brand_pack(pack_id).await;
    let z2 = ctx.export_brand_pack(pack_id).await;
    assert_eq!(sha256(&z1), sha256(&z2));
}
```

### 5.10 `brand_pack_rls_isolation_test.rs`

```rust
#[tokio::test]
async fn tenant_a_cannot_read_tenant_b_brand_pack() {
    let ctx = TestContext::with_two_tenants().await;
    let pack_id = ctx.as_tenant("b").create_brand_pack(default_palette(), valid_asset_png()).await.unwrap().id;
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT id FROM portal_brand_packs WHERE id=$1")
        .bind(pack_id).fetch_all(ctx.pool_as("a")).await.unwrap_or_default();
    assert_eq!(rows.len(), 0);
}
```

---

## §6 — Implementation skeleton

### 6.1 WCAG contrast formula

```rust
// services/portal/src/brand/validate.rs
pub fn relative_luminance(hex: &str) -> f64 {
    let (r, g, b) = parse_hex(hex);
    let lin = |c: f64| if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) };
    0.2126 * lin(r as f64 / 255.0) + 0.7152 * lin(g as f64 / 255.0) + 0.0722 * lin(b as f64 / 255.0)
}

pub fn contrast_ratio(fg: &str, bg: &str) -> f64 {
    let l1 = relative_luminance(fg);
    let l2 = relative_luminance(bg);
    let (l, d) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
    (l + 0.05) / (d + 0.05)
}

pub fn validate_wcag_aa(palette: &Palette) -> Result<(), ValidationError> {
    let pairs = [
        ("primary", "background", 4.5),
        ("error", "background", 4.5),
        ("primary", "surface", 3.0),
    ];
    for (fg_name, bg_name, required) in pairs {
        let ratio = contrast_ratio(palette.get(fg_name), palette.get(bg_name));
        if ratio < required {
            return Err(ValidationError::WcagViolation {
                offending_pair: [fg_name, bg_name],
                actual_ratio: ratio,
                required_ratio: required,
            });
        }
    }
    Ok(())
}
```

### 6.2 Magic-bytes check

```rust
pub fn validate_magic_bytes(claimed_mime: &str, content: &[u8]) -> Result<(), ValidationError> {
    match claimed_mime {
        "image/png" => {
            if content.len() < 8 || &content[0..8] != b"\x89PNG\r\n\x1a\n" {
                return Err(ValidationError::InvalidAssetContent { claimed: claimed_mime.into() });
            }
        }
        "image/svg+xml" => {
            let head = std::str::from_utf8(&content[..content.len().min(256)]).unwrap_or("");
            if !head.trim_start().starts_with("<?xml") && !head.trim_start().starts_with("<svg") {
                return Err(ValidationError::InvalidAssetContent { claimed: claimed_mime.into() });
            }
        }
        _ => return Err(ValidationError::UnsupportedMimeType(claimed_mime.into())),
    }
    Ok(())
}
```

### 6.3 ACME issuance via instant-acme

```rust
pub async fn issue_cert(ctx: &AppCtx, cname: &str) -> Result<IssuedCert, AcmeError> {
    let order = ctx.acme_account.new_order(&[Identifier::Dns(cname.into())]).await?;
    let challenges = order.authorizations().await?;
    for auth in challenges {
        let challenge = auth.find_http_01().ok_or(AcmeError::NoHttp01Challenge)?;
        ctx.acme_challenge_server.serve(&challenge.token, &challenge.key_authorization()).await;
        challenge.ready().await?;
    }
    order.poll_ready(Duration::from_secs(120)).await?;
    let (cert_chain_pem, key_pem) = order.finalize().await?;
    Ok(IssuedCert {
        cert_pem: cert_chain_pem.first_cert_pem(),
        chain_pem: cert_chain_pem.intermediate_pem(),
        key_pem,
        expires_at: parse_x509_expiry(&cert_chain_pem)?,
    })
}
```

---

## §7 — Dependencies

**Upstream (depends_on):**
- **TASK-TEN-101** Self-serve signup — tenant exists before brand pack can be created.

**Cross-module (related_tasks):**
- **TASK-PORTAL-001** Scoped read-only views — consumes brand pack at render time.
- **TASK-PORTAL-003** External IdP — brand applied at IdP redirect login page.
- **TASK-PORTAL-005** Branded Genie — brand pack applied to embedded chat.
- **TASK-TEN-103** Residency provisioning — CDN edge selection per residency.
- **TASK-AUTH-101** RBAC — `tenant_admin` role gate.
- **TASK-EMAIL-001** Transactional email — consumes fallback chain for tenant overrides.
- **TASK-DOC-001** Documents — brand pack may apply to PDF export headers.
- **TASK-AI-003** memory audit-row bridge — 8 new kinds.
- **TASK-MEMORY-111** PII scrubbing — validation that no PII enters assets.

**Downstream (blocks):** None.

---

## §8 — Example payloads

### 8.1 `POST /brand-pack` request

```json
{
  "palette": {
    "primary":    "#1A73E8",
    "secondary":  "#34A853",
    "accent":     "#FBBC04",
    "background": "#FFFFFF",
    "surface":    "#F8F9FA",
    "error":      "#D93025"
  },
  "email_overrides": {
    "welcome": "Welcome to {{ tenant_name }}'s portal!\n\nClick {{ magic_link }} to sign in."
  },
  "assets": [
    { "kind": "header_logo", "mime_type": "image/png", "base64_content": "iVBORw0KGgo..." }
  ]
}
```

### 8.2 `portal.brand_pack_activated` memory row

```json
{
  "kind": "portal.brand_pack_activated",
  "severity": 2,
  "tenant_id": "8a2f...",
  "actor_id": "user.tenant_admin.789",
  "trace_id": "0af7651916cd43dd8448eb211c80319c",
  "occurred_at": "2026-05-17T09:14:32.847Z",
  "payload": {
    "pack_id": 42,
    "version": 3,
    "previous_pack_id": 41
  }
}
```

### 8.3 CNAME setup response (201)

```json
{
  "cname": "portal.acme.com",
  "dns_verify_record": "_cyberos-portal-verify TXT \"abc123def456...\"",
  "status": "pending_dns",
  "next_step": "Add the TXT record above to your DNS provider, then POST /cname/{id}/verify"
}
```

### 8.4 WCAG violation response (400)

```json
{
  "error": "wcag_contrast_violation",
  "offending_pair": ["primary", "background"],
  "actual_ratio": 1.13,
  "required_ratio": 4.5,
  "remediation_hint": "Try a darker primary or lighter background"
}
```

---

## §9 — Open questions

All resolved for slice 1. Deferred:

- **Deferred:** Multi-CNAME per tenant (region-specific landing pages) — slice 2.
- **Deferred:** Email template GUI editor (vs raw Tera) — slice 2.
- **Deferred:** Brand-pack inheritance (parent tenant → child engagement) — slice 2.
- **Deferred:** Custom favicon variants beyond 32×32 (16×16, Apple touch, etc.) — slice 2.
- **Deferred:** SVG runtime sanitization (defang scripts inside SVG) — slice 2; for now SVG accepted as-is with magic-bytes only.
- **Deferred:** Brand-pack analytics (which version most-served) — slice 3.
- **Deferred:** Dark-mode palette variants — slice 2 (6 light-mode slots only at slice 1).
- **Deferred:** Custom CSS overrides beyond palette — slice 3 (security review needed).

---

## §10 — Failure modes inventory

| Failure | Detection | Outcome | Recovery |
|---|---|---|---|
| WCAG contrast violation | save-time check | 400 + `wcag_contrast_violation` + remediation hint + `portal.brand_pack_validation_rejected` | Tenant picks conformant colours |
| Magic-bytes mismatch (asset spoof) | upload validation | 400 + `invalid_asset_content` | Tenant re-uploads valid asset |
| Asset > 1 MiB | size check | 413 + `asset_too_large` | Tenant optimises asset |
| Total brand pack > 5 MiB | aggregate check | 400 + `brand_pack_total_too_large` | Tenant reduces asset sizes |
| Rate limit 10/day hit | per-tenant counter | 429 + `Retry-After: 86400` | Tenant waits 24h or contacts support |
| Tera template with disallowed tag | AST walk | 400 + `tera_disallowed_tag` + tag name | Tenant removes the tag |
| DNS verify TXT not found | DNS resolver returns NXDOMAIN | 424 + `dns_record_not_found` | Tenant adds TXT + waits propagation; client retries |
| ACME HTTP-01 challenge fails (port 80 closed) | ACME poll returns invalid | sev-2 alert `portal.cname_tls_issuance_failed`; cname status remains `dns_verified` | Tenant opens port 80; manual re-trigger |
| ACME rate-limited (LE rate limits) | ACME 429 | Backoff 1h + retry; sev-2 if persistent | Inherent — wait + retry |
| Daily renewal job fails | row's `tls_renewal_failures` increments | After 3 consecutive → sev-1; > 7 days at < 30d expiry → page on-call | On-call investigates; manual renewal via CLI |
| Cert expired (renewal job didn't run) | renewal job's stale-job detector | sev-1 immediate | On-call runs manual ACME issuance |
| Tenant CNAME conflicts with existing | unique index `uniq_cname_global` | 409 + `cname_taken` | Tenant picks alternate CNAME |
| KMS unavailable when decrypting asset | KMS timeout | 503 + sev-2 `portal.kms_unavailable`; CDN serves stale-cached if available | AWS KMS recovers; cache rebuilds |
| CDN cache invalidation NATS publish fails | NATS error | Cache staleness up to 5min (TTL); sev-3 informational | Inherent — TTL eventual consistency |
| Image pipeline (vips) crash | vips returns error | 500 + sev-2 alert; asset NOT persisted | Operator investigates; libvips upgrade may be needed |
| Rollback to non-existent pack_id | FK constraint fails | 404 + `target_pack_not_found` | Tenant chooses valid pack from list |
| Multiple concurrent activations (race) | UPSERT on portal_brand_pack_active is atomic | Last writer wins; previous activation memory row still exists for audit | Inherent — last-write-wins acceptable |
| Tenant uploads SVG with embedded `<script>` | slice 1 accepts as-is | XSS risk if SVG served inline | Slice 2 adds SVG sanitization; slice 1 mitigates by serving SVG with `Content-Disposition: attachment` for now (downloaded, not inline) |
| Email override `{% include %}` outside per-tenant path | sandbox path constraint | 400 + `tera_include_path_violation` | Tenant uses relative path |
| Renewal job runs on revoked cert | status check | Skip + log informational; no audit row | Inherent — guard at job entry |
| Brand pack export size > 100 MiB | size check before zip stream | 413 + `export_too_large` | Tenant exports per-asset instead |

---

## §11 — Implementation notes

**§11.1** The W3C relative luminance formula (§6.1) is the standard WCAG 2.x algorithm; do not invent variations. The `lin` curve handles the sRGB gamma correction.

**§11.2** `csscolorparser` crate handles hex parsing + future RGB/HSL formats; slice 1 accepts only `#RRGGBB` hex.

**§11.3** `libvips` binding (`libvips-rs`) is a Rust wrapper around the C library; deployment requires `libvips-dev` package. Production image processing is faster + safer than alternatives (ImageMagick has historical CVE volume).

**§11.4** ACME via `instant-acme` crate is the recommended Rust ACME client. Account key persisted in `portal_cname_configs.acme_account_key_kms_blob` at deployment time (one global account; per-cname order).

**§11.5** HTTP-01 challenge server runs as a sub-handler at `/.well-known/acme-challenge/{token}` exposing the keyAuthorization for the active order. Multi-tenant: dispatch by `Host` header to the correct order's response.

**§11.6** Tera sandbox uses a custom `tera::Tera` instance with `register_function` calls limited to a whitelist; `register_filter` not exposed; `register_tester` not exposed. AST walk uses tera's `Context` introspection.

**§11.7** CDN edge: CloudFront in front of our origin; cache key includes `?v=<sha16>` query param so different versions never collide. 5-min TTL at edge.

**§11.8** ETag format: SHA-256-truncated 16 hex chars of the canonical asset bytes; matches TASK-MCP-005's PRM ETag pattern.

**§11.9** Brand pack JSON export uses canonical-JSON (sorted keys) for deterministic byte equality across runs (task-audit skill rule 27).

**§11.10** ZIP archive determinism: `ZIP_DEFLATED level 6 + fixed mtime 2000-01-01T00:00:00Z + mode 0o644 + sorted entries` (consistent with AGENTS.md §10 portability pattern).

**§11.11** The `created_by_subject_id` + `activated_by_subject_id` columns enable per-actor brand-change auditing (which CMO/admin made the change).

**§11.12** SVG sanitization deferred to slice 2: slice 1 serves SVG with `Content-Disposition: attachment` to prevent inline rendering + XSS. Tenants who want inline SVG can use PNG (which can't carry script).

**§11.13** ACME account renewal at year-7 (LE accounts expire in 10 years; renew at 7). Out-of-scope at slice 1 (next renewal is 2033).

**§11.14** The renewal job is idempotent on `(cname_id, run_date)` — running twice the same day is a no-op after first success.

**§11.15** Per-tenant rate limit (10 saves/day) uses Redis sliding-window like TASK-TEN-101 §1 #5; same infrastructure.

**§11.16** Email-override fallback chain is TASK-EMAIL-001 internal mechanism; this FR populates the override files via `portal_brand_packs.email_overrides` JSONB → on-disk Tera fragment.

**§11.17** CNAME global uniqueness (`uniq_cname_global`) prevents two tenants from claiming the same custom domain — first-come-first-served; second tenant gets `cname_taken` until first revokes.

**§11.18** Activation timestamp + actor_id retention: never delete activation history; rollback creates a new activation row pointing to the historic pack.

**§11.19** Brand pack JSON schema versioning: slice 1 implicit v1; if v2 changes shape, the loader uses `schema_version` field (added in slice 2).

**§11.20** Per-asset `content_sha256` enables cache-hit at the asset table level — if a tenant uploads the same logo twice (re-paste), the SHA dedup avoids re-storage (slice 2 enhancement; slice 1 stores duplicates).

---

*End of TASK-PORTAL-002 spec.*
