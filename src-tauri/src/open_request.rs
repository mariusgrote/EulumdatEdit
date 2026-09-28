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

/// Hands paths to the frontend in order, live or queued until it is ready.
pub fn deliver(app: &AppHandle, paths: &[PathBuf]) {
    if paths.is_empty() {
        return;
    }
    let paths: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let state = app.state::<AppState>();
    if !state.frontend_ready.load(Ordering::SeqCst) {
        state.pending_open.lock().unwrap().extend(paths);
    } else if let Some(window) = focus_window(app) {
        let _ = app.emit_to(
            EventTarget::webview_window(window.label()),
            "open-file",
            &paths,
        );
    } else {
        for (index, path) in paths.iter().enumerate() {
            if crate::commands::open_path_window(app, Path::new(path)).is_ok() {
                state
                    .pending_open
                    .lock()
                    .unwrap()
                    .extend(paths[index + 1..].iter().cloned());
                break;
            }
        }
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

/// Finds photometric files among launch arguments, skipping the
/// executable path. Relative paths are resolved against `cwd`, the working
/// directory of the process that received them.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn photometric_paths_from_args<I, S>(args: I, cwd: &Path) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter()
        .skip(1)
        .map(|a| PathBuf::from(a.as_ref()))
        .filter(|p| is_photometric(p))
        .map(|p| if p.is_absolute() { p } else { cwd.join(p) })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_the_executable_path() {
        let args = ["/opt/app/weird.ldt"];
        assert!(photometric_paths_from_args(args, Path::new("/")).is_empty());
    }

    #[test]
    fn keeps_all_ldt_arguments_in_order() {
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
            photometric_paths_from_args(args, Path::new("/")),
            vec![first, second]
        );
    }

    #[test]
    fn resolves_relative_paths_against_cwd() {
        let cwd = std::env::current_dir().unwrap();
        let args = ["app", "lamp.ldt"];
        assert_eq!(
            photometric_paths_from_args(args, &cwd),
            vec![cwd.join("lamp.ldt")]
        );
    }

    #[test]
    fn accepts_ies_files_case_insensitively() {
        let args = ["app", "lamp.IES"];
        assert_eq!(
            photometric_paths_from_args(args, Path::new("/tmp")),
            vec![PathBuf::from("/tmp/lamp.IES")]
        );
    }

    #[test]
    fn ignores_non_photometric_arguments() {
        let args = ["app", "notes.txt", "ldt"];
        assert!(photometric_paths_from_args(args, Path::new("/")).is_empty());
    }
}
