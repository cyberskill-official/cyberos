use std::fmt;

/// Structured error type for all GAM backend operations.
#[derive(Debug)]
pub enum GamError {
    /// Git CLI not found or not installed.
    GitNotFound,
    /// Git command execution failed.
    GitFailed(String),
    /// Filesystem I/O error.
    Io(std::io::Error),
    /// JSON serialization / deserialization error.
    Serde(String),
    /// Requested resource (alias, group, etc.) not found.
    NotFound(String),
    /// RwLock was poisoned.
    LockPoisoned(String),
}

impl fmt::Display for GamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GamError::GitNotFound => write!(f, "Git is not installed or not found in PATH"),
            GamError::GitFailed(msg) => write!(f, "Git error: {msg}"),
            GamError::Io(err) => write!(f, "I/O error: {err}"),
            GamError::Serde(msg) => write!(f, "Serialization error: {msg}"),
            GamError::NotFound(what) => write!(f, "Not found: {what}"),
            GamError::LockPoisoned(what) => write!(f, "Lock poisoned: {what}"),
        }
    }
}

impl std::error::Error for GamError {}

impl From<std::io::Error> for GamError {
    fn from(err: std::io::Error) -> Self {
        if err.kind() == std::io::ErrorKind::NotFound {
            GamError::GitNotFound
        } else {
            GamError::Io(err)
        }
    }
}

impl From<serde_json::Error> for GamError {
    fn from(err: serde_json::Error) -> Self {
        GamError::Serde(err.to_string())
    }
}

impl From<GamError> for String {
    fn from(err: GamError) -> Self {
        err.to_string()
    }
}

/// Helper to convert an `error_code` string suitable for IPC.
impl GamError {
    pub fn code(&self) -> &'static str {
        match self {
            GamError::GitNotFound => "GIT_NOT_FOUND",
            GamError::GitFailed(_) => "GIT_FAILED",
            GamError::Io(_) => "IO_ERROR",
            GamError::Serde(_) => "SERDE_ERROR",
            GamError::NotFound(_) => "NOT_FOUND",
            GamError::LockPoisoned(_) => "LOCK_POISONED",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_git_not_found() {
        let err = GamError::GitNotFound;
        assert!(err.to_string().contains("not installed"));
        assert_eq!(err.code(), "GIT_NOT_FOUND");
    }

    #[test]
    fn display_git_failed() {
        let err = GamError::GitFailed("fatal: not a git repository".into());
        assert!(err.to_string().contains("fatal"));
        assert_eq!(err.code(), "GIT_FAILED");
    }

    #[test]
    fn from_io_error_not_found() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let err: GamError = io_err.into();
        assert!(matches!(err, GamError::GitNotFound));
    }

    #[test]
    fn from_io_error_other() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err: GamError = io_err.into();
        assert!(matches!(err, GamError::Io(_)));
        assert_eq!(err.code(), "IO_ERROR");
    }

    #[test]
    fn into_string_conversion() {
        let err = GamError::NotFound("alias 'co'".into());
        let s: String = err.into();
        assert!(s.contains("alias 'co'"));
    }

    #[test]
    fn lock_poisoned_display() {
        let err = GamError::LockPoisoned("git_service".into());
        assert!(err.to_string().contains("git_service"));
        assert_eq!(err.code(), "LOCK_POISONED");
    }
}
