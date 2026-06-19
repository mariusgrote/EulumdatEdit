//! Serializable data-transfer objects mirroring the `eulumdat-core` model.
//!
//! The frontend never sees `eulumdat-core` types directly; it exchanges these
//! plain serde structs. Enums are transported as their EULUMDAT raw integer
//! values so the round-trip is lossless.

use eulumdat_core::{
    Distribution, Eulumdat, EulumdatError, LampSet, Symmetry, TypeIndicator, ValidationWarning,
};
use serde::{Deserialize, Serialize};

/// A full luminaire model as exchanged with the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EulumdatDto {
    pub identification: String,
    /// EULUMDAT type indicator raw value (1, 2, or 3).
    pub type_indicator: u8,
    /// EULUMDAT symmetry indicator raw value (0..=4).
    pub symmetry: u8,
    pub c_plane_step: f64,
    pub gamma_step: f64,
    pub measurement_report_number: String,
    pub luminaire_name: String,
    pub luminaire_number: String,
    pub file_name: String,
    pub date_user: String,
    pub luminaire_length: f64,
    pub luminaire_width: f64,
    pub luminaire_height: f64,
    pub luminous_area_length: f64,
    pub luminous_area_width: f64,
    pub luminous_area_height_c0: f64,
    pub luminous_area_height_c90: f64,
    pub luminous_area_height_c180: f64,
    pub luminous_area_height_c270: f64,
    pub downward_flux_fraction: f64,
    pub light_output_ratio: f64,
    pub conversion_factor: f64,
    pub tilt: f64,
    pub lamps: Vec<LampSetDto>,
    pub direct_ratios: [f64; 10],
    pub c_planes: Vec<f64>,
    pub gamma_angles: Vec<f64>,
    pub intensities: Vec<Vec<f64>>,
}

/// One standard lamp set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LampSetDto {
    pub lamp_count: u32,
    pub lamp_type: String,
    pub total_luminous_flux: f64,
    pub color_temperature: String,
    pub color_rendering_index: String,
    pub wattage_including_ballast: f64,
}

/// A validation warning surfaced to the UI.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WarningDto {
    pub field: String,
    pub message: String,
}

/// Derived photometric values shown alongside the editor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotometryDto {
    pub total_output: f64,
    pub calculated_downward_flux_fraction: f64,
    pub beam_angle_c0_c180: Option<f64>,
    pub beam_angle_c90_c270: Option<f64>,
    pub field_angle_c0_c180: Option<f64>,
    pub field_angle_c90_c270: Option<f64>,
    pub c_plane_count: usize,
    pub gamma_count: usize,
}

/// The bundle returned after opening or mutating a document.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocResponse {
    pub doc: EulumdatDto,
    pub warnings: Vec<WarningDto>,
    pub photometry: PhotometryDto,
    pub path: Option<String>,
    pub dirty: bool,
}

impl From<&LampSet> for LampSetDto {
    fn from(l: &LampSet) -> Self {
        Self {
            lamp_count: l.lamp_count,
            lamp_type: l.lamp_type.clone(),
            total_luminous_flux: l.total_luminous_flux,
            color_temperature: l.color_temperature.clone(),
            color_rendering_index: l.color_rendering_index.clone(),
            wattage_including_ballast: l.wattage_including_ballast,
        }
    }
}

impl From<&LampSetDto> for LampSet {
    fn from(l: &LampSetDto) -> Self {
        Self {
            lamp_count: l.lamp_count,
            lamp_type: l.lamp_type.clone(),
            total_luminous_flux: l.total_luminous_flux,
            color_temperature: l.color_temperature.clone(),
            color_rendering_index: l.color_rendering_index.clone(),
            wattage_including_ballast: l.wattage_including_ballast,
        }
    }
}

impl From<&Eulumdat> for EulumdatDto {
    fn from(m: &Eulumdat) -> Self {
        Self {
            identification: m.identification.clone(),
            type_indicator: m.type_indicator.raw_value(),
            symmetry: m.symmetry.raw_value(),
            c_plane_step: m.c_plane_step,
            gamma_step: m.gamma_step,
            measurement_report_number: m.measurement_report_number.clone(),
            luminaire_name: m.luminaire_name.clone(),
            luminaire_number: m.luminaire_number.clone(),
            file_name: m.file_name.clone(),
            date_user: m.date_user.clone(),
            luminaire_length: m.luminaire_length,
            luminaire_width: m.luminaire_width,
            luminaire_height: m.luminaire_height,
            luminous_area_length: m.luminous_area_length,
            luminous_area_width: m.luminous_area_width,
            luminous_area_height_c0: m.luminous_area_height_c0,
            luminous_area_height_c90: m.luminous_area_height_c90,
            luminous_area_height_c180: m.luminous_area_height_c180,
            luminous_area_height_c270: m.luminous_area_height_c270,
            downward_flux_fraction: m.downward_flux_fraction,
            light_output_ratio: m.light_output_ratio,
            conversion_factor: m.conversion_factor,
            tilt: m.tilt,
            lamps: m.lamps.iter().map(LampSetDto::from).collect(),
            direct_ratios: m.direct_ratios,
            c_planes: m.c_planes.clone(),
            gamma_angles: m.gamma_angles.clone(),
            intensities: m.intensities.clone(),
        }
    }
}

impl EulumdatDto {
    /// Builds a fresh `eulumdat-core` model from this DTO.
    ///
    /// Scalar/text fields are copied directly. The photometric grid is applied
    /// through `replace_distribution` so the stored C-plane count and table
    /// shape stay internally consistent.
    pub fn to_model(&self) -> Result<Eulumdat, EulumdatError> {
        let type_indicator = TypeIndicator::from_raw(self.type_indicator)?;
        let symmetry = Symmetry::from_raw(self.symmetry)?;

        let mut m = Eulumdat {
            identification: self.identification.clone(),
            type_indicator,
            symmetry,
            c_plane_step: self.c_plane_step,
            gamma_step: self.gamma_step,
            measurement_report_number: self.measurement_report_number.clone(),
            luminaire_name: self.luminaire_name.clone(),
            luminaire_number: self.luminaire_number.clone(),
            file_name: self.file_name.clone(),
            date_user: self.date_user.clone(),
            luminaire_length: self.luminaire_length,
            luminaire_width: self.luminaire_width,
            luminaire_height: self.luminaire_height,
            luminous_area_length: self.luminous_area_length,
            luminous_area_width: self.luminous_area_width,
            luminous_area_height_c0: self.luminous_area_height_c0,
            luminous_area_height_c90: self.luminous_area_height_c90,
            luminous_area_height_c180: self.luminous_area_height_c180,
            luminous_area_height_c270: self.luminous_area_height_c270,
            downward_flux_fraction: self.downward_flux_fraction,
            light_output_ratio: self.light_output_ratio,
            conversion_factor: self.conversion_factor,
            tilt: self.tilt,
            lamps: self.lamps.iter().map(LampSet::from).collect(),
            direct_ratios: self.direct_ratios,
            ..Eulumdat::default()
        };

        m.replace_distribution(Distribution {
            symmetry,
            c_plane_step: self.c_plane_step,
            gamma_step: self.gamma_step,
            c_planes: self.c_planes.clone(),
            gamma_angles: self.gamma_angles.clone(),
            intensities: self.intensities.clone(),
        })?;

        Ok(m)
    }
}

impl PhotometryDto {
    pub fn from_model(m: &Eulumdat) -> Self {
        Self {
            total_output: m.total_output(),
            calculated_downward_flux_fraction: m.calculated_downward_flux_fraction(),
            beam_angle_c0_c180: m.beam_angle_c0_c180(),
            beam_angle_c90_c270: m.beam_angle_c90_c270(),
            field_angle_c0_c180: m.field_angle_c0_c180(),
            field_angle_c90_c270: m.field_angle_c90_c270(),
            c_plane_count: m.c_plane_count(),
            gamma_count: m.gamma_count(),
        }
    }
}

pub fn warnings_to_dto(warnings: &[ValidationWarning]) -> Vec<WarningDto> {
    warnings
        .iter()
        .map(|w| WarningDto {
            field: w.field.clone(),
            message: w.message.clone(),
        })
        .collect()
}
