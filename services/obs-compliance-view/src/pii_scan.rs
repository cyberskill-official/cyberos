//! Defence-in-depth PII scan (TASK-OBS-008 §1 #12). The audit chain already stores placeholders, not raw
//! PII (`email_hash16`, `<VN_CCCD_1>`), so a compliance response should never contain raw PII. Before a
//! view is served, its rendered body is scanned; any match is a sev-1 and a 500 rather than a leak. This
//! is the pure detector - the HTTP layer decides what to do with a non-empty result.

use std::sync::OnceLock;

use regex::Regex;

/// A raw-PII match: the kind and the byte offset in the scanned body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiiMatch {
    pub kind: &'static str,
    pub at: usize,
}

fn email_re() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"[a-zA-Z0-9._%+\-]+@[a-zA-Z0-9.\-]+\.[a-zA-Z]{2,}").unwrap())
}

fn vn_cccd_re() -> &'static Regex {
    // The VN citizen ID (CCCD) is exactly 12 digits; the chain stores it as <VN_CCCD_n>.
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"\b\d{12}\b").unwrap())
}

fn vn_phone_re() -> &'static Regex {
    // A VN mobile number: +84 then 9 digits, or 0 then 9 digits.
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"(?:\+84|0)\d{9}\b").unwrap())
}

/// Scan `body` for raw PII patterns the audit chain should never contain. Returns every match, sorted by
/// offset; an empty result means the body is clean to serve.
pub fn scan(body: &str) -> Vec<PiiMatch> {
    let mut out = Vec::new();
    for (re, kind) in [
        (email_re(), "email"),
        (vn_cccd_re(), "vn_cccd"),
        (vn_phone_re(), "vn_phone"),
    ] {
        for m in re.find_iter(body) {
            out.push(PiiMatch {
                kind,
                at: m.start(),
            });
        }
    }
    out.sort_by_key(|m| m.at);
    out
}

/// True if the body carries no raw PII and is safe to serve.
pub fn is_clean(body: &str) -> bool {
    scan(body).is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_placeholdered_response_is_clean() {
        // The shapes the chain actually stores - no raw PII.
        let body = r#"{"rows":[{"actor":"email_hash16:a1b2c3d4","cccd":"<VN_CCCD_1>","trace_id":"7f3ac9be12d4"}]}"#;
        assert!(is_clean(body), "matches: {:?}", scan(body));
    }

    #[test]
    fn raw_email_is_caught() {
        let m = scan(r#"{"actor":"stephen@cyberskill.world"}"#);
        assert!(m.iter().any(|x| x.kind == "email"));
    }

    #[test]
    fn raw_vn_cccd_is_caught() {
        let m = scan(r#"{"cccd":"079123456789"}"#);
        assert!(m.iter().any(|x| x.kind == "vn_cccd"));
    }

    #[test]
    fn raw_vn_phone_is_caught() {
        assert!(scan("call +84906878091 now")
            .iter()
            .any(|x| x.kind == "vn_phone"));
        assert!(scan("call 0906878091 now")
            .iter()
            .any(|x| x.kind == "vn_phone"));
    }
}
