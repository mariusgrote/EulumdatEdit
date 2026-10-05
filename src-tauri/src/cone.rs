//! Half-peak downward cone and horizontal-plane illuminance.
use eulumdat_core::{Eulumdat, Symmetry};
use serde::Deserialize;
use std::fmt::Write;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConeOptions {
    pub size: u32,
    pub planes: Vec<String>,
    pub max_distance: f64,
}

struct Cone {
    left: f64,
    right: f64,
    axial_cd: f64,
    edge_cd: f64,
}

// EULUMDAT stores only the independent rows, but lists all C-plane angles.
fn profile(model: &Eulumdat, c: f64) -> Option<&[f64]> {
    let effective = match model.symmetry {
        Symmetry::Rotational => return model.intensities.first().map(Vec::as_slice),
        Symmetry::None => c,
        Symmetry::C0C180 => {
            if c > 180.0 {
                360.0 - c
            } else {
                c
            }
        }
        Symmetry::C90C270 => {
            if c < 90.0 {
                180.0 - c
            } else if c > 270.0 {
                540.0 - c
            } else {
                c
            }
        }
        Symmetry::C0C180AndC90C270 => {
            let c = if c > 180.0 { 360.0 - c } else { c };
            if c > 90.0 {
                180.0 - c
            } else {
                c
            }
        }
    };
    let index = model
        .c_planes
        .iter()
        .position(|v| (v - effective).abs() < 1e-6)?;
    let offset = if model.symmetry == Symmetry::C90C270 {
        model
            .c_planes
            .iter()
            .position(|v| (v - 90.0).abs() < 1e-6)?
    } else {
        0
    };
    model
        .intensities
        .get(index.checked_sub(offset)?)
        .map(Vec::as_slice)
}

fn crossing(angles: &[f64], values: &[f64], threshold: f64) -> Option<f64> {
    for i in 1..angles.len() {
        if values[i] < threshold {
            let fraction = (values[i - 1] - threshold) / (values[i - 1] - values[i]);
            let angle = angles[i - 1] + fraction * (angles[i] - angles[i - 1]);
            return (angle > 0.0 && angle < 90.0).then_some(angle);
        }
        if angles[i] >= 90.0 {
            break;
        }
    }
    None
}

fn calculate(model: &Eulumdat, plane: &str) -> Result<Cone, String> {
    let (a, b) = match plane {
        "c0c180" => (0.0, 180.0),
        "c90c270" => (90.0, 270.0),
        _ => return Err("Unknown cone plane".into()),
    };
    let unavailable = || "Selected C-plane data unavailable".to_string();
    let right = profile(model, a).ok_or_else(unavailable)?;
    let left = profile(model, b).ok_or_else(unavailable)?;
    let angles = &model.gamma_angles;
    if angles.len() < 2
        || angles[0].abs() > 1e-6
        || left.len() != angles.len()
        || right.len() != angles.len()
        || angles.iter().any(|v| !v.is_finite())
        || angles.windows(2).any(|v| v[0] >= v[1])
        || left.iter().chain(right).any(|v| !v.is_finite() || *v < 0.0)
    {
        return Err("Cone requires valid intensity data starting at gamma 0°".into());
    }
    let peak = left.iter().chain(right).copied().fold(0.0, f64::max);
    let threshold = peak * 0.5;
    if peak <= 0.0 || left[0] < threshold || right[0] < threshold {
        return Err("No half-peak downward cone around gamma 0°".into());
    }
    let message = || "Half-peak cone must close below gamma 90°".to_string();
    let axial_stored = (left[0] + right[0]) * 0.5;
    let left = crossing(angles, left, threshold).ok_or_else(message)?;
    let right_angle = crossing(angles, right, threshold).ok_or_else(message)?;
    // Lamp sets are alternative configurations. Field 26c already includes lamp count.
    let flux = model
        .lamps
        .first()
        .map_or(0.0, |lamp| lamp.total_luminous_flux);
    let factor = model.conversion_factor * flux / 1000.0;
    if !flux.is_finite()
        || flux <= 0.0
        || !model.conversion_factor.is_finite()
        || model.conversion_factor <= 0.0
        || !factor.is_finite()
    {
        return Err("Lux values require positive lamp flux and conversion factor".into());
    }
    let axial_cd = axial_stored * factor;
    let edge_cd = threshold * factor;
    if !axial_cd.is_finite() || !edge_cd.is_finite() {
        return Err("Intensity values are too large".into());
    }
    Ok(Cone {
        left,
        right: right_angle,
        axial_cd,
        edge_cd,
    })
}

pub fn render(model: &Eulumdat, options: &ConeOptions) -> Result<String, String> {
    if !(200..=4096).contains(&options.size)
        || !options.max_distance.is_finite()
        || !(0.1..=100.0).contains(&options.max_distance)
    {
        return Err("Distance must be 0.1–100 m".into());
    }
    if options.planes.is_empty() || options.planes.len() > 2 {
        return Err("Select at least one cone plane".into());
    }
    let cones = options
        .planes
        .iter()
        .map(|plane| {
            calculate(model, plane)
                .map(|cone| (plane, cone))
                .map_err(|error| format!("{plane}: {error}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let left_max = cones
        .iter()
        .map(|(_, c)| c.left.to_radians().tan())
        .fold(0.0, f64::max);
    let right_max = cones
        .iter()
        .map(|(_, c)| c.right.to_radians().tan())
        .fold(0.0, f64::max);
    let scale = 290.0 / (left_max + right_max);
    let x = 60.0 + left_max * scale;
    let mut svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 600 600" role="img" aria-label="Half-peak cone diagram"><rect width="600" height="600" fill="#fff"/><g font-family="Arial, Helvetica, sans-serif" font-size="14" fill="#444">"##,
        size = options.size
    );
    for (index, (plane, cone)) in cones.iter().enumerate() {
        let label = if plane.as_str() == "c0c180" {
            "C0/C180"
        } else {
            "C90/C270"
        };
        let color = if plane.as_str() == "c0c180" {
            "#ff6b6b"
        } else {
            "#7375ff"
        };
        let left = x - cone.left.to_radians().tan() * scale;
        let right = x + cone.right.to_radians().tan() * scale;
        let y = 26 + index * 22;
        let _ = write!(
            svg,
            r##"<text x="30" y="{y}" fill="{color}">{label} · Half-peak beam {:.1}°</text><path d="M {x:.2} 82 L {left:.2} 472 L {right:.2} 472 Z" fill="{color}" fill-opacity="0.08" stroke="{color}" stroke-width="1.5"/>"##,
            cone.left + cone.right
        );
    }
    let _ = write!(
        svg,
        r##"<path d="M {x:.2} 82 V 472" stroke="#bbb" stroke-dasharray="4 4"/>"##
    );
    for i in 1..=6 {
        let fraction = i as f64 / 6.0;
        let d = options.max_distance * fraction;
        let y = 82.0 + 390.0 * fraction;
        let axial = cones[0].1.axial_cd / d.powi(2);
        let _ = write!(
            svg,
            r##"<path d="M 30 {y:.2} H 570" stroke="#ddd"/><text x="30" y="{:.2}">{d:.2}</text><text x="375" y="{:.2}" font-size="12">E(0°)</text><text x="570" y="{:.2}" text-anchor="end">{axial:.0}</text>"##,
            y - 8.0,
            y - 44.0,
            y - 44.0
        );
        for (index, (plane, cone)) in cones.iter().enumerate() {
            let color = if plane.as_str() == "c0c180" {
                "#ff6b6b"
            } else {
                "#7375ff"
            };
            let diameter = d * (cone.left.to_radians().tan() + cone.right.to_radians().tan());
            let left_lux = cone.edge_cd * cone.left.to_radians().cos().powi(3) / d.powi(2);
            let right_lux = cone.edge_cd * cone.right.to_radians().cos().powi(3) / d.powi(2);
            let row_y = y - 26.0 + index as f64 * 18.0;
            let _ = write!(
                svg,
                r##"<g fill="{color}"><text x="205" y="{row_y:.2}" text-anchor="middle">{diameter:.2}</text><text x="375" y="{row_y:.2}" font-size="12">E(edge L/R)</text><text x="570" y="{row_y:.2}" text-anchor="end" font-size="12">{left_lux:.0} / {right_lux:.0}</text></g>"##
            );
        }
    }
    svg.push_str(r##"<text x="30" y="500" font-size="12">Distance [m]</text><text x="205" y="500" text-anchor="middle" font-size="12">Cone diameter [m]</text><text x="570" y="500" text-anchor="end" font-size="12">Illuminance [lx]</text><text x="30" y="530" font-size="12">Horizontal receiving plane · First lamp set · Edges at 50% of peak</text><text x="30" y="550" font-size="12">Distance and width scaled separately. Point source, without reflections.</text></g></svg>"##);
    Ok(svg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eulumdat_core::LampSet;
    fn fixture() -> Eulumdat {
        Eulumdat {
            symmetry: Symmetry::Rotational,
            c_planes: vec![0.0],
            gamma_angles: vec![0.0, 30.0, 60.0, 90.0],
            intensities: vec![vec![1000.0, 500.0, 0.0, 0.0]],
            conversion_factor: 2.0,
            lamps: vec![LampSet {
                total_luminous_flux: 2000.0,
                lamp_count: 3,
                lamp_type: String::new(),
                color_temperature: String::new(),
                color_rendering_index: String::new(),
                wattage_including_ballast: 0.0,
            }],
            ..Eulumdat::default()
        }
    }
    #[test]
    fn cone_geometry_and_absolute_lux() {
        let cone = calculate(&fixture(), "c0c180").unwrap();
        assert!((cone.left - 30.0).abs() < 1e-10);
        assert!((cone.right - 30.0).abs() < 1e-10);
        assert_eq!(cone.axial_cd, 4000.0);
        assert_eq!(cone.axial_cd / 2_f64.powi(2), 1000.0);
        assert!((cone.edge_cd * cone.left.to_radians().cos().powi(3) - 1299.0381057).abs() < 1e-6);
        assert!(
            (2.0 * (cone.left.to_radians().tan() + cone.right.to_radians().tan()) - 2.3094010768)
                .abs()
                < 1e-8
        );
    }
    #[test]
    fn asymmetric_edges_and_alternative_lamp_sets() {
        let mut model = fixture();
        model.symmetry = Symmetry::None;
        model.c_planes = vec![0.0, 180.0];
        model.intensities.push(vec![1000.0, 750.0, 250.0, 0.0]);
        model.lamps.push(LampSet {
            total_luminous_flux: 9000.0,
            lamp_count: 1,
            lamp_type: String::new(),
            color_temperature: String::new(),
            color_rendering_index: String::new(),
            wattage_including_ballast: 0.0,
        });
        let cone = calculate(&model, "c0c180").unwrap();
        assert_eq!(cone.left, 45.0);
        assert_eq!(cone.right, 30.0);
        assert_eq!(cone.axial_cd, 4000.0);
        assert!(calculate(&model, "c90c270").is_err());
    }
    #[test]
    fn maps_mirror_symmetries_to_independent_rows() {
        for symmetry in [Symmetry::C0C180, Symmetry::C0C180AndC90C270] {
            let mut model = fixture();
            model.symmetry = symmetry;
            model.c_planes = vec![0.0, 90.0, 180.0, 270.0];
            model.intensities = vec![
                vec![1000.0, 500.0, 0.0, 0.0],
                vec![1000.0, 750.0, 250.0, 0.0],
            ];
            if symmetry == Symmetry::C0C180 {
                model.intensities.push(vec![1000.0, 750.0, 250.0, 0.0]);
            }
            let cone = calculate(&model, "c90c270").unwrap();
            assert_eq!(cone.left, 45.0);
            assert_eq!(cone.right, 45.0);
        }
    }

    #[test]
    fn rejects_uplight_wide_beams_and_missing_flux() {
        let mut model = fixture();
        model.intensities[0] = vec![0.0, 0.0, 500.0, 1000.0];
        assert!(calculate(&model, "c0c180").is_err());
        model.intensities[0] = vec![1000.0; 4];
        assert!(calculate(&model, "c0c180").is_err());
        model = fixture();
        model.lamps.clear();
        assert!(calculate(&model, "c0c180").is_err());
    }
    #[test]
    fn maps_c90_symmetry_and_renders_six_steps() {
        let mut model = fixture();
        model.symmetry = Symmetry::C90C270;
        model.c_planes = vec![0.0, 90.0, 180.0, 270.0];
        model.intensities = vec![model.intensities[0].clone(); 3];
        assert!(calculate(&model, "c0c180").is_ok());
        let svg = render(
            &model,
            &ConeOptions {
                size: 520,
                planes: vec!["c90c270".into()],
                max_distance: 3.0,
            },
        )
        .unwrap();
        assert_eq!(svg.matches("E(0°)").count(), 6);
        assert!(svg.contains("3.00"));
        assert!(svg.contains(">444</text>"));
        assert!(render(
            &model,
            &ConeOptions {
                size: 520,
                planes: vec!["c0c180".into()],
                max_distance: f64::NAN
            }
        )
        .is_err());
    }
}
