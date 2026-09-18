//! Tauri commands — the bridge between the SvelteKit UI and `eulumdat-core`.
//!
//! The Rust model is the single source of truth. Every mutating command
//! validates and returns a fresh [`DocResponse`] (model + warnings + derived
//! photometry) so the frontend never has to re-derive anything.

use eulumdat_core::{
    Eulumdat, IntensityMode, PlanePair, PolarDiagramOptions, Symmetry, TypeIndicator,
    ValidationSettings,
};
use serde::Deserialize;
use std::sync::atomic::Ordering;
use tauri::{AppHandle, State, WebviewWindow, WebviewWindowBuilder};

use crate::dto::{warnings_to_dto, DocResponse, EulumdatDto, PhotometryDto, UgrDto};
use crate::state::{AppState, OpenDoc};

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

/// Runs `f` on the document shown in `window`, creating an empty one on first use.
fn with_doc<T>(
    state: &AppState,
    window: &WebviewWindow,
    f: impl FnOnce(&mut OpenDoc) -> Result<T, String>,
) -> Result<T, String> {
    let mut docs = state.docs.lock().unwrap();
    f(docs.entry(window.label().to_string()).or_default())
}

/// Reads and parses `path`, naming the path in the error so a failed open
/// shows which file the backend was actually asked for.
fn load(path: &str) -> Result<Eulumdat, String> {
    Eulumdat::from_path(path)
        .map(|(model, _warnings)| model)
        .map_err(|e| format!("Could not open {path:?}: {e}"))
}

/// Creates a new luminaire from a built-in default template.
#[tauri::command]
pub fn new_from_template(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let model = template_model();
    with_doc(&state, &window, |doc| {
        doc.model = Some(model.clone());
        doc.path = None;
        doc.dirty = false;
        respond(&model, None, false, doc.strict_validation)
    })
}

/// Opens and parses a `.ldt` file from disk.
#[tauri::command]
pub fn open_file(
    path: String,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let model = load(&path)?;
    with_doc(&state, &window, |doc| {
        doc.model = Some(model.clone());
        doc.path = Some(path.clone());
        doc.dirty = false;
        respond(&model, Some(path), false, doc.strict_validation)
    })
}

/// Opens `path` (or a new template document when `None`) in a new window.
///
/// Parse errors are returned to the calling window before any window is
/// created. Async because creating a window from a sync command deadlocks on
/// Windows.
#[tauri::command]
pub async fn open_window(
    path: Option<String>,
    window: WebviewWindow,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let model = match &path {
        Some(path) => load(path)?,
        None => template_model(),
    };
    let strict_validation = with_doc(&state, &window, |doc| Ok(doc.strict_validation))?;

    let id = state.next_window_id.fetch_add(1, Ordering::SeqCst);
    let label = format!("doc-{id}");
    state.docs.lock().unwrap().insert(
        label.clone(),
        OpenDoc {
            model: Some(model),
            path,
            dirty: false,
            strict_validation,
        },
    );

    let built = build_window(&app, &window, &label);
    if built.is_err() {
        state.docs.lock().unwrap().remove(&label);
    }
    built.map_err(|e| e.to_string())
}

/// Creates a window like the configured main one, cascaded from `parent`.
fn build_window(app: &AppHandle, parent: &WebviewWindow, label: &str) -> tauri::Result<()> {
    let mut config = app
        .config()
        .app
        .windows
        .first()
        .cloned()
        .unwrap_or_default();
    config.label = label.to_string();
    let mut builder = WebviewWindowBuilder::from_config(app, &config)?;
    if let (Ok(pos), Ok(scale)) = (parent.outer_position(), parent.scale_factor()) {
        let pos = pos.to_logical::<f64>(scale);
        builder = builder.position(pos.x + 28.0, pos.y + 28.0);
    }
    builder.build()?;
    Ok(())
}

/// Returns the document already loaded for this window, if any. A window
/// created by [`open_window`] starts with its document in place.
#[tauri::command]
pub fn current_document(
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<Option<DocResponse>, String> {
    with_doc(&state, &window, |doc| match &doc.model {
        Some(model) => respond(model, doc.path.clone(), doc.dirty, doc.strict_validation).map(Some),
        None => Ok(None),
    })
}

/// Clears this window's document and returns it to the empty state.
#[tauri::command]
pub fn close_document(window: WebviewWindow, state: State<'_, AppState>) -> Result<(), String> {
    with_doc(&state, &window, |doc| {
        *doc = OpenDoc::default();
        Ok(())
    })
}

/// Whether any window other than the calling one holds unsaved changes.
#[tauri::command]
pub fn other_windows_dirty(window: WebviewWindow, state: State<'_, AppState>) -> bool {
    state
        .docs
        .lock()
        .unwrap()
        .iter()
        .any(|(label, doc)| label != window.label() && doc.dirty)
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
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let model = doc.to_model().map_err(|e| e.to_string())?;
    with_doc(&state, &window, |state_doc| {
        state_doc.model = Some(model.clone());
        state_doc.dirty = true;
        respond(
            &model,
            state_doc.path.clone(),
            true,
            state_doc.strict_validation,
        )
    })
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
        model.write_path(&path).map_err(|e| e.to_string())?;
        doc.dirty = false;
        respond(&model, Some(path), false, doc.strict_validation)
    })
}

/// Saves the current model to a new path and associates the document with it.
#[tauri::command]
pub fn save_as(
    path: String,
    window: WebviewWindow,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    with_doc(&state, &window, |doc| {
        let model = doc
            .model
            .clone()
            .ok_or_else(|| "No document open".to_string())?;
        model.write_path(&path).map_err(|e| e.to_string())?;
        doc.path = Some(path.clone());
        doc.dirty = false;
        respond(&model, Some(path), false, doc.strict_validation)
    })
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
    let docs = state.docs.lock().unwrap();
    let model = docs
        .get(window.label())
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
