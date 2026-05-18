//! FR-AUTH-106 slice 2 — GeoIP resolver. Pluggable behind a trait so production
//! reads MaxMind GeoLite2-City.mmdb, dev/test uses a hand-rolled in-memory
//! map, and a null resolver lets the rest of AUTH degrade gracefully when no
//! DB is configured.
//!
//! Wired into `travel::record_login_and_assess` so `country_iso` + `lat` +
//! `lon` are populated on every `login_history_geo` insert. Once those columns
//! are non-null on two consecutive rows, the kind-2 (`cross_continent_velocity`)
//! and kind-3 (`geo_velocity_exceeded`) detectors fire.
//!
//! Environment:
//!   * `AUTH_GEOIP_DB`  — absolute path to `GeoLite2-City.mmdb`. Optional;
//!                       absent ⇒ NullResolver (degradation matches slice-1).
//!   * `AUTH_GEOIP_REQUIRED=1` — fail startup if `AUTH_GEOIP_DB` is absent or
//!                              unreadable. Used in production envs where we
//!                              don't want silent slice-1 degradation.

use std::net::IpAddr;
use std::sync::Arc;

#[derive(Debug, Clone, Default)]
pub struct GeoLookup {
    pub country_iso: Option<String>, // ISO 3166-1 alpha-2 — e.g. "VN", "US", "DE"
    pub region: Option<String>,      // subdivision name, may be None
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    /// Two-letter continent code — "AS"/"EU"/"NA"/"SA"/"AF"/"OC"/"AN".
    /// We don't write this to DB but the kind-2 detector uses it.
    pub continent: Option<String>,
}

/// FR-AUTH-106 slice-3 — Anonymous-IP DB lookup result. MaxMind's
/// GeoIP2-Anonymous-IP DB flags an IP as VPN, hosting provider, public
/// proxy, residential proxy, or Tor exit. Any TRUE flag is considered
/// "anonymous" for the purposes of `block_anonymous_ip`.
#[derive(Debug, Clone, Copy, Default)]
pub struct AnonLookup {
    pub is_anonymous: bool,
    pub is_vpn: bool,
    pub is_tor_exit: bool,
    pub is_hosting_provider: bool,
}

pub trait GeoIpResolver: Send + Sync + 'static {
    fn lookup(&self, ip: IpAddr) -> GeoLookup;
    /// Anonymous-IP lookup. Default impl returns "not anonymous" so the
    /// NullResolver and the test resolver don't need to override.
    fn anon_lookup(&self, _ip: IpAddr) -> AnonLookup {
        AnonLookup::default()
    }
}

// ---------------------------------------------------------------------------
// NullResolver — no-op, matches slice-1 behaviour (everything stays NULL).
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NullResolver;

impl GeoIpResolver for NullResolver {
    fn lookup(&self, _ip: IpAddr) -> GeoLookup {
        GeoLookup::default()
    }
}

// ---------------------------------------------------------------------------
// MaxMind GeoLite2-City reader.
// ---------------------------------------------------------------------------

pub struct MaxMindResolver {
    reader: maxminddb::Reader<Vec<u8>>,
    /// Optional Anonymous-IP reader. None when AUTH_GEOIP_ANONYMOUS_DB is
    /// unset — VPN/Tor flagging then quietly returns "not anonymous".
    anon_reader: Option<maxminddb::Reader<Vec<u8>>>,
}

impl MaxMindResolver {
    pub fn from_path(path: &str) -> Result<Self, GeoIpError> {
        let reader = maxminddb::Reader::open_readfile(path)
            .map_err(|e| GeoIpError::Open(format!("opening {path}: {e}")))?;
        Ok(Self { reader, anon_reader: None })
    }

    /// Attach the Anonymous-IP DB. Optional — when absent, `anon_lookup`
    /// quietly returns "not anonymous" and the `block_anonymous_ip` policy
    /// is effectively no-op even when enabled.
    pub fn with_anonymous_db(mut self, path: &str) -> Result<Self, GeoIpError> {
        let r = maxminddb::Reader::open_readfile(path)
            .map_err(|e| GeoIpError::Open(format!("opening {path}: {e}")))?;
        self.anon_reader = Some(r);
        Ok(self)
    }
}

impl GeoIpResolver for MaxMindResolver {
    fn lookup(&self, ip: IpAddr) -> GeoLookup {
        // GeoLite2-City record shape — we only pull the fields we record.
        let record: Result<maxminddb::geoip2::City, _> = self.reader.lookup(ip);
        match record {
            Ok(city) => {
                let country_iso = city
                    .country
                    .as_ref()
                    .and_then(|c| c.iso_code.as_ref())
                    .map(|s| s.to_string());
                let continent = city
                    .continent
                    .as_ref()
                    .and_then(|c| c.code.as_ref())
                    .map(|s| s.to_string());
                let region = city
                    .subdivisions
                    .as_ref()
                    .and_then(|subs| subs.first())
                    .and_then(|s| s.names.as_ref())
                    .and_then(|n| n.get("en"))
                    .map(|s| s.to_string());
                let (lat, lon) = city
                    .location
                    .as_ref()
                    .map(|l| (l.latitude, l.longitude))
                    .unwrap_or((None, None));
                GeoLookup { country_iso, region, lat, lon, continent }
            }
            Err(_) => GeoLookup::default(),
        }
    }

    fn anon_lookup(&self, ip: IpAddr) -> AnonLookup {
        let Some(r) = self.anon_reader.as_ref() else {
            return AnonLookup::default();
        };
        let rec: Result<maxminddb::geoip2::AnonymousIp, _> = r.lookup(ip);
        match rec {
            Ok(a) => AnonLookup {
                is_anonymous: a.is_anonymous.unwrap_or(false),
                is_vpn: a.is_anonymous_vpn.unwrap_or(false),
                is_tor_exit: a.is_tor_exit_node.unwrap_or(false),
                is_hosting_provider: a.is_hosting_provider.unwrap_or(false),
            },
            Err(_) => AnonLookup::default(),
        }
    }
}

// ---------------------------------------------------------------------------
// In-memory test resolver — handy for tests + dev without an mmdb file.
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod testing {
    use super::*;
    use std::collections::HashMap;

    pub struct StaticResolver(pub HashMap<IpAddr, GeoLookup>);
    impl GeoIpResolver for StaticResolver {
        fn lookup(&self, ip: IpAddr) -> GeoLookup {
            self.0.get(&ip).cloned().unwrap_or_default()
        }
    }
}

// ---------------------------------------------------------------------------
// Factory — honours AUTH_GEOIP_DB / AUTH_GEOIP_REQUIRED.
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum GeoIpError {
    #[error("opening GeoIP DB: {0}")]
    Open(String),
    #[error("AUTH_GEOIP_DB unset but AUTH_GEOIP_REQUIRED=1")]
    Required,
}

pub fn from_env() -> Result<Arc<dyn GeoIpResolver>, GeoIpError> {
    let required = std::env::var("AUTH_GEOIP_REQUIRED")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    match std::env::var("AUTH_GEOIP_DB") {
        Ok(path) if !path.is_empty() => {
            let mut r = MaxMindResolver::from_path(&path)?;
            tracing::info!(path = %path, "GeoIP enrichment enabled (MaxMind)");
            // Optional Anonymous-IP DB — when present, VPN/Tor flagging
            // activates. Per FR-AUTH-106 slice-3, this is supplementary —
            // missing DB does not error.
            if let Ok(anon_path) = std::env::var("AUTH_GEOIP_ANONYMOUS_DB") {
                if !anon_path.is_empty() {
                    r = r.with_anonymous_db(&anon_path)?;
                    tracing::info!(path = %anon_path, "Anonymous-IP DB loaded — VPN/Tor flagging active");
                }
            }
            Ok(Arc::new(r))
        }
        _ if required => Err(GeoIpError::Required),
        _ => {
            tracing::info!("GeoIP enrichment disabled (AUTH_GEOIP_DB unset) — kind-2/3 detectors inactive");
            Ok(Arc::new(NullResolver))
        }
    }
}

// ---------------------------------------------------------------------------
// Great-circle distance helper — used by the kind-3 detector. Implemented
// here (not in travel.rs) so the geo module owns all coordinate math.
// ---------------------------------------------------------------------------

/// Haversine distance in kilometres between two (lat, lon) points.
pub fn haversine_km(a: (f64, f64), b: (f64, f64)) -> f64 {
    const R_KM: f64 = 6371.0088; // mean Earth radius (km)
    let (lat1, lon1) = (a.0.to_radians(), a.1.to_radians());
    let (lat2, lon2) = (b.0.to_radians(), b.1.to_radians());
    let dlat = lat2 - lat1;
    let dlon = lon2 - lon1;
    let h =
        (dlat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (dlon / 2.0).sin().powi(2);
    2.0 * R_KM * h.sqrt().asin()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn haversine_zero_distance() {
        assert!(haversine_km((10.0, 20.0), (10.0, 20.0)).abs() < 1e-9);
    }

    #[test]
    fn haversine_singapore_to_london_approx_10860km() {
        // SG: 1.3521 N, 103.8198 E ; LON: 51.5074 N, -0.1278 W
        let d = haversine_km((1.3521, 103.8198), (51.5074, -0.1278));
        assert!((d - 10850.0).abs() < 50.0, "got {d} km");
    }

    #[test]
    fn null_resolver_returns_empty_lookup() {
        let r = NullResolver;
        let g = r.lookup("203.0.113.42".parse().unwrap());
        assert!(g.country_iso.is_none() && g.lat.is_none() && g.continent.is_none());
    }

    #[test]
    fn from_env_with_no_db_returns_null_resolver() {
        std::env::remove_var("AUTH_GEOIP_DB");
        std::env::remove_var("AUTH_GEOIP_REQUIRED");
        let r = from_env().expect("should fall back to NullResolver");
        let g = r.lookup("203.0.113.42".parse().unwrap());
        assert!(g.country_iso.is_none());
    }

    #[test]
    fn from_env_required_without_db_errors() {
        std::env::remove_var("AUTH_GEOIP_DB");
        std::env::set_var("AUTH_GEOIP_REQUIRED", "1");
        // Can't use `.unwrap_err()` because `Arc<dyn GeoIpResolver>` is not Debug.
        // Pattern-match the Result directly instead.
        match from_env() {
            Err(GeoIpError::Required) => {} // expected
            Err(other) => panic!("expected GeoIpError::Required, got {other:?}"),
            Ok(_) => panic!("expected error, got Ok resolver"),
        }
        std::env::remove_var("AUTH_GEOIP_REQUIRED");
    }
}
