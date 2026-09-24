//! Application state: documents are independent from the windows that show them.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32};
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

/// The ordered tabs shown by one native window.
#[derive(Debug, Default)]
pub struct WindowTabs {
    pub tabs: Vec<String>,
    pub active: Option<String>,
}

/// All document and window relationships guarded by one lock so moving a tab
/// can never leave it in two windows, or in none.
#[derive(Debug, Default)]
pub struct Workspace {
    pub docs: HashMap<String, OpenDoc>,
    pub windows: HashMap<String, WindowTabs>,
}

/// Shared, mutex-guarded application state.
#[derive(Debug, Default)]
pub struct AppState {
    pub workspace: Mutex<Workspace>,
    /// Source of unique document/tab ids.
    pub next_doc_id: AtomicU32,
    /// Source of unique labels for windows opened after `main`.
    pub next_window_id: AtomicU32,
    /// A file the OS asked us to open (file association / `open with`) before
    /// the frontend was ready to receive it. Drained by `take_pending_open`.
    pub pending_open: Mutex<Option<String>>,
    /// Set once the frontend has drained `pending_open`. After this, OS open
    /// requests are delivered live via the `open-file` event instead.
    pub frontend_ready: AtomicBool,
}
