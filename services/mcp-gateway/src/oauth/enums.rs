//! TASK-MCP-004 closed OAuth enums (DEC-807, DEC-808, clauses #5, #4, #16, #17).
//!
//! These mirror the Postgres `CREATE TYPE ... AS ENUM` declarations in the OAuth migrations. Each is
//! a closed set with a cardinality test that fails if a variant is added without updating the
//! assertion - a governance tripwire, the same pattern as `naming::Sep986Verb`. The RFC 6749 §5.2
//! error code enum lives in [`super::error::OAuthErrorCode`] (6 variants); this module carries the
//! grant-type, client-type, and the two state machines.

use std::fmt;

/// The two grant types this slice supports (DEC-807, clause #5). `client_credentials` is deferred to
/// TASK-MCP-007; implicit and resource-owner-password are prohibited by OAuth 2.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrantType {
    /// `authorization_code`
    AuthorizationCode,
    /// `refresh_token`
    RefreshToken,
}

impl GrantType {
    /// All variants in declaration order (used by the cardinality test).
    pub fn all_variants() -> &'static [GrantType] {
        &[GrantType::AuthorizationCode, GrantType::RefreshToken]
    }

    /// The canonical wire string.
    pub fn as_str(&self) -> &'static str {
        match self {
            GrantType::AuthorizationCode => "authorization_code",
            GrantType::RefreshToken => "refresh_token",
        }
    }

    /// Parse a wire string, or `None` if it is not a supported grant type. An unsupported but
    /// well-formed grant (e.g. `client_credentials`, `password`) returns `None`; the token endpoint
    /// maps that to `400 unsupported_grant_type`.
    pub fn from_wire(s: &str) -> Option<GrantType> {
        match s {
            "authorization_code" => Some(GrantType::AuthorizationCode),
            "refresh_token" => Some(GrantType::RefreshToken),
            _ => None,
        }
    }
}

impl fmt::Display for GrantType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Public vs confidential clients (DEC-808, clause #4). Public clients have no secret and prove
/// possession via PKCE; confidential clients authenticate at the token endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientType {
    /// `public` - CLIs, desktop and native apps; PKCE-only.
    Public,
    /// `confidential` - server-side clients with a secret.
    Confidential,
}

impl ClientType {
    /// All variants in declaration order (used by the cardinality test).
    pub fn all_variants() -> &'static [ClientType] {
        &[ClientType::Public, ClientType::Confidential]
    }

    /// The canonical wire string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ClientType::Public => "public",
            ClientType::Confidential => "confidential",
        }
    }

    /// Parse a wire string, or `None` if it is not a known client type.
    pub fn from_wire(s: &str) -> Option<ClientType> {
        match s {
            "public" => Some(ClientType::Public),
            "confidential" => Some(ClientType::Confidential),
            _ => None,
        }
    }
}

impl fmt::Display for ClientType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Authorization-code lifecycle (clause #16). A code is `active` until it is exchanged
/// (`consumed`) or its 30-second TTL elapses (`expired`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeState {
    /// `active` - issued, not yet exchanged, within TTL.
    Active,
    /// `consumed` - exchanged once; a second exchange is a replay (clause #15).
    Consumed,
    /// `expired` - TTL elapsed before exchange.
    Expired,
}

impl CodeState {
    /// All variants in declaration order (used by the cardinality test).
    pub fn all_variants() -> &'static [CodeState] {
        &[CodeState::Active, CodeState::Consumed, CodeState::Expired]
    }

    /// The canonical wire string.
    pub fn as_str(&self) -> &'static str {
        match self {
            CodeState::Active => "active",
            CodeState::Consumed => "consumed",
            CodeState::Expired => "expired",
        }
    }

    /// Parse a wire string, or `None` if it is not a known code state.
    pub fn from_wire(s: &str) -> Option<CodeState> {
        match s {
            "active" => Some(CodeState::Active),
            "consumed" => Some(CodeState::Consumed),
            "expired" => Some(CodeState::Expired),
            _ => None,
        }
    }
}

impl fmt::Display for CodeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Refresh-token family lifecycle (clause #17). Rotation moves a token `active` -> `used`; presenting
/// a `used` token is reuse, which marks the whole family `compromised` (clause #9, DEC-806).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshState {
    /// `active` - the current valid token in its family.
    Active,
    /// `used` - rotated out; presenting it again is reuse.
    Used,
    /// `compromised` - the family was poisoned by a reuse; every descendant is invalid.
    Compromised,
}

impl RefreshState {
    /// All variants in declaration order (used by the cardinality test).
    pub fn all_variants() -> &'static [RefreshState] {
        &[
            RefreshState::Active,
            RefreshState::Used,
            RefreshState::Compromised,
        ]
    }

    /// The canonical wire string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RefreshState::Active => "active",
            RefreshState::Used => "used",
            RefreshState::Compromised => "compromised",
        }
    }

    /// Parse a wire string, or `None` if it is not a known refresh state.
    pub fn from_wire(s: &str) -> Option<RefreshState> {
        match s {
            "active" => Some(RefreshState::Active),
            "used" => Some(RefreshState::Used),
            "compromised" => Some(RefreshState::Compromised),
            _ => None,
        }
    }
}

impl fmt::Display for RefreshState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- cardinality tripwires (the spec mandates a CI cardinality test per enum) ----

    #[test]
    fn grant_type_has_exactly_two_variants() {
        assert_eq!(GrantType::all_variants().len(), 2);
    }

    #[test]
    fn client_type_has_exactly_two_variants() {
        assert_eq!(ClientType::all_variants().len(), 2);
    }

    #[test]
    fn code_state_has_exactly_three_variants() {
        assert_eq!(CodeState::all_variants().len(), 3);
    }

    #[test]
    fn refresh_state_has_exactly_three_variants() {
        assert_eq!(RefreshState::all_variants().len(), 3);
    }

    // ---- round-trips: every variant's wire string parses back to itself ----

    #[test]
    fn grant_type_round_trips() {
        for v in GrantType::all_variants() {
            assert_eq!(GrantType::from_wire(v.as_str()), Some(*v));
        }
    }

    #[test]
    fn client_type_round_trips() {
        for v in ClientType::all_variants() {
            assert_eq!(ClientType::from_wire(v.as_str()), Some(*v));
        }
    }

    #[test]
    fn code_state_round_trips() {
        for v in CodeState::all_variants() {
            assert_eq!(CodeState::from_wire(v.as_str()), Some(*v));
        }
    }

    #[test]
    fn refresh_state_round_trips() {
        for v in RefreshState::all_variants() {
            assert_eq!(RefreshState::from_wire(v.as_str()), Some(*v));
        }
    }

    // ---- unsupported / unknown inputs return None, not a wrong variant ----

    #[test]
    fn unsupported_grant_types_are_rejected() {
        for s in ["client_credentials", "password", "implicit", ""] {
            assert_eq!(GrantType::from_wire(s), None);
        }
    }

    #[test]
    fn unknown_states_are_rejected() {
        assert_eq!(CodeState::from_wire("revoked"), None);
        assert_eq!(RefreshState::from_wire("rotated"), None);
        assert_eq!(ClientType::from_wire("hybrid"), None);
    }
}
