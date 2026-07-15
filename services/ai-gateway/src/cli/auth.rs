//! TASK-AI-021 §1 #6 — Operator token authentication and role gating.

use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::Deserialize;

/// Operator claims extracted from the JWT.
#[derive(Debug, Clone, Deserialize)]
pub struct OperatorClaims {
    /// Operator ID (kebab-case email).
    pub operator_id: String,
    /// Roles assigned to the operator.
    pub roles: Vec<Role>,
    /// Expiration timestamp.
    pub exp: Option<u64>,
}

/// Operator role.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Read,
    Mutate,
    Admin,
}

impl std::fmt::Display for Role {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Role::Read => write!(f, "read"),
            Role::Mutate => write!(f, "mutate"),
            Role::Admin => write!(f, "admin"),
        }
    }
}

/// Authentication error.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("missing CYBEROS_AI_OPERATOR_TOKEN")]
    MissingToken,
    #[error("invalid token: {reason}")]
    InvalidToken { reason: String },
    #[error("token expired")]
    Expired,
    #[error("insufficient_role: needed {needed:?}; have {has:?}")]
    InsufficientRole { needed: Role, has: Vec<Role> },
}

impl AuthError {
    pub fn needed(&self) -> Role {
        match self {
            AuthError::InsufficientRole { needed, .. } => needed.clone(),
            _ => Role::Read,
        }
    }
    pub fn has(&self) -> Vec<Role> {
        match self {
            AuthError::InsufficientRole { has, .. } => has.clone(),
            _ => vec![],
        }
    }
}

/// Parse and validate the operator token from the environment.
pub fn require_token() -> Result<OperatorClaims, AuthError> {
    let token = std::env::var("CYBEROS_AI_OPERATOR_TOKEN").map_err(|_| AuthError::MissingToken)?;
    parse_token(&token)
}

/// Parse a JWT token string.
pub fn parse_token(token: &str) -> Result<OperatorClaims, AuthError> {
    let secret = std::env::var("CYBEROS_AI_OPERATOR_SECRET")
        .unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let token_data = decode::<OperatorClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AuthError::InvalidToken {
        reason: e.to_string(),
    })?;

    let claims = token_data.claims;

    if let Some(exp) = claims.exp {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > exp {
            return Err(AuthError::Expired);
        }
    }

    Ok(claims)
}

/// Verify the operator has the required role.
pub fn require_role(claims: &OperatorClaims, needed: &Role) -> Result<(), AuthError> {
    if claims.roles.contains(&Role::Admin) {
        return Ok(());
    }
    if *needed == Role::Read {
        return Ok(());
    }
    if *needed == Role::Mutate && claims.roles.contains(&Role::Mutate) {
        return Ok(());
    }
    Err(AuthError::InsufficientRole {
        needed: needed.clone(),
        has: claims.roles.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_role() {
        assert_eq!(Role::Read.to_string(), "read");
        assert_eq!(Role::Mutate.to_string(), "mutate");
        assert_eq!(Role::Admin.to_string(), "admin");
    }

    #[test]
    fn admin_passes_all_roles() {
        let claims = OperatorClaims {
            operator_id: "test@cyberos.world".into(),
            roles: vec![Role::Admin],
            exp: None,
        };
        assert!(require_role(&claims, &Role::Read).is_ok());
        assert!(require_role(&claims, &Role::Mutate).is_ok());
        assert!(require_role(&claims, &Role::Admin).is_ok());
    }

    #[test]
    fn read_cannot_mutate() {
        let claims = OperatorClaims {
            operator_id: "test@cyberos.world".into(),
            roles: vec![Role::Read],
            exp: None,
        };
        assert!(require_role(&claims, &Role::Read).is_ok());
        assert!(require_role(&claims, &Role::Mutate).is_err());
    }

    #[test]
    fn mutate_can_mutate_but_not_admin() {
        let claims = OperatorClaims {
            operator_id: "test@cyberos.world".into(),
            roles: vec![Role::Mutate],
            exp: None,
        };
        assert!(require_role(&claims, &Role::Read).is_ok());
        assert!(require_role(&claims, &Role::Mutate).is_ok());
        // mutate cannot grant admin — but admin check is "contains Admin", so mutate fails
        assert!(require_role(&claims, &Role::Admin).is_err());
    }
}
