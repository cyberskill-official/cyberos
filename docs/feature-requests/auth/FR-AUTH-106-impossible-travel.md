---
id: FR-AUTH-106
title: "Impossible-travel detection + adaptive MFA challenge"
module: AUTH
priority: SHOULD
status: implementing
accepted_at: 2026-05-16
accepted_by: Stephen Cheng
verify: T
phase: P2
milestone: P2 · auth-hardening
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: null
memory_chain_hash: null
related_frs: [FR-AUTH-002, FR-AUTH-005, FR-AUTH-102, FR-AUTH-101, FR-MEMORY-111]
depends_on: [FR-AUTH-002, FR-AUTH-102]
blocks: []

source_pages:
  - website/docs/modules/auth.html#impossible-travel
source_decisions:
  - DEC-740 2026-05-16 — Use Haversine great-circle distance between IP-geolocated coordinates as travel metric
  - DEC-741 2026-05-16 — Threshold = distance / time-delta exceeds 900 km/h (commercial airliner cruise speed)
  - DEC-742 2026-05-16 — Per-tenant `impossible_travel_speed_kmh` override [200, 5000], default 900
  - DEC-743 2026-05-16 — On detection, require fresh MFA (FR-AUTH-102 challenge) — do NOT block login outright
  - DEC-744 2026-05-16 — Closed 4-value `travel_decision` enum (allowed, challenged, allowed_after_challenge, blocked)
  - DEC-745 2026-05-16 — IP geo lookup uses MaxMind GeoIP2 City DB (locally bundled) — no third-party network call per login
  - DEC-746 2026-05-16 — Per-tenant `impossible_travel_action` enum (challenge | block | warn_only), default `challenge`
  - DEC-747 2026-05-16 — Sliding window = last 24h of successful logins; older entries ignored
  - DEC-748 2026-05-16 — Cache per-subject last-login table in Postgres with prior_login(latitude, longitude, occurred_at, ip_country, ip_asn)
  - DEC-749 2026-05-16 — Same-country same-ASN within 24h bypass (carrier IP roaming false-positive guard)
  - DEC-750 2026-05-16 — VPN / Tor exit-node detection via MaxMind anonymous-IP DB — VPN flagged separately, not auto-blocked
  - DEC-751 2026-05-16 — MaxMind DB refresh weekly via CI; stale DB > 30 days emits sev-2 memory audit
  - DEC-752 2026-05-16 — Login itself is never blocked on impossible-travel — the auth handler returns `challenge_required` and the client re-submits with MFA token
  - DEC-753 2026-05-16 — Adaptive challenge is sticky for 30min — re-challenging within 30min from same subject + same IP is suppressed (sev-3 audit)
  - DEC-754 2026-05-16 — 6 memory audit kinds + each carries scrubbed reason via FR-MEMORY-111
  - DEC-755 2026-05-16 — IP allowlist per-tenant bypasses impossible-travel for office IPs (CIDR list, validated at policy save)

build_envelope:
  language: rust 1.81
  service: cyberos/services/auth/
  new_files:
    - services/auth/src/travel/mod.rs
    - services/auth/src/travel/geoip.rs
    - services/auth/src/travel/detector.rs
    - services/auth/src/travel/policy.rs
    - services/auth/src/travel/asn_bypass.rs
    - services/auth/src/travel/anonymous_ip.rs
    - services/auth/migrations/0016_login_history_geo.sql
    - services/auth/migrations/0017_travel_audit.sql
    - services/auth/tests/travel_normal_test.rs
    - services/auth/tests/travel_impossible_test.rs
    - services/auth/tests/travel_vpn_test.rs
    - services/auth/tests/travel_allowlist_test.rs
    - services/auth/tests/travel_policy_test.rs
  modified_files:
    - services/auth/src/handlers/login.rs (invoke travel::evaluate after successful credential check)
    - services/auth/src/handlers/mfa_challenge.rs (mark challenge as travel-driven for audit linkage)
  allowed_tools:
    - file_read: services/auth/**
    - file_write: services/auth/{src,tests,migrations}/**
    - bash: cargo test -p cyberos-auth
    - file_read: vendor/maxmind/GeoLite2-City.mmdb (read-only)
  disallowed_tools:
    - send IP address to any external geolocation API
    - bypass travel::evaluate for any login code path
    - block login outright on travel detection (must use challenge, not block, by default)

effort_hours: 8
sub_tasks:
  - "0.5h: GeoIP2 wrapper (maxminddb crate)"
  - "1.0h: Haversine distance + speed math + travel decision"
  - "1.0h: per-subject prior-login table + 24h sliding-window query"
  - "1.0h: ASN bypass + VPN detection wiring"
  - "1.0h: per-tenant policy loader (speed + action + allowlist CIDR)"
  - "1.0h: login handler integration + MFA challenge linkage"
  - "0.5h: MaxMind DB refresh staleness check"
  - "2.0h: integration tests (NYC → Tokyo within 30min + carrier-roaming false-positive + VPN flag + office allowlist)"
risk_if_skipped: "Without impossible-travel detection, credential-stuffing attackers can sign in from any geography with no friction. Combined with FR-AUTH-107 (HIBP) which catches the 'known-bad password' vector, this catches the 'stolen credential, different geography' vector — both required for defense-in-depth credential security."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** evaluate every successful credential verification against the subject's recent login geography and require an adaptive MFA challenge when the apparent travel speed since the prior login exceeds the per-tenant impossible-travel threshold.

1. **MUST** invoke `travel::evaluate(subject_id, ip, occurred_at)` after successful credential verification (password + Argon2 match passes) but BEFORE issuing the AUTH session JWT. The evaluator returns one of: `Allowed`, `ChallengeRequired`, `Blocked` (per per-tenant action policy in DEC-746).

2. **MUST** geolocate the source IP using the locally-bundled MaxMind GeoLite2 City database (DEC-745). The lookup returns `(latitude, longitude, country_iso2, asn, is_anonymous_proxy)`. No external geolocation API call MUST be made — every login's IP must be resolvable from the local DB.

3. **MUST** compute the great-circle distance between the current login's geo coordinates and the most-recent prior login's coordinates using the Haversine formula on the WGS-84 sphere. The Haversine formula is sufficient (accuracy ≤ 0.5%); we don't need Vincenty's full ellipsoid math.

4. **MUST** compute the apparent travel speed as `distance_km / hours_since_prior_login`. Speeds with `hours_since_prior_login < 0.01` (< 36 seconds) are treated as the same-session — no travel evaluation, the result is `Allowed`.

5. **MUST** flag the login as impossible-travel when the apparent speed exceeds `tenants.impossible_travel_speed_kmh` (default 900 per DEC-741, range [200, 5000] per DEC-742).

6. **MUST** apply the per-tenant `impossible_travel_action` policy (DEC-746):
   - `challenge`: return `200 OK { result: "challenge_required", challenge_id, methods: [...] }` (mfa challenge linkage); the session JWT is NOT issued until challenge-completion endpoint succeeds (FR-AUTH-102 challenge flow).
   - `block`: return `403 FORBIDDEN { error: "impossible_travel_blocked", prior_country, current_country, speed_kmh, threshold_kmh }`. The session JWT is NOT issued; the subject must contact ops.
   - `warn_only`: issue the session JWT but emit a sev-2 memory audit row `auth.travel_warn_only`. No user friction.

7. **MUST** bypass the impossible-travel check when the same-subject's prior login was within 24h AND same ISO country AND same ASN (DEC-749). This guards against carrier-IP roaming false positives (e.g., a Vietnamese subject's mobile session switches from Viettel cell tower at HCMC to one near the Cambodian border — different geo by 100km, same country same ASN, well under 24h, real human movement).

8. **MUST** bypass the impossible-travel check when the current IP matches one of the per-tenant `impossible_travel_ip_allowlist_cidrs` entries (DEC-755). The list is a JSON array of CIDR strings (e.g., `["203.0.113.0/24", "198.51.100.42/32"]`), validated at policy save (each entry MUST parse as `IpNet`; total list ≤ 64 entries). Bypassed logins emit a sev-3 audit `auth.travel_allowlist_bypass`.

9. **MUST** define a closed 4-value `travel_decision` Postgres enum (`allowed`, `challenged`, `allowed_after_challenge`, `blocked`). CI cardinality test asserts exactly 4 (DEC-744).

10. **MUST** define a closed 3-value `impossible_travel_action` Postgres enum (`challenge`, `block`, `warn_only`). CI cardinality test asserts exactly 3 (DEC-746).

11. **MUST** persist every login (challenged + allowed + blocked) to the `login_history_geo` append-only table per FR-AUTH-103's existing login_history pattern, extended with `(latitude, longitude, country_iso2, asn, is_anonymous_proxy, travel_decision, speed_kmh_vs_prior)`. The table is REVOKE UPDATE, DELETE FROM cyberos_app + privileged auth_writer role.

12. **MUST** maintain a 24-hour sliding window for prior-login lookup (DEC-747). The query: `SELECT latitude, longitude, occurred_at, country_iso2, asn FROM login_history_geo WHERE subject_id = $1 AND travel_decision IN ('allowed', 'allowed_after_challenge') AND occurred_at > now() - INTERVAL '24 hours' ORDER BY occurred_at DESC LIMIT 1`. Older entries are not considered (a login 25h ago from Paris doesn't trigger when the next login is from New York 24h+1min later — but new-subject + sparse-login users are normal and should not be flagged).

13. **MUST** detect anonymous IP networks (VPN, Tor exit, proxy) via the MaxMind anonymous-IP database (DEC-750). When `is_anonymous_proxy = true`, the evaluator emits a sev-2 memory audit row `auth.travel_anonymous_ip` regardless of speed. By default, anonymous-IP logins ARE allowed (we don't auto-block — some users legitimately use VPNs). The per-tenant policy `block_anonymous_ip` (boolean, default false) can flip the default to block.

14. **MUST** validate the MaxMind DB freshness at process startup. If `mmdb.metadata().build_timestamp` is more than 30 days old, emit a sev-2 memory audit row `auth.travel_geoip_stale` (DEC-751). The service still starts and uses the stale DB — the staleness audit is the operational signal, not a hard block.

15. **MUST** route the adaptive challenge through FR-AUTH-102 (the existing MFA challenge issuer). The travel-evaluator creates a challenge with `kind = "impossible_travel"`, returns the challenge_id to the client. The client submits MFA proof via `POST /v1/auth/mfa/challenge/{id}/verify` per FR-AUTH-102. On verify-success, the AUTH session JWT is finally issued and a `login_history_geo` row is recorded with `travel_decision = 'allowed_after_challenge'`.

16. **MUST** suppress repeat challenges within 30 minutes from the same subject + same IP (DEC-753). This avoids friction for a user who completes an impossible-travel challenge from a coffee-shop wifi then immediately re-authenticates after a session timeout. The 30-min window emits a sev-3 audit `auth.travel_challenge_suppressed_repeat`.

17. **MUST** restrict policy mutation (`impossible_travel_speed_kmh`, `impossible_travel_action`, `block_anonymous_ip`, `impossible_travel_ip_allowlist_cidrs`) to the `security_admin` role per FR-AUTH-101 + a sev-2 memory audit row with reason ≥10 chars.

18. **MUST** validate per-tenant `impossible_travel_speed_kmh` at policy write time. Range [200, 5000]. Out-of-range returns `400 BAD_REQUEST` with `{error: "speed_threshold_out_of_range", min: 200, max: 5000}`.

19. **MUST** validate per-tenant `impossible_travel_ip_allowlist_cidrs` at policy write time. Each CIDR parses via Rust `ipnetwork::IpNet`; invalid entries return `400`. List length ≤ 64. Per-entry max-prefix-length: IPv4 /0 forbidden, /8 forbidden (too broad); IPv6 /0 forbidden, /16 forbidden. Tightest acceptable prefix is /9 for IPv4 and /17 for IPv6.

20. **MUST** scrub all reason-bearing audit text (policy reason, block error message extensions) via FR-MEMORY-111 before memory chain emission. Geo coordinates + country + ASN are operational metadata, not PII; they pass through.

21. **MUST** emit 6 closed memory audit kinds (DEC-754):
    - `auth.travel_allowed` (sev-3, every login under threshold)
    - `auth.travel_challenged` (sev-2, every login over threshold that triggered challenge)
    - `auth.travel_allowed_after_challenge` (sev-2, challenge-completion success)
    - `auth.travel_blocked` (sev-2, action=block tenant policy)
    - `auth.travel_anonymous_ip` (sev-2, VPN/Tor/proxy)
    - `auth.travel_geoip_stale` (sev-2, MaxMind DB > 30d old at startup)
    - `auth.travel_policy_changed` (sev-2, per security_admin mutation)
    - `auth.travel_allowlist_bypass` (sev-3, allowlisted IP bypass)
    - `auth.travel_challenge_suppressed_repeat` (sev-3, 30-min window suppression)
    - `auth.travel_warn_only` (sev-2, action=warn_only would-have-triggered)

22. **MUST** treat IPv4 and IPv6 uniformly — both are geolocated via the same MaxMind DB. NAT64 / dual-stack edge cases that yield different geo for v4 vs v6 from the same client are accepted as-is; the evaluator uses whichever IP the request arrived on.

23. **MUST** support a `dry_run` mode on the policy update endpoint: `dry_run=true` returns the proposed effective config + a list of last-7-days logins that would have been challenged/blocked under the proposed policy. No DB mutation, no audit row emitted.

24. **MUST** maintain a per-subject `recent_login_geo_cache` in-process LRU (Arc<Mutex<lru::LruCache<SubjectId, (lat, lon, occurred_at, country, asn)>>>>) sized to 50000 entries. On evaluator call, check the cache first; cache hit short-circuits the Postgres SELECT. Cache miss falls back to the Postgres query in clause #12. The cache is per-AUTH-process and is invalidated on every successful login (write-through).

25. **MUST** treat any IP that fails geolocation (private network, reserved space, unallocated v6) as `travel_decision = 'allowed'` with a sev-3 audit row `auth.travel_geoip_unresolvable`. This avoids false positives for development environments + reserved corporate networks.

26. **MUST** emit the prior-login geo coordinates in the memory audit chain row (not in the user-facing API response). The chain row carries `{subject_id, current_lat, current_lon, current_country, current_asn, prior_lat, prior_lon, prior_country, prior_asn, hours_delta, distance_km, speed_kmh, threshold_kmh, decision}`. The user-facing response on `block` includes only `{current_country, prior_country, speed_kmh, threshold_kmh}` — not the exact coordinates (the user knows where they are; the operator gets the precise geo in the chain).

27. **MUST** define a stale-cache invalidation pathway: when a security_admin updates the per-tenant policy, the in-process cache for that tenant's subjects is NOT flushed (the cache holds last-login geo, which is not policy-dependent). Policy changes take effect on the next login per subject. This is intentional: re-evaluating historical decisions under new policy is out of scope (it would require replaying every login row, an expensive operation we accept as future work).

---

## §2 — Rationale (informative — preserve all 22 paragraphs)

**§2.1  Why Haversine and not Vincenty.** Haversine on a sphere is accurate to ~0.5% for distances < 10000 km (most logins are within this range). Vincenty's iterative ellipsoid method is accurate to ~0.5 mm but adds CPU cycles for nothing — we're comparing against an 900 km/h threshold; sub-percent accuracy in distance contributes < 1 km/h to the speed calculation, far below the threshold's noise floor.

**§2.2  Why 900 km/h default threshold.** DEC-741. Commercial airliners cruise at 800-900 km/h. Setting the threshold below cruise speed would false-positive on every long-haul flight. Setting it above means we miss in-flight credential abuse (impossible at our scale). 900 km/h is the right number for the kinds of users we expect (knowledge workers commuting locally or flying intercontinental).

**§2.3  Why challenge, not block, by default.** DEC-743 + DEC-752 + clause #6 (`challenge` is default action). Blocking outright punishes a legitimate user who took a red-eye flight to a conference and tries to log in upon landing. The challenge path requires fresh MFA — defeating credential-stuffing attackers (who don't have the second factor) without breaking the legitimate user. The `block` action exists for high-security tenants who explicitly opt in.

**§2.4  Why local MaxMind DB and not a third-party API.** DEC-745. Third-party geo APIs (ipinfo, ipstack, MaxMind's hosted) add a network round-trip per login + a third-party data-flow + a cost. Local DB is a one-time-per-week refresh, free, and the lookup is a 1-microsecond mmap. The accuracy difference vs hosted is negligible.

**§2.5  Why 24-hour sliding window.** DEC-747. Beyond 24h the prior login is too stale to draw conclusions — most users have multiple logins per day, so the most-recent-within-24h is the right comparison point. For users who log in once a week (some admins) the absence of a within-24h prior login means the current login can't be "impossible" relative to a stale comparison; treat as a first-login of the session.

**§2.6  Why ASN bypass for carrier roaming.** DEC-749 + clause #7. Mobile carriers assign IPs dynamically across geographic cell towers. A single subject's mobile session can traverse 200km in 30 minutes (e.g., driving from HCMC to Phan Thiet). The ASN-and-country-match within 24h heuristic suppresses these false positives without weakening the cross-country defense.

**§2.7  Why VPN flagged not blocked by default.** DEC-750 + clause #13. Legitimate users use VPNs (corporate, privacy-conscious, expats accessing home-country services). Auto-blocking would punish them. Flagging at sev-2 lets operators investigate suspicious VPN+travel combinations without breaking normal users. Per-tenant `block_anonymous_ip` lets paranoid tenants opt in.

**§2.8  Why 30-day MaxMind staleness threshold.** DEC-751 + clause #14. MaxMind publishes weekly. 4-week (30-day) gap means we missed 3+ updates — likely a CI failure. The sev-2 audit at startup is the operational signal; we don't auto-block startup because a stale DB still works (geo data doesn't shift wildly week-over-week).

**§2.9  Why CIDR allowlist per tenant.** DEC-755 + clause #8. Office IPs (corporate egress NAT) are the legitimate "known safe" geos. A subject logging in from the office + then from their hotel in the same day should not be challenged. The allowlist lets the tenant configure this without a backdoor.

**§2.10  Why CIDR list capped at 64 entries.** Clause #19. Office IPs are few (CIDR aggregation handles most corp networks). A 64-entry cap prevents operators from accidentally allowlisting entire countries (e.g., /8 of US). The /9-minimum prefix is the defense.

**§2.11  Why challenge suppression at 30min.** DEC-753 + clause #16. A coffee-shop user who completes MFA for impossible-travel then session-timeouts 25min later should not be re-challenged on the next login from the same shop. 30min is the user's tolerance for "I just did this 5 minutes ago, why again". Beyond 30min, the challenge re-arms (defense-in-depth against extended sessions on hostile networks).

**§2.12  Why dry-run on policy update.** Clause #23. Security_admins want to see "how many of my recent logins would have been challenged if I tightened the threshold to 500 km/h?" before flipping the policy live. Dry-run returns the count + a sample list. No DB mutation; no audit row.

**§2.13  Why per-subject in-process cache.** Clause #24. The Postgres SELECT for prior login is a few ms — but multiplied by every login across many AUTH processes, it adds up. The 50000-entry LRU caches the hot subjects; on cache miss, the SELECT runs and the result is cached. Cache invalidation on every login (write-through) keeps it correct.

**§2.14  Why we don't store full IP history.** The `login_history_geo` carries one row per login with coordinates — that's the historical record. We don't store every IP indefinitely as a separate table because (a) coordinates + country + ASN are sufficient for forensics, (b) raw IPs are mildly PII-sensitive and we want to scrub-on-export. The geo row IS the forensic record.

**§2.15  Why we redact exact coordinates from the user-facing block response.** Clause #26. A user who sees their exact prior coordinates in an error message might feel surveilled. The chain row (operator-visible) carries the precise geo; the user response shows only country + speed. This is the same pattern as the FR-AUTH-107 audit: operator sees count, user sees vague reject.

**§2.16  Why the MaxMind DB is vendored, not pulled at runtime.** Pulling at runtime means an unreachable MaxMind CDN bricks new AUTH process starts. Vendoring (committing the .mmdb file to a release artifact, refreshed weekly via CI) makes startup deterministic.

**§2.17  Why we don't auto-challenge on country-change without speed-based reasoning.** A user living in San Francisco who logs in from London 24h later has traveled by airliner — legitimate. A user in San Francisco who logs in from London 30 minutes later did NOT travel — that's the impossible-travel signal. Country-change alone false-positives too often (border-region users, dual-citizenship workers); speed-based is the right metric.

**§2.18  Why first-login of a new subject is always allowed.** A new subject has no prior login in the 24h sliding window. The evaluator returns `Allowed` and the audit row marks it as a first-login. This is the correct semantic: we can't reason about travel from no data.

**§2.19  Why we run the evaluator AFTER credential check.** Clause #1. The credential check is fast (Argon2 = 100ms). The travel evaluator is faster (mmap + cache lookup = sub-ms). Running travel first would burn travel-eval cycles on every credential-stuffing attempt; running travel after credentials means only successful credentials get the geo evaluation. This is cheap by ordering.

**§2.20  Why per-tenant action and not global.** DEC-746 + clause #6. Tenant security postures vary: a healthcare customer wants `block` for compliance; a startup wants `warn_only` for low friction. Global action would force one shape; per-tenant respects the customer's risk tolerance.

**§2.21  Why we don't expose travel decisions via REST to tenants.** Operators see chain audit rows; the tenant's subjects see only the challenge prompt or block error. We don't expose a "your last 100 login geos" API to the subject — that's an attack surface (account takeover sees where the legitimate user was, useful for spear-phishing). The tenant admin's audit-export (FR-DOC-009 future) is the right surface for operator review.

**§2.22  Why the evaluator is in-process and not a separate microservice.** A separate service would add a network hop on the hot login path. The evaluator's footprint (MaxMind DB + cache + Postgres SELECT) is small enough to colocate with AUTH. We revisit if AUTH becomes CPU-bound on travel evaluation.

---

## §3 — API & schema

### §3.1 — Migration 0016: login_history_geo

```sql
-- services/auth/migrations/0016_login_history_geo.sql

CREATE TYPE travel_decision AS ENUM ('allowed', 'challenged', 'allowed_after_challenge', 'blocked');
CREATE TYPE impossible_travel_action AS ENUM ('challenge', 'block', 'warn_only');

CREATE TABLE login_history_geo (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subject_id          UUID NOT NULL REFERENCES subjects(id),
    tenant_id           UUID NOT NULL REFERENCES tenants(id),
    occurred_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    ip                  INET NOT NULL,
    latitude            DOUBLE PRECISION,
    longitude           DOUBLE PRECISION,
    country_iso2        CHAR(2),
    asn                 BIGINT,
    is_anonymous_proxy  BOOLEAN NOT NULL DEFAULT false,
    travel_decision     travel_decision NOT NULL,
    speed_kmh_vs_prior  DOUBLE PRECISION,  -- nullable when no prior login
    challenge_id        UUID REFERENCES mfa_challenges(id),
    memory_chain_hash    CHAR(64) NOT NULL CHECK (memory_chain_hash ~ '^[0-9a-f]{64}$')
);

CREATE INDEX login_geo_subject_time ON login_history_geo (subject_id, occurred_at DESC);
CREATE INDEX login_geo_tenant_time ON login_history_geo (tenant_id, occurred_at DESC);
CREATE INDEX login_geo_decision ON login_history_geo (travel_decision) WHERE travel_decision != 'allowed';

REVOKE UPDATE, DELETE ON login_history_geo FROM cyberos_app;
GRANT INSERT, SELECT ON login_history_geo TO auth_writer;
GRANT SELECT ON login_history_geo TO security_admin;

ALTER TABLE login_history_geo ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON login_history_geo
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);
```

### §3.2 — Migration 0017: per-tenant policy columns

```sql
-- services/auth/migrations/0017_travel_audit.sql

ALTER TABLE tenants
    ADD COLUMN IF NOT EXISTS impossible_travel_speed_kmh INTEGER NOT NULL DEFAULT 900
        CHECK (impossible_travel_speed_kmh BETWEEN 200 AND 5000),
    ADD COLUMN IF NOT EXISTS impossible_travel_action impossible_travel_action NOT NULL DEFAULT 'challenge',
    ADD COLUMN IF NOT EXISTS block_anonymous_ip BOOLEAN NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS impossible_travel_ip_allowlist_cidrs JSONB NOT NULL DEFAULT '[]'::jsonb;

-- Audit table for policy changes (separate from login_history_geo)
CREATE TABLE travel_policy_audit (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL REFERENCES tenants(id),
    actor_id        UUID NOT NULL REFERENCES subjects(id),
    occurred_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    old_config      JSONB NOT NULL,
    new_config      JSONB NOT NULL,
    reason          TEXT NOT NULL CHECK (length(reason) >= 10 AND length(reason) <= 1000),
    memory_chain_hash CHAR(64) NOT NULL
);

REVOKE UPDATE, DELETE ON travel_policy_audit FROM cyberos_app;
GRANT INSERT, SELECT ON travel_policy_audit TO auth_writer;
GRANT SELECT ON travel_policy_audit TO security_admin;

ALTER TABLE travel_policy_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON travel_policy_audit
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);
```

### §3.3 — Detector with Haversine + policy enforcement

```rust
// services/auth/src/travel/detector.rs

use maxminddb::geoip2;
use std::net::IpAddr;

pub struct GeoLookup {
    pub latitude: f64,
    pub longitude: f64,
    pub country_iso2: String,
    pub asn: u64,
    pub is_anonymous_proxy: bool,
}

pub enum TravelResult {
    Allowed,
    AllowlistBypass,
    AsnRoamBypass,
    Challenge { challenge_id: Uuid, prior: GeoLookup, current: GeoLookup, speed_kmh: f64 },
    Block { prior: GeoLookup, current: GeoLookup, speed_kmh: f64 },
    WarnOnly { speed_kmh: f64 },
    FirstLogin,
    Unresolvable,
}

pub async fn evaluate(
    pool: &PgPool,
    geoip: &GeoIp,
    subject_id: Uuid,
    tenant_id: Uuid,
    ip: IpAddr,
    occurred_at: DateTime<Utc>,
) -> Result<TravelResult, EvalError> {
    // Policy
    let policy: TravelPolicy = sqlx::query_as!(
        TravelPolicy,
        r#"SELECT impossible_travel_speed_kmh AS speed_kmh,
                  impossible_travel_action AS "action: ImpossibleTravelAction",
                  block_anonymous_ip,
                  impossible_travel_ip_allowlist_cidrs AS allowlist
           FROM tenants WHERE id = $1"#,
        tenant_id
    ).fetch_one(pool).await?;

    // Allowlist bypass
    if cidr_match(&policy.allowlist, ip) {
        return Ok(TravelResult::AllowlistBypass);
    }

    // Geo-lookup
    let current = match geoip.lookup(ip) {
        Ok(c) => c,
        Err(_) => return Ok(TravelResult::Unresolvable),
    };

    // Anonymous proxy
    if current.is_anonymous_proxy && policy.block_anonymous_ip {
        return Ok(TravelResult::Block { prior: current.clone(), current, speed_kmh: 0.0 });
    }

    // Prior login lookup (cache → DB)
    let prior = match prior_login_within_24h(pool, subject_id).await? {
        Some(p) => p,
        None => return Ok(TravelResult::FirstLogin),
    };

    // ASN + country bypass
    if prior.country_iso2 == current.country_iso2 && prior.asn == current.asn {
        return Ok(TravelResult::AsnRoamBypass);
    }

    // Haversine speed
    let distance_km = haversine_km(prior.latitude, prior.longitude, current.latitude, current.longitude);
    let hours = (occurred_at - prior.occurred_at).num_milliseconds() as f64 / 3_600_000.0;
    if hours < 0.01 {
        return Ok(TravelResult::Allowed);
    }
    let speed_kmh = distance_km / hours;

    if speed_kmh < policy.speed_kmh as f64 {
        return Ok(TravelResult::Allowed);
    }

    // Threshold exceeded — apply action
    match policy.action {
        ImpossibleTravelAction::Challenge => {
            let challenge_id = mfa::issue_challenge(pool, subject_id, "impossible_travel").await?;
            Ok(TravelResult::Challenge { challenge_id, prior, current, speed_kmh })
        }
        ImpossibleTravelAction::Block => Ok(TravelResult::Block { prior, current, speed_kmh }),
        ImpossibleTravelAction::WarnOnly => Ok(TravelResult::WarnOnly { speed_kmh }),
    }
}

fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let r = 6371.0;
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1.to_radians().cos() * lat2.to_radians().cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    r * c
}
```

---

## §4 — Acceptance criteria

1. `travel_decision` enum exactly 4 values.
2. `impossible_travel_action` enum exactly 3 values.
3. Login from NYC then login from Tokyo 30min later (with default 900 km/h) triggers `challenged`.
4. Same login pair under `block` action returns `403 impossible_travel_blocked` with prior_country + current_country + speed_kmh.
5. Same login pair under `warn_only` action issues session JWT + emits `auth.travel_warn_only` sev-2 audit.
6. NYC → Tokyo within 30min under `challenge` action returns `200 { challenge_required: true, challenge_id }`.
7. Subsequent `POST /v1/auth/mfa/challenge/{id}/verify` with valid MFA proof issues session JWT + records `allowed_after_challenge`.
8. Same-country same-ASN within 24h bypasses with sev-3 audit `auth.travel_asn_roam_bypass` (not explicitly listed but covered by suppression).
9. First-login (no prior within 24h) returns `Allowed` + audit row records `allowed` with `speed_kmh_vs_prior = NULL`.
10. Allowlisted CIDR bypass emits sev-3 `auth.travel_allowlist_bypass`.
11. Anonymous-IP login emits sev-2 `auth.travel_anonymous_ip`; default does NOT block; tenant policy `block_anonymous_ip = true` blocks.
12. MaxMind DB > 30 days old at startup emits sev-2 `auth.travel_geoip_stale`.
13. Repeat challenge within 30min from same subject + same IP suppressed; sev-3 `auth.travel_challenge_suppressed_repeat`.
14. Policy mutation requires `security_admin` role + reason ≥ 10 chars + sev-2 `auth.travel_policy_changed`.
15. Speed threshold < 200 or > 5000 returns `400 speed_threshold_out_of_range`.
16. CIDR list with /0 prefix or /8 IPv4 returns `400`.
17. CIDR list > 64 entries returns `400`.
18. CIDR list with invalid CIDR string returns `400`.
19. Login never blocked outright on impossible-travel under default `challenge` action.
20. User-facing block response includes only country + speed; never lat/lon.
21. memory audit chain row includes full geo coordinates + ASN + country for both prior and current.
22. Postgres `login_history_geo` REVOKE UPDATE/DELETE confirmed at `\dp`.
23. RLS prevents cross-tenant SELECT on `login_history_geo`.
24. Per-subject in-process cache reduces Postgres SELECTs (verified by query log count = 1 for 10 consecutive logins of same subject).
25. Cache invalidation on every successful login (write-through) — next login sees fresh prior data.
26. IPv4 and IPv6 geolocate via same DB; mixed-stack subjects handled uniformly.
27. Unresolvable IP (e.g., 10.0.0.1 private) returns `Allowed` + sev-3 `auth.travel_geoip_unresolvable`.
28. Dry-run policy update returns affected-login count for last 7 days; no DB write; no audit row.
29. CI cardinality tests assert 4 + 3 enum sizes.
30. Travel evaluator runs AFTER credential check (instrumentation hook).
31. Geo lookup latency p99 < 1ms (in-memory MaxMind mmap).
32. NYC → Tokyo within 30min reports speed ≈ 21500 km/h (well above 900 km/h threshold).

---

## §5 — Verification (CI tests)

- `cardinality_test_decision` — 4.
- `cardinality_test_action` — 3.
- `nyc_to_tokyo_30min_test` — challenge under default action.
- `nyc_to_tokyo_block_test` — block action returns 403.
- `nyc_to_tokyo_warn_test` — warn_only issues JWT + sev-2 audit.
- `challenge_then_verify_test` — challenge → MFA verify → JWT issued + `allowed_after_challenge`.
- `asn_roam_bypass_test` — same country + ASN within 24h, 300km distance → bypass.
- `first_login_test` — no prior login → `Allowed` + `speed_kmh_vs_prior = NULL`.
- `allowlist_bypass_test` — office CIDR → bypass + sev-3 audit.
- `anonymous_ip_test` — Tor exit IP → sev-2; default allow; with `block_anonymous_ip` → block.
- `geoip_stale_test` — mock DB metadata 31 days old → sev-2 startup audit.
- `challenge_suppression_test` — challenge completed at T; new login at T+25min from same IP → suppressed.
- `challenge_re-arm_test` — challenge completed at T; new login at T+31min → re-challenge fires.
- `policy_mutation_acl_test` — non-security_admin → 403.
- `policy_threshold_range_test` — 199 and 5001 → 400.
- `policy_cidr_validation_test` — invalid CIDR strings → 400; /0 + /8 IPv4 → 400.
- `policy_cidr_count_test` — 65-entry list → 400.
- `block_response_redaction_test` — assert response body has no `latitude`/`longitude` fields.
- `chain_audit_full_geo_test` — assert memory chain row carries prior + current lat/lon.
- `rls_isolation_test` — two tenants, cross-query empty.
- `cache_invalidation_test` — login → cache hit on next; insert direct → next login sees the inserted row.
- `mixed_stack_test` — IPv4 then IPv6 from same client; same evaluator behavior.
- `unresolvable_ip_test` — 10.0.0.1 → allowed + sev-3.
- `dry_run_test` — policy update with dry_run=true → preview + no DB write.
- `evaluator_after_credential_test` — instrumentation asserts credential check fires before travel::evaluate.
- `latency_test` — 1000 random logins; p99 evaluator latency < 5ms (1ms geo + DB SELECT).
- `nyc_tokyo_speed_math_test` — speed_kmh > 20000.

---

## §6 — File skeleton

```
services/auth/
├── src/
│   ├── travel/
│   │   ├── mod.rs          # pub evaluate()
│   │   ├── geoip.rs        # MaxMind mmap wrapper
│   │   ├── detector.rs     # Haversine + policy enforcement (§3.3)
│   │   ├── policy.rs       # per-tenant policy loader
│   │   ├── asn_bypass.rs   # same-country+ASN check
│   │   ├── anonymous_ip.rs # VPN/Tor flagging
│   │   ├── cache.rs        # per-subject LRU
│   │   ├── audit.rs        # memory audit emission for 10 kinds
│   │   └── error.rs
│   └── handlers/
│       ├── login.rs              # MODIFIED: invoke travel::evaluate after credential check
│       └── travel_policy.rs      # NEW: PUT /v1/admin/tenants/{id}/travel-policy
├── migrations/
│   ├── 0016_login_history_geo.sql
│   └── 0017_travel_audit.sql
└── tests/
    ├── travel_normal_test.rs
    ├── travel_impossible_test.rs
    ├── travel_vpn_test.rs
    ├── travel_allowlist_test.rs
    └── travel_policy_test.rs
vendor/maxmind/
└── GeoLite2-City.mmdb   # refreshed weekly by CI
```

---

## §7 — Dependencies & blast-radius

**Depends on**: FR-AUTH-002 (subject_create — login flow), FR-AUTH-102 (MFA challenge — adaptive challenge issuer).

**Blocks**: nothing (terminal leaf in P2 auth hardening; complements FR-AUTH-107).

**Blast radius if broken**:
- **False positive**: legitimate user challenged unnecessarily; bounded by allowlist + ASN bypass.
- **False negative**: impossible-travel signal missed; defense-in-depth via FR-AUTH-102 MFA still active.
- **GeoIP DB stale**: gradual quality degradation; sev-2 audit at startup.
- **MaxMind unreachable for CI refresh**: weekly refresh fails silently; vendored DB stays as-is until manual intervention.

---

## §8 — Payload examples

### §8.1 — Normal login (clean travel)

```
POST /v1/auth/login
{ "email": "...", "password": "..." }

200 OK
{ "session_token": "...", "expires_at": "..." }
```

### §8.2 — Impossible-travel triggers challenge

```
POST /v1/auth/login
{ "email": "...", "password": "..." }

200 OK
{
  "result": "challenge_required",
  "reason": "impossible_travel",
  "challenge_id": "chl_xyz",
  "methods": ["totp", "webauthn"]
}
```

### §8.3 — Block action

```
POST /v1/auth/login
{ "email": "...", "password": "..." }

403 Forbidden
{
  "error": "impossible_travel_blocked",
  "prior_country": "US",
  "current_country": "JP",
  "speed_kmh": 21450.7,
  "threshold_kmh": 900,
  "contact": "security@your-tenant.com"
}
```

### §8.4 — Policy mutation

```
PUT /v1/admin/tenants/{id}/travel-policy
Authorization: Bearer <security_admin>
{
  "impossible_travel_speed_kmh": 500,
  "impossible_travel_action": "challenge",
  "block_anonymous_ip": false,
  "impossible_travel_ip_allowlist_cidrs": ["203.0.113.0/24"],
  "reason": "Tightening threshold per audit finding"
}

200 OK
{ "applied": true }
```

### §8.5 — Dry-run policy preview

```
PUT /v1/admin/tenants/{id}/travel-policy?dry_run=true
{ "impossible_travel_speed_kmh": 500 }

200 OK
{
  "would_apply": true,
  "affected_logins_last_7d": 12,
  "sample": [
    { "subject_id": "sub_abc", "speed_kmh": 612.3, "would_decision": "challenged" }
  ]
}
```

---

## §9 — Open questions

- **OQ-1** (closed by DEC-741): 900 km/h threshold.
- **OQ-2** (closed by DEC-743): challenge not block by default.
- **OQ-3** (closed by DEC-745): local MaxMind DB.
- **OQ-4** (open): should we publish the user's "last 5 login geos" via subject-facing API for transparency? Currently no (§2.21). Revisit if customer demands.
- **OQ-5** (open): should we offer machine-learning-based anomaly detection beyond speed (e.g., device fingerprint, time-of-day, behavioral)? Not in scope; speed is the cheap operationally-understandable metric.

---

## §10 — Failure modes (32 rows)

| # | Failure | Detection | Sev | Handler |
|---|---------|-----------|-----|---------|
| 1 | GeoIP DB file missing at startup | mmap error | 1 | AUTH refuses to start |
| 2 | GeoIP DB corrupted | maxminddb parse error | 1 | AUTH refuses to start |
| 3 | GeoIP DB stale > 30d | metadata check | 2 | sev-2 startup audit; service starts |
| 4 | IP geolocation lookup returns no city record | maxminddb None | 3 | Allowed + sev-3 unresolvable audit |
| 5 | Private/reserved IP space | RFC 1918 check | 3 | Allowed + sev-3 unresolvable audit |
| 6 | Postgres SELECT for prior login fails | sqlx error | 1 | Treat as first-login + sev-2 audit |
| 7 | Per-subject cache miss + Postgres slow | latency p99 alarm | 2 | Bounded by 60s SLA at sev-2 |
| 8 | Policy load fails | sqlx error | 1 | Default policy used + sev-2 audit |
| 9 | CIDR allowlist malformed in DB | parse error at load | 1 | Treat as empty list + sev-2 audit |
| 10 | MFA challenge issuance fails (FR-AUTH-102 down) | upstream error | 1 | Login rejected with 503 + sev-1 |
| 11 | Cross-tenant RLS misconfiguration | rls_isolation_test fails | 1 | CI blocks deploy |
| 12 | speed_kmh = NaN from same-instant logins | hours < 0.01 guard | 3 | Allowed (clause #4) |
| 13 | Anonymous proxy DB missing | mmdb open error | 2 | Treat all IPs as non-anonymous + sev-2 |
| 14 | Policy mutation by non-admin | RBAC | 2 | 403 + sev-2 |
| 15 | Policy threshold out of [200, 5000] | CHECK constraint | 3 | 400 |
| 16 | Policy action enum invalid | DB enum | 3 | 400 |
| 17 | CIDR list > 64 entries | length validator | 3 | 400 |
| 18 | CIDR /0 or /8 IPv4 | prefix validator | 3 | 400 |
| 19 | CIDR invalid string | ipnetwork parse | 3 | 400 |
| 20 | Reason for mutation < 10 chars | CHECK | 3 | 400 |
| 21 | login_history_geo INSERT permission denied | GRANT misconfig | 1 | Login rejected; sev-1 audit |
| 22 | memory_chain_hash regex fails | CHECK constraint | 1 | Login rejected; sev-1 audit |
| 23 | Cache invalidation race (two concurrent logins) | LRU put serialization | 3 | Later-write-wins; bounded |
| 24 | Travel evaluator invoked before credential check | ordering test | 2 | CI blocks |
| 25 | Block response includes lat/lon | redaction test | 1 | CI blocks |
| 26 | Chain audit row missing geo fields | schema validation | 1 | CI blocks |
| 27 | Stale MaxMind DB in nightly CI | weekly refresh job | 2 | Sev-2 ops alarm |
| 28 | Repeat-challenge suppression bug (always suppresses) | challenge_re-arm_test | 2 | CI blocks |
| 29 | Dry-run mutates state | dry_run_test | 2 | CI blocks |
| 30 | Same IP both v4 and v6 yields different geo | mixed_stack_test | 3 | Use whichever arrived; sev-3 if speed-flag |
| 31 | Subject deleted between cache write and read | foreign key | 3 | Bounded by FR-AUTH-105 deletion ordering |
| 32 | Tenant terminated (FR-TEN-104) — old login geos still query | RLS + tenant state | 3 | Read-only after termination; allowed |

---

## §11 — Implementation notes

**§11.1** The MaxMind GeoLite2-City.mmdb is committed as a release artifact, not as source code. Weekly CI refreshes it via `wget https://download.maxmind.com/...` (license key in CI secret).

**§11.2** The anonymous-IP DB is a separate `.mmdb` file (`GeoIP2-Anonymous-IP.mmdb`); both are mmap'd at startup.

**§11.3** Haversine constants: Earth radius = 6371 km. The formula uses `sin/cos/asin` which are fast enough that we don't memoize.

**§11.4** The per-subject cache is keyed on subject_id only (not tenant_id); the prior_login query is already tenant-scoped via RLS.

**§11.5** Cache invalidation is write-through: every successful login writes to both the in-process cache and Postgres in one async block. The Postgres write happens first (durable); the cache write happens after (on success).

**§11.6** The challenge_id returned in the `challenge_required` response is the same challenge type as FR-AUTH-102 user-initiated MFA — the verify endpoint is shared.

**§11.7** The MaxMind staleness check at startup runs once at process boot. Subsequent runtime staleness (e.g., a 60-day-running process) is detected on the next restart.

**§11.8** The 30-min challenge suppression is implemented via a small in-process map (subject_id + ip → last_challenge_at, ttl 30min). Cross-process visibility isn't needed because each AUTH process handles independent login streams.

**§11.9** The `block_anonymous_ip` policy is a simple boolean; we don't allow per-VPN-vendor whitelisting (that complicates the model and the tenant probably doesn't know their VPN vendor's IP ranges anyway — they'd use the CIDR allowlist).

**§11.10** ASN-and-country bypass is a single SQL row comparison (no aggregation). Latency is negligible.

**§11.11** The `is_anonymous_proxy` field combines four MaxMind categories (anonymous_vpn, public_proxy, tor_exit_node, hosting_provider). For our purposes any of these triggers the same audit kind; finer-grained handling is out of scope.

**§11.12** Tests use a fixture MaxMind DB with known IPs for repeatable assertions (no live MaxMind calls in CI).

**§11.13** The dry_run handler runs the same evaluator over the last 7 days of `login_history_geo` rows for the tenant. For tenants with > 1M logins/week, the query is paginated; the sample in the response is the first 20 hits.

**§11.14** The `evaluator_after_credential_test` uses a code-instrumented hook (or a tracing span) to verify the ordering. If the test fails, the login handler was refactored incorrectly.

**§11.15** The `cache_invalidation_test` directly INSERTs a prior login row, then performs a login from a different geo, and asserts the evaluator sees the inserted row. This catches stale-cache bugs.

**§11.16** The `unresolvable_ip_test` covers RFC 1918 (10.0.0.0/8), 127.0.0.0/8, fe80::/10 link-local — common in development environments. The audit row makes the unresolvable case visible without breaking dev.

**§11.17** Tracing spans wrap each `travel::evaluate` call with the subject_id + decision. The span shows up in FR-OBS-001 OTEL traces, enabling forensic walks across login + travel + MFA spans.

**§11.18** The login handler emits the `auth.login_attempted` audit row before `travel::evaluate` and the `auth.travel_*` audit row after — both link to the same `trace_id`.

**§11.19** The CIDR allowlist is checked with the `ipnetwork::IpNetwork::contains(ip)` method; we don't roll our own bit manipulation. The list is loaded once per request from `tenants` (which is itself cached for 60s).

**§11.20** The MaxMind reader is `maxminddb::Reader<Vec<u8>>` (owned buffer) for thread-safety. The buffer is loaded once at startup and shared via `Arc`.

**§11.21** First-login of a new subject is identified by the absence of prior rows in the 24h window. The `FirstLogin` audit kind logs sev-3 — operationally interesting but expected.

**§11.22** Per-tenant policy cache (60s TTL) is invalidated via pg_notify on policy mutation, mirroring the FR-AUTH-107 pattern.

---

*End of FR-AUTH-106 spec.*
