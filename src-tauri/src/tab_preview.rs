//! A click-through native window for the document tab being dragged.

use tauri::{
    AppHandle, Manager, PhysicalPosition, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::state::AppState;

const PREFIX: &str = "tab-preview-";

pub fn is_preview_label(label: &str) -> bool {
    label.starts_with(PREFIX)
}

fn preview_label(source: &str, id: &str) -> String {
    format!("{PREFIX}{source}-{id}")
}

fn position_at_cursor(source: &WebviewWindow, preview: &WebviewWindow) -> Result<(), String> {
    let cursor = source.cursor_position().map_err(|e| e.to_string())?;
    let offset = source.scale_factor().map_err(|e| e.to_string())? * 12.0;
    preview
        .set_position(PhysicalPosition::new(cursor.x + offset, cursor.y + offset))
        .map_err(|e| e.to_string())
}

/// A generation token prevents an old drag's end/move from affecting a new drag.
#[tauri::command]
pub async fn start_tab_preview(
    id: String,
    encoded_title: String,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let source = window.label().to_string();
    if is_preview_label(&source)
        || id.len() > 64
        || id.is_empty()
        || !id.bytes().all(|c| c.is_ascii_hexdigit() || c == b'-')
        || encoded_title.len() > 4096
    {
        return Err("Invalid tab preview request".into());
    }
    let label = preview_label(&source, &id);
    let old_id = state
        .tab_previews
        .lock()
        .unwrap()
        .insert(source.clone(), id.clone());
    if let Some(old_id) = old_id {
        if let Some(old) = app.get_webview_window(&preview_label(&source, &old_id)) {
            let _ = old.destroy();
        }
    }

    let url = WebviewUrl::App(format!("drag-preview?title={encoded_title}").into());
    let result = WebviewWindowBuilder::new(&app, &label, url)
        .title("")
        .inner_size(280.0, 38.0)
        .decorations(false)
        .resizable(false)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focusable(false)
        .focused(false)
        .visible(false)
        .build();
    let preview = match result {
        Ok(preview) => preview,
        Err(error) => {
            let mut previews = state.tab_previews.lock().unwrap();
            if previews.get(&source) == Some(&id) {
                previews.remove(&source);
            }
            return Err(error.to_string());
        }
    };

    let still_active = state.tab_previews.lock().unwrap().get(&source) == Some(&id);
    if !still_active || app.get_webview_window(&source).is_none() {
        let _ = preview.destroy();
        return Ok(());
    }
    let result = preview
        .set_ignore_cursor_events(true)
        .map_err(|e| e.to_string())
        .and_then(|_| position_at_cursor(&window, &preview))
        .and_then(|_| preview.show().map_err(|e| e.to_string()));
    if let Err(error) = result {
        let _ = preview.destroy();
        let mut previews = state.tab_previews.lock().unwrap();
        if previews.get(&source) == Some(&id) {
            previews.remove(&source);
        }
        return Err(error.to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn move_tab_preview(
    id: String,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if state.tab_previews.lock().unwrap().get(window.label()) != Some(&id) {
        return Ok(());
    }
    if let Some(preview) = app.get_webview_window(&preview_label(window.label(), &id)) {
        position_at_cursor(&window, &preview)?;
    }
    Ok(())
}

#[tauri::command]
pub fn end_tab_preview(
    id: String,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) {
    let source = window.label();
    let mut previews = state.tab_previews.lock().unwrap();
    if previews.get(source) != Some(&id) {
        return;
    }
    previews.remove(source);
    drop(previews);
    if let Some(preview) = app.get_webview_window(&preview_label(source, &id)) {
        let _ = preview.destroy();
    }
}

pub fn remove_source(app: &AppHandle, source: &str) {
    let state = app.state::<AppState>();
    let id = state.tab_previews.lock().unwrap().remove(source);
    if let Some(preview) = id.and_then(|id| app.get_webview_window(&preview_label(source, &id))) {
        let _ = preview.destroy();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_labels_cannot_be_mistaken_for_document_windows() {
        assert!(is_preview_label(&preview_label("doc-1", "abc")));
        assert!(!is_preview_label("doc-1"));
        assert!(!is_preview_label("main"));
    }
}
