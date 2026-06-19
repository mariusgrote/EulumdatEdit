//! EulumdatEdit Tauri backend.

mod commands;
mod dto;
mod state;

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
            commands::update_document,
            commands::save,
            commands::save_as,
            commands::resample_gamma,
            commands::scale_to_100_percent,
            commands::set_strict_validation,
            commands::render_polar_svg,
            commands::write_bytes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
