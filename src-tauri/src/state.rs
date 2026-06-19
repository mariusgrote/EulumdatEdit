//! Application state: the currently open document plus editor metadata.

use std::sync::atomic::AtomicBool;
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
    /// A file the OS asked us to open (file association / `open with`) before
    /// the frontend was ready to receive it. Drained by `take_pending_open`.
    pub pending_open: Mutex<Option<String>>,
    /// Set once the frontend has drained `pending_open`. After this, OS open
    /// requests are delivered live via the `open-file` event instead.
    pub frontend_ready: AtomicBool,
}
