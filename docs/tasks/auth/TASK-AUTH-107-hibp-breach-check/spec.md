---
id: TASK-AUTH-107
title: "HIBP password breach check (k-anonymity) on signup + rotation"
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
eu_ai_act_risk_class: not_ai
# UNREVIEWED: auto-set by the 2026-07-14 schema migration; a human MUST confirm before this task leaves draft
ai_authorship: generated_then_reviewed
client_visible: false
type: feature
created_at: 2026-05-16T00:00:00+07:00
department: engineering
author: "@stephencheng"
template: task@1
module: AUTH
priority: p1
status: done
accepted_at: 2026-05-16
accepted_by: Stephen Cheng
verify: T
phase: P2
milestone: P2 · auth-hardening
slice: 1
owner: Stephen Cheng
created: 2026-05-16
shipped: 2026-05-23
memory_chain_hash: null
related_tasks: [TASK-AUTH-002, TASK-AUTH-005, TASK-AUTH-101, TASK-MEMORY-111]
depends_on: [TASK-AUTH-002, TASK-AUTH-005]
blocks: []

source_pages:
  - website/docs/modules/auth.html#breach-check
source_decisions:
  - DEC-720 2026-05-16 — Use Have I Been Pwned k-anonymity API (5-hex SHA-1 prefix, no full hash leaves the box)
  - DEC-721 2026-05-16 — Block when count ≥ THRESHOLD (default 5); below threshold passes with sev-3 audit
  - DEC-722 2026-05-16 — Per-tenant `hibp_block_threshold` override [1, 10000], default 5
  - DEC-723 2026-05-16 — HIBP-only when network egress permitted; air-gapped tenants get local k-anon dump fallback
  - DEC-724 2026-05-16 — Cache last-seen prefix → count mapping for 1 hour (in-process LRU, max 10000 entries)
  - DEC-725 2026-05-16 — Fail-closed when HIBP unreachable AND no cache hit AND tenant policy says fail-closed; fail-open otherwise (default fail-open with sev-2 audit)
  - DEC-726 2026-05-16 — Per-tenant `hibp_unreachable_policy` enum (fail_closed | fail_open), default fail_open
  - DEC-727 2026-05-16 — Audit emission carries only prefix + count + decision; never the full password hash or plaintext (DEC-722 PII contract)
  - DEC-728 2026-05-16 — Apply at signup, rotation, password set on existing account; NOT at login (login doesn't have plaintext after Argon2)
  - DEC-729 2026-05-16 — Rotation-time check returns 422 with explicit error_code so UI can prompt "this password appeared in breaches — choose another"
  - DEC-730 2026-05-16 — Network timeout 2000ms; one retry on connect-fail (not on 4xx/5xx); after retry treat as unreachable

language: rust 1.81
service: cyberos/services/auth/
new_files:
  - services/auth/src/hibp/mod.rs
  - services/auth/src/hibp/client.rs
  - services/auth/src/hibp/cache.rs
  - services/auth/src/hibp/local_dump.rs
  - services/auth/src/hibp/policy.rs
  - services/auth/migrations/0015_hibp_audit.sql
  - services/auth/tests/geoip_test.rs
  - services/auth/tests/rls_isolation_test.rs
  - services/auth/tests/rbac_catalogue_test.rs
  - services/auth/tests/rbac_catalogue_test.rs
modified_files:
  - services/auth/src/handlers/signup.rs (call hibp::check before Argon2)
  - services/auth/src/handlers/password_rotate.rs (call hibp::check before Argon2)
  - services/auth/src/handlers/admin_set_password.rs (call hibp::check before Argon2)
allowed_tools:
  - file_read: services/auth/**
  - file_write: services/auth/{src,tests,migrations}/**
  - bash: cargo test -p cyberos-auth
  - net: GET https://api.pwnedpasswords.com/range/{prefix} (only this endpoint)
disallowed_tools:
  - send full SHA-1 or plaintext password to any network endpoint
  - log password plaintext or full hash at any level
  - bypass hibp::check for any new password-setting code path

effort_hours: 4
subtasks:
  - "0.5h: HIBP client with 5-prefix range query + 2s timeout + 1 retry"
  - "0.5h: in-process LRU cache (prefix → count map, 1h TTL, 10000 entries)"
  - "0.5h: local k-anon dump fallback loader (env-toggled)"
  - "0.5h: per-tenant policy loader (threshold + unreachable_policy)"
  - "0.5h: 3 handler integrations (signup + rotation + admin-set)"
  - "0.5h: hibp_audit migration with prefix + count + decision + sev"
  - "1.0h: integration tests (mock HTTP server + tenant fixture)"
risk_if_skipped: "Without breach-check, attackers can credential-stuff CyberOS with known-bad passwords from public dumps (Collection #1 et al). One compromised tenant subject = full RLS scope of that subject. Stripe + 2FA still required for full kill chain, but the upstream control is missing."
---

## §1 — Description (BCP-14 normative)

The AUTH service **MUST** check every new or rotated password against the Have I Been Pwned (HIBP) breach corpus via the k-anonymity range API (no plaintext or full hash leaves the local network) and reject passwords whose breach-count meets or exceeds the configured per-tenant threshold.

1. **MUST** apply the breach check at three code paths: signup (TASK-AUTH-002 subject_create with password), password rotation (`POST /v1/auth/password/rotate`), and admin-set-password (`POST /v1/admin/subjects/{id}/password`). The check **MUST NOT** apply at login (DEC-728); at login the plaintext is verified against Argon2, then discarded, and we never re-check the plaintext against HIBP. Rationale: login is the high-traffic hot path; breach checking belongs at password-set time, not at every authentication.

2. **MUST** compute SHA-1 of the plaintext password in-process, take the first 5 uppercase hex characters as the **prefix**, and send only the prefix in an HTTP GET request to `https://api.pwnedpasswords.com/range/{prefix}` (DEC-720). The full SHA-1, the plaintext, and any other identifying value **MUST NOT** leave the AUTH service process.

3. **MUST** parse the HIBP response (each line is `<suffix35>:<count>`), find the line whose suffix matches the remaining 35 hex chars of the local SHA-1, and extract the `count` integer. If no line matches, count = 0.

4. **MUST** reject the password if `count >= tenants.hibp_block_threshold` (default 5 per DEC-721, range [1, 10000] per DEC-722). The rejection returns:
- At signup: `422 UNPROCESSABLE_ENTITY` with `{error: "password_breached", count, threshold}`.
- At rotation: `422 UNPROCESSABLE_ENTITY` with the same body (DEC-729 enables UI prompt).
- At admin-set: `422 UNPROCESSABLE_ENTITY` with the same body; the admin gets to choose a different password for the subject.

5. **MUST** emit one memory audit row per breach-check call. The row carries `{prefix, count, decision: "allowed" | "rejected" | "unreachable", tenant_id, subject_id, code_path: "signup" | "rotation" | "admin_set"}`. The row never carries the full SHA-1 or the plaintext (DEC-727). The sev tier is sev-3 for `allowed`, sev-2 for `rejected`, sev-2 for `unreachable` when fail-closed, sev-3 for `unreachable` when fail-open.

6. **MUST** maintain an in-process LRU cache keyed on the 5-prefix, value = (HIBP response body parsed as `Map<suffix, count>`, fetched_at). TTL = 1 hour, max entries = 10000 (DEC-724). On cache hit, no HTTP request is made; the suffix lookup happens in-memory. Cache is per-process; restart loses it. A single prefix-fetch returns approximately 500 suffix-count pairs (HIBP averages ≈500 per prefix), so 10000 prefixes ≈ 5M entries ≈ 60 MiB RSS. The cache is bounded.

7. **MUST** enforce a 2000ms timeout on each HIBP HTTP request (DEC-730). On connect failure, retry once. On any non-2xx response (4xx, 5xx) or timeout-after-retry, the request is treated as unreachable. Network-level retries beyond 1 are forbidden because (a) the timeout already cost 2s, and (b) password-set is interactive — a 4s+ delay degrades UX.

8. **MUST** resolve the unreachable case per the per-tenant `hibp_unreachable_policy` (DEC-726). Default `fail_open`: the password passes the breach-check but a sev-3 memory audit row records `decision = "unreachable"`. Alternative `fail_closed`: return `503 SERVICE_UNAVAILABLE` with `{error: "hibp_unreachable", retry_after_seconds: 60}` and sev-2 audit. The CFO/security role mutates the policy with a sev-2 audit row + reason ≥10 chars.

9. **MUST** support a local k-anon dump fallback (DEC-723). When env `CYBEROS_HIBP_LOCAL_DUMP_PATH` is set, the AUTH service skips the network request entirely and looks up the prefix in a local file (HIBP's downloadable dump split by 5-char prefix). The local dump is used by air-gapped deployments. The dump file format is one prefix per file: `<dump_root>/<prefix>.txt` with the same line-per-suffix:count format. Mode is determined at process start by env presence; runtime switching is not supported.

10. **MUST** validate per-tenant `hibp_block_threshold` at policy write time. Range [1, 10000]. Out-of-range mutation returns `400 BAD_REQUEST` with `{error: "hibp_threshold_out_of_range", min: 1, max: 10000}`. Policy mutation requires the `security_admin` role per TASK-AUTH-101 + a sev-2 memory audit row + reason ≥10 chars.

11. **MUST** define a closed 3-value `hibp_decision` Postgres enum (`allowed`, `rejected`, `unreachable`). The CI cardinality test asserts exactly 3 values.

12. **MUST** record every breach-check call in the `hibp_audit_log` append-only table — REVOKE UPDATE, DELETE FROM cyberos_app. The row carries `(id, tenant_id, subject_id_at_call_time, prefix, count, decision, code_path, occurred_at, memory_chain_hash)`. The memory_chain_hash cross-links the Postgres row to the memory audit row.

13. **MUST** route the breach-check call BEFORE the Argon2 hash computation. Argon2 is intentionally slow (~100ms per call); calling HIBP first means a rejected password short-circuits the Argon2 cost. The order: input validation → HIBP check → (if allowed) Argon2 hash → persist.

14. **MUST** scrub the prefix from any structured log lines that escape the process. Even though the prefix is k-anonymous (≈100k passwords share a prefix), structured logs that get exported (e.g., to a SIEM) could be cross-referenced with HIBP downloads to materially shrink the candidate set for a specific user. Audit emissions (clause #5) are an exception — they go through TASK-MEMORY-111 PII scrubbing, which knows the prefix is operational metadata and lets it through.

15. **MUST** treat the User-Agent header as `cyberos-auth/<service-version>` per HIBP's required-UA contract. Requests without a UA may be rate-limited or rejected by HIBP. The version string is the same string emitted by `auth --version`.

16. **MUST NOT** accept the HIBP "Add-Padding" header (`Add-Padding: true`). Padding adds dummy lines that change response sizes; we don't need it because the in-process cache hides response sizes from any external observer. Adding padding would only inflate cache memory.

17. **MUST** define a closed 2-value `hibp_unreachable_policy` enum (`fail_open`, `fail_closed`). The CI cardinality test asserts exactly 2 values.

18. **MUST** apply the breach check to password resets initiated by the subject (TASK-AUTH-102 password-reset flow) as well as admin-initiated resets. The reset-flow handler invokes the same `hibp::check` function as signup/rotation/admin-set — there is one canonical check function with three call sites.

19. **MUST** refuse to start the AUTH service if `CYBEROS_HIBP_LOCAL_DUMP_PATH` is set but the directory is unreadable or empty. The startup error message: `hibp_local_dump_unreachable: <path> exists but contains 0 .txt files`. This is fail-loud: an air-gapped operator should know immediately that their dump is missing, not discover it via a fail-closed reject on the first user signup.

20. **MUST** maintain a CI test that asserts the live HIBP API contract is unchanged: a fixed prefix `21BD1` MUST return a response containing the suffix-count line for the well-known weak password "password" (count > 1,000,000 as of 2026). This canary breaks intentionally if HIBP changes its API or removes its hall-of-fame entries; we update the canary deliberately rather than discovering breakage in production.

21. **MUST** apply rate-limit awareness: HIBP's free tier is 1 request per ~1.5s per IP. The in-process cache (clause #6) handles repeated checks of the same prefix; for unique prefixes flooding in faster than the rate limit, the AUTH service tokens via a semaphore (10 concurrent requests max). At semaphore saturation, requests queue up to 5 seconds; longer waits return as unreachable per DEC-730.

22. **MUST** treat 429 (HIBP rate limit) as unreachable + emit a sev-2 `hibp.rate_limited` memory audit row. Repeated 429s in a 5-minute window MUST escalate to sev-1 (signals our deployment is exceeding the free-tier budget — operations decision needed).

23. **MUST** allow per-tenant whitelist of well-known development passwords ONLY in development environments. The `CYBEROS_HIBP_DEV_WHITELIST` env (`"password,admin,test,changeme"`) bypasses the breach check for those plaintexts. The env is honored ONLY when `CYBEROS_ENV=development`; in `staging` or `production` it is silently ignored AND a sev-1 memory audit row records the attempted bypass at startup. This prevents accidental shipment of dev whitelists to prod.

24. **MUST** support a `dry_run` mode on the policy update endpoint where the caller previews what would change without persisting. The `dry_run=true` query param returns the proposed effective config + a list of currently-active sessions that would be affected on next password rotation. No DB mutation, no memory audit emitted.

25. **MUST** emit 6 closed memory audit kinds:
- `auth.hibp_check_passed` (sev-3, on count < threshold)
- `auth.hibp_check_rejected` (sev-2, on count >= threshold)
- `auth.hibp_unreachable_fail_open` (sev-3, network down + tenant fail_open)
- `auth.hibp_unreachable_fail_closed` (sev-2, network down + tenant fail_closed)
- `auth.hibp_rate_limited` (sev-2 per event; sev-1 escalation when 3+ in 5min)
- `auth.hibp_policy_changed` (sev-2 per mutation)

---

## §2 — Rationale (informative — preserve all 22 paragraphs)

**§2.1  Why HIBP and not a self-maintained corpus.** Building our own credential-stuffing corpus would mean ingesting + indexing > 10B leaked passwords + keeping the dataset current as new breaches drop. HIBP does this full-time, with a public API, and offers k-anonymity so we never leak the password itself. The cost (network dependency on a third party) is bounded by clause #9 (local dump fallback for air-gapped) + clause #8 (per-tenant policy on unreachable).

**§2.2  Why k-anonymity and not full-hash submission.** HIBP also offers an unauthenticated full-hash check via `POST /unprotected/api/v2/pwned`, but sending a full SHA-1 hash to a third party — even one we trust — is a step too close to "we sent a fingerprint of our user's password to a third party". The k-anonymous range API sends only 5 hex chars (~1M possible prefixes, ~100k passwords per prefix). The remaining 35 chars are matched locally. The plaintext, full hash, user identity, and IP are never linked at HIBP's end.

**§2.3  Why threshold = 5 by default.** DEC-721. HIBP returns the count of times each password has been seen across all aggregated breaches. A password seen once might be a leak that hasn't been deduplicated; a password seen 5+ times across breaches is observed real-world. The threshold balances false-positive (rejecting passwords that are unusual but happen to coincide with a long-tail leak) against false-negative (accepting passwords on the cusp). Tenants who want stricter (1) or laxer (50, 100) override per-tenant per DEC-722.

**§2.4  Why fail-open by default and per-tenant override.** DEC-725 + DEC-726. Fail-open keeps signup working when HIBP is down; the only cost is that one password — the one being set during the outage — might be in HIBP's corpus and we let it through. The sev-2 audit row makes the gap visible. Fail-closed (for high-security tenants) means HIBP outage blocks signup entirely; for those tenants, the local dump (clause #9) is the operational mitigation.

**§2.5  Why check BEFORE Argon2.** Clause #13. Argon2 is intentionally compute-heavy (~100ms tuned). If we Argon2'd first and then HIBP'd, every rejected password would cost us 100ms of CPU. With HIBP first, rejected passwords short-circuit at the network call (~50ms p50 cached, ~500ms p99 uncached). The reverse ordering would multiply signup CPU under credential-stuffing attack scenarios.

**§2.6  Why not check at login.** DEC-728 + clause #1. At login, the plaintext is held briefly to verify Argon2; calling HIBP on every login would (a) add 50-500ms to every authentication, and (b) re-check passwords we already approved at set-time. The right semantic is: a password is breach-checked at set-time; if HIBP later adds it to the corpus, the user's existing password remains valid until they rotate (then re-checked). For tenants that want compliance-style "force rotation on every breach", clause #5's audit row + a scheduled job that scans rotation_due is the future direction (out of scope here).

**§2.7  Why 1-hour cache and not 24-hour.** DEC-724 + clause #6. HIBP's corpus grows over time; a 24-hour cache would let a freshly-added breach go unmasked for up to a day. 1 hour caps that gap. The cache hit-rate is high for popular prefixes (the top-1000 prefixes likely cover most signup attempts), so the cost of 24× more HTTP requests is modest.

**§2.8  Why 5-hex prefix and not 4 or 6.** HIBP's API contract is fixed at 5 hex chars. 4 chars (16x fewer prefixes, ~1.6M passwords per prefix) would leak more anonymity; 6 chars (16x more prefixes, ~6k per prefix) would leak less but isn't supported. We follow the API.

**§2.9  Why 10000-entry cache and not unbounded.** Clause #6. Unbounded cache leaks memory when prefix-distribution is uniform random (and at our scale, signup prefixes ARE distributed roughly uniformly because passwords are diverse). 10000 entries × ~6 KiB per entry ≈ 60 MiB. Manageable. The LRU eviction keeps the cache focused on recently-active prefixes.

**§2.10  Why we don't enable Add-Padding.** Clause #16. The padding feature inflates each HIBP response with dummy lines to defeat traffic analysis. We hide HIBP traffic from external observers by (a) running AUTH inside our cluster, (b) caching responses locally. Adding padding inflates cache memory without security benefit at our deployment shape.

**§2.11  Why air-gapped local dump option.** DEC-723 + clause #9. Some tenants (regulated industries, government) prohibit any third-party network call from production. The local k-anon dump (downloadable from HIBP, ~30 GiB compressed) lets those tenants run with full breach-check coverage at the cost of monthly dump-refresh discipline. The fallback is intentionally simpler than the live API — no caching needed, file system is fast enough; no rate-limit handling needed.

**§2.12  Why the live-API canary test.** Clause #20. HIBP could change its API contract (URL, response format, padding default). Our implementation depends on the current contract. A CI test against the live API for a known weak password ("password" prefix `21BD1`) breaks loudly if contract changes; we update the implementation deliberately rather than discovering breakage when a user signup mysteriously rejects.

**§2.13  Why the dev whitelist with guardrails.** Clause #23. Local development against the live HIBP from each dev's laptop would burn through the rate limit fast + force every dev signup form to use a strong password (annoying). The env-toggled whitelist lets devs sign up with "password" in development. The fail-loud sev-1 audit at startup when whitelist is set in non-dev catches operator error — someone accidentally promoting a dev config to prod.

**§2.14  Why 6 closed audit kinds and not fewer.** Clause #25. Each kind reflects a different operational signal: success-pass, deliberate reject, network-failure-fail-open, network-failure-fail-closed, rate-limited, policy-changed. Collapsing to fewer (e.g., "hibp_event" with reason field) would force every dashboard to do free-text parsing.

**§2.15  Why we don't expose HIBP results to the user UI beyond reject.** Clause #4 returns the count in the error body. The UI could surface "this password has appeared in 1,247 breaches" to motivate the user. We don't put that in the UI by default for two reasons: (a) it confuses non-technical users ("breach" is a loaded word), and (b) the count fluctuates as HIBP merges new dumps and the user thinks the number is about their breach specifically. The audit row carries the count for operators; the UI just says "this password has appeared in known breaches — choose another".

**§2.16  Why we don't allow per-tenant choice of breach-check provider.** Multiple breach-check services exist (e.g., Enzoic, SpyCloud). Supporting a pluggable backend would multiply integration complexity for marginal benefit; HIBP is the de facto industry standard, free, and well-instrumented. If a competitor offers a meaningfully better corpus or API, we revisit then.

**§2.17  Why we don't include user-PII in the HIBP request.** HIBP doesn't see anything but a 5-char prefix. No email, no user ID, no IP correlation (HIBP doesn't log the requesting IP). Clause #2 + DEC-720 + DEC-727 codify this — defense-in-depth against a future change that would leak user context.

**§2.18  Why per-tenant threshold range is [1, 10000].** Clause #10. Threshold = 1 is the strictest (reject any password seen even once). Threshold = 10000 is effectively "warn-only without rejecting most things" — only the most common ~50 passwords ever exceed 10000 in the HIBP corpus. Beyond 10000 the threshold is meaningless (almost nothing exceeds it; the policy is "accept everything"). The cap prevents confused operators from setting threshold to 1B and silently disabling the check.

**§2.19  Why we surface password-breach errors as 422 not 400.** Clause #4. 400 is "bad input you can fix syntactically"; 422 is "semantically valid but business-rule-rejected". A breached password is well-formed and meets length+complexity rules; it just happens to be known-compromised. 422 is the standard HTTP semantic for that.

**§2.20  Why we never log the plaintext or full SHA-1 at any level.** Build-envelope `disallowed_tools`. A debug log with the plaintext is a credentials-in-logs incident. A debug log with the full SHA-1 is reversible to the plaintext for any password in the HIBP corpus. Both are forbidden by code review + by the static-analysis pass in CI that greps for `password = "{}"` patterns.

**§2.21  Why startup fails loudly when local-dump is set but empty.** Clause #19. The alternative — silent fallback to network HIBP when local-dump is empty — defeats the purpose of choosing air-gapped mode. Air-gapped operators picked local-dump for a reason; the system should fail at startup, not on the first signup.

**§2.22  Why we don't ship our own HIBP dump.** Hosting a 30 GiB dump as part of the CyberOS distribution would (a) bloat the deployment package, (b) make us responsible for keeping it fresh, (c) force tenants to update CyberOS to get new corpus. Letting operators fetch HIBP's dump directly keeps refresh on their schedule.

---

## §3 — API & schema

### §3.1 — Migration: hibp_audit_log + per-tenant policy

```sql
-- services/auth/migrations/0015_hibp_audit.sql

CREATE TYPE hibp_decision AS ENUM ('allowed', 'rejected', 'unreachable');
CREATE TYPE hibp_unreachable_policy AS ENUM ('fail_open', 'fail_closed');
CREATE TYPE hibp_code_path AS ENUM ('signup', 'rotation', 'admin_set', 'password_reset');

CREATE TABLE hibp_audit_log (
    id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id        UUID NOT NULL REFERENCES tenants(id),
    subject_id       UUID,  -- nullable for signup-before-subject-exists
    prefix           CHAR(5) NOT NULL CHECK (prefix ~ '^[0-9A-F]{5}$'),
    count            BIGINT NOT NULL CHECK (count >= 0),
    decision         hibp_decision NOT NULL,
    code_path        hibp_code_path NOT NULL,
    occurred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    memory_chain_hash CHAR(64) NOT NULL CHECK (memory_chain_hash ~ '^[0-9a-f]{64}$')
);

CREATE INDEX hibp_audit_tenant_time ON hibp_audit_log (tenant_id, occurred_at DESC);
CREATE INDEX hibp_audit_subject     ON hibp_audit_log (subject_id) WHERE subject_id IS NOT NULL;

REVOKE UPDATE, DELETE ON hibp_audit_log FROM cyberos_app;
GRANT INSERT, SELECT ON hibp_audit_log TO auth_writer;
GRANT SELECT ON hibp_audit_log TO security_admin;

ALTER TABLE hibp_audit_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation ON hibp_audit_log
    USING (tenant_id = current_setting('cyberos.tenant_id')::uuid)
    WITH CHECK (tenant_id = current_setting('cyberos.tenant_id')::uuid);

-- Per-tenant policy
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS hibp_block_threshold INTEGER NOT NULL DEFAULT 5
    CHECK (hibp_block_threshold BETWEEN 1 AND 10000);
ALTER TABLE tenants ADD COLUMN IF NOT EXISTS hibp_unreachable_policy hibp_unreachable_policy NOT NULL DEFAULT 'fail_open';
```

### §3.2 — HIBP client

```rust
// services/auth/src/hibp/client.rs

use sha1::{Sha1, Digest};
use std::time::Duration;

pub struct HibpClient {
    http: reqwest::Client,
    cache: Arc<Mutex<lru::LruCache<String, (HashMap<String, u64>, Instant)>>>,
    local_dump_root: Option<PathBuf>,
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl HibpClient {
    pub fn new(local_dump: Option<PathBuf>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_millis(2000))
            .user_agent(format!("cyberos-auth/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("HIBP HTTP client");
        Self {
            http,
            cache: Arc::new(Mutex::new(lru::LruCache::new(NonZeroUsize::new(10000).unwrap()))),
            local_dump_root: local_dump,
            semaphore: Arc::new(tokio::sync::Semaphore::new(10)),
        }
    }

    pub async fn lookup_count(&self, password: &str) -> Result<u64, HibpUnreachable> {
        let hash = format!("{:X}", Sha1::digest(password.as_bytes()));
        let prefix = &hash[..5];
        let suffix = &hash[5..];

        // Cache check
        {
            let mut c = self.cache.lock().await;
            if let Some((map, fetched)) = c.peek(prefix) {
                if fetched.elapsed() < Duration::from_secs(3600) {
                    return Ok(*map.get(suffix).unwrap_or(&0));
                }
            }
        }

        // Local dump path
        if let Some(root) = &self.local_dump_root {
            let path = root.join(format!("{}.txt", prefix));
            let map = parse_dump_file(&path).await?;
            self.cache_insert(prefix, &map).await;
            return Ok(*map.get(suffix).unwrap_or(&0));
        }

        // Network path with semaphore
        let _permit = self.semaphore.acquire().await.map_err(|_| HibpUnreachable::Semaphore)?;
        let url = format!("https://api.pwnedpasswords.com/range/{}", prefix);
        let resp = match self.http.get(&url).send().await {
            Ok(r) => r,
            Err(_) => {
                // One retry on connect-failure
                self.http.get(&url).send().await
                    .map_err(|_| HibpUnreachable::Network)?
            }
        };
        if resp.status() == 429 {
            return Err(HibpUnreachable::RateLimited);
        }
        if !resp.status().is_success() {
            return Err(HibpUnreachable::Network);
        }
        let body = resp.text().await.map_err(|_| HibpUnreachable::Network)?;
        let map: HashMap<String, u64> = body.lines().filter_map(|line| {
            let mut parts = line.splitn(2, ':');
            let sfx = parts.next()?.to_string();
            let cnt = parts.next()?.parse::<u64>().ok()?;
            Some((sfx, cnt))
        }).collect();
        self.cache_insert(prefix, &map).await;
        Ok(*map.get(suffix).unwrap_or(&0))
    }
}

pub enum HibpUnreachable {
    Network, RateLimited, Semaphore, LocalDumpMissing(PathBuf)
}
```

### §3.3 — Check function with policy enforcement

```rust
// services/auth/src/hibp/mod.rs

pub async fn check(
    pool: &PgPool,
    client: &HibpClient,
    tenant_id: Uuid,
    subject_id: Option<Uuid>,
    password: &str,
    code_path: CodePath,
) -> Result<(), CheckError> {
    let threshold: i64 = sqlx::query_scalar!(
        "SELECT hibp_block_threshold FROM tenants WHERE id = $1", tenant_id
    ).fetch_one(pool).await?;
    let unreachable_policy: HibpUnreachablePolicy = sqlx::query_scalar!(
        r#"SELECT hibp_unreachable_policy AS "p: HibpUnreachablePolicy" FROM tenants WHERE id = $1"#,
        tenant_id
    ).fetch_one(pool).await?;

    let prefix = format!("{:X}", Sha1::digest(password.as_bytes()))[..5].to_string();

    match client.lookup_count(password).await {
        Ok(count) => {
            let decision = if count >= threshold as u64 { Decision::Rejected } else { Decision::Allowed };
            emit_audit(pool, tenant_id, subject_id, &prefix, count, decision, code_path).await?;
            if decision == Decision::Rejected {
                return Err(CheckError::Breached { count, threshold: threshold as u64 });
            }
            Ok(())
        }
        Err(unreachable) => {
            let (decision, sev) = match unreachable_policy {
                HibpUnreachablePolicy::FailOpen => (Decision::Unreachable, 3),
                HibpUnreachablePolicy::FailClosed => (Decision::Unreachable, 2),
            };
            emit_audit_unreachable(pool, tenant_id, subject_id, &prefix, decision, sev, unreachable, code_path).await?;
            match unreachable_policy {
                HibpUnreachablePolicy::FailOpen => Ok(()),
                HibpUnreachablePolicy::FailClosed => Err(CheckError::ServiceUnavailable),
            }
        }
    }
}
```

---

## §4 — Acceptance criteria

1. `hibp_decision` enum exactly 3 values; CI cardinality test asserts.
2. `hibp_unreachable_policy` enum exactly 2 values.
3. `hibp_code_path` enum exactly 4 values (signup, rotation, admin_set, password_reset).
4. Signup with HIBP-known password (count >= threshold) returns `422 password_breached` with count + threshold.
5. Signup with non-breached password proceeds; Argon2 hash computed; subject created.
6. Rotation with breached password returns `422 password_breached`.
7. Admin-set with breached password returns `422`.
8. Password reset flow (TASK-AUTH-102) routes through same check.
9. Login NEVER invokes HIBP (verified via test that mocks HIBP and asserts zero calls).
10. HIBP request sends only 5-char prefix; full hash never appears in network capture (test uses wiremock to assert request body).
11. Cache hit on second check with same prefix; no HTTP request issued.
12. Cache TTL of 1h; after 3600s+1, second check issues a fresh HTTP request.
13. Cache LRU eviction when 10001 unique prefixes accessed; least-recent prefix re-fetches.
14. Network timeout at 2000ms triggers retry; after retry-fail, unreachable.
15. 429 from HIBP treated as unreachable + sev-2 audit `auth.hibp_rate_limited`.
16. Three 429s in 5-min window escalates to sev-1.
17. Tenant `fail_open` policy lets request proceed on unreachable; sev-3 audit.
18. Tenant `fail_closed` returns `503 hibp_unreachable` with `retry_after_seconds`; sev-2 audit.
19. Per-tenant `hibp_block_threshold` mutation requires `security_admin` role + reason ≥10 chars + sev-2 audit.
20. Threshold out of [1, 10000] returns `400 hibp_threshold_out_of_range`.
21. `CYBEROS_HIBP_LOCAL_DUMP_PATH` set → skips network, reads `<root>/<prefix>.txt`.
22. Local-dump root set but empty → AUTH service refuses to start (clause #19).
23. CI canary test against live HIBP for prefix `21BD1` returns suffix line for "password" with count > 1,000,000.
24. `CYBEROS_HIBP_DEV_WHITELIST` honored only when `CYBEROS_ENV=development`; sev-1 audit at startup if set in prod.
25. `hibp_audit_log` REVOKE UPDATE/DELETE confirmed at `\dp`.
26. Cross-tenant SELECT on `hibp_audit_log` returns empty (RLS enforced).
27. Audit row carries prefix + count + decision + code_path but never plaintext or full SHA-1.
28. HIBP request UA header is `cyberos-auth/<version>`.
29. HIBP request does NOT include `Add-Padding: true` (clause #16).
30. Semaphore at 10 concurrent HIBP requests; 11th waits; 5-second wait timeout treated as unreachable.
31. `dry_run=true` on policy update returns effective preview + affected subject count; no DB mutation; no audit row.
32. Argon2 hash NOT computed when HIBP rejects (clause #13 ordering verified via timing test or instrumentation hook).

---

## §5 — Verification (CI tests)

- `cardinality_test_decision` — 3 enum values.
- `cardinality_test_unreachable_policy` — 2.
- `cardinality_test_code_path` — 4.
- `signup_breached_test` — wiremock returns count=5+; signup returns 422.
- `signup_clean_test` — wiremock returns count=0; signup succeeds.
- `rotation_breached_test` — same for rotation path.
- `admin_set_breached_test` — same for admin path.
- `password_reset_breached_test` — same for reset path.
- `login_no_hibp_test` — login invokes HIBP zero times.
- `network_capture_test` — assert request body is only 5 hex chars + UA header.
- `cache_hit_test` — two consecutive checks with same prefix → one HTTP request.
- `cache_ttl_test` — advance clock 3601s; second check → fresh HTTP request.
- `cache_lru_eviction_test` — fill cache to 10000; access 10001st; oldest evicted.
- `timeout_retry_test` — wiremock delays 2.5s; retry fires; eventually unreachable.
- `rate_limited_429_test` — 429 → unreachable + sev-2 audit.
- `rate_limited_escalation_test` — 3 429s in 5min → sev-1 audit.
- `fail_open_test` — wiremock unreachable + tenant policy fail_open → signup succeeds + sev-3.
- `fail_closed_test` — wiremock unreachable + tenant policy fail_closed → 503 + sev-2.
- `threshold_mutation_acl_test` — non-security_admin role → 403.
- `threshold_range_test` — threshold = 0 and 10001 → 400.
- `local_dump_test` — set env; provide fixture dir; lookup uses dump not network.
- `local_dump_empty_test` — set env + empty dir; AUTH refuses to start.
- `live_hibp_canary_test` — actual GET against api.pwnedpasswords.com/range/21BD1; assert count > 1M for "password".
- `dev_whitelist_dev_test` — CYBEROS_ENV=development; whitelist password "password" bypasses HIBP.
- `dev_whitelist_prod_test` — CYBEROS_ENV=production + whitelist set → ignored + sev-1 startup audit.
- `audit_no_plaintext_test` — assert hibp_audit_log row never contains plaintext or full hash.
- `rls_isolation_test` — two tenants; cross-query returns empty.
- `ua_header_test` — wiremock asserts UA matches `cyberos-auth/x.y.z`.
- `no_padding_header_test` — wiremock asserts Add-Padding absent.
- `semaphore_test` — fire 11 parallel checks; 11th waits; instrument confirms.
- `argon2_after_hibp_test` — wiremock returns reject; instrumented Argon2 NOT called.
- `dry_run_test` — dry_run=true returns preview; no DB row written; no audit row.

---

## §6 — File skeleton

```
services/auth/
├── src/
│   ├── hibp/
│   │   ├── mod.rs          # pub check() function
│   │   ├── client.rs       # HibpClient
│   │   ├── cache.rs        # LRU cache wrapper
│   │   ├── local_dump.rs   # air-gapped fallback
│   │   ├── policy.rs       # per-tenant policy load + mutation
│   │   ├── audit.rs        # memory audit emission
│   │   └── error.rs        # CheckError + HibpUnreachable enums
│   └── handlers/
│       ├── signup.rs              # MODIFIED: invoke hibp::check before Argon2
│       ├── password_rotate.rs     # MODIFIED
│       ├── admin_set_password.rs  # MODIFIED
│       └── password_reset.rs      # MODIFIED
├── migrations/
│   └── 0015_hibp_audit.sql
└── tests/
    ├── hibp_signup_test.rs
    ├── hibp_rotation_test.rs
    ├── hibp_unreachable_test.rs
    ├── hibp_cache_test.rs
    ├── hibp_canary_test.rs       # live API test (CI-only)
    ├── hibp_local_dump_test.rs
    └── hibp_dev_whitelist_test.rs
```

---

## §7 — Dependencies & blast-radius

**Depends on**: TASK-AUTH-002 (subject_create — wiring point), TASK-AUTH-005 (admin REST — wiring point).

**Blocks**: nothing (terminal leaf in P2 auth hardening).

**Blast radius if broken**:
- **False positives** (clean password rejected): user friction during signup; bounded by the threshold tuning.
- **False negatives** (breached password accepted): one user's account is credential-stuffing vulnerable; mitigated by MFA at TASK-AUTH-102.
- **HIBP outage with fail-open**: short window of unchecked passwords; sev-2 audit makes it visible.
- **HIBP outage with fail-closed**: signup unavailable for those tenants until network or operator switches to fail-open.

---

## §8 — Payload examples

### §8.1 — Successful signup (clean password)

```
POST /v1/auth/signup
{ "email": "a@b.com", "password": "<strong-random-13-char>", "tenant_id": "ten_abc" }

200 OK
{ "subject_id": "sub_xyz", "session_token": "..." }
```

### §8.2 — Signup with breached password

```
POST /v1/auth/signup
{ "email": "a@b.com", "password": "Password123", "tenant_id": "ten_abc" }

422 Unprocessable Entity
{
  "error": "password_breached",
  "count": 1234,
  "threshold": 5,
  "hint": "This password has appeared in known breaches. Please choose another."
}
```

### §8.3 — HIBP unreachable, tenant fail_closed

```
POST /v1/auth/password/rotate
{ "old_password": "...", "new_password": "..." }

503 Service Unavailable
{
  "error": "hibp_unreachable",
  "retry_after_seconds": 60,
  "policy": "fail_closed"
}
```

### §8.4 — Policy mutation (security_admin)

```
PUT /v1/admin/tenants/{id}/hibp-policy
Authorization: Bearer <security_admin>
{
  "hibp_block_threshold": 1,
  "hibp_unreachable_policy": "fail_closed",
  "reason": "Customer SLA requires fail-closed mode"
}

200 OK
{ "applied": true, "effective_at": "2026-05-16T11:00:00Z" }
```

### §8.5 — Dry-run policy preview

```
PUT /v1/admin/tenants/{id}/hibp-policy?dry_run=true
{ "hibp_block_threshold": 1 }

200 OK
{
  "would_apply": true,
  "current": { "hibp_block_threshold": 5 },
  "proposed": { "hibp_block_threshold": 1 },
  "affected_subjects_on_next_rotation": 0
}
```

---

## §9 — Open questions

- **OQ-1** (closed by DEC-721): threshold default = 5. Confirmed.
- **OQ-2** (closed by DEC-723): local k-anon dump fallback for air-gapped. Confirmed.
- **OQ-3** (closed by DEC-725): default fail-open with sev-2 audit. Confirmed.
- **OQ-4** (open): should we provide a scheduled job to re-check all stored Argon2 hashes against newly-added HIBP entries? Currently NOT in scope (clause #1 + §2.6). Re-checking requires storing the SHA-1 alongside Argon2, which defeats Argon2's purpose. Defer indefinitely.
- **OQ-5** (open): should we explore the paid HIBP API for the higher rate limit + dump-mirror automation? Watch traffic; revisit if free-tier rate-limit pressure becomes operational.

---

## §10 — Failure modes (32 rows)

| # | Failure | Detection | Sev | Handler |
|---|---------|-----------|-----|---------|
| 1 | HIBP network unreachable | timeout 2s | 2-3 | Per-tenant policy: fail_open + sev-3 or fail_closed + sev-2 |
| 2 | HIBP 429 rate-limited | response code | 2 | Treat as unreachable + sev-2; 3 in 5min → sev-1 |
| 3 | HIBP 5xx response | response code | 2 | Treat as unreachable; same policy as #1 |
| 4 | DNS failure to api.pwnedpasswords.com | DNS error | 2 | Treat as unreachable |
| 5 | TLS handshake failure | TLS error | 2 | Treat as unreachable |
| 6 | Connection reset mid-response | read error | 2 | Retry once; if still fail, unreachable |
| 7 | Response body truncated | line parsing | 3 | Use partial map; count from missing suffix = 0; sev-3 audit |
| 8 | Cache memory pressure | LRU eviction | 3 | Oldest entries evicted; no audit needed |
| 9 | Local dump file missing for prefix | file open error | 1 | Sev-1 startup audit (loader checks all prefixes at start); treat individual miss as unreachable |
| 10 | Local dump file corrupted | parse error | 1 | Sev-1; treat as unreachable for this prefix |
| 11 | Local dump path empty at startup | dir scan | 1 | AUTH refuses to start |
| 12 | Plaintext leaked to log | static-analysis CI grep | 1 | CI blocks deploy |
| 13 | Full SHA-1 leaked to log | same | 1 | CI blocks deploy |
| 14 | Threshold mutation by non-admin | RBAC | 2 | 403 + sev-2 |
| 15 | Threshold out of [1, 10000] | CHECK constraint | 3 | 400 |
| 16 | Reason for mutation < 10 chars | length validator | 3 | 400 |
| 17 | Dev whitelist set in prod | startup config scan | 1 | Whitelist ignored + sev-1 startup audit |
| 18 | Live-canary CI test fails | CI run | 2 | Deploy blocked; investigate HIBP API change |
| 19 | Semaphore saturation > 5s wait | wait timeout | 3 | Treat as unreachable |
| 20 | Argon2 invoked before HIBP | code review + ordering test | 2 | Test fails; CI blocks |
| 21 | Audit row missing memory_chain_hash | CHECK constraint | 1 | DB rejects insert |
| 22 | RLS cross-tenant leak | rls_isolation_test | 1 | CI blocks |
| 23 | Network capture shows full SHA-1 | network_capture_test | 1 | CI blocks |
| 24 | Cache invalidation on policy change | policy mutation handler | 3 | Cache flushed on threshold or policy change |
| 25 | HIBP API contract change (URL) | live canary | 2 | Code fix + redeploy |
| 26 | HIBP response format change | parse error | 2 | Treat as unreachable; sev-2; investigate |
| 27 | Add-Padding inadvertently enabled | request capture test | 2 | Code fix |
| 28 | UA header missing | request capture test | 3 | Code fix |
| 29 | Cache TTL > 1h | manual config or code review | 2 | Code fix |
| 30 | Audit row contains plaintext or full hash | audit_no_plaintext_test | 1 | CI blocks |
| 31 | Login path inadvertently invokes HIBP | login_no_hibp_test | 2 | CI blocks |
| 32 | dry_run=true mutates state | dry_run_test | 2 | CI blocks |

---

## §11 — Implementation notes

**§11.1** The HIBP client is one shared instance per AUTH service process (Arc'd). The LRU + semaphore + HTTP client are all behind the Arc.

**§11.2** SHA-1 is chosen by HIBP, not by us. The hash is not used for password storage (Argon2 does that); it's used only as a corpus index key.

**§11.3** The audit emission is async + after the COMMIT — a failed audit row never blocks signup. Failed audits are retried via the task-OBS WAL queue pattern.

**§11.4** Per-tenant policy is cached in the AUTH process with a 60s TTL (matching the TASK-AUTH-109 active-tenant pattern). Mutation invalidates the cache via pg_notify.

**§11.5** The 2s timeout is conservative; HIBP's p99 response time is typically < 500ms. The slack absorbs network jitter without degrading UX.

**§11.6** The semaphore at 10 concurrent requests prevents one AUTH process from consuming more than 10× the free-tier rate at any moment. At 10 concurrent + ~500ms p50 + 1.5s rate limit = ~6.6 req/s sustained, well within free tier.

**§11.7** The 1h cache TTL means under heavy traffic, the same prefix is fetched at most once per hour. With ~1M signup attempts/day spread over ~1M possible prefixes (uniform-ish), cache hit rate at our scale will be modest; the rate-limiter still protects us.

**§11.8** Local-dump format follows HIBP's "split-by-prefix" download layout. Each file `<prefix>.txt` is identical to the HIBP API response body — same parser works for both.

**§11.9** The dev whitelist is intentionally narrow: only the few literal strings devs use during local development. A wildcard or pattern-based whitelist would be too easy to misuse.

**§11.10** Argon2 ordering is enforced by the handler structure: `validate_password_complexity()` → `hibp::check()` → `argon2::hash()` → `persist()`. A timing instrumentation hook in CI verifies HIBP is invoked first.

**§11.11** The error body's `hint` field is the UI-facing message. It's intentionally vague to avoid confusing the user with breach counts.

**§11.12** The `retry_after_seconds: 60` on fail_closed is a hint, not a guarantee. The client UI uses it for the retry button countdown.

**§11.13** Live-canary CI runs only on the nightly build, not every PR — it makes a real HIBP API call and we don't want PR-CI to consume that rate budget.

**§11.14** Local-dump root is read once at startup into an in-memory index of "which prefixes exist on disk"; subsequent lookups check the index + open the file. The index is `O(N)` where N = 1,048,576 prefixes × 32-byte stat entry ≈ 32 MiB RSS.

**§11.15** The audit_log table's `subject_id` is nullable because at signup the subject doesn't yet exist (the breach-check runs before subject_create). After signup, the row's subject_id can be filled by a post-create trigger if needed; today it stays NULL for signup rows and is set for rotation/admin/reset rows.

**§11.16** The 429-escalation logic (3 in 5min → sev-1) is implemented as a sliding window counter per AUTH process. Cross-process visibility happens via the memory audit chain itself — operators see 429s across the fleet by aggregating chain rows.

**§11.17** The `prefix` column on `hibp_audit_log` is `CHAR(5)` with a regex CHECK to ensure exactly 5 uppercase hex chars. This prevents log analyzers from accidentally treating it as variable text.

**§11.18** The `memory_chain_hash` cross-link to the memory row enables ledger walks: given an audit row, jump to memory chain hash; given a chain hash, query back to Postgres row. Mirrors the TASK-TEN-004 dual-write pattern.

**§11.19** Tests use `wiremock` for HTTP mock + `sqlx::testcontainers` for Postgres. The live-canary test uses an actual HTTP call against api.pwnedpasswords.com — gated by `#[cfg(feature = "live_canary")]` to keep it out of unit-test runs.

**§11.20** The cache invalidation on policy change flushes the entire cache (not just the affected prefix) because threshold changes affect every check decision, not just specific prefixes.

**§11.21** The PII scrubber (TASK-MEMORY-111) is configured with `hibp.*` as a structured-payload allowlist — the prefix, count, decision are operational metadata, not user PII.

**§11.22** The fail-loud startup behavior for empty local-dump (clause #19) is implemented as a single `glob("<root>/*.txt") | count == 0` check at process start. The error message is unambiguous so operators know exactly what to fix.

---

*End of TASK-AUTH-107 spec.*
