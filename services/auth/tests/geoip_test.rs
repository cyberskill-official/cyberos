//! TASK-AUTH-106 — integration test for GeoIP resolution.
//!
//! These tests exercise the **real** MaxMind path. They skip cleanly when
//! `AUTH_GEOIP_DB` is unset, so the suite stays green on dev laptops that
//! haven't run `services/auth/scripts/install-geoip.sh` yet. In CI we set
//! the env var via the workflow's GeoIP install step.

use cyberos_auth::geoip::{self, GeoIpResolver, NullResolver};
use std::sync::Arc;

fn skip_if_no_db() -> bool {
    std::env::var("AUTH_GEOIP_DB")
        .ok()
        .is_none_or(|s| s.is_empty())
}

#[test]
fn null_resolver_returns_empty_lookup_for_arbitrary_ip() {
    let r: Arc<dyn GeoIpResolver> = Arc::new(NullResolver);
    let g = r.lookup("8.8.8.8".parse().unwrap());
    assert!(g.country_iso.is_none());
    assert!(g.lat.is_none());
}

#[test]
fn from_env_falls_back_to_null_when_db_absent() {
    if !skip_if_no_db() {
        eprintln!("skipping null-fallback test — AUTH_GEOIP_DB is set");
        return;
    }
    std::env::remove_var("AUTH_GEOIP_DB");
    std::env::remove_var("AUTH_GEOIP_REQUIRED");
    let r = geoip::from_env().expect("should fall back to NullResolver");
    let g = r.lookup("8.8.8.8".parse().unwrap());
    assert!(g.country_iso.is_none(), "NullResolver must return None");
}

#[test]
fn maxmind_resolves_google_public_dns_to_us() {
    if skip_if_no_db() {
        eprintln!("skipping MaxMind test — AUTH_GEOIP_DB unset");
        return;
    }
    let r = geoip::from_env().expect("MaxMind init");
    // 8.8.8.8 is Google Public DNS — historically resolves to a US datacenter.
    let g = r.lookup("8.8.8.8".parse().unwrap());
    assert_eq!(
        g.country_iso.as_deref(),
        Some("US"),
        "expected 8.8.8.8 → US, got {:?}",
        g.country_iso
    );
    assert!(g.lat.is_some(), "MaxMind should populate lat for 8.8.8.8");
}

#[test]
fn maxmind_resolves_singapore_ip_to_sg() {
    if skip_if_no_db() {
        eprintln!("skipping MaxMind test — AUTH_GEOIP_DB unset");
        return;
    }
    let r = geoip::from_env().expect("MaxMind init");
    // 165.21.0.0 — Singapore's SingNet. Stable assignment for decades.
    let g = r.lookup("165.21.0.1".parse().unwrap());
    assert_eq!(g.country_iso.as_deref(), Some("SG"));
}

#[test]
fn haversine_singapore_to_london_in_window() {
    use cyberos_auth::geoip::haversine_km;
    let d = haversine_km((1.3521, 103.8198), (51.5074, -0.1278));
    // Real great-circle: ~10852 km. Verify within 50 km.
    assert!(
        (d - 10852.0).abs() < 50.0,
        "haversine SG↔LON should be ~10852km, got {d}"
    );
}
