//! The macOS app menu. Custom items emit a `menu` event carrying their id to
//! the focused window (see `lib.rs`), so each window handles its own document.
//!
//! macOS only: on Windows/Linux the menu would be rendered as a per-window
//! menu bar, which clashes with the app's custom title bar. Those platforms
//! rely on the webview keyboard handler for the same shortcuts instead.

use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::{AppHandle, Wry};

pub fn build(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let app_menu = SubmenuBuilder::new(app, "EulumdatEdit")
        .about(Some(AboutMetadata {
            name: Some("EulumdatEdit".to_string()),
            ..Default::default()
        }))
        .separator()
        .hide()
        .hide_others()
        .show_all()
        .separator()
        // Custom item instead of the predefined Quit: native termination skips
        // the window close-request handler and its unsaved-changes guard.
        .item(
            &MenuItemBuilder::with_id("quit", "Quit EulumdatEdit")
                .accelerator("CmdOrCtrl+Q")
                .build(app)?,
        )
        .build()?;

    let file_menu = SubmenuBuilder::new(app, "File")
        .item(
            &MenuItemBuilder::with_id("new", "New")
                .accelerator("CmdOrCtrl+N")
                .build(app)?,
        )
        .item(
            &MenuItemBuilder::with_id("open", "Open…")
                .accelerator("CmdOrCtrl+O")
                .build(app)?,
        )
        .separator()
        // Cmd+W closes the document rather than the window.
        .item(
            &MenuItemBuilder::with_id("close", "Close")
                .accelerator("CmdOrCtrl+W")
                .build(app)?,
        )
        .close_window_with_text("Close Window")
        .build()?;

    let edit_menu = SubmenuBuilder::new(app, "Edit")
        .undo()
        .redo()
        .separator()
        .cut()
        .copy()
        .paste()
        .select_all()
        .build()?;

    let view_menu = SubmenuBuilder::new(app, "View").fullscreen().build()?;

    let window_menu = SubmenuBuilder::new(app, "Window")
        .minimize()
        .separator()
        .close_window()
        .build()?;

    MenuBuilder::new(app)
        .items(&[&app_menu, &file_menu, &edit_menu, &view_menu, &window_menu])
        .build()
}
