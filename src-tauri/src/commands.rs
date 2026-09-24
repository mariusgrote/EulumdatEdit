//! Tauri commands — the bridge between the SvelteKit UI and `eulumdat-core`.
//!
//! The Rust model is the single source of truth. Every mutating command
//! validates and returns a fresh [`DocResponse`] (model + warnings + derived
//! photometry) so the frontend never has to re-derive anything.

use eulumdat_core::{
    Eulumdat, IntensityMode, PlanePair, PolarDiagramOptions, PolarDiagramPresentation, Symmetry,
    TypeIndicator, ValidationSettings,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, State, WebviewWindow, WebviewWindowBuilder};

use crate::dto::{warnings_to_dto, DocResponse, EulumdatDto, PhotometryDto, UgrDto};
use crate::ies;
use crate::state::{AppState, OpenDoc, Workspace};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TabSummary {
    pub id: String,
    pub title: String,
    pub path: Option<String>,
    pub dirty: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowStateResponse {
    pub document: Option<DocResponse>,
    pub tabs: Vec<TabSummary>,
    pub active_tab_id: Option<String>,
}

/// Builds the standard response bundle from the current document.
///
/// `strict` selects the legacy EULUMDAT text-length limits; when false the
/// model is validated with `ValidationSettings::unrestricted()` (the default).
fn respond(
    model: &Eulumdat,
    path: Option<String>,
    dirty: bool,
    strict: bool,
) -> Result<DocResponse, String> {
    let settings = if strict {
        ValidationSettings::restricted()
    } else {
        ValidationSettings::unrestricted()
    };
    let warnings = model.validate(settings).map_err(|e| e.to_string())?;
    Ok(DocResponse {
        doc: EulumdatDto::from(model),
        warnings: warnings_to_dto(&warnings),
        photometry: PhotometryDto::from_model(model),
        ugr: UgrDto::from_model(model),
        path,
        dirty,
        strict_validation: strict,
    })
}

fn active_doc_id(workspace: &Workspace, window_label: &str) -> Result<String, String> {
    workspace
        .windows
        .get(window_label)
        .and_then(|window| window.active.clone())
        .ok_or_else(|| "No document open".to_string())
}

/// Runs `f` on the active document shown in `window`.
fn with_doc<T>(
    state: &AppState,
    window: &WebviewWindow,
    f: impl FnOnce(&mut OpenDoc) -> Result<T, String>,
) -> Result<T, String> {
    let mut workspace = state.workspace.lock().unwrap();
    let id = active_doc_id(&workspace, window.label())?;
    let doc = workspace
        .docs
        .get_mut(&id)
        .ok_or_else(|| "Active document is missing".to_string())?;
    f(doc)
}

fn tab_title(doc: &OpenDoc) -> String {
    doc.path
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled")
        .to_string()
}

fn window_state(workspace: &Workspace, window_label: &str) -> Result<WindowStateResponse, String> {
    let Some(window) = workspace.windows.get(window_label) else {
        return Ok(WindowStateResponse {
            document: None,
            tabs: Vec::new(),
            active_tab_id: None,
        });
    };
    let tabs = window
        .tabs
        .iter()
        .filter_map(|id| {
            workspace.docs.get(id).map(|doc| TabSummary {
                id: id.clone(),
                title: tab_title(doc),
                path: doc.path.clone(),
                dirty: doc.dirty,
            })
        })
        .collect();
    let document = window
        .active
        .as_ref()
        .and_then(|id| workspace.docs.get(id))
        .and_then(|doc| {
            doc.model
                .as_ref()
                .map(|model| respond(model, doc.path.clone(), doc.dirty, doc.strict_validation))
        })
        .transpose()?;
    Ok(WindowStateResponse {
        document,
        tabs,
        active_tab_id: window.active.clone(),
    })
}

fn insert_document(
    state: &AppState,
    window_label: &str,
    model: Eulumdat,
    path: Option<String>,
    strict_validation: bool,
) -> Result<WindowStateResponse, String> {
    let id = format!("tab-{}", state.next_doc_id.fetch_add(1, Ordering::SeqCst));
    let mut workspace = state.workspace.lock().unwrap();
    workspace.docs.insert(
        id.clone(),
        OpenDoc {
            model: Some(model),
            path,
            dirty: false,
            strict_validation,
        },
    );
    let window = workspace
        .windows
        .entry(window_label.to_string())
        .or_default();
    window.tabs.push(id.clone());
    window.active = Some(id);
    window_state(&workspace, window_label)
}

/// Reads and parses `path`, naming the path in the error so a failed open
/// shows which file the backend was actually asked for.
fn load(path: &str) -> Result<(Eulumdat, String), String> {
    let canonical =
        std::fs::canonicalize(path).map_err(|e| format!("Could not open {path:?}: {e}"))?;
    let model = match file_format(&canonical)? {
        "ies" => {
            let contents =
                std::fs::read(&canonical).map_err(|e| format!("Could not open {path:?}: {e}"))?;
            let name = canonical.file_name().unwrap_or_default().to_string_lossy();
            ies::parse_bytes(&contents, &name)
                .map_err(|e| format!("Could not open {path:?}: {e}"))?
        }
        _ => Eulumdat::from_path(&canonical)
            .map(|(model, _warnings)| model)
            .map_err(|e| format!("Could not open {path:?}: {e}"))?,
    };
    Ok((model, canonical.to_string_lossy().into_owned()))
}

fn file_format(path: &Path) -> Result<&str, String> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("ies") => Ok("ies"),
        Some(extension) if extension.eq_ignore_ascii_case("ldt") => Ok("ldt"),
        _ => Err("File must have an .ldt or .ies extension".into()),
    }
}

/// Writes a document and returns the model that opening the written file
/// would produce. IES omits EULUMDAT-only fields and uses an absolute flux
/// basis, so the editor must show the saved representation after writing it.
fn write_document(model: &Eulumdat, path: &Path) -> Result<Eulumdat, String> {
    match file_format(path)? {
        "ies" => {
            let text = ies::serialize(model)?;
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let saved = ies::parse(&text, &name)?;
            std::fs::write(path, text).map_err(|e| format!("Could not write {path:?}: {e}"))?;
            Ok(saved)
        }
        _ => {
            model.write_path(path).map_err(|e| e.to_string())?;
            Ok(model.clone())
        }
    }
}

fn canonical_destination(path: &str) -> Result<String, String> {
    let candidate = PathBuf::from(path);
    if candidate.exists() {
        return std::fs::canonicalize(&candidate)
            .map(|path| path.to_string_lossy().into_owned())
            .map_err(|error| format!("Could not resolve {path:?}: {error}"));
    }
    let file_name = candidate
        .file_name()
        .ok_or_else(|| "Save path has no file name".to_string())?;
    let parent = candidate
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let parent = std::fs::canonicalize(parent)
        .map_err(|error| format!("Could not resolve {path:?}: {error}"))?;
    Ok(parent.join(file_name).to_string_lossy().into_owned())
}

/// Creates a new luminaire from a built-in default template.
#[tauri::command]
pub fn new_from_template(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let strict_validation = active_strict_validation(&state, window.label());
    insert_document(
        &state,
        window.label(),
        template_model(),
        None,
        strict_validation,
    )
}

/// Opens and parses a `.ldt` or `.ies` file from disk.
#[tauri::command]
pub fn open_file(
    path: String,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let (model, canonical_path) = load(&path)?;

    let existing = {
        let workspace = state.workspace.lock().unwrap();
        workspace
            .docs
            .iter()
            .find(|(_, doc)| doc.path.as_deref() == Some(canonical_path.as_str()))
            .and_then(|(id, _)| {
                workspace.windows.iter().find_map(|(label, tabs)| {
                    tabs.tabs.contains(id).then(|| (label.clone(), id.clone()))
                })
            })
    };
    if let Some((label, id)) = existing {
        let response = {
            let mut workspace = state.workspace.lock().unwrap();
            if let Some(tabs) = workspace.windows.get_mut(&label) {
                tabs.active = Some(id);
            }
            window_state(&workspace, window.label())?
        };
        if let Some(existing_window) = app.get_webview_window(&label) {
            let _ = existing_window.unminimize();
            let _ = existing_window.set_focus();
            let _ = existing_window.emit("workspace-changed", ());
        }
        return Ok(response);
    }

    let strict_validation = active_strict_validation(&state, window.label());
    insert_document(
        &state,
        window.label(),
        model,
        Some(canonical_path),
        strict_validation,
    )
}

/// Reloads the active tab from disk, discarding its in-memory edits.
#[tauri::command]
pub fn reload_document(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    with_doc(&state, &window, |doc| {
        let path = doc
            .path
            .clone()
            .ok_or_else(|| "No file path set".to_string())?;
        let (model, _) = load(&path)?;
        let response = respond(&model, Some(path), false, doc.strict_validation)?;
        doc.model = Some(model);
        doc.dirty = false;
        Ok(response)
    })
}

fn active_strict_validation(state: &AppState, window_label: &str) -> bool {
    let workspace = state.workspace.lock().unwrap();
    workspace
        .windows
        .get(window_label)
        .and_then(|window| window.active.as_ref())
        .and_then(|id| workspace.docs.get(id))
        .is_some_and(|doc| doc.strict_validation)
}

fn build_window_at(app: &AppHandle, label: &str, x: f64, y: f64) -> tauri::Result<()> {
    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .unwrap_or_default();
    config.label = label.to_string();
    WebviewWindowBuilder::from_config(app, &config)?
        .position(x, y)
        .build()?;
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn open_empty_window(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let id = state.next_window_id.fetch_add(1, Ordering::SeqCst);
    let label = format!("doc-{id}");
    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .unwrap_or_default();
    config.label = label;
    WebviewWindowBuilder::from_config(app, &config)
        .and_then(|builder| builder.build())
        .map(|_| ())
        .map_err(|error| error.to_string())
}

pub fn open_new_document_window(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let window_id = state.next_window_id.fetch_add(1, Ordering::SeqCst);
    let label = format!("doc-{window_id}");
    insert_document(&state, &label, template_model(), None, false)?;

    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .unwrap_or_default();
    config.label = label.clone();
    let built = WebviewWindowBuilder::from_config(app, &config).and_then(|builder| builder.build());
    if let Err(error) = built {
        remove_window(&state, &label);
        return Err(error.to_string());
    }
    Ok(())
}

pub fn open_path_window(app: &AppHandle, path: &Path) -> Result<(), String> {
    let (model, canonical_path) = load(&path.to_string_lossy())?;
    let state = app.state::<AppState>();
    let window_id = state.next_window_id.fetch_add(1, Ordering::SeqCst);
    let label = format!("doc-{window_id}");
    insert_document(&state, &label, model, Some(canonical_path), false)?;

    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .unwrap_or_default();
    config.label = label.clone();
    let built = WebviewWindowBuilder::from_config(app, &config).and_then(|builder| builder.build());
    if let Err(error) = built {
        remove_window(&state, &label);
        return Err(error.to_string());
    }
    Ok(())
}

/// Returns the active document and ordered tabs for this window.
#[tauri::command]
pub fn current_document(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let mut workspace = state.workspace.lock().unwrap();
    workspace
        .windows
        .entry(window.label().to_string())
        .or_default();
    window_state(&workspace, window.label())
}

/// Closes a tab and activates its right-hand neighbour, or its left-hand
/// neighbour when it was last in the row.
#[tauri::command]
pub fn close_document(
    tab_id: Option<String>,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let mut workspace = state.workspace.lock().unwrap();
    let id = tab_id
        .or_else(|| {
            workspace
                .windows
                .get(window.label())
                .and_then(|tabs| tabs.active.clone())
        })
        .ok_or_else(|| "No document open".to_string())?;
    let tabs = workspace
        .windows
        .get_mut(window.label())
        .ok_or_else(|| "Window is missing".to_string())?;
    let index = tabs
        .tabs
        .iter()
        .position(|candidate| candidate == &id)
        .ok_or_else(|| "Tab does not belong to this window".to_string())?;
    tabs.tabs.remove(index);
    if tabs.active.as_deref() == Some(&id) {
        tabs.active = tabs
            .tabs
            .get(index)
            .or_else(|| index.checked_sub(1).and_then(|i| tabs.tabs.get(i)))
            .cloned();
    }
    workspace.docs.remove(&id);
    window_state(&workspace, window.label())
}

#[tauri::command]
pub fn activate_tab(
    tab_id: String,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let mut workspace = state.workspace.lock().unwrap();
    let tabs = workspace
        .windows
        .get_mut(window.label())
        .ok_or_else(|| "Window is missing".to_string())?;
    if !tabs.tabs.contains(&tab_id) {
        return Err("Tab does not belong to this window".to_string());
    }
    tabs.active = Some(tab_id);
    window_state(&workspace, window.label())
}

fn move_tab_in_workspace(
    workspace: &mut Workspace,
    tab_id: &str,
    source_label: &str,
    target_label: &str,
    target_index: Option<usize>,
) -> Result<(), String> {
    let source = workspace
        .windows
        .get_mut(source_label)
        .ok_or_else(|| "Source window is missing".to_string())?;
    let source_index = source
        .tabs
        .iter()
        .position(|id| id == tab_id)
        .ok_or_else(|| "Tab does not belong to the source window".to_string())?;
    let target_index = target_index.map(|index| {
        if source_label == target_label && index > source_index {
            index - 1
        } else {
            index
        }
    });
    source.tabs.remove(source_index);
    if source.active.as_deref() == Some(tab_id) {
        source.active = source
            .tabs
            .get(source_index)
            .or_else(|| {
                source_index
                    .checked_sub(1)
                    .and_then(|index| source.tabs.get(index))
            })
            .cloned();
    }

    let target = workspace
        .windows
        .entry(target_label.to_string())
        .or_default();
    let index = target_index
        .unwrap_or(target.tabs.len())
        .min(target.tabs.len());
    target.tabs.insert(index, tab_id.to_string());
    target.active = Some(tab_id.to_string());
    Ok(())
}

#[tauri::command]
pub fn move_tab(
    tab_id: String,
    target_window: String,
    target_index: Option<usize>,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let source_label = window.label().to_string();
    let response = {
        let mut workspace = state.workspace.lock().unwrap();
        move_tab_in_workspace(
            &mut workspace,
            &tab_id,
            &source_label,
            &target_window,
            target_index,
        )?;
        window_state(&workspace, &source_label)?
    };
    if let Some(target) = app.get_webview_window(&target_window) {
        let _ = target.set_focus();
        let _ = target.emit("workspace-changed", ());
    }
    Ok(response)
}

#[tauri::command]
pub async fn detach_tab(
    tab_id: String,
    x: f64,
    y: f64,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<WindowStateResponse, String> {
    let source_label = window.label().to_string();
    let id = state.next_window_id.fetch_add(1, Ordering::SeqCst);
    let target_label = format!("doc-{id}");
    {
        let mut workspace = state.workspace.lock().unwrap();
        move_tab_in_workspace(&mut workspace, &tab_id, &source_label, &target_label, None)?;
    }

    let built = build_window_at(&app, &target_label, x, y);
    if let Err(error) = built {
        let mut workspace = state.workspace.lock().unwrap();
        let _ = move_tab_in_workspace(&mut workspace, &tab_id, &target_label, &source_label, None);
        workspace.windows.remove(&target_label);
        return Err(error.to_string());
    }

    let workspace = state.workspace.lock().unwrap();
    window_state(&workspace, &source_label)
}

pub fn remove_window(state: &AppState, label: &str) {
    let mut workspace = state.workspace.lock().unwrap();
    if let Some(window) = workspace.windows.remove(label) {
        for id in window.tabs {
            workspace.docs.remove(&id);
        }
    }
}

/// Whether any document other than the calling window's active tab is dirty.
#[tauri::command]
pub fn other_documents_dirty(window: WebviewWindow, state: State<'_, AppState>) -> bool {
    let workspace = state.workspace.lock().unwrap();
    let active = workspace
        .windows
        .get(window.label())
        .and_then(|tabs| tabs.active.as_deref());
    workspace
        .docs
        .iter()
        .any(|(id, doc)| Some(id.as_str()) != active && doc.dirty)
}

/// Exits the application unconditionally.
///
/// The frontend calls this only after its unsaved-changes guard passes; Rust
/// cannot see uncommitted drafts, so it does not check the dirty flag itself.
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// Returns and clears any file the OS queued for opening before the UI was
/// ready (e.g. launching the app by double-clicking a `.ldt` file).
#[tauri::command]
pub fn take_pending_open(state: State<'_, AppState>) -> Option<String> {
    state.frontend_ready.store(true, Ordering::SeqCst);
    state.pending_open.lock().unwrap().take()
}

/// Replaces the in-memory model with an edited DTO from the UI.
///
/// This is called after each edit; it rebuilds, validates, and recomputes.
#[tauri::command]
pub fn update_document(
    doc: EulumdatDto,
    tab_id: String,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let model = doc.to_model().map_err(|e| e.to_string())?;
    let mut workspace = state.workspace.lock().unwrap();
    let belongs_to_window = workspace
        .windows
        .get(window.label())
        .is_some_and(|tabs| tabs.tabs.contains(&tab_id));
    if !belongs_to_window {
        return Err("Tab does not belong to this window".to_string());
    }
    let state_doc = workspace
        .docs
        .get_mut(&tab_id)
        .ok_or_else(|| "Document is missing".to_string())?;
    state_doc.model = Some(model.clone());
    state_doc.dirty = true;
    respond(
        &model,
        state_doc.path.clone(),
        true,
        state_doc.strict_validation,
    )
}

/// Saves the current model to its existing path.
#[tauri::command]
pub fn save(window: WebviewWindow, state: State<'_, AppState>) -> Result<DocResponse, String> {
    with_doc(&state, &window, |doc| {
        let path = doc
            .path
            .clone()
            .ok_or_else(|| "No file path set; use Save As".to_string())?;
        let model = doc
            .model
            .clone()
            .ok_or_else(|| "No document open".to_string())?;
        let saved = write_document(&model, Path::new(&path))?;
        doc.model = Some(saved.clone());
        doc.dirty = false;
        respond(&saved, Some(path), false, doc.strict_validation)
    })
}

/// Saves the current model to a new path and associates the document with it.
#[tauri::command]
pub fn save_as(
    path: String,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let canonical_path = canonical_destination(&path)?;
    let mut workspace = state.workspace.lock().unwrap();
    let id = active_doc_id(&workspace, window.label())?;
    if workspace.docs.iter().any(|(other_id, doc)| {
        other_id != &id && doc.path.as_deref() == Some(canonical_path.as_str())
    }) {
        return Err("This file is already open in another tab".to_string());
    }
    let doc = workspace
        .docs
        .get_mut(&id)
        .ok_or_else(|| "No document open".to_string())?;
    let model = doc
        .model
        .clone()
        .ok_or_else(|| "No document open".to_string())?;
    let saved = write_document(&model, Path::new(&canonical_path))?;
    doc.model = Some(saved.clone());
    doc.path = Some(canonical_path.clone());
    doc.dirty = false;
    respond(&saved, Some(canonical_path), false, doc.strict_validation)
}

/// Writes an IES copy without changing the document's path or dirty state.
#[tauri::command]
pub fn export_ies(
    path: String,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if !path.to_ascii_lowercase().ends_with(".ies") {
        return Err("IES export path must end in .ies".into());
    }
    let destination = canonical_destination(&path)?;
    let workspace = state.workspace.lock().unwrap();
    if workspace
        .docs
        .values()
        .any(|doc| doc.path.as_deref() == Some(destination.as_str()))
    {
        return Err("Cannot export over an open document".into());
    }
    let id = active_doc_id(&workspace, window.label())?;
    let model = workspace
        .docs
        .get(&id)
        .and_then(|doc| doc.model.as_ref())
        .ok_or("No document open")?;
    write_document(model, Path::new(&destination)).map(|_| ())
}

/// Resamples the gamma table to a new angular step.
#[tauri::command]
pub fn resample_gamma(
    step: u32,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    with_doc(&state, &window, |doc| {
        let mut model = doc
            .model
            .clone()
            .ok_or_else(|| "No document open".to_string())?;
        model.resample_gamma(step).map_err(|e| e.to_string())?;
        doc.model = Some(model.clone());
        doc.dirty = true;
        respond(&model, doc.path.clone(), true, doc.strict_validation)
    })
}

/// Scales the distribution so its peak reaches 100% (1000 cd/klm at peak C-plane).
#[tauri::command]
pub fn scale_to_100_percent(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    with_doc(&state, &window, |doc| {
        let mut model = doc
            .model
            .clone()
            .ok_or_else(|| "No document open".to_string())?;
        model.scale_to_100_percent();
        doc.model = Some(model.clone());
        doc.dirty = true;
        respond(&model, doc.path.clone(), true, doc.strict_validation)
    })
}

/// Enables or disables the legacy strict text-length validation and
/// re-validates the open document under the new setting.
#[tauri::command]
pub fn set_strict_validation(
    enabled: bool,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    with_doc(&state, &window, |doc| {
        doc.strict_validation = enabled;
        let model = doc
            .model
            .clone()
            .ok_or_else(|| "No document open".to_string())?;
        respond(&model, doc.path.clone(), doc.dirty, enabled)
    })
}

/// Options for the polar diagram, as sent from the UI.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolarOptionsDto {
    pub width: u32,
    pub height: u32,
    /// Plane identifiers: "c0c180", "c90c270", "c45c225", "c135c315".
    pub planes: Vec<String>,
    pub show_grid: bool,
    pub show_legend: bool,
    pub show_axis_labels: bool,
    /// "stored" or "converted".
    pub intensity_mode: String,
    /// "classic" or "focused".
    pub presentation: String,
    pub title: Option<String>,
}

fn plane_from_str(s: &str) -> Option<PlanePair> {
    match s.to_ascii_lowercase().as_str() {
        "c0c180" => Some(PlanePair::C0C180),
        "c90c270" => Some(PlanePair::C90C270),
        "c45c225" => Some(PlanePair::C45C225),
        "c135c315" => Some(PlanePair::C135C315),
        _ => None,
    }
}

/// Renders the current model's polar luminous-intensity diagram as SVG markup.
#[tauri::command]
pub fn render_polar_svg(
    options: PolarOptionsDto,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let workspace = state.workspace.lock().unwrap();
    let id = active_doc_id(&workspace, window.label())?;
    let model = workspace
        .docs
        .get(&id)
        .and_then(|doc| doc.model.as_ref())
        .ok_or_else(|| "No document open".to_string())?;

    let planes: Vec<PlanePair> = options
        .planes
        .iter()
        .filter_map(|s| plane_from_str(s))
        .collect();

    let intensity_mode = match options.intensity_mode.as_str() {
        "converted" => IntensityMode::ConvertedByFactor,
        _ => IntensityMode::StoredCandelaPerKilolumen,
    };
    let presentation = match options.presentation.as_str() {
        "focused" => PolarDiagramPresentation::Focused,
        _ => PolarDiagramPresentation::Classic,
    };

    let opts = PolarDiagramOptions {
        width: options.width,
        height: options.height,
        margin: 56.0,
        title: options.title,
        planes: if planes.is_empty() {
            vec![PlanePair::C0C180, PlanePair::C90C270]
        } else {
            planes
        },
        show_grid: options.show_grid,
        show_legend: options.show_legend,
        show_axis_labels: options.show_axis_labels,
        intensity_mode,
        presentation,
    };

    model.to_polar_svg(&opts).map_err(|e| e.to_string())
}

/// Writes raw bytes to a path on disk. Used by the UI to save exported graphs
/// (SVG markup or rasterized PNG) to a user-chosen location. Restricted to
/// those export formats so this command is not a general write primitive.
#[tauri::command]
pub fn write_bytes(path: String, contents: Vec<u8>) -> Result<(), String> {
    let allowed = std::path::Path::new(&path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("svg") || e.eq_ignore_ascii_case("png"));
    if !allowed {
        return Err("write_bytes only writes .svg or .png graph exports".to_string());
    }
    std::fs::write(&path, contents).map_err(|e| e.to_string())
}

/// A minimal valid luminaire used for "New".
fn template_model() -> Eulumdat {
    use eulumdat_core::{Distribution, LampSet};

    let gamma_angles: Vec<f64> = (0..=18).map(|i| f64::from(i) * 10.0).collect();
    // Rotationally symmetric cosine-like beam as a sensible starting point.
    let intensities: Vec<Vec<f64>> = vec![gamma_angles
        .iter()
        .map(|g| (1000.0 * g.to_radians().cos()).max(0.0))
        .collect()];

    let mut model = Eulumdat {
        identification: "EulumdatEdit".to_string(),
        type_indicator: TypeIndicator::PointSourceWithSymmetry,
        symmetry: Symmetry::Rotational,
        measurement_report_number: String::new(),
        luminaire_name: "New luminaire".to_string(),
        luminaire_number: String::new(),
        file_name: "new.ldt".to_string(),
        date_user: String::new(),
        luminaire_length: 100.0,
        luminaire_width: 100.0,
        luminaire_height: 50.0,
        luminous_area_length: 80.0,
        luminous_area_width: 80.0,
        downward_flux_fraction: 100.0,
        light_output_ratio: 100.0,
        conversion_factor: 1.0,
        tilt: 0.0,
        lamps: vec![LampSet {
            lamp_count: 1,
            lamp_type: "LED".to_string(),
            total_luminous_flux: 1000.0,
            color_temperature: "4000K".to_string(),
            color_rendering_index: "80".to_string(),
            wattage_including_ballast: 10.0,
        }],
        direct_ratios: [0.0; 10],
        ..Eulumdat::default()
    };

    model
        .replace_distribution(Distribution {
            symmetry: Symmetry::Rotational,
            c_plane_step: 0.0,
            gamma_step: 10.0,
            c_planes: vec![0.0],
            gamma_angles,
            intensities,
        })
        .expect("template distribution should be valid");

    model
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::{EulumdatDto, UgrDto};
    use eulumdat_core::{Eulumdat, ValidationSettings};

    #[test]
    fn template_is_valid_and_serializes() {
        let model = template_model();
        model
            .validate(ValidationSettings::restricted())
            .expect("template should validate");
        let text = model.to_text();
        let (reparsed, _) = Eulumdat::parse(&text).expect("template should reparse");
        assert_eq!(reparsed.luminaire_name, model.luminaire_name);
    }

    #[test]
    fn ies_file_round_trips_through_file_commands() {
        let path = temp_export_path("round-trip.ies");
        let mut model = template_model();
        model.lamps[0].total_luminous_flux = 2000.0;
        let saved = write_document(&model, &path).expect("IES file should be written");
        let (reloaded, _) = load(&path.to_string_lossy()).expect("IES file should open");
        assert_eq!(saved, reloaded);
        assert_eq!(saved.lamps[0].total_luminous_flux, 1000.0);
        assert_ne!(saved.intensities, model.intensities);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn dto_round_trips_through_model() {
        let model = template_model();
        let dto = EulumdatDto::from(&model);
        let rebuilt = dto.to_model().expect("dto should rebuild a model");
        assert_eq!(rebuilt.to_text(), model.to_text());
    }

    #[test]
    fn ugr_table_is_available_on_a_fine_angle_grid() {
        let mut model = template_model();
        model.resample_gamma(5).expect("template should resample");
        let UgrDto::Available(table) = UgrDto::from_model(&model) else {
            panic!("UGR table should apply to the resampled template");
        };
        assert_eq!(table.reflectances.len(), 5);
        assert_eq!(table.lamp_flux_values.rows.len(), 19);
        assert_eq!(table.lamp_flux_values.rows[10].x_h, 4);
        assert_eq!(table.lamp_flux_values.rows[10].y_h, 8);
        assert_eq!(
            table.lamp_flux_values.data_sheet_crosswise,
            table.lamp_flux_values.rows[10].crosswise[0]
        );
    }

    #[test]
    fn ugr_blockers_point_to_their_fields() {
        let mut model = template_model();
        model.luminous_area_length = 0.0;
        model.lamps[0].total_luminous_flux = 0.0;
        let UgrDto::Blocked { blockers } = UgrDto::from_model(&model) else {
            panic!("UGR table should be blocked");
        };
        let keys: Vec<_> = blockers
            .iter()
            .map(|b| (b.field_key.as_deref(), b.lamp_index))
            .collect();
        assert!(keys.contains(&(Some("luminousAreaLength"), None)));
        assert!(keys.contains(&(Some("totalLuminousFlux"), Some(0))));
        // The 10° gamma grid of the template is too coarse; it has no field.
        assert!(keys.contains(&(None, None)));
    }

    #[test]
    fn renders_polar_svg() {
        let model = template_model();
        let svg = model
            .to_polar_svg(&PolarDiagramOptions::default())
            .expect("polar svg should render");
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn moving_active_tab_selects_neighbour_and_activates_destination() {
        let mut workspace = Workspace::default();
        workspace.windows.insert(
            "source".to_string(),
            crate::state::WindowTabs {
                tabs: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                active: Some("b".to_string()),
            },
        );
        workspace.windows.insert(
            "target".to_string(),
            crate::state::WindowTabs {
                tabs: vec!["d".to_string()],
                active: Some("d".to_string()),
            },
        );

        move_tab_in_workspace(&mut workspace, "b", "source", "target", Some(0)).unwrap();

        assert_eq!(workspace.windows["source"].tabs, ["a", "c"]);
        assert_eq!(workspace.windows["source"].active.as_deref(), Some("c"));
        assert_eq!(workspace.windows["target"].tabs, ["b", "d"]);
        assert_eq!(workspace.windows["target"].active.as_deref(), Some("b"));
    }

    #[test]
    fn reordering_within_one_window_keeps_exactly_one_copy() {
        let mut workspace = Workspace::default();
        workspace.windows.insert(
            "main".to_string(),
            crate::state::WindowTabs {
                tabs: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                active: Some("a".to_string()),
            },
        );

        move_tab_in_workspace(&mut workspace, "a", "main", "main", Some(3)).unwrap();

        assert_eq!(workspace.windows["main"].tabs, ["b", "c", "a"]);
        assert_eq!(workspace.windows["main"].active.as_deref(), Some("a"));
    }

    /// Runs `eulumdat-core`'s validator on a model that trips every warning and
    /// checks each label translates to a field key. Fails when upstream rewords
    /// a warning label, instead of warning navigation silently breaking.
    #[test]
    fn every_validator_warning_maps_to_a_field_key() {
        let long = "x".repeat(1000);
        let mut model = template_model();
        model.identification = long.clone();
        model.measurement_report_number = long.clone();
        model.luminaire_name = long.clone();
        model.luminaire_number = long.clone();
        model.file_name = long.clone();
        model.date_user = long.clone();
        model.luminaire_length = 0.0;
        model.luminaire_width = -1.0;
        model.luminaire_height = -1.0;
        model.luminous_area_length = -1.0;
        model.luminous_area_width = -1.0;
        model.luminous_area_height_c0 = -1.0;
        model.luminous_area_height_c90 = -1.0;
        model.luminous_area_height_c180 = -1.0;
        model.luminous_area_height_c270 = -1.0;
        model.downward_flux_fraction = 101.0;
        model.light_output_ratio = 101.0;
        model.conversion_factor = 11.0;
        model.tilt = 181.0;
        let lamp = &mut model.lamps[0];
        lamp.lamp_count = 0;
        lamp.lamp_type = long.clone();
        lamp.total_luminous_flux = 0.0;
        lamp.color_temperature = long.clone();
        lamp.color_rendering_index = long;
        lamp.wattage_including_ballast = 0.0;
        model.direct_ratios[0] = 11.0;

        let warnings = model
            .validate(ValidationSettings::restricted())
            .expect("warnings should not be hard errors");
        let dtos = warnings_to_dto(&warnings);

        for dto in &dtos {
            assert!(
                dto.field_key.is_some() || dto.field.starts_with("k["),
                "no field key for validator warning {:?}",
                dto.field
            );
        }

        let mut keys: Vec<&str> = dtos.iter().filter_map(|d| d.field_key.as_deref()).collect();
        keys.sort_unstable();
        let mut expected = vec![
            "identification",
            "measurementReportNumber",
            "luminaireName",
            "luminaireNumber",
            "fileName",
            "dateUser",
            "luminaireLength",
            "luminaireWidth",
            "luminaireHeight",
            "luminousAreaLength",
            "luminousAreaWidth",
            "luminousAreaHeightC0",
            "luminousAreaHeightC90",
            "luminousAreaHeightC180",
            "luminousAreaHeightC270",
            "downwardFluxFraction",
            "lightOutputRatio",
            "conversionFactor",
            "tilt",
            "lampCount",
            "lampType",
            "totalLuminousFlux",
            "colorTemperature",
            "colorRenderingIndex",
            "wattageIncludingBallast",
        ];
        expected.sort_unstable();
        assert_eq!(keys, expected);
    }

    /// A unique temp path per test so parallel runs don't collide.
    fn temp_export_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("eulumdat-edit-{}-{name}", std::process::id()))
    }

    #[test]
    fn write_bytes_writes_svg_exports() {
        let path = temp_export_path("graph.svg");
        write_bytes(path.to_string_lossy().into_owned(), b"<svg/>".to_vec())
            .expect("svg export should be written");
        assert_eq!(std::fs::read(&path).unwrap(), b"<svg/>");
        std::fs::remove_file(&path).unwrap();
    }

    #[test]
    fn write_bytes_rejects_other_extensions() {
        let path = temp_export_path("notes.txt");
        let result = write_bytes(path.to_string_lossy().into_owned(), b"data".to_vec());
        assert!(result.is_err());
        assert!(!path.exists());
    }

    #[test]
    fn write_bytes_extension_check_is_case_insensitive() {
        let path = temp_export_path("graph.PNG");
        write_bytes(
            path.to_string_lossy().into_owned(),
            vec![0x89, b'P', b'N', b'G'],
        )
        .expect("uppercase .PNG should be accepted");
        assert!(path.exists());
        std::fs::remove_file(&path).unwrap();
    }
}
