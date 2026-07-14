//! TASK-OBS-001 §1 #2 + §3 — Bearer-token file management.
//!
//! Format (one entry per line):
//!
//! ```text
//! <service-name>   <bearer-token>
//! ai-gateway       8b2f…
//! auth             7c1e…
//! ```
//!
//! Comments (`#`) and blank lines are allowed. The otelcol bearertokenauth extension
//! reads the file at startup; for rotation, the supervisor signals SIGHUP after
//! writing a new file.

use std::collections::HashMap;
use std::path::Path;

use thiserror::Error;

/// Errors from [`TokenFile::load`].
#[derive(Debug, Error)]
pub enum AuthError {
    /// I/O failure reading the token file.
    #[error("read: {0}")]
    Read(#[from] std::io::Error),
    /// Malformed line.
    #[error("parse line {line_no}: {reason}")]
    Parse {
        /// 1-based line number.
        line_no: usize,
        /// Reason the line was rejected.
        reason: String,
    },
}

/// Parsed token file mapping `service_name → bearer_token`.
#[derive(Debug, Default, Clone)]
pub struct TokenFile {
    /// Service-name → bearer-token map.
    pub tokens: HashMap<String, String>,
}

impl TokenFile {
    /// Load and parse a token file.
    pub fn load(path: &Path) -> Result<Self, AuthError> {
        let raw = std::fs::read_to_string(path)?;
        Self::parse(&raw)
    }

    /// Parse from a string (separated so tests don't need a temp file).
    pub fn parse(raw: &str) -> Result<Self, AuthError> {
        let mut tokens = HashMap::new();
        for (i, line) in raw.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let mut parts = trimmed.split_whitespace();
            let svc = parts.next().ok_or(AuthError::Parse {
                line_no: i + 1,
                reason: "no service name".into(),
            })?;
            let tok = parts.next().ok_or(AuthError::Parse {
                line_no: i + 1,
                reason: "no token after service name".into(),
            })?;
            if parts.next().is_some() {
                return Err(AuthError::Parse {
                    line_no: i + 1,
                    reason: "extra columns after token".into(),
                });
            }
            tokens.insert(svc.to_string(), tok.to_string());
        }
        Ok(Self { tokens })
    }

    /// Lookup a token by service name.
    pub fn token_for(&self, service: &str) -> Option<&str> {
        self.tokens.get(service).map(|s| s.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_canonical() {
        let raw = "# header\nai-gateway   token-ai\nauth          token-auth\n\n";
        let tf = TokenFile::parse(raw).unwrap();
        assert_eq!(tf.token_for("ai-gateway"), Some("token-ai"));
        assert_eq!(tf.token_for("auth"), Some("token-auth"));
        assert_eq!(tf.token_for("missing"), None);
    }

    #[test]
    fn parse_rejects_extra_columns() {
        let raw = "ai-gateway token1 stray-third-column\n";
        assert!(TokenFile::parse(raw).is_err());
    }
}
