//! Canonical JSON export.

use crate::error::ViewError;
use crate::views::ViewResponse;

/// Serialize deterministically.
pub fn render_json(view: &ViewResponse) -> Result<Vec<u8>, ViewError> {
    serde_json::to_vec(view).map_err(|err| ViewError::ExportFailed(err.to_string()))
}
