//! TASK-AUTH-110 §1 #10 + DEC-2486 - the OIDC id_token.
//!
//! The id_token is what an RP (Mattermost, portal) reads to learn who logged in.
//! It is a distinct shape from the platform access-token [`crate::jwt::Claims`]:
//! `aud` is the RP `client_id`, and it carries the OIDC profile claims plus
//! `tenant_id` + `roles` so the RP can place the user without a second call.
//!
//! Signing reuses the TASK-AUTH-004 RS256 path against `auth_signing_keys`
//! (DEC-2481, one JWKS for the platform). [`build_id_token_claims`] is pure and
//! unit-tested here; [`sign_id_token`] is the thin RS256 wrapper. The active key
//! is loaded at the call site (slice 1b) so this module stays database-free.

use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::Serialize;

use super::errors::OpError;

/// OIDC id_token claim set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdTokenClaims {
    pub iss: String,
    pub sub: String,
    /// The RP client_id (single audience).
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub auth_time: i64,
    /// Echoed from the RP's authorize request when supplied (OIDC Core).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<String>,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub preferred_username: String,
    pub tenant_id: String,
    pub roles: Vec<String>,
}

/// Assemble the id_token claims. Pure - no key, no clock side effects (the caller
/// passes `iat`), so it is fully unit-testable.
#[allow(clippy::too_many_arguments)]
pub fn build_id_token_claims(
    issuer: &str,
    subject_id: &str,
    client_id: &str,
    email: &str,
    email_verified: bool,
    name: &str,
    preferred_username: &str,
    tenant_id: &str,
    roles: Vec<String>,
    nonce: Option<String>,
    iat: i64,
    ttl_secs: i64,
) -> IdTokenClaims {
    IdTokenClaims {
        iss: issuer.to_string(),
        sub: subject_id.to_string(),
        aud: client_id.to_string(),
        exp: iat + ttl_secs,
        iat,
        auth_time: iat,
        nonce,
        email: email.to_string(),
        email_verified,
        name: name.to_string(),
        preferred_username: preferred_username.to_string(),
        tenant_id: tenant_id.to_string(),
        roles,
    }
}

/// Sign the id_token with the active TASK-AUTH-004 RSA private key (PEM) under
/// `kid`. The `kid` MUST be a key published in `/.well-known/jwks.json` so the RP
/// can verify. Same RS256 path as [`crate::jwt::JwtService`] minting.
pub fn sign_id_token(
    claims: &IdTokenClaims,
    kid: &str,
    private_pem: &str,
) -> Result<String, OpError> {
    let mut header = Header::new(Algorithm::RS256);
    header.kid = Some(kid.to_string());
    let key =
        EncodingKey::from_rsa_pem(private_pem.as_bytes()).map_err(|_| OpError::ServerError)?;
    encode(&header, claims, &key).map_err(|_| OpError::ServerError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aud_is_the_client_id_and_exp_is_iat_plus_ttl() {
        let c = build_id_token_claims(
            "https://auth.cyberos.world",
            "cf0f35f7-7770-4598-a656-50493e635351",
            "cyberos-chat",
            "[email protected]",
            true,
            "Thai Anh Trinh",
            "thai-anh.trinh",
            "00000000-0000-0000-0000-000000000000",
            vec!["tenant-admin".to_string()],
            Some("n-0S6_WzA2Mj".to_string()),
            1_751_155_200,
            3600,
        );
        assert_eq!(c.aud, "cyberos-chat");
        assert_eq!(c.sub, "cf0f35f7-7770-4598-a656-50493e635351");
        assert_eq!(c.exp, 1_751_155_200 + 3600);
        assert_eq!(c.auth_time, 1_751_155_200);
        assert_eq!(c.roles, vec!["tenant-admin".to_string()]);
        assert_eq!(c.nonce.as_deref(), Some("n-0S6_WzA2Mj"));
    }

    #[test]
    fn nonce_absent_is_omitted_from_json() {
        let c = build_id_token_claims(
            "https://auth",
            "sub-1",
            "cyberos-chat",
            "[email protected]",
            false,
            "U",
            "u",
            "t-1",
            vec![],
            None,
            1000,
            60,
        );
        let v = serde_json::to_value(&c).unwrap();
        assert!(v.get("nonce").is_none(), "nonce must be omitted when None");
        assert_eq!(v["aud"], "cyberos-chat");
        assert_eq!(v["email_verified"], false);
    }
}
