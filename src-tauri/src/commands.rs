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
use tauri::State;

use crate::dto::{warnings_to_dto, DocResponse, EulumdatDto, PhotometryDto};
use crate::state::AppState;

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
        path,
        dirty,
        strict_validation: strict,
    })
}

/// Creates a new luminaire from a built-in default template.
#[tauri::command]
pub fn new_from_template(state: State<'_, AppState>) -> Result<DocResponse, String> {
    let model = template_model();
    let mut doc = state.doc.lock().unwrap();
    doc.model = Some(model.clone());
    doc.path = None;
    doc.dirty = false;
    respond(&model, None, false, doc.strict_validation)
}

/// Opens and parses a `.ldt` file from disk.
#[tauri::command]
pub fn open_file(path: String, state: State<'_, AppState>) -> Result<DocResponse, String> {
    let (model, _warnings) = Eulumdat::from_path(&path).map_err(|e| e.to_string())?;
    let mut doc = state.doc.lock().unwrap();
    doc.model = Some(model.clone());
    doc.path = Some(path.clone());
    doc.dirty = false;
    respond(&model, Some(path), false, doc.strict_validation)
}

/// Clears the open document and returns the app to its empty state.
#[tauri::command]
pub fn close_document(state: State<'_, AppState>) -> Result<(), String> {
    let mut doc = state.doc.lock().unwrap();
    *doc = crate::state::OpenDoc::default();
    Ok(())
}

/// Returns and clears any file the OS queued for opening before the UI was
/// ready (e.g. launching the app by double-clicking a `.ldt` file).
#[tauri::command]
pub fn take_pending_open(state: State<'_, AppState>) -> Option<String> {
    state
        .frontend_ready
        .store(true, std::sync::atomic::Ordering::SeqCst);
    state.pending_open.lock().unwrap().take()
}

/// Replaces the in-memory model with an edited DTO from the UI.
///
/// This is called after each edit; it rebuilds, validates, and recomputes.
#[tauri::command]
pub fn update_document(
    doc: EulumdatDto,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let model = doc.to_model().map_err(|e| e.to_string())?;
    let mut state_doc = state.doc.lock().unwrap();
    state_doc.model = Some(model.clone());
    state_doc.dirty = true;
    let path = state_doc.path.clone();
    respond(&model, path, true, state_doc.strict_validation)
}

/// Saves the current model to its existing path.
#[tauri::command]
pub fn save(state: State<'_, AppState>) -> Result<DocResponse, String> {
    let mut doc = state.doc.lock().unwrap();
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
}

/// Saves the current model to a new path and associates the document with it.
#[tauri::command]
pub fn save_as(path: String, state: State<'_, AppState>) -> Result<DocResponse, String> {
    let mut doc = state.doc.lock().unwrap();
    let model = doc
        .model
        .clone()
        .ok_or_else(|| "No document open".to_string())?;
    model.write_path(&path).map_err(|e| e.to_string())?;
    doc.path = Some(path.clone());
    doc.dirty = false;
    respond(&model, Some(path), false, doc.strict_validation)
}

/// Resamples the gamma table to a new angular step.
#[tauri::command]
pub fn resample_gamma(step: u32, state: State<'_, AppState>) -> Result<DocResponse, String> {
    let mut doc = state.doc.lock().unwrap();
    let mut model = doc
        .model
        .clone()
        .ok_or_else(|| "No document open".to_string())?;
    model.resample_gamma(step).map_err(|e| e.to_string())?;
    doc.model = Some(model.clone());
    doc.dirty = true;
    let path = doc.path.clone();
    respond(&model, path, true, doc.strict_validation)
}

/// Scales the distribution so its peak reaches 100% (1000 cd/klm at peak C-plane).
#[tauri::command]
pub fn scale_to_100_percent(state: State<'_, AppState>) -> Result<DocResponse, String> {
    let mut doc = state.doc.lock().unwrap();
    let mut model = doc
        .model
        .clone()
        .ok_or_else(|| "No document open".to_string())?;
    model.scale_to_100_percent();
    doc.model = Some(model.clone());
    doc.dirty = true;
    let path = doc.path.clone();
    respond(&model, path, true, doc.strict_validation)
}

/// Enables or disables the legacy strict text-length validation and
/// re-validates the open document under the new setting.
#[tauri::command]
pub fn set_strict_validation(
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<DocResponse, String> {
    let mut doc = state.doc.lock().unwrap();
    doc.strict_validation = enabled;
    let model = doc
        .model
        .clone()
        .ok_or_else(|| "No document open".to_string())?;
    let path = doc.path.clone();
    let dirty = doc.dirty;
    respond(&model, path, dirty, enabled)
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
    state: State<'_, AppState>,
) -> Result<String, String> {
    let doc = state.doc.lock().unwrap();
    let model = doc
        .model
        .as_ref()
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
/// (SVG markup or rasterized PNG) to a user-chosen location.
#[tauri::command]
pub fn write_bytes(path: String, contents: Vec<u8>) -> Result<(), String> {
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
    use crate::dto::EulumdatDto;
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
    fn renders_polar_svg() {
        let model = template_model();
        let svg = model
            .to_polar_svg(&PolarDiagramOptions::default())
            .expect("polar svg should render");
        assert!(svg.contains("<svg"));
    }
}
