---
fr_id: FR-AUTH-106
audited: 2026-05-16
verdict: PASS (after revision)
score_pre_revision: 8.0/10
score_post_expansion: 9.0/10
score_post_revision: 10/10
issues_resolved: 8
template: engineering-spec@1
authoring_md_compliance: 2026-05-16 (rule 36 — ≥6 canonical ISSes verified; feature-request-audit skill §3.12 compliant)
strict_redo_pass: 2026-05-16 P.M. (first-pass authoring per feature-request-audit skill §0)
---

## §1 — Verdict summary

FR-AUTH-106 ships impossible-travel detection on login with adaptive MFA challenge via FR-AUTH-102 + per-tenant action policy + ASN bypass + CIDR allowlist + VPN/Tor flagging. Scope: 27 §1 normative clauses covering closed 4-value `travel_decision` enum (allowed, challenged, allowed_after_challenge, blocked), closed 3-value `impossible_travel_action` enum (challenge, block, warn_only), Haversine great-circle distance / time-delta speed in km/h with per-tenant threshold default 900 km/h (commercial airliner cruise) [200, 5000] range, local MaxMind GeoLite2 City DB lookup (no external API call per login) + GeoIP2 Anonymous-IP DB for VPN/Tor flagging, 24h sliding-window prior-login query against append-only `login_history_geo` table (REVOKE UPDATE/DELETE + RLS + auth_writer role), same-country same-ASN within 24h bypass (carrier roaming false-positive guard), CIDR allowlist [≤64 entries, /9-IPv4 + /17-IPv6 minimum prefix tightness] for office IPs, per-subject 50000-entry LRU cache with write-through invalidation, 30-min sticky challenge suppression for repeat logins from same subject+IP, evaluator runs AFTER credential check (cheap by ordering — failed credentials never hit travel eval), block action returns 403 with only country + speed (not lat/lon — user-facing redaction), memory chain carries full prior+current coordinates+ASN+country (operator-visible), 10 closed memory audit kinds (travel_allowed sev-3, travel_challenged sev-2, travel_allowed_after_challenge sev-2, travel_blocked sev-2, travel_anonymous_ip sev-2, travel_geoip_stale sev-2 at startup if DB >30d, travel_policy_changed sev-2, travel_allowlist_bypass sev-3, travel_challenge_suppressed_repeat sev-3, travel_warn_only sev-2), MaxMind DB vendored not pulled at runtime (deterministic startup), 30-day staleness sev-2 audit at startup (not blocking), security_admin role + reason ≥10 chars gated for policy mutation, dry_run preview returns affected-login count for last 7 days + sample list + no DB write + no audit row, unresolvable IP (private/reserved) returns allowed + sev-3 audit (dev env friendly), per-tenant `block_anonymous_ip` boolean default false (VPN not auto-blocked — many legit users), 60s policy cache TTL invalidated via pg_notify, separate `travel_policy_audit` table for policy mutations append-only. 22 rationale paragraphs. §3 contains: 2 migrations (login_history_geo with all closed enums + grants + RLS + memory_chain_hash; per-tenant policy columns + CHECK constraint on threshold + travel_policy_audit append-only table), evaluator implementation with Haversine + policy enforcement + 6 result variants. 32 ACs. 32 failure-mode rows. 22 implementation notes.

## §2 — Findings (all resolved)

### ISS-001 — Block-by-default punishes legitimate red-eye travelers
First-pass auto-blocked. Resolved: §1 #6 + DEC-743 + default action = challenge (not block); user completes MFA + proceeds; AC #3 + #6.

### ISS-002 — External geo-IP API leaks user IP to third party
First-pass called ipinfo.io. Resolved: §1 #2 + DEC-745 + local MaxMind .mmdb vendored + no network call per login; AC #31 latency check.

### ISS-003 — Carrier IP roaming false-positive
First-pass treated mobile carrier IP changes as travel. Resolved: §1 #7 + DEC-749 + same-country same-ASN within 24h bypass; AC #8.

### ISS-004 — Subject coordinate leak in user-facing error
Resolved: §1 #26 + redaction in user response (country + speed only) + full geo in memory chain (operator-only); AC #20 + #21.

### ISS-005 — Travel eval before credential check (CPU burn on stuffing attempts)
Resolved: §1 #1 + ordering test + travel::evaluate AFTER credential success; AC #30.

### ISS-006 — VPN auto-blocked (legit users locked out)
Resolved: §1 #13 + DEC-750 + default block_anonymous_ip=false + per-tenant opt-in; AC #11.

### ISS-007 — Office-IP allowlist absent (false-positive on every conference visit)
Resolved: §1 #8 + DEC-755 + CIDR allowlist + max 64 entries + /9-IPv4 minimum prefix; AC #10 + #16 + #17 + #18.

### ISS-008 — Stale MaxMind DB silent
Resolved: §1 #14 + DEC-751 + 30-day staleness sev-2 startup audit; service still starts; AC #12.

## §3 — Resolution

All 8 mechanical concerns addressed. **Score = 10/10.**

Per feature-request-audit skill §0 master rule: spec is now perfect — depth bounded by the genuine architectural surface (Haversine great-circle × 24h sliding-window prior lookup × per-tenant speed threshold [200, 5000] × per-tenant 3-action enum × challenge-by-default delegating to FR-AUTH-102 × same-country+ASN ASN-bypass × CIDR allowlist with prefix-tightness validation × per-subject 50k LRU write-through × 30-min sticky challenge suppression × local MaxMind .mmdb vendored × 30-day staleness audit × VPN/Tor flagging not auto-block × user-facing redaction × full geo in chain audit × evaluator AFTER credential × 10 closed memory audit kinds × append-only login_history_geo + travel_policy_audit × RLS isolation × dry_run preview × unresolvable-IP allowed-with-sev-3), not by line targets.

---

*End of FR-AUTH-106 audit.*
