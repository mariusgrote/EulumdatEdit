//! EulumdatEdit Tauri backend.

mod commands;
mod dto;
mod ies;
#[cfg(target_os = "macos")]
mod menu;
mod open_request;
mod state;
mod tab_preview;

use tauri::{Emitter, Manager};

use state::AppState;

/// Builds and runs the Tauri application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default();

    // Windows/Linux launch a new process for every "Open with". Forward its
    // file to the running app instead; this must be the first plugin.
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    let builder = builder.plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
        open_request::focus_window(app);
        if let Some(path) =
            open_request::photometric_path_from_args(&args, std::path::Path::new(&cwd))
        {
            open_request::deliver(app, &path);
        }
    }));

    builder
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::new_from_template,
            commands::open_file,
            commands::reload_document,
            commands::current_document,
            commands::close_document,
            commands::activate_tab,
            commands::move_tab,
            commands::detach_tab,
            tab_preview::start_tab_preview,
            tab_preview::move_tab_preview,
            tab_preview::end_tab_preview,
            commands::other_documents_dirty,
            commands::quit_app,
            commands::take_pending_open,
            commands::update_document,
            commands::save,
            commands::save_as,
            commands::export_ies,
            commands::resample_gamma,
            commands::scale_to_100_percent,
            commands::set_strict_validation,
            commands::render_polar_svg,
            commands::write_bytes,
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                if tab_preview::is_preview_label(window.label()) {
                    return;
                }
                tab_preview::remove_source(window.app_handle(), window.label());
                let state = window.state::<AppState>();
                commands::remove_window(&state, window.label());
            }
        })
        .on_menu_event(|app, event| {
            // Menu items act on the focused window and its active tab.
            if let Some(window) = open_request::target_window(app) {
                let _ = app.emit_to(
                    tauri::EventTarget::webview_window(window.label()),
                    "menu",
                    event.id().as_ref(),
                );
            } else if event.id() == "quit" {
                app.exit(0);
            } else if event.id() == "new" {
                let _ = commands::open_new_document_window(app);
            }
        })
        .setup(|_app| {
            #[cfg(target_os = "macos")]
            _app.set_menu(menu::build(_app.handle())?)?;

            // Windows/Linux pass the file that launched the app as an argument.
            #[cfg(any(target_os = "windows", target_os = "linux"))]
            if let Some(path) = std::env::current_dir()
                .ok()
                .and_then(|cwd| open_request::photometric_path_from_args(std::env::args_os(), &cwd))
            {
                open_request::deliver(_app.handle(), &path);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app, _event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } = _event
            {
                let _ = commands::open_empty_window(_app);
            }

            // macOS/iOS/Android deliver file-association / "Open with" requests here.
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
            if let tauri::RunEvent::Opened { urls } = _event {
                if let Some(path) = urls
                    .iter()
                    .filter_map(|u| u.to_file_path().ok())
                    .find(|p| open_request::is_photometric(p))
                {
                    open_request::deliver(_app, &path);
                }
            }
        });
}
