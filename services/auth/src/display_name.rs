//! FR-AUTH-111 — where a person's display name comes from, for every SSO path.
//!
//! Today `oidc.rs` and `saml.rs` both bind the person's EMAIL into `subjects.display_name`. Everyone
//! provisioned through Google therefore renders as `van-anh.vu@cyberskill.world` wherever a name belongs:
//! the channel list, above every message they have sent, mentions, the member picker, and any screenshot or
//! export. It reads as broken software, it puts an address where only a name was intended, and it
//! contradicts the privacy policy we have already published — which tells people we take their name from
//! their Google account. We do receive it. We then drop it on the floor.
//!
//! This module exists so the fix has exactly ONE implementation. FR-AUTH-111 §1 #6 requires the same
//! resolution chain and the same no-clobber rule in both `oidc.rs` and `saml.rs`; two copies of a rule are
//! two rules, and they drift. Both call sites import from here.
//!
//! Two things are deliberately absent, and both are load-bearing:
//!
//! * **No transformation.** The resolver copies what the IdP said and never prettifies, title-cases, or
//!   splits on separators (§1 #3). If Google says the person is called `nguyenvana`, that is their name.
//!   Guessing at capitalisation is how a product mangles Vietnamese names, and this is a Vietnamese company.
//!
//! * **No `picture`.** The claim is right there in the ID token and an avatar would make the product nicer.
//!   Persisting it would silently widen the published Data Safety declaration and the privacy policy, both
//!   of which enumerate what we collect. That is a decision with paperwork attached, not a line of code
//!   (§1 #9). `Profile` has no field to deserialise it into, which is the point.

use uuid::Uuid;

/// The name-bearing claims we read. Standard OIDC (`name`, `given_name`, `family_name`,
/// `preferred_username`); the SAML side maps its attributes onto the same shape so one chain serves both.
#[derive(Debug, Default, serde::Deserialize)]
pub struct Profile {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub given_name: Option<String>,
    #[serde(default)]
    pub family_name: Option<String>,
    #[serde(default)]
    pub preferred_username: Option<String>,
}

/// §1 #2 — the resolution chain. Ordered, total, and deliberately dumb.
///
/// Returns the RUNG that matched alongside the value, so the caller can log how it resolved without logging
/// what it resolved (§1 #8). A `tracing::debug!(?name)` is a privacy incident; a `debug!(rung)` is a
/// diagnostic. Getting both properties costs one `&'static str`.
///
/// The final rung returns an empty string when there is no email at all. The caller substitutes the handle:
/// a person rendering as `@oidc-1043…` is poor, and rendering as nothing at all is worse.
///
/// `name` is the first rung, not the `given`+`family` join, and that ordering is not arbitrary. Vietnamese
/// convention puts the family name first; Google already orders the `name` claim per the account's own
/// locale. The join is a fallback that will sometimes produce the wrong order, and building a locale-aware
/// formatter to guard that edge would still be wrong for anyone whose IdP disagrees with their culture. The
/// `name` claim is the person's own answer. Prefer it.
pub fn resolve(p: &Profile, email: Option<&str>) -> (&'static str, String) {
    let clean = |s: &Option<String>| -> Option<String> {
        s.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
    };

    if let Some(n) = clean(&p.name) {
        return ("name", n);
    }
    match (clean(&p.given_name), clean(&p.family_name)) {
        (Some(g), Some(f)) => return ("given_name+family_name", format!("{g} {f}")),
        (Some(g), None) => return ("given_name", g),
        (None, Some(f)) => return ("family_name", f),
        (None, None) => {}
    }
    if let Some(u) = clean(&p.preferred_username) {
        return ("preferred_username", u);
    }
    match email {
        // The local part, never the full address (§1 #2). This rung is what the bug should have been.
        Some(e) if e.contains('@') => (
            "email_local_part",
            e.split('@').next().unwrap_or_default().to_string(),
        ),
        Some(e) => ("email", e.to_string()),
        None => ("handle", String::new()),
    }
}

/// §1 #4 + #5 — repair a display name that was never really set, and never touch one that was.
///
/// **Why this is a standalone UPDATE and not a `CASE` inside the JIT upsert**, which is what FR-AUTH-111 §3
/// sketches: because the upsert is unreachable for exactly the people who need repairing. Both
/// `oidc::resolve_subject` and `saml::resolve_subject` short-circuit on the existing-link fast path and
/// `return Ok(sid)` before any `INSERT` runs. Every colleague who has already signed in through Google — that
/// is, every person affected today — takes that path. A `CASE` in the `ON CONFLICT` clause would fix new
/// provisioning, do nothing for the existing damage, and look like it worked. §1 #5 promises the fix is
/// self-healing with no migration and no administrator action; this is the statement that keeps the promise.
///
/// So the rule lives here, applied to whichever subject_id resolved, on every path. Idempotent by
/// construction: once the name is repaired the predicate stops matching.
///
/// The predicate is the whole of the no-clobber guarantee (§1 #4). We refresh only when the stored value is
/// NULL, empty, or byte-equal to the subject's own email — the three shapes that mean "nobody ever set this".
/// A value that DIFFERS from the email was put there deliberately: by an administrator repairing it by hand,
/// or by the person themselves through `PATCH /v1/auth/me`, which ships today. Silently reverting someone's
/// own decision on their next sign-in, with no error and no trace, is a worse bug than the one we are fixing.
///
/// `resolved` is refused when empty so a blank or whitespace-only claim can never blank a stored name (§1 #9
/// of the failure inventory): the chain falls through rather than writing nothing over something.
pub async fn heal(
    pg: &sqlx::PgPool,
    tenant_id: Uuid,
    subject_id: Uuid,
    resolved: &str,
) -> Result<(), sqlx::Error> {
    if resolved.trim().is_empty() {
        return Ok(());
    }
    let mut tx = pg.begin().await?;
    sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
        .bind(tenant_id.to_string())
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "UPDATE subjects
            SET display_name = $2, updated_at = NOW()
          WHERE id = $1
            AND (display_name IS NULL
                 OR display_name = ''
                 OR display_name = email)",
    )
    .bind(subject_id)
    .bind(resolved)
    .execute(&mut *tx)
    .await?;
    tx.commit().await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(
        name: Option<&str>,
        given: Option<&str>,
        family: Option<&str>,
        preferred: Option<&str>,
    ) -> Profile {
        Profile {
            name: name.map(str::to_string),
            given_name: given.map(str::to_string),
            family_name: family.map(str::to_string),
            preferred_username: preferred.map(str::to_string),
        }
    }

    /// AC 1-5 — every rung, in order.
    #[test]
    fn the_chain_walks_in_order() {
        let e = Some("van-anh.vu@cyberskill.world");

        // AC 1 — `name` wins, and the diacritics survive byte-for-byte. This is the assertion that would
        // catch a well-meaning `.to_title_case()` appearing here later.
        let (rung, v) = resolve(&p(Some("Trịnh Thái Anh"), None, None, None), e);
        assert_eq!((rung, v.as_str()), ("name", "Trịnh Thái Anh"));

        // AC 2 — both parts join.
        let (rung, v) = resolve(&p(None, Some("Thái Anh"), Some("Trịnh"), None), e);
        assert_eq!(
            (rung, v.as_str()),
            ("given_name+family_name", "Thái Anh Trịnh")
        );

        // AC 3 — a single part is used alone, not padded into " Trịnh" or "Thái Anh ".
        let (rung, v) = resolve(&p(None, Some("Thái Anh"), None, None), e);
        assert_eq!((rung, v.as_str()), ("given_name", "Thái Anh"));
        let (rung, v) = resolve(&p(None, None, Some("Trịnh"), None), e);
        assert_eq!((rung, v.as_str()), ("family_name", "Trịnh"));

        // AC 4 — preferred_username.
        let (rung, v) = resolve(&p(None, None, None, Some("stephen")), e);
        assert_eq!((rung, v.as_str()), ("preferred_username", "stephen"));

        // AC 5 — the local part, NEVER the full address.
        let (rung, v) = resolve(&p(None, None, None, None), e);
        assert_eq!((rung, v.as_str()), ("email_local_part", "van-anh.vu"));
    }

    /// AC 6 — the bug itself, pinned. No rung may ever produce the full email address.
    #[test]
    fn no_rung_ever_yields_the_full_email() {
        let email = "a.b@c.com";
        for profile in [
            p(Some("A B"), None, None, None),
            p(None, Some("A"), Some("B"), None),
            p(None, None, None, Some("ab")),
            p(None, None, None, None),
        ] {
            let (_, v) = resolve(&profile, Some(email));
            assert_ne!(v, email, "display_name must never be bound from the email");
        }
    }

    /// AC 9 — a whitespace-only claim falls THROUGH rather than writing a blank. A person who renders as
    /// nothing at all is worse off than one who renders as their email.
    #[test]
    fn blank_claims_fall_through_and_never_blank_a_name() {
        let (rung, v) = resolve(&p(Some("   "), None, None, None), Some("a.b@c.com"));
        assert_eq!((rung, v.as_str()), ("email_local_part", "a.b"));

        let (rung, v) = resolve(
            &p(Some(""), Some("\t"), Some(" "), Some("\n")),
            Some("a.b@c.com"),
        );
        assert_eq!((rung, v.as_str()), ("email_local_part", "a.b"));
    }

    /// The final arm: no claims and no email. Empty, so the caller substitutes the handle rather than
    /// writing an empty string over a name.
    #[test]
    fn no_email_and_no_claims_yields_the_handle_rung() {
        let (rung, v) = resolve(&p(None, None, None, None), None);
        assert_eq!(rung, "handle");
        assert!(v.is_empty());
    }

    /// An email with no `@` cannot be split; it is used whole rather than panicking on `split.next()`.
    #[test]
    fn a_malformed_email_does_not_panic() {
        let (rung, v) = resolve(&p(None, None, None, None), Some("not-an-email"));
        assert_eq!((rung, v.as_str()), ("email", "not-an-email"));
    }

    /// `picture` is not a field on Profile, so it cannot be persisted by accident (§1 #9, AC 11). Serde
    /// ignores unknown keys, which is what we want: the claim arriving changes nothing.
    #[test]
    fn the_picture_claim_has_nowhere_to_land() {
        let claims = serde_json::json!({
            "name": "Trịnh Thái Anh",
            "picture": "https://lh3.googleusercontent.com/photo.jpg",
        });
        let profile: Profile = serde_json::from_value(claims).expect("deserialise");
        let (_, v) = resolve(&profile, None);
        assert_eq!(v, "Trịnh Thái Anh");
        // The only way to reintroduce the leak is to add a field here, which is a visible diff.
        assert!(!format!("{profile:?}").contains("photo.jpg"));
    }
}
