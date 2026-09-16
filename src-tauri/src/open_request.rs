//! Routes OS "open this file" requests (file association, `Open with`, a
//! second launch) to the frontend.
//!
//! macOS delivers these as `RunEvent::Opened`. Windows and Linux start the
//! executable with the file path as a command-line argument instead, both on
//! first launch and, through the single-instance plugin, on later launches.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager};

use crate::state::AppState;

/// Hands `path` to the frontend: live via the `open-file` event once the UI
/// is listening, otherwise queued for `take_pending_open`.
pub fn deliver(app: &AppHandle, path: &Path) {
    let path = path.to_string_lossy().into_owned();
    let state = app.state::<AppState>();
    if state.frontend_ready.load(Ordering::SeqCst) {
        let _ = app.emit("open-file", &path);
    } else {
        *state.pending_open.lock().unwrap() = Some(path);
    }
}

/// Whether `path` has a `.ldt` extension (case-insensitive).
pub fn is_ldt(path: &Path) -> bool {
    path.extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("ldt"))
}

/// Finds the first `.ldt` file among launch arguments, skipping the
/// executable path. Relative paths are resolved against `cwd`, the working
/// directory of the process that received them.
#[cfg_attr(target_os = "macos", allow(dead_code))]
pub fn ldt_path_from_args<I, S>(args: I, cwd: &Path) -> Option<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    args.into_iter()
        .skip(1)
        .map(|a| PathBuf::from(a.as_ref()))
        .find(|p| is_ldt(p))
        .map(|p| if p.is_absolute() { p } else { cwd.join(p) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_the_executable_path() {
        let args = ["/opt/app/weird.ldt"];
        assert_eq!(ldt_path_from_args(args, Path::new("/")), None);
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
        assert_eq!(ldt_path_from_args(args, Path::new("/")), Some(first));
    }

    #[test]
    fn resolves_relative_paths_against_cwd() {
        let cwd = std::env::current_dir().unwrap();
        let args = ["app", "lamp.ldt"];
        assert_eq!(ldt_path_from_args(args, &cwd), Some(cwd.join("lamp.ldt")));
    }

    #[test]
    fn ignores_non_ldt_arguments() {
        let args = ["app", "notes.txt", "ldt"];
        assert_eq!(ldt_path_from_args(args, Path::new("/")), None);
    }
}
