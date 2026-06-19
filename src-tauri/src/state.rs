//! Application state: the currently open document plus editor metadata.

use std::sync::Mutex;

use eulumdat_core::Eulumdat;

/// The open document and its on-disk association.
#[derive(Debug, Default)]
pub struct OpenDoc {
    pub model: Option<Eulumdat>,
    /// Absolute path the document was opened from / last saved to.
    pub path: Option<String>,
    /// Whether the in-memory model differs from the last saved state.
    pub dirty: bool,
    /// When set, enforces the legacy EULUMDAT text-length limits (8.3-era
    /// filename, etc.). Off by default; modern files are unrestricted.
    pub strict_validation: bool,
}

/// Shared, mutex-guarded application state.
#[derive(Debug, Default)]
pub struct AppState {
    pub doc: Mutex<OpenDoc>,
}
