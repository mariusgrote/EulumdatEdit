//! Serializable data-transfer objects mirroring the `eulumdat-core` model.
//!
//! The frontend never sees `eulumdat-core` types directly; it exchanges these
//! plain serde structs. Enums are transported as their EULUMDAT raw integer
//! values so the round-trip is lossless.

use eulumdat_core::{
    Distribution, Eulumdat, EulumdatError, FluxBasis, LampSet, Symmetry, TypeIndicator, UgrBlocker,
    UgrTable, ValidationWarning, UGR_REFLECTANCES,
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
    /// Human-readable field label from `eulumdat-core`, for display only.
    pub field: String,
    /// Stable camelCase key of the form field the warning belongs to, e.g.
    /// `luminaireLength`. Lamp warnings carry the lamp set property key
    /// (`lampCount`) and are combined with `lamp_index` by the frontend.
    /// `None` when the field has no form control (direct ratios `k[n]`).
    pub field_key: Option<String>,
    pub message: String,
    /// Zero-based lamp set index when the warning belongs to a repeated lamp
    /// field; `None` for document-level fields.
    pub lamp_index: Option<usize>,
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

/// UGR table after the CIE tabular method, or the reasons it does not apply.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum UgrDto {
    Available(UgrTableDto),
    Blocked { blockers: Vec<UgrBlockerDto> },
}

/// A computed UGR table for both flux bases.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UgrTableDto {
    /// Lamp flux in lm that `lamp_flux_values` refer to.
    pub lamp_flux: f64,
    /// `8·log10(Φ / 1000)`, the offset between both flux bases.
    pub flux_correction: f64,
    /// Column reflectances as `[ceiling, walls, floor]` fractions.
    pub reflectances: Vec<[f64; 3]>,
    /// Values for the lamp flux of the file.
    pub lamp_flux_values: UgrValuesDto,
    /// Values for a lamp flux of 1000 lm.
    pub normalized_values: UgrValuesDto,
}

/// UGR values for one flux basis. Values are unrounded; `None` marks a cell
/// without glare contribution.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UgrValuesDto {
    pub rows: Vec<UgrRowDto>,
    /// Room 4H × 8H, reflectances 70/50/20.
    pub data_sheet_crosswise: Option<f64>,
    pub data_sheet_endwise: Option<f64>,
}

/// One room of the UGR table.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UgrRowDto {
    pub x_h: u8,
    pub y_h: u8,
    pub crosswise: Vec<Option<f64>>,
    pub endwise: Vec<Option<f64>>,
}

/// A reason why the UGR tabular method does not apply.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UgrBlockerDto {
    pub message: String,
    /// Form field key of the offending input, as in [`WarningDto`]. `None` for
    /// blockers caused by the intensity distribution itself.
    pub field_key: Option<String>,
    pub lamp_index: Option<usize>,
}

/// The bundle returned after opening or mutating a document.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocResponse {
    pub doc: EulumdatDto,
    pub warnings: Vec<WarningDto>,
    pub photometry: PhotometryDto,
    pub ugr: UgrDto,
    pub path: Option<String>,
    pub dirty: bool,
    /// Whether legacy strict text-length validation is currently enabled.
    pub strict_validation: bool,
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

impl UgrDto {
    pub fn from_model(m: &Eulumdat) -> Self {
        match m.ugr_table() {
            Ok(table) => Self::Available(UgrTableDto::from(&table)),
            Err(blockers) => Self::Blocked {
                blockers: blockers.iter().map(UgrBlockerDto::from).collect(),
            },
        }
    }
}

impl From<&UgrTable> for UgrTableDto {
    fn from(table: &UgrTable) -> Self {
        Self {
            lamp_flux: table.lamp_flux(),
            flux_correction: table.flux_correction(),
            reflectances: UGR_REFLECTANCES
                .iter()
                .map(|r| [r.ceiling, r.walls, r.floor])
                .collect(),
            lamp_flux_values: UgrValuesDto::new(table, FluxBasis::LampFlux),
            normalized_values: UgrValuesDto::new(table, FluxBasis::Normalized1000Lm),
        }
    }
}

impl UgrValuesDto {
    fn new(table: &UgrTable, basis: FluxBasis) -> Self {
        let (data_sheet_crosswise, data_sheet_endwise) = table.data_sheet_value(basis);
        Self {
            rows: table
                .rows(basis)
                .map(|row| UgrRowDto {
                    x_h: row.room.x_h,
                    y_h: row.room.y_h,
                    crosswise: row.crosswise.to_vec(),
                    endwise: row.endwise.to_vec(),
                })
                .collect(),
            data_sheet_crosswise,
            data_sheet_endwise,
        }
    }
}

impl From<&UgrBlocker> for UgrBlockerDto {
    fn from(blocker: &UgrBlocker) -> Self {
        let (field_key, lamp_index) = match blocker {
            UgrBlocker::NoLuminousArea => (Some("luminousAreaLength"), None),
            UgrBlocker::NoLampFlux => (Some("totalLuminousFlux"), Some(0)),
            UgrBlocker::NoLightOutputRatio => (Some("lightOutputRatio"), None),
            _ => (None, None),
        };
        Self {
            message: blocker.to_string(),
            field_key: field_key.map(str::to_string),
            lamp_index,
        }
    }
}

pub fn warnings_to_dto(warnings: &[ValidationWarning]) -> Vec<WarningDto> {
    warnings
        .iter()
        .map(|w| WarningDto {
            field: w.field.clone(),
            field_key: field_key_for(w).map(str::to_string),
            message: w.message.clone(),
            lamp_index: w.lamp_index,
        })
        .collect()
}

/// Translates `eulumdat-core`'s display label for a warning into the stable
/// field key used by the frontend (`data-field-key`).
///
/// `eulumdat-core` exposes no machine-readable field identifier, so this match
/// is the one place coupled to its wording. The tests in `commands.rs` run it
/// against real validator output, so an upstream rename fails `cargo test`.
fn field_key_for(warning: &ValidationWarning) -> Option<&'static str> {
    let key = match warning.field.as_str() {
        "Identification" => "identification",
        "Measurement report number" => "measurementReportNumber",
        "Luminaire name" => "luminaireName",
        "Luminaire number" => "luminaireNumber",
        "File name" => "fileName",
        "Date/user" => "dateUser",
        "Length/diameter of luminaire" => "luminaireLength",
        "Width of luminaire" => "luminaireWidth",
        "Height of luminaire" => "luminaireHeight",
        "Length/diameter of luminous area" => "luminousAreaLength",
        "Width of luminous area" => "luminousAreaWidth",
        "Height of luminous area C0-plane" => "luminousAreaHeightC0",
        "Height of luminous area C90-plane" => "luminousAreaHeightC90",
        "Height of luminous area C180-plane" => "luminousAreaHeightC180",
        "Height of luminous area C270-plane" => "luminousAreaHeightC270",
        "Downward flux fraction" => "downwardFluxFraction",
        "Light output ratio of luminaire" => "lightOutputRatio",
        "Conversion factor for luminous intensities" => "conversionFactor",
        "Tilt of luminaire during measurement" => "tilt",
        "Number of lamps" => "lampCount",
        "Type of lamps" => "lampType",
        "Total luminous flux of lamps" => "totalLuminousFlux",
        "Color temperature of lamps" => "colorTemperature",
        "Color rendering index" => "colorRenderingIndex",
        "Wattage including ballast" => "wattageIncludingBallast",
        _ => return None,
    };
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn warning(field: &str, lamp_index: Option<usize>) -> ValidationWarning {
        ValidationWarning {
            field: field.to_string(),
            message: String::new(),
            lamp_index,
        }
    }

    #[test]
    fn maps_general_geometry_and_lamp_fields() {
        assert_eq!(
            field_key_for(&warning("Luminaire name", None)),
            Some("luminaireName")
        );
        assert_eq!(
            field_key_for(&warning("Height of luminous area C90-plane", None)),
            Some("luminousAreaHeightC90")
        );
        assert_eq!(
            field_key_for(&warning("Number of lamps", Some(2))),
            Some("lampCount")
        );
    }

    #[test]
    fn direct_ratio_fields_have_no_key() {
        assert_eq!(field_key_for(&warning("k[3]", None)), None);
    }

    #[test]
    fn dto_carries_field_key_and_lamp_index() {
        let dto = warnings_to_dto(&[warning("Type of lamps", Some(1))]);
        assert_eq!(dto[0].field_key.as_deref(), Some("lampType"));
        assert_eq!(dto[0].lamp_index, Some(1));
    }
}
