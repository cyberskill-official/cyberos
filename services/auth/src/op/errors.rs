//! FR-AUTH-110 - closed error type for the OIDC-provider surface.
//!
//! Codes are the RFC 6749 / OIDC error strings used either as the `error=` query
//! param on a redirect back to the RP, or as the `error` field of a JSON body on
//! the token / userinfo / admin paths. `http_status` is for the JSON paths only;
//! `UnknownClient` and `RedirectMismatch` are rendered as an error page and are
//! never redirected (open-redirect defense, DEC-2491), so their status is the
//! page status.

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum OpError {
    /// `client_id` is not a registered active RP. Rendered, not redirected.
    #[error("unknown_client")]
    UnknownClient,
    /// `redirect_uri` is not an exact match of the RP's registered set. Rendered,
    /// not redirected.
    #[error("redirect_mismatch")]
    RedirectMismatch,
    /// Missing / malformed required parameter (e.g. no PKCE challenge).
    #[error("invalid_request")]
    InvalidRequest,
    /// `response_type` other than `code` (implicit / hybrid forbidden).
    #[error("unsupported_response_type")]
    UnsupportedResponseType,
    /// The resolved subject is revoked (FR-AUTH-005). The kick.
    #[error("access_denied")]
    AccessDenied,
    /// Auth code missing / expired / replayed, or PKCE verifier mismatch.
    #[error("invalid_grant")]
    InvalidGrant,
    /// RP client authentication failed (bad client_secret).
    #[error("invalid_client")]
    InvalidClient,
    /// A unique constraint was violated (e.g. duplicate client_id on register).
    #[error("conflict")]
    Conflict,
    /// Internal failure (key load, signing, db).
    #[error("server_error")]
    ServerError,
}

impl OpError {
    /// The RFC 6749 / OIDC error code string.
    pub fn code(&self) -> &'static str {
        match self {
            OpError::UnknownClient => "unknown_client",
            OpError::RedirectMismatch => "redirect_mismatch",
            OpError::InvalidRequest => "invalid_request",
            OpError::UnsupportedResponseType => "unsupported_response_type",
            OpError::AccessDenied => "access_denied",
            OpError::InvalidGrant => "invalid_grant",
            OpError::InvalidClient => "invalid_client",
            OpError::Conflict => "conflict",
            OpError::ServerError => "server_error",
        }
    }

    /// HTTP status for the JSON error paths (token / userinfo / admin). The two
    /// rendered-not-redirected variants return the error-page status.
    pub fn http_status(&self) -> u16 {
        match self {
            OpError::UnknownClient
            | OpError::RedirectMismatch
            | OpError::InvalidRequest
            | OpError::UnsupportedResponseType
            | OpError::InvalidGrant => 400,
            OpError::InvalidClient => 401,
            OpError::AccessDenied => 403,
            OpError::Conflict => 409,
            OpError::ServerError => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_the_rfc_strings() {
        assert_eq!(OpError::AccessDenied.code(), "access_denied");
        assert_eq!(OpError::InvalidGrant.code(), "invalid_grant");
        assert_eq!(OpError::UnknownClient.code(), "unknown_client");
        // Display matches the code (thiserror message).
        assert_eq!(format!("{}", OpError::InvalidClient), "invalid_client");
    }

    #[test]
    fn statuses_map_per_oauth_semantics() {
        assert_eq!(OpError::InvalidClient.http_status(), 401);
        assert_eq!(OpError::AccessDenied.http_status(), 403);
        assert_eq!(OpError::InvalidGrant.http_status(), 400);
        assert_eq!(OpError::ServerError.http_status(), 500);
    }
}
