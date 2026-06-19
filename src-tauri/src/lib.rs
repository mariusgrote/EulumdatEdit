//! EulumdatEdit Tauri backend.

mod commands;
mod dto;
mod state;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
use std::sync::atomic::Ordering;

#[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
use tauri::{Emitter, Manager};

use state::AppState;

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::new_from_template,
            commands::open_file,
            commands::take_pending_open,
            commands::update_document,
            commands::save,
            commands::save_as,
            commands::resample_gamma,
            commands::scale_to_100_percent,
            commands::set_strict_validation,
            commands::render_polar_svg,
            commands::write_bytes,
        ])
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {
            // macOS/iOS/Android deliver file-association / "Open with" requests here.
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
            if let tauri::RunEvent::Opened { urls } = _event {
                if let Some(path) = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("ldt")))
                {
                    let path = path.to_string_lossy().into_owned();
                    let state = _app.state::<AppState>();
                    if state.frontend_ready.load(Ordering::SeqCst) {
                        let _ = _app.emit("open-file", &path);
                    } else {
                        *state.pending_open.lock().unwrap() = Some(path);
                    }
                }
            }
        });
}
