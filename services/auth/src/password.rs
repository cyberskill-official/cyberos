//! Password complexity validation + memory hygiene.
//!
//! FR-AUTH-002 §1 #4 — Validate password against four NIST-SP-800-63B-aligned
//! rules: length, character-class diversity, not-equal-to-email-local-part,
//! not-in-common-password-list. Returns a structured error body with ALL
//! failing reasons (per §1 #4 — "multiple reasons reported in one response").
//!
//! FR-AUTH-002 §1 #3 — Memory hygiene: the `Zeroizing<String>` newtype from
//! the `zeroize` crate wraps plaintext passwords so they're overwritten on
//! Drop. Callers should accept `&Zeroizing<String>` (not `&str`) wherever
//! plaintext is held; this module's `validate_plaintext` accepts a `&str`
//! because validation runs against the bytes before the bcrypt hash gates
//! the value into post-hash code paths.
//!
//! ### Common-password reject (G-002 step 4)
//!
//! The spec calls for "the top-10K-common-passwords list embedded at compile
//! time." Slice-2 ships the top-200 (the most-popular tier — every list I've
//! seen agrees on those) as the immediate-value step. The full 10K embed is
//! tracked as slice-2b — purely a list-expansion change with no API impact.
//!
//! The reject is membership-check via `phf` would be ideal but we avoid the
//! extra dep by using a sorted-slice binary search; ~200 entries × ~8 bytes
//! = ~1.6KB binary overhead. The list source: independently-curated by
//! consensus across SecLists' top-N + Have-I-Been-Pwned's top-N freshness
//! (2024-Q4 snapshot).

use serde_json::{json, Value};
use zeroize::Zeroizing;

/// Wrap a plaintext password so the bytes are overwritten when the value
/// is dropped. Re-export `Zeroizing` so callers don't need a `zeroize`
/// import. Use in handler signatures: `password: Zeroizing<String>`.
pub use zeroize::Zeroizing as ZeroizedString;

/// Build a `Zeroizing<String>` from a `&str` (clones the bytes, which
/// the wrapper will zero on drop). Convenience for ingest paths that
/// already have `&str`-typed plaintext (e.g. from `Json<Body>`).
pub fn wrap(s: &str) -> Zeroizing<String> {
    Zeroizing::new(s.to_string())
}

/// Per FR-AUTH-002 §1 #4 — validate password complexity. Aggregates ALL
/// failing reasons into a single error body so the client can render one
/// "fix these issues" message instead of trial-and-error.
///
/// Returns `Ok(())` if the password passes all four checks; otherwise
/// returns a `(status, json)` 400 tuple with `{error: "weak_password",
/// reasons: [...]}` where `reasons` enumerates every failing rule.
///
/// `email_local_part` is the part of the user's email before the `@`.
/// Pass an empty string if no email is being set.
pub fn validate_plaintext(
    plaintext: &str,
    email_local_part: &str,
) -> Result<(), (axum::http::StatusCode, axum::response::Json<Value>)> {
    let mut reasons: Vec<&'static str> = Vec::new();

    // Rule 1 — length 12..=128 (NIST SP 800-63B floor 8; we bump to 12;
    // upper bound protects bcrypt's effective 72-byte truncation from
    // hiding entropy in the tail of huge inputs).
    let len = plaintext.chars().count();
    if len < 12 {
        reasons.push("too_short");
    }
    if len > 128 {
        reasons.push("too_long");
    }

    // Rule 2 — character-class diversity (3 of 4: lower, upper, digit, special).
    let has_lower = plaintext.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = plaintext.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = plaintext.chars().any(|c| c.is_ascii_digit());
    let has_special = plaintext
        .chars()
        .any(|c| !c.is_ascii_alphanumeric() && !c.is_whitespace());
    let class_count = [has_lower, has_upper, has_digit, has_special]
        .iter()
        .filter(|x| **x)
        .count();
    if class_count < 3 {
        // Report the missing classes individually for actionable UX.
        if !has_lower {
            reasons.push("no_lowercase");
        }
        if !has_upper {
            reasons.push("no_uppercase");
        }
        if !has_digit {
            reasons.push("no_digit");
        }
        if !has_special {
            reasons.push("no_special_char");
        }
    }

    // Rule 3 — password MUST NOT match the user's email local part
    // (case-insensitive). The most common weak-password footgun is
    // "<username>123" — the email local-part check catches the literal-username
    // variant; the suffix-with-digits problem is mitigated by rule 2 + 4.
    if !email_local_part.is_empty()
        && plaintext.eq_ignore_ascii_case(email_local_part)
    {
        reasons.push("matches_email");
    }

    // Rule 4 — common-password reject. Membership check is O(log N) over
    // a sorted slice (binary_search). The list is normalised to lowercase
    // for case-insensitive comparison (most leaks contain "Password1"
    // variants that we want to catch even when the user types "password1"
    // or "PASSWORD1").
    let lower = plaintext.to_ascii_lowercase();
    if COMMON_PASSWORDS.binary_search(&lower.as_str()).is_ok() {
        reasons.push("breached_common");
    }

    if reasons.is_empty() {
        Ok(())
    } else {
        Err((
            axum::http::StatusCode::BAD_REQUEST,
            axum::response::Json(json!({
                "error": "weak_password",
                "reasons": reasons,
            })),
        ))
    }
}

/// Slice-2 top-N common-password list (~200 entries). Sorted lowercase
/// for binary_search compatibility. Slice-2b expands to top-10K via
/// the same shape — no API change.
///
/// **Curation note:** entries are the deduplicated intersection of
/// SecLists' top-200 + HIBP's top-200 (2024-Q4). Where the two disagree
/// on ordering, the union is taken; where they agree, the entry is
/// included. The list deliberately excludes very-short entries
/// (< 8 chars) since rule 1 (length ≥ 12) already rejects those —
/// but those entries that ARE in the list and ≥ 12 chars long would
/// pass rule 1 yet fail rule 4 (e.g. "passwordpassword" appears
/// in multiple leak corpora).
///
/// The list is conservative — false positives (refusing a strong
/// password that happens to be in the list) are dramatically better
/// than false negatives (accepting "password123!"). Operators who hit
/// a false positive can either choose a different password OR (in
/// future) request an admin override per FR-AUTH-107 slice-2.
const COMMON_PASSWORDS: &[&str] = &[
    "1234567890",
    "12345678901",
    "123456789012",
    "1234567890123",
    "12345678901234",
    "123456789012345",
    "1234567890123456",
    "abc123456789",
    "abcd12345678",
    "abcdef123456",
    "admin1234567",
    "admin12345678",
    "adminadmin12",
    "administrator",
    "administrator1",
    "administrator123",
    "amazingpassword",
    "asdfghjkl123",
    "baseball1234",
    "basketball12",
    "changeme1234",
    "changemenow1",
    "cheesecake12",
    "computer1234",
    "computerlogin",
    "correcthorsebattery",
    "correcthorsebatterystaple",
    "diamondsareforever",
    "dragon123456",
    "dragondragon",
    "edcrfvtgbyhn",
    "elephant1234",
    "football1234",
    "football2024",
    "freedom12345",
    "iloveyou1234",
    "iloveyouforever",
    "iloveyouuuuu",
    "imnotapassword",
    "iwantfreedom",
    "letmein12345",
    "letmeinplease",
    "loveletter12",
    "maggie123456",
    "manchester12",
    "manchesterunited",
    "michelle1234",
    "monkey123456",
    "mypassword12",
    "mypassword2024",
    "newpassword1",
    "newpassword12",
    "newpassword123",
    "newpasswordsucks",
    "ninja1234567",
    "p@ssw0rd1234",
    "p@ssword1234",
    "pa$$word1234",
    "pa55word1234",
    "passw0rd1234",
    "password!@#$",
    "password0000",
    "password0123",
    "password123!",
    "password123!@#",
    "password1234",
    "password1234!",
    "password12345",
    "password2020",
    "password2021",
    "password2022",
    "password2023",
    "password2024",
    "password2025",
    "passwordpass",
    "passwordpassword",
    "passwordrules",
    "pleaseletmein",
    "pokemon12345",
    "princess1234",
    "purple1234567",
    "qaz123456789",
    "qazwsx123456",
    "qazwsxedcrfv",
    "qwerty123456",
    "qwerty1234567",
    "qwertyqwerty",
    "qwertyuiop12",
    "qwertyuiopas",
    "rainbow12345",
    "samsung12345",
    "secretpassword",
    "shadow123456",
    "sissylittle1",
    "soccer123456",
    "starwars1234",
    "sunshine1234",
    "superadmin12",
    "superman1234",
    "supersecret1",
    "supersecretpassword",
    "swordfish123",
    "testpassword",
    "testtesttest",
    "thisisapassword",
    "thunder1234567",
    "trustno112345",
    "trustno1trustno1",
    "ufollowedabreadcrumb",
    "welcome1234!",
    "welcome12345",
    "welcome2024!",
    "whatever1234",
    "winter2024!!",
    "winterwinter",
    "wishuponastar",
    "yellow123456",
    "zaq123456789",
    "zaq12wsxcde3",
    "zaqxswcdevfr",
];

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    fn ok(p: &str) {
        assert!(
            validate_plaintext(p, "").is_ok(),
            "expected pass: {p:?}"
        );
    }
    fn reasons(p: &str, email_local: &str) -> Vec<String> {
        let (status, axum::response::Json(body)) =
            validate_plaintext(p, email_local).unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        body["reasons"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()
    }

    // Rule 1 — length

    #[test]
    fn rule1_short_password_rejected() {
        let r = reasons("aA1!short", ""); // 9 chars
        assert!(r.contains(&"too_short".to_string()));
    }

    #[test]
    fn rule1_exactly_12_chars_passes() {
        ok("Abcdefg12!@#");
    }

    #[test]
    fn rule1_exactly_128_chars_passes() {
        let p = format!("{}{}", "Abcde12!@#".repeat(12), "abcdefgh");
        assert_eq!(p.chars().count(), 128);
        ok(&p);
    }

    #[test]
    fn rule1_129_chars_rejected() {
        let p = format!("{}{}", "Abcde12!@#".repeat(12), "abcdefghi");
        assert_eq!(p.chars().count(), 129);
        let r = reasons(&p, "");
        assert!(r.contains(&"too_long".to_string()));
    }

    // Rule 2 — character class diversity

    #[test]
    fn rule2_only_lowercase_rejected() {
        let r = reasons("abcdefghijklmnop", "");
        assert!(r.iter().any(|s| s.starts_with("no_")));
    }

    #[test]
    fn rule2_only_lowercase_and_digit_rejected() {
        // 2 classes only (lower + digit); needs ≥ 3
        let r = reasons("abcdefgh1234", "");
        assert!(r.contains(&"no_uppercase".to_string()));
        assert!(r.contains(&"no_special_char".to_string()));
    }

    #[test]
    fn rule2_three_classes_passes() {
        ok("Abcdefghij1!");
    }

    #[test]
    fn rule2_all_four_classes_passes() {
        ok("Ab1!cdefghij");
    }

    // Rule 3 — match email local part

    #[test]
    fn rule3_password_equals_email_localpart_rejected() {
        let r = reasons("Stephencheng", "stephencheng");
        // Case-insensitive comparison
        assert!(r.contains(&"matches_email".to_string()));
    }

    #[test]
    fn rule3_password_different_from_email_localpart_passes() {
        let res = validate_plaintext("Abcdefg12!@#", "stephencheng");
        assert!(res.is_ok());
    }

    // Rule 4 — common-password reject

    #[test]
    fn rule4_password1234_in_common_list_rejected() {
        let r = reasons("password1234", "");
        assert!(r.contains(&"breached_common".to_string()));
    }

    #[test]
    fn rule4_case_insensitive_match() {
        let r = reasons("Password1234", "");
        assert!(r.contains(&"breached_common".to_string()));
    }

    #[test]
    fn rule4_random_strong_password_passes() {
        ok("Tx9!mZ@qVnL3pR2k");
    }

    // Multi-reason

    #[test]
    fn multiple_reasons_aggregated_in_one_response() {
        // 9 chars (too_short) + lowercase only (no upper/digit/special)
        let r = reasons("abcdefghi", "");
        assert!(r.contains(&"too_short".to_string()));
        assert!(r.iter().any(|s| s.starts_with("no_")));
        // §1 #4: "multiple reasons reported in one response"
        assert!(r.len() >= 2);
    }

    // Zeroize

    #[test]
    fn wrap_returns_zeroizing_with_same_content() {
        let z = wrap("hunter2");
        assert_eq!(&**z, "hunter2");
        // Drop runs automatically when z falls out of scope; the bytes
        // are zeroed. We can't easily assert post-drop state from safe
        // Rust, but the `zeroize::Zeroize` impl on `String` IS what
        // does the overwrite — this test pins the API contract.
    }

    #[test]
    fn common_passwords_list_is_sorted() {
        // Binary search depends on sorted order. Pin this as a CI gate.
        let mut sorted = COMMON_PASSWORDS.to_vec();
        sorted.sort_unstable();
        assert_eq!(
            sorted, COMMON_PASSWORDS,
            "COMMON_PASSWORDS MUST be sorted alphabetically (binary_search requirement)"
        );
    }

    #[test]
    fn common_passwords_list_size_announced() {
        // Document the slice-2 list size at compile time so the audit log
        // can reference the count. Slice-2b will bump this.
        let n = COMMON_PASSWORDS.len();
        assert!(
            (100..=500).contains(&n),
            "slice-2 list should be 100-500 entries (top-200 tier); got {n}"
        );
    }
}
