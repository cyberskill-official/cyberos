//! FR-AUTH-107 — Have I Been Pwned breach check (k-anonymity protocol).
//!
//! Computes SHA-1(password) → splits into 5-char prefix + 35-char suffix.
//! Sends the 5-char prefix to <https://api.pwnedpasswords.com/range/{prefix}>.
//! HIBP responds with all 35-char suffixes that share the prefix + their
//! observed breach counts. We scan for our suffix locally; never send the
//! full hash, never send the password.
//!
//! The k-anonymity protocol is documented at
//! <https://haveibeenpwned.com/API/v3#PwnedPasswords>. The protocol design
//! is the documented PDPL Art. 25 + GDPR Art. 25 data-minimisation alignment.

use sha1::{Digest, Sha1};
use std::time::Duration;
use thiserror::Error;

const HIBP_RANGE_URL: &str = "https://api.pwnedpasswords.com/range/";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Error)]
pub enum HibpError {
    #[error("network: {0}")]
    Network(String),
    #[error("non-200 from HIBP: {0}")]
    BadStatus(u16),
}

/// Outcome of a breach check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HibpOutcome {
    /// Password is not in any known breach.
    Allowed,
    /// Password was found; carries the observed count (operator may surface).
    Breached { count: u64 },
    /// HIBP API was unreachable. Per policy, treat as ALLOWED but audit the
    /// failure — fail-open on transient network issues, fail-closed on persistent.
    ApiUnreachable,
}

/// Compute the 5-char SHA-1 prefix + 35-char suffix used by the k-anonymity
/// protocol. Stable + side-effect-free; used for logging without leaking the
/// full hash.
pub fn sha1_split(password: &str) -> (String, String) {
    let mut h = Sha1::new();
    h.update(password.as_bytes());
    let hex: String = h.finalize().iter().map(|b| format!("{:02X}", b)).collect();
    (hex[..5].to_string(), hex[5..].to_string())
}

/// Hit the HIBP range endpoint. Returns the body as text or an error.
pub async fn fetch_range(prefix: &str) -> Result<String, HibpError> {
    if prefix.len() != 5 || !prefix.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(HibpError::Network(format!("bad prefix: {prefix:?}")));
    }
    let url = format!("{HIBP_RANGE_URL}{prefix}");
    let client = reqwest::Client::builder()
        .timeout(REQUEST_TIMEOUT)
        .user_agent("cyberos-auth/0.1 (+https://cyberos.cyberskill.world)")
        .build()
        .map_err(|e| HibpError::Network(e.to_string()))?;
    let resp = client
        .get(&url)
        .header("Add-Padding", "true") // HIBP recommendation: prevents traffic-pattern fingerprinting
        .send()
        .await
        .map_err(|e| HibpError::Network(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(HibpError::BadStatus(resp.status().as_u16()));
    }
    resp.text().await.map_err(|e| HibpError::Network(e.to_string()))
}

/// Scan an HIBP range body for our suffix. Returns Some(count) if found.
pub fn scan_for_suffix(body: &str, suffix: &str) -> Option<u64> {
    for line in body.lines() {
        // HIBP returns lines like `<35-char suffix>:<count>` (uppercase suffix).
        let (s, count) = match line.split_once(':') {
            Some(t) => t,
            None => continue,
        };
        if s.trim().eq_ignore_ascii_case(suffix) {
            return count.trim().parse::<u64>().ok();
        }
    }
    None
}

/// End-to-end check. Returns `HibpOutcome`. Never panics; network failures map
/// to `ApiUnreachable` (caller decides whether to allow or block).
pub async fn check_password(password: &str) -> HibpOutcome {
    let (prefix, suffix) = sha1_split(password);
    match fetch_range(&prefix).await {
        Ok(body) => match scan_for_suffix(&body, &suffix) {
            Some(count) => HibpOutcome::Breached { count },
            None => HibpOutcome::Allowed,
        },
        Err(_) => HibpOutcome::ApiUnreachable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_split_matches_known_vector() {
        // SHA-1("password") = 5BAA61E4C9B93F3F0682250B6CF8331B7EE68FD8
        let (prefix, suffix) = sha1_split("password");
        assert_eq!(prefix, "5BAA6");
        assert_eq!(suffix, "1E4C9B93F3F0682250B6CF8331B7EE68FD8");
    }

    #[test]
    fn scan_for_suffix_finds_uppercase_match() {
        let body = "1E4C9B93F3F0682250B6CF8331B7EE68FD8:9659365\nOTHERSUFFIX:123";
        assert_eq!(
            scan_for_suffix(body, "1E4C9B93F3F0682250B6CF8331B7EE68FD8"),
            Some(9659365)
        );
    }

    #[test]
    fn scan_for_suffix_is_case_insensitive() {
        let body = "1e4c9b93f3f0682250b6cf8331b7ee68fd8:42";
        assert_eq!(
            scan_for_suffix(body, "1E4C9B93F3F0682250B6CF8331B7EE68FD8"),
            Some(42)
        );
    }

    #[test]
    fn scan_returns_none_when_absent() {
        let body = "OTHERSUFFIX:123\nANOTHER:456";
        assert_eq!(scan_for_suffix(body, "NOTPRESENT"), None);
    }

    #[test]
    fn scan_handles_garbage_lines() {
        let body = "garbage\n\nDEADBEEF:notanumber\nABCDEF:7";
        assert_eq!(scan_for_suffix(body, "ABCDEF"), Some(7));
    }
}
