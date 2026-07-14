//! Time-range validation for a compliance view (TASK-OBS-008 §1 #6). Windows are validated on epoch
//! seconds - the HTTP layer parses the ISO-8601 `since` / `until` params - and a window over 365 days is
//! rejected so an auditor paginates rather than asking the service to render a year-plus in one shot.

/// The maximum view window: 365 days, in seconds (§1 #6).
pub const MAX_WINDOW_SECS: i64 = 365 * 24 * 60 * 60;

/// Why a window is invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum WindowError {
    #[error("until is before since")]
    Inverted,
    #[error("window exceeds 365 days; paginate")]
    TooWide,
}

/// Validate `[since, until]` in epoch seconds: `until` must be at or after `since`, and the span at most
/// 365 days.
pub fn validate(since_secs: i64, until_secs: i64) -> Result<(), WindowError> {
    if until_secs < since_secs {
        return Err(WindowError::Inverted);
    }
    if until_secs - since_secs > MAX_WINDOW_SECS {
        return Err(WindowError::TooWide);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverted_window_is_rejected() {
        assert_eq!(validate(100, 50), Err(WindowError::Inverted));
    }

    #[test]
    fn a_window_at_the_limit_is_allowed() {
        assert_eq!(validate(0, MAX_WINDOW_SECS), Ok(()));
        assert_eq!(validate(1000, 1000), Ok(())); // zero-width is fine
    }

    #[test]
    fn a_window_over_the_limit_is_rejected() {
        assert_eq!(validate(0, MAX_WINDOW_SECS + 1), Err(WindowError::TooWide));
    }
}
