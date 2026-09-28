//! Routes OS "open this file" requests (file association, `Open with`, a
//! second launch) to the frontend.
//!
//! macOS delivers these as `RunEvent::Opened`. Windows and Linux start the
//! executable with the file path as a command-line argument instead, both on
//! first launch and, through the single-instance plugin, on later launches.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, EventTarget, Manager, WebviewWindow};

use crate::state::AppState;

/// Hands `path` to the frontend: live via an `open-file` event to one window
/// once the UI is listening, otherwise queued for `take_pending_open`. The
/// receiving window opens it in a tab, or focuses its existing tab.
pub fn deliver(app: &AppHandle, path: &Path) {
    let path = path.to_string_lossy().into_owned();
    let state = app.state::<AppState>();
    if !state.frontend_ready.load(Ordering::SeqCst) {
        *state.pending_open.lock().unwrap() = Some(path);
    } else if let Some(window) = focus_window(app) {
        let _ = app.emit_to(
            EventTarget::webview_window(window.label()),
            "open-file",
            &path,
        );
    } else {
        let _ = crate::commands::open_path_window(app, Path::new(&path));
    }
}

/// The window app-level requests (menu items, OS opens) should act on: the
/// focused one, or any window when the app is in the background.
pub fn target_window(app: &AppHandle) -> Option<WebviewWindow> {
    let windows = app.webview_windows();
    windows
        .values()
        .filter(|w| !crate::tab_preview::is_preview_label(w.label()))
        .find(|w| w.is_focused().unwrap_or(false))
        .or_else(|| {
            windows
                .values()
                .find(|w| !crate::tab_preview::is_preview_label(w.label()))
        })
        .cloned()
}

/// Brings the [`target_window`] to the front and returns it.
pub fn focus_window(app: &AppHandle) -> Option<WebviewWindow> {
    let window = target_window(app)?;
    let _ = window.unminimize();
    let _ = window.set_focus();
    Some(window)
}

/// Whether `path` has a `.ldt` or `.ies` extension (case-insensitive).
pub fn is_photometric(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("ldt") || e.eq_ignore_ascii_case("ies"))
}

/// Finds the first photometric file among launch arguments, skipping the
/// executable path. Relative paths are resolved against `cwd`, the working
/// directory of the process that received them.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn photometric_path_from_args<I, S>(args: I, cwd: &Path) -> Option<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter()
        .skip(1)
        .map(|a| PathBuf::from(a.as_ref()))
        .find(|p| is_photometric(p))
        .map(|p| if p.is_absolute() { p } else { cwd.join(p) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_the_executable_path() {
        let args = ["/opt/app/weird.ldt"];
        assert_eq!(photometric_path_from_args(args, Path::new("/")), None);
    }

    #[test]
    fn picks_the_first_ldt_argument() {
        let cwd = std::env::current_dir().unwrap();
        let first = cwd.join("a.LDT");
        let second = cwd.join("b.ldt");
        let args = [
            "app".to_string(),
            "--flag".to_string(),
            first.to_string_lossy().into_owned(),
            second.to_string_lossy().into_owned(),
        ];
        assert_eq!(
            photometric_path_from_args(args, Path::new("/")),
            Some(first)
        );
    }

    #[test]
    fn resolves_relative_paths_against_cwd() {
        let cwd = std::env::current_dir().unwrap();
        let args = ["app", "lamp.ldt"];
        assert_eq!(
            photometric_path_from_args(args, &cwd),
            Some(cwd.join("lamp.ldt"))
        );
    }

    #[test]
    fn accepts_ies_files_case_insensitively() {
        let args = ["app", "lamp.IES"];
        assert_eq!(
            photometric_path_from_args(args, Path::new("/tmp")),
            Some(PathBuf::from("/tmp/lamp.IES"))
        );
    }

    #[test]
    fn ignores_non_photometric_arguments() {
        let args = ["app", "notes.txt", "ldt"];
        assert_eq!(photometric_path_from_args(args, Path::new("/")), None);
    }
}
