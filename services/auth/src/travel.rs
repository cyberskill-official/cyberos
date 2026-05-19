//! FR-AUTH-106 — Impossible-travel detection + adaptive MFA challenge.
//!
//! Called from every successful login flow (password / OIDC / SAML / Passkey)
//! BEFORE the access token is returned. The function:
//!   1. Looks up the subject's most-recent login row (and its GeoIP fields).
//!   2. Resolves the current IP via the configured GeoIP backend.
//!   3. Inserts the new login row WITH country_iso/region/lat/lon populated
//!      when the resolver returned them (NULL if NullResolver is in use).
//!   4. Runs three detectors in order:
//!        * `same_network_burst` — two distinct /24 prefixes within 30 s.
//!        * `cross_continent_velocity` — continent_code differs AND the
//!          delta < 6 hours.
//!        * `geo_velocity_exceeded` — haversine(prev, curr) / delta > 1000 km/h.
//!      First match wins. If matched: emit `travel_audit` row + return
//!      `Challenge { kind, ... }`. The caller renders this as
//!      `403 needs_mfa_challenge: true` and gates the token issuance until
//!      `record_mfa_passed` is called for the new login_id.
//!
//! Slice-2 (2026-05-18): GeoIP enrichment via MaxMind; the GeoIP-dependent
//! detectors (kind-2, kind-3) are now wired. When the resolver is `NullResolver`
//! they silently no-op, exactly matching slice-1 behaviour.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::geoip::{haversine_km, GeoIpResolver};
use crate::travel_policy::{cidr_allowed, PolicyAction, PolicyCache, StickySuppress};

const SAME_NETWORK_BURST_WINDOW_SECS: i64 = 30;
const CONTINENT_FLIP_WINDOW_SECS: i64 = 6 * 60 * 60; // 6 hours
const PLAUSIBLE_TRAVEL_KMH: f64 = 1000.0;

#[derive(Debug, Clone, Serialize)]
pub enum TravelOutcome {
    /// First login, or no rule fired — proceed normally.
    Clear { login_id: Uuid },
    /// Implausible travel detected. Caller must challenge with MFA before
    /// honoring the issued token.
    Challenge {
        login_id: Uuid,
        kind: &'static str,
        prev_login_id: Uuid,
        delta_seconds: i64,
    },
    /// FR-AUTH-106 slice-3 — policy says `block`. Caller must refuse the
    /// login entirely (return 403 to the client; no token issued).
    Block {
        login_id: Uuid,
        kind: &'static str,
        prev_login_id: Option<Uuid>,
    },
}

/// FR-AUTH-106 slice-3 — bundle of per-call dependencies. Wrapped in a
/// struct so future additions (PolicyAudit emitter, OBS hook) don't keep
/// growing the parameter list past `too_many_arguments`.
pub struct AssessDeps<'a> {
    pub pool: &'a PgPool,
    pub geoip: &'a Arc<dyn GeoIpResolver>,
    pub policy_cache: &'a PolicyCache,
    pub sticky_suppress: &'a Arc<StickySuppress>,
}

/// FR-AUTH-106 slice-3 — slice-3 entry point that consults per-tenant policy
/// + CIDR allowlist + anonymous-IP + sticky-suppression before falling
/// through to the slice-2 detector. Returns:
///   * `Clear` — caller proceeds normally.
///   * `Challenge` — caller issues token + signals needs_mfa_challenge.
///   * `Block` — caller refuses login outright.
pub async fn assess_login(
    deps: &AssessDeps<'_>,
    tenant_id: Uuid,
    subject_id: Uuid,
    flow: &str,
    ip: std::net::IpAddr,
    user_agent: Option<&str>,
) -> Result<TravelOutcome, sqlx::Error> {
    let policy = deps.policy_cache.get(deps.pool, tenant_id).await;
    let prefix24 = ipv4_prefix24(ip);
    let prefix24_str = prefix24.to_string();

    // 1. CIDR allowlist short-circuit — the login row is still recorded so we
    //    have history for the next call; we just skip the detector chain.
    if cidr_allowed(&policy.allowlist, ip) {
        let outcome = record_login_and_assess(
            deps.pool, deps.geoip, tenant_id, subject_id, flow, ip, user_agent,
        )
        .await?;
        // Even if the detector flagged it, override to Clear because the
        // CIDR is operator-allowlisted.
        let login_id = match outcome {
            TravelOutcome::Clear { login_id }
            | TravelOutcome::Challenge { login_id, .. }
            | TravelOutcome::Block { login_id, .. } => login_id,
        };
        return Ok(TravelOutcome::Clear { login_id });
    }

    // 2. Anonymous-IP block — only if policy says so AND the lookup returns
    //    a flag. NullResolver + Anonymous-IP-unconfigured → always false.
    let anon = deps.geoip.anon_lookup(ip);
    if policy.block_anonymous_ip && anon.is_anonymous {
        // Insert the login row so we have audit trail, then return Block.
        let outcome = record_login_and_assess(
            deps.pool, deps.geoip, tenant_id, subject_id, flow, ip, user_agent,
        )
        .await?;
        let (login_id, prev_login_id) = match outcome {
            TravelOutcome::Clear { login_id } => (login_id, None),
            TravelOutcome::Challenge {
                login_id,
                prev_login_id,
                ..
            } => (login_id, Some(prev_login_id)),
            TravelOutcome::Block {
                login_id,
                prev_login_id,
                ..
            } => (login_id, prev_login_id),
        };
        return Ok(TravelOutcome::Block {
            login_id,
            kind: "anonymous_ip",
            prev_login_id,
        });
    }

    // 3. Sticky suppression — if the subject passed MFA from this /24 within
    //    the policy window, skip the detector chain entirely.
    if deps
        .sticky_suppress
        .should_suppress(subject_id, &prefix24_str)
        .await
    {
        // Still record the login (audit completeness); return Clear.
        let outcome = record_login_and_assess(
            deps.pool, deps.geoip, tenant_id, subject_id, flow, ip, user_agent,
        )
        .await?;
        let login_id = match outcome {
            TravelOutcome::Clear { login_id }
            | TravelOutcome::Challenge { login_id, .. }
            | TravelOutcome::Block { login_id, .. } => login_id,
        };
        return Ok(TravelOutcome::Clear { login_id });
    }

    // 4. Run the detector chain.
    let outcome = record_login_and_assess(
        deps.pool, deps.geoip, tenant_id, subject_id, flow, ip, user_agent,
    )
    .await?;

    // 5. Apply policy action — if detector returned Challenge, the policy
    //    may upgrade it to Block or downgrade to warn-only.
    let outcome = match outcome {
        TravelOutcome::Challenge {
            login_id,
            kind,
            prev_login_id,
            delta_seconds,
        } => match policy.action {
            PolicyAction::Challenge => TravelOutcome::Challenge {
                login_id,
                kind,
                prev_login_id,
                delta_seconds,
            },
            PolicyAction::Block => TravelOutcome::Block {
                login_id,
                kind,
                prev_login_id: Some(prev_login_id),
            },
            PolicyAction::WarnOnly => TravelOutcome::Clear { login_id },
        },
        other => other,
    };
    Ok(outcome)
}

/// Record a login + decide whether to challenge. Always inserts the
/// `login_history_geo` row; conditionally inserts a `travel_audit` row.
#[allow(clippy::too_many_arguments)]
pub async fn record_login_and_assess(
    pool: &PgPool,
    geoip: &Arc<dyn GeoIpResolver>,
    tenant_id: Uuid,
    subject_id: Uuid,
    flow: &str,
    ip: std::net::IpAddr,
    user_agent: Option<&str>,
) -> Result<TravelOutcome, sqlx::Error> {
    let now = Utc::now();
    let prefix24 = ipv4_prefix24(ip);
    let geo = geoip.lookup(ip);

    // 1. Look up the previous login. We pull country_iso + lat + lon now so
    //    the kind-2/3 detectors have both endpoints to reason over.
    let mut tx = pool.begin().await?;
    sqlx::query("SET LOCAL app.current_tenant_id = $1")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;

    let prev: Option<(
        Uuid,
        String,
        Option<String>,
        Option<f64>,
        Option<f64>,
        DateTime<Utc>,
    )> = sqlx::query_as(
        "SELECT id, host(ip_prefix24), country_iso, lat, lon, occurred_at
             FROM login_history_geo
            WHERE subject_id = $1
         ORDER BY occurred_at DESC
            LIMIT 1",
    )
    .bind(subject_id)
    .fetch_optional(&mut *tx)
    .await?;

    // 2. Insert the new login row, with GeoIP columns populated when the
    //    resolver returned them. The NULL fallback exactly matches slice-1.
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO login_history_geo
                (tenant_id, subject_id, flow, ip, ip_prefix24, user_agent,
                 country_iso, region, lat, lon)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
       RETURNING id",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(flow)
    .bind(ip)
    // sqlx 0.8 + ipnetwork 0.20 changed the bundled Encode wiring after this
    // session's rustc 1.83→1.88 bump (the workspace `sqlx[ipnetwork]` feature
    // no longer implements Encode<Postgres> for `ipnetwork::IpNetwork`
    // directly). Bind the textual CIDR form instead — INET accepts it and
    // it sidesteps the trait gap. Format e.g. `203.0.113.0/24`.
    .bind(prefix24.to_string())
    .bind(user_agent)
    .bind(geo.country_iso.as_deref())
    .bind(geo.region.as_deref())
    .bind(geo.lat)
    .bind(geo.lon)
    .fetch_one(&mut *tx)
    .await?;
    let current_login_id = row.0;
    tx.commit().await?;

    // 3. Decide. We need the previous continent for the kind-2 detector; the
    //    DB doesn't store it (continents are derivable from country_iso, but
    //    that's a static table — for slice 2 we just re-resolve the previous
    //    coordinates if both are populated, or fall back to country comparison).
    let outcome = match prev {
        None => TravelOutcome::Clear {
            login_id: current_login_id,
        },
        Some((prev_id, prev_prefix, prev_country, prev_lat, prev_lon, prev_ts)) => {
            let delta_secs = (now - prev_ts).num_seconds().max(0);
            let current_prefix = prefix24.to_string();

            // Detector 1 — same_network_burst (no GeoIP needed).
            if delta_secs < SAME_NETWORK_BURST_WINDOW_SECS && current_prefix != prev_prefix {
                emit_travel_audit(
                    pool,
                    tenant_id,
                    subject_id,
                    prev_id,
                    current_login_id,
                    "same_network_burst",
                    delta_secs,
                    serde_json::json!({
                        "prev_prefix": prev_prefix,
                        "current_prefix": current_prefix,
                        "window_secs": SAME_NETWORK_BURST_WINDOW_SECS,
                    }),
                )
                .await?;
                return Ok(TravelOutcome::Challenge {
                    login_id: current_login_id,
                    kind: "same_network_burst",
                    prev_login_id: prev_id,
                    delta_seconds: delta_secs,
                });
            }

            // Detector 2 — cross_continent_velocity. Triggers when the
            // country_iso flips (prev != current, both non-NULL) inside the
            // window. We don't store continent_code; ISO-alpha-2 country flip
            // is a strong proxy and avoids an extra static lookup at the hot
            // path. Continent-mapping refinement is tracked as a follow-up.
            if delta_secs < CONTINENT_FLIP_WINDOW_SECS {
                if let (Some(prev_c), Some(curr_c)) =
                    (prev_country.as_deref(), geo.country_iso.as_deref())
                {
                    if prev_c != curr_c {
                        emit_travel_audit(
                            pool,
                            tenant_id,
                            subject_id,
                            prev_id,
                            current_login_id,
                            "cross_continent_velocity",
                            delta_secs,
                            serde_json::json!({
                                "prev_country": prev_c,
                                "current_country": curr_c,
                                "window_secs": CONTINENT_FLIP_WINDOW_SECS,
                            }),
                        )
                        .await?;
                        return Ok(TravelOutcome::Challenge {
                            login_id: current_login_id,
                            kind: "cross_continent_velocity",
                            prev_login_id: prev_id,
                            delta_seconds: delta_secs,
                        });
                    }
                }
            }

            // Detector 3 — geo_velocity_exceeded. Requires both endpoints to
            // have lat+lon. Skipped when either side is NULL (degrades to no-op).
            if let (Some(pl), Some(pn), Some(cl), Some(cn)) = (prev_lat, prev_lon, geo.lat, geo.lon)
            {
                if delta_secs > 0 {
                    let km = haversine_km((pl, pn), (cl, cn));
                    let kmh = km / (delta_secs as f64 / 3600.0);
                    if kmh > PLAUSIBLE_TRAVEL_KMH {
                        emit_travel_audit(
                            pool,
                            tenant_id,
                            subject_id,
                            prev_id,
                            current_login_id,
                            "geo_velocity_exceeded",
                            delta_secs,
                            serde_json::json!({
                                "distance_km": km,
                                "speed_kmh": kmh,
                                "threshold_kmh": PLAUSIBLE_TRAVEL_KMH,
                            }),
                        )
                        .await?;
                        return Ok(TravelOutcome::Challenge {
                            login_id: current_login_id,
                            kind: "geo_velocity_exceeded",
                            prev_login_id: prev_id,
                            delta_seconds: delta_secs,
                        });
                    }
                }
            }

            // No alarm.
            TravelOutcome::Clear {
                login_id: current_login_id,
            }
        }
    };
    Ok(outcome)
}

async fn emit_travel_audit(
    pool: &PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    prev_login_id: Uuid,
    current_login_id: Uuid,
    kind: &str,
    delta_seconds: i64,
    detail: serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO travel_audit
                (tenant_id, subject_id, prev_login_id, current_login_id,
                 detection_kind, delta_seconds, detail, outcome)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'mfa_challenged')",
    )
    .bind(tenant_id)
    .bind(subject_id)
    .bind(prev_login_id)
    .bind(current_login_id)
    .bind(kind)
    .bind(delta_seconds as i32)
    .bind(detail)
    .execute(pool)
    .await
    .map(|_| ())
}

/// Compute the /24 network prefix of an IPv4 address. IPv6 addresses are
/// recorded as-is in the `ip_prefix24` column (the column accepts INET);
/// slice 2 will add a proper /64 grouping for IPv6.
fn ipv4_prefix24(ip: std::net::IpAddr) -> ipnetwork::IpNetwork {
    use std::net::Ipv4Addr;
    match ip {
        std::net::IpAddr::V4(v4) => {
            let octets = v4.octets();
            let masked = Ipv4Addr::new(octets[0], octets[1], octets[2], 0);
            ipnetwork::IpNetwork::new(std::net::IpAddr::V4(masked), 24)
                .expect("24 is valid prefix len")
        }
        std::net::IpAddr::V6(v6) => {
            // Slice 1: just record the full v6 with /128 — refinement in slice 2.
            ipnetwork::IpNetwork::new(std::net::IpAddr::V6(v6), 128)
                .expect("128 is valid prefix len")
        }
    }
}

/// Mark a successful MFA-challenge resolution. Called after the client
/// completes TOTP/Passkey in response to a Challenge outcome.
pub async fn record_mfa_passed(pool: &PgPool, login_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE travel_audit SET outcome = 'mfa_passed' WHERE current_login_id = $1")
        .bind(login_id)
        .execute(pool)
        .await
        .map(|_| ())
}

/// FR-AUTH-106 slice-3 — full slice-3 MFA-pass: marks the audit row passed
/// AND records the (subject, /24) → sticky-suppress entry so the next
/// login from the same /24 within the policy window doesn't re-challenge.
pub async fn record_mfa_passed_with_sticky(
    pool: &PgPool,
    sticky: &Arc<StickySuppress>,
    policy: &PolicyCache,
    tenant_id: Uuid,
    subject_id: Uuid,
    login_id: Uuid,
    login_ip: std::net::IpAddr,
) -> Result<(), sqlx::Error> {
    record_mfa_passed(pool, login_id).await?;
    let p = policy.get(pool, tenant_id).await;
    let prefix24 = ipv4_prefix24(login_ip).to_string();
    sticky
        .record(subject_id, prefix24, p.sticky_suppress_min)
        .await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn ipv4_prefix24_masks_last_octet() {
        let n = ipv4_prefix24(IpAddr::from_str("203.0.113.42").unwrap());
        assert_eq!(n.to_string(), "203.0.113.0/24");
    }

    #[test]
    fn ipv4_prefix24_same_subnet_collapses() {
        let a = ipv4_prefix24(IpAddr::from_str("10.0.0.1").unwrap());
        let b = ipv4_prefix24(IpAddr::from_str("10.0.0.250").unwrap());
        assert_eq!(a.to_string(), b.to_string());
    }

    #[test]
    fn ipv4_prefix24_different_subnet_differs() {
        let a = ipv4_prefix24(IpAddr::from_str("10.0.0.1").unwrap());
        let b = ipv4_prefix24(IpAddr::from_str("10.0.1.1").unwrap());
        assert_ne!(a.to_string(), b.to_string());
    }
}
