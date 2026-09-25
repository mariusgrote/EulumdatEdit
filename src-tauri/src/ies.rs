//! LM-63 Type C import and export. The editor's model uses cd/klm; LM-63
//! stores candela, so the conversion keeps the physical intensity explicit.

use eulumdat_core::{Distribution, Eulumdat, LampSet, Symmetry, TypeIndicator, ValidationSettings};

fn number<'a>(tokens: &mut impl Iterator<Item = &'a str>, field: &str) -> Result<f64, String> {
    let value = tokens
        .next()
        .ok_or_else(|| format!("IES file is missing {field}"))?
        .parse::<f64>()
        .map_err(|_| format!("IES file has an invalid {field}"))?;
    if !value.is_finite() {
        return Err(format!("IES file has a non-finite {field}"));
    }
    Ok(value)
}

fn count<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    field: &str,
    max: usize,
) -> Result<usize, String> {
    let value = number(tokens, field)?;
    if value < 1.0 || value > max as f64 || value.fract() != 0.0 {
        return Err(format!("IES file has an unsupported {field}"));
    }
    Ok(value as usize)
}

fn angles<'a>(
    tokens: &mut impl Iterator<Item = &'a str>,
    count: usize,
    field: &str,
) -> Result<Vec<f64>, String> {
    let values: Vec<_> = (0..count)
        .map(|_| number(tokens, field))
        .collect::<Result<_, _>>()?;
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(format!("IES {field} must increase strictly"));
    }
    Ok(values)
}

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

/// IES numeric data is ASCII, but published metadata may use Windows-1252.
pub fn parse_bytes(input: &[u8], file_name: &str) -> Result<Eulumdat, String> {
    if let Ok(text) = std::str::from_utf8(input) {
        return parse(text, file_name);
    }
    let (text, _, _) = encoding_rs::WINDOWS_1252.decode(input);
    parse(&text, file_name)
}

pub fn parse(text: &str, file_name: &str) -> Result<Eulumdat, String> {
    let mut lines = text.lines().map(str::trim);
    let first = lines
        .next()
        .unwrap_or_default()
        .trim_start_matches('\u{feff}');
    if !matches!(
        first,
        "IES:LM-63-2019" | "IESNA:LM-63-2002" | "IESNA:LM-63-1995" | "IESNA91" | "IESNA:LM-63-1986"
    ) {
        return Err(
            "Unsupported IES version (expected LM-63-2019, 2002, 1995, 1991 or 1986)".into(),
        );
    }
    let mut name = String::new();
    let mut manufacturer = String::new();
    let mut data = None;
    for line in lines.by_ref() {
        if let Some(value) = line.strip_prefix("[LUMINAIRE]") {
            name = value.trim().to_string();
        }
        if let Some(value) = line.strip_prefix("[MANUFAC]") {
            manufacturer = value.trim().to_string();
        }
        if let Some(tilt) = line.strip_prefix("TILT=") {
            if !tilt.eq_ignore_ascii_case("NONE") {
                return Err("IES TILT data is not supported; use a file with TILT=NONE".into());
            }
            data = Some(lines.collect::<Vec<_>>().join(" "));
            break;
        }
    }
    let data = data.ok_or("IES file is missing TILT=NONE")?;
    let mut tokens = data.split_whitespace();
    let lamp_count = count(&mut tokens, "lamp count", 100_000)?;
    let lumens_per_lamp = number(&mut tokens, "lumens per lamp")?;
    let multiplier = number(&mut tokens, "candela multiplier")?;
    let vertical_count = count(&mut tokens, "vertical angle count", 361)?;
    let horizontal_count = count(&mut tokens, "horizontal angle count", 722)?;
    let photometric_type = number(&mut tokens, "photometric type")?;
    if photometric_type != 1.0 {
        return Err("Only Type C IES photometry is supported".into());
    }
    let units = number(&mut tokens, "unit type")?;
    let millimeters = match units {
        1.0 => 304.8,
        2.0 => 1000.0,
        _ => return Err("IES unit type must be 1 or 2".into()),
    };
    let width = number(&mut tokens, "luminous width")? * millimeters;
    let length = number(&mut tokens, "luminous length")? * millimeters;
    let height = number(&mut tokens, "luminous height")? * millimeters;
    let ballast = number(&mut tokens, "ballast factor")?;
    let ballast_lamp = number(&mut tokens, "ballast-lamp factor")?;
    let watts = number(&mut tokens, "input watts")?;
    let gamma = angles(&mut tokens, vertical_count, "vertical angles")?;
    let horizontal = angles(&mut tokens, horizontal_count, "horizontal angles")?;
    if !near(gamma[0], 0.0) || gamma.last().is_some_and(|last| *last > 180.0) {
        return Err("IES Type C vertical angles must start at 0 and end at or before 180".into());
    }
    if !near(horizontal[0], 0.0) {
        return Err("IES Type C horizontal angles must start at 0".into());
    }
    let end = *horizontal.last().unwrap();
    let symmetry = if horizontal_count == 1 {
        Symmetry::Rotational
    } else if near(end, 90.0) {
        Symmetry::C0C180AndC90C270
    } else if near(end, 180.0) {
        Symmetry::C0C180
    } else if near(end, 360.0) {
        Symmetry::None
    } else {
        return Err("IES horizontal angles must end at 90, 180 or 360".into());
    };
    let mut rows = Vec::with_capacity(horizontal_count);
    for _ in 0..horizontal_count {
        rows.push(
            (0..vertical_count)
                .map(|_| number(&mut tokens, "candela value"))
                .collect::<Result<Vec<_>, _>>()?,
        );
    }
    let scale = multiplier * ballast * ballast_lamp;
    if !scale.is_finite() || scale <= 0.0 || rows.iter().flatten().any(|value| *value < 0.0) {
        return Err(
            "IES candela values and factors must be non-negative, with positive factors".into(),
        );
    }
    let lamp_flux = if lumens_per_lamp > 0.0 {
        lumens_per_lamp * lamp_count as f64
    } else if near(lumens_per_lamp, -1.0) {
        1000.0
    } else {
        return Err("IES lumens per lamp must be positive or -1 for absolute photometry".into());
    };
    if !lamp_flux.is_finite() || lamp_flux <= 0.0 {
        return Err("IES lamp flux is invalid".into());
    }
    for row in &mut rows {
        for value in row {
            *value *= scale * 1000.0 / lamp_flux;
            if !value.is_finite() {
                return Err("IES candela scaling exceeds the supported range".into());
            }
        }
    }
    let c_planes = match symmetry {
        Symmetry::Rotational => vec![0.0],
        Symmetry::C0C180AndC90C270 => {
            let mut half = horizontal.clone();
            half.extend(horizontal.iter().rev().skip(1).map(|angle| 180.0 - angle));
            let mut full = half.clone();
            full.extend(
                half.iter()
                    .rev()
                    .skip(1)
                    .take(half.len() - 2)
                    .map(|angle| 360.0 - angle),
            );
            full
        }
        Symmetry::C0C180 => {
            let mut full = horizontal.clone();
            full.extend(
                horizontal
                    .iter()
                    .rev()
                    .skip(1)
                    .take(horizontal.len() - 2)
                    .map(|angle| 360.0 - angle),
            );
            full
        }
        Symmetry::None => {
            if rows.len() < 5
                || !horizontal.iter().any(|angle| near(*angle, 90.0))
                || !horizontal.iter().any(|angle| near(*angle, 180.0))
                || !horizontal.iter().any(|angle| near(*angle, 270.0))
            {
                return Err("IES full-circle data needs C0, C90, C180 and C270".into());
            }
            horizontal.clone()
        }
        Symmetry::C90C270 => unreachable!(),
    };
    if c_planes.len() > 721 {
        return Err("IES horizontal angle grid is too large for EULUMDAT".into());
    }
    let mut model = Eulumdat {
        identification: manufacturer,
        type_indicator: TypeIndicator::PointSourceWithoutSymmetry,
        luminaire_name: if name.is_empty() {
            file_name.to_string()
        } else {
            name
        },
        file_name: file_name.to_string(),
        luminous_area_length: length,
        luminous_area_width: width,
        luminous_area_height_c0: height,
        luminous_area_height_c90: height,
        luminous_area_height_c180: height,
        luminous_area_height_c270: height,
        light_output_ratio: 100.0,
        conversion_factor: 1.0,
        lamps: vec![LampSet {
            lamp_count: lamp_count as u32,
            lamp_type: "IES lamp".into(),
            total_luminous_flux: lamp_flux,
            color_temperature: String::new(),
            color_rendering_index: String::new(),
            wattage_including_ballast: watts,
        }],
        ..Eulumdat::default()
    };
    model
        .replace_distribution(Distribution {
            symmetry,
            c_plane_step: horizontal.get(1).map_or(0.0, |next| next - horizontal[0]),
            gamma_step: gamma.get(1).map_or(0.0, |next| next - gamma[0]),
            c_planes,
            gamma_angles: gamma,
            intensities: rows,
        })
        .map_err(|e| e.to_string())?;
    model.downward_flux_fraction = model.calculated_downward_flux_fraction();
    model
        .validate(ValidationSettings::unrestricted())
        .map_err(|e| format!("IES cannot be represented as EULUMDAT: {e}"))?;
    Ok(model)
}

pub fn serialize(model: &Eulumdat) -> Result<String, String> {
    model
        .validate(ValidationSettings::unrestricted())
        .map_err(|e| e.to_string())?;
    let lamp_flux: f64 = model
        .lamps
        .iter()
        .map(|lamp| lamp.total_luminous_flux)
        .sum();
    if !lamp_flux.is_finite()
        || lamp_flux <= 0.0
        || !model.conversion_factor.is_finite()
        || model.conversion_factor <= 0.0
    {
        return Err("IES export needs positive lamp flux and conversion factor".into());
    }
    let indices: Vec<usize> = match model.symmetry {
        Symmetry::Rotational => vec![0],
        Symmetry::C0C180AndC90C270 => model
            .c_planes
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (*c <= 90.0).then_some(i))
            .collect(),
        Symmetry::C0C180 => model
            .c_planes
            .iter()
            .enumerate()
            .filter_map(|(i, c)| (*c <= 180.0).then_some(i))
            .collect(),
        _ => (0..model.c_planes.len()).collect(),
    };
    let stored_indices: Vec<usize> = if model.symmetry == Symmetry::C90C270 {
        let mut stored_angles: Vec<f64> = model
            .c_planes
            .iter()
            .copied()
            .filter(|c| *c >= 270.0 || *c <= 90.0)
            .collect();
        stored_angles.sort_by(|a, b| {
            (a - 270.0)
                .rem_euclid(360.0)
                .total_cmp(&(b - 270.0).rem_euclid(360.0))
        });
        model
            .c_planes
            .iter()
            .map(|c| {
                let representative = if *c > 90.0 && *c < 270.0 {
                    (180.0 - c).rem_euclid(360.0)
                } else {
                    *c
                };
                stored_angles
                    .iter()
                    .position(|stored| near(*stored, representative))
                    .ok_or("Missing C-plane for IES export")
            })
            .collect::<Result<_, _>>()?
    } else {
        indices.clone()
    };
    let mut out = format!("IESNA:LM-63-2002\n[TEST] EulumdatEdit export\n[TESTLAB] Unknown\n[ISSUEDATE] Unknown\n[MANUFAC] {}\n[LUMINAIRE] {}\nTILT=NONE\n", model.identification.replace(['\r', '\n'], " "), model.luminaire_name.replace(['\r', '\n'], " "));
    if indices.is_empty() {
        return Err("No C-planes to export".into());
    }
    let append_c360 = model.symmetry == Symmetry::C90C270
        || (model.symmetry == Symmetry::None
            && model
                .c_planes
                .last()
                .is_some_and(|last| !near(*last, 360.0)));
    let horizontal_count = indices.len() + usize::from(append_c360);
    let watts: f64 = model
        .lamps
        .iter()
        .map(|lamp| lamp.wattage_including_ballast)
        .sum();
    out.push_str(&format!(
        "1 -1 1 {} {} 1 2\n{} {} {}\n1 1 {}\n",
        model.gamma_angles.len(),
        horizontal_count,
        model.luminous_area_width / 1000.0,
        model.luminous_area_length / 1000.0,
        model.luminous_area_height_c0 / 1000.0,
        watts
    ));
    out.push_str(
        &model
            .gamma_angles
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(" "),
    );
    out.push('\n');
    for index in &indices {
        out.push_str(&format!("{} ", model.c_planes[*index]));
    }
    if append_c360 {
        out.push_str("360 ");
    }
    out.push('\n');
    let first_row = *stored_indices.first().ok_or("Missing first C-plane row")?;
    for index in stored_indices
        .into_iter()
        .chain(append_c360.then_some(first_row))
    {
        let row = model
            .intensities
            .get(index)
            .ok_or("Missing C-plane intensity row")?;
        for value in row {
            let candela = value * model.conversion_factor * lamp_flux / 1000.0;
            if !candela.is_finite() || candela < 0.0 {
                return Err("IES candela value is invalid".into());
            }
            out.push_str(&format!("{candela} "));
        }
        out.push('\n');
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture(horizontal: &str, count: usize, rows: &str, lumens: &str) -> String {
        format!("IESNA:LM-63-2002\n[MANUFAC] Maker\n[LUMINAIRE] Test lamp\nTILT=NONE\n1 {lumens} 1 3 {count} 1 2\n0.2 0.4 0.1\n1 1 12\n0 90 180\n{horizontal}\n{rows}\n")
    }

    #[test]
    fn imports_quarter_symmetry_and_preserves_candela_on_export() {
        let input = fixture("0 90", 2, "100 50 10 200 80 20", "1000");
        let model = parse(&input, "lamp.ies").unwrap();
        assert_eq!(model.symmetry, Symmetry::C0C180AndC90C270);
        assert_eq!(model.c_planes, vec![0.0, 90.0, 180.0, 270.0]);
        assert_eq!(model.intensities[1][0], 200.0);
        assert_eq!(model.luminous_area_length, 400.0);
        assert_eq!(model.luminous_area_width, 200.0);
        assert_eq!(model.luminaire_length, 0.0);
        assert_eq!(model.luminaire_width, 0.0);
        let exported = serialize(&model).unwrap();
        let reparsed = parse(&exported, "copy.ies").unwrap();
        assert_eq!(reparsed.intensities, model.intensities);
        assert_eq!(reparsed.c_planes, model.c_planes);
    }

    #[test]
    fn imports_absolute_full_circle_and_preserves_c360() {
        let input = fixture(
            "0 90 180 270 360",
            5,
            "100 50 10 200 80 20 300 90 30 400 100 40 101 51 11",
            "-1",
        );
        let model = parse(&input, "full.ies").unwrap();
        assert_eq!(model.symmetry, Symmetry::None);
        assert_eq!(model.c_planes, vec![0.0, 90.0, 180.0, 270.0, 360.0]);
        assert_eq!(model.intensities.len(), 5);
        assert_ne!(model.intensities[0], model.intensities[4]);
        let exported = serialize(&model).unwrap();
        let reparsed = parse(&exported, "again.ies").unwrap();
        assert_eq!(reparsed.intensities, model.intensities);
    }

    #[test]
    fn imports_windows_1252_metadata() {
        let text =
            fixture("0 90", 2, "100 50 10 200 80 20", "1000").replace("Test lamp", "Lampe °");
        let (bytes, _, _) = encoding_rs::WINDOWS_1252.encode(&text);
        let model = parse_bytes(&bytes, "lamp.ies").unwrap();
        assert_eq!(model.luminaire_name, "Lampe °");
    }

    #[test]
    fn applies_candela_and_ballast_factors_on_import() {
        let input = fixture("0 90", 2, "100 50 10 200 80 20", "2000")
            .replace("1 2000 1 3", "1 2000 2 3")
            .replace("1 1 12", "0.5 1 12");
        let model = parse(&input, "factors.ies").unwrap();
        assert_eq!(model.intensities[0][0], 50.0);
        assert_eq!(model.lamps[0].total_luminous_flux, 2000.0);
        let exported = serialize(&model).unwrap();
        let reparsed = parse(&exported, "exported.ies").unwrap();
        assert_eq!(
            reparsed.intensities[0][0] * reparsed.lamps[0].total_luminous_flux / 1000.0,
            100.0
        );
    }

    #[test]
    fn rejects_unsupported_type_and_tilt() {
        let input = fixture("0 90", 2, "100 50 10 200 80 20", "1000");
        assert!(
            parse(&input.replace("TILT=NONE", "TILT=INCLUDE"), "bad.ies")
                .unwrap_err()
                .contains("TILT")
        );
        assert!(parse(&input.replace("2 1 2", "2 2 2"), "bad.ies")
            .unwrap_err()
            .contains("Type C"));
    }

    #[test]
    fn exports_c90_c270_symmetry_with_the_correct_plane_rows() {
        let mut model = parse(
            &fixture(
                "0 90 180 270 360",
                5,
                "100 50 10 200 80 20 100 50 10 300 90 30 100 50 10",
                "-1",
            ),
            "planes.ies",
        )
        .unwrap();
        model
            .replace_distribution(Distribution {
                symmetry: Symmetry::C90C270,
                c_plane_step: 90.0,
                gamma_step: 90.0,
                c_planes: vec![0.0, 90.0, 180.0, 270.0],
                gamma_angles: vec![0.0, 90.0, 180.0],
                intensities: vec![
                    vec![300.0, 90.0, 30.0],
                    vec![100.0, 50.0, 10.0],
                    vec![200.0, 80.0, 20.0],
                ],
            })
            .unwrap();
        let exported = serialize(&model).unwrap();
        let reparsed = parse(&exported, "copy.ies").unwrap();
        assert_eq!(reparsed.intensities[0][0], 100.0);
        assert_eq!(reparsed.intensities[1][0], 200.0);
        assert_eq!(reparsed.intensities[2][0], 100.0);
        assert_eq!(reparsed.intensities[3][0], 300.0);
    }

    /// Optional compatibility check for downloaded IES files kept outside the
    /// repository. Set IES_QA_DIR to a directory of Type C, TILT=NONE files.
    #[test]
    #[ignore = "requires external IES samples via IES_QA_DIR"]
    fn external_ies_files_keep_candela_after_export() {
        let directory = std::env::var("IES_QA_DIR").expect("set IES_QA_DIR to a sample directory");
        let mut paths: Vec<_> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("ies"))
            })
            .collect();
        paths.sort();
        assert!(!paths.is_empty(), "IES_QA_DIR contains no IES files");
        for path in paths {
            let bytes = std::fs::read(&path).unwrap();
            let name = path.file_name().unwrap().to_string_lossy();
            let model =
                parse_bytes(&bytes, &name).unwrap_or_else(|error| panic!("{path:?}: {error}"));
            let (ldt, _) = Eulumdat::parse(&model.to_text())
                .unwrap_or_else(|error| panic!("{path:?}: LDT conversion failed: {error}"));
            assert_eq!(model.c_planes, ldt.c_planes, "{path:?}");
            assert_eq!(model.gamma_angles, ldt.gamma_angles, "{path:?}");
            assert_eq!(model.intensities, ldt.intensities, "{path:?}");
            let exported = serialize(&model).unwrap_or_else(|error| panic!("{path:?}: {error}"));
            let reparsed =
                parse(&exported, &name).unwrap_or_else(|error| panic!("{path:?}: {error}"));
            assert_eq!(model.c_planes, reparsed.c_planes, "{path:?}");
            assert_eq!(model.gamma_angles, reparsed.gamma_angles, "{path:?}");
            assert_eq!(
                model.intensities.len(),
                reparsed.intensities.len(),
                "{path:?}"
            );
            assert!(
                model
                    .intensities
                    .iter()
                    .zip(&reparsed.intensities)
                    .all(|(a, b)| a.len() == b.len()),
                "{path:?}"
            );
            let original_flux: f64 = model
                .lamps
                .iter()
                .map(|lamp| lamp.total_luminous_flux)
                .sum();
            let exported_flux: f64 = reparsed
                .lamps
                .iter()
                .map(|lamp| lamp.total_luminous_flux)
                .sum();
            for (original, exported) in model
                .intensities
                .iter()
                .flatten()
                .zip(reparsed.intensities.iter().flatten())
            {
                let before = original * original_flux * model.conversion_factor / 1000.0;
                let after = exported * exported_flux * reparsed.conversion_factor / 1000.0;
                assert!(
                    (before - after).abs() <= before.abs().max(1.0) * 1e-6,
                    "{path:?}: candela changed from {before} to {after}"
                );
            }
        }
    }
}
