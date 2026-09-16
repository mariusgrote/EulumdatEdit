// TypeScript mirror of the Rust DTOs in src-tauri/src/dto.rs.

export interface LampSet {
  lampCount: number;
  lampType: string;
  totalLuminousFlux: number;
  colorTemperature: string;
  colorRenderingIndex: string;
  wattageIncludingBallast: number;
}

export interface EulumdatDoc {
  identification: string;
  /** EULUMDAT type indicator raw value: 1, 2, or 3. */
  typeIndicator: number;
  /** EULUMDAT symmetry indicator raw value: 0..=4. */
  symmetry: number;
  cPlaneStep: number;
  gammaStep: number;
  measurementReportNumber: string;
  luminaireName: string;
  luminaireNumber: string;
  fileName: string;
  dateUser: string;
  luminaireLength: number;
  luminaireWidth: number;
  luminaireHeight: number;
  luminousAreaLength: number;
  luminousAreaWidth: number;
  luminousAreaHeightC0: number;
  luminousAreaHeightC90: number;
  luminousAreaHeightC180: number;
  luminousAreaHeightC270: number;
  downwardFluxFraction: number;
  lightOutputRatio: number;
  conversionFactor: number;
  tilt: number;
  lamps: LampSet[];
  directRatios: number[];
  cPlanes: number[];
  gammaAngles: number[];
  /** Stored intensity rows: indexed by stored C-plane, then gamma angle. */
  intensities: number[][];
}

export interface Warning {
  /** Human-readable field label from the validator; display only. */
  field: string;
  /**
   * Stable camelCase field key (e.g. "luminaireLength"); null when the field has
   * no form control. Lamp warnings carry the lamp set property key ("lampCount"),
   * combined with `lampIndex` to address a specific set.
   */
  fieldKey: string | null;
  message: string;
  /** Zero-based lamp set index for repeated lamp fields; null for document-level fields. */
  lampIndex: number | null;
}

export interface Photometry {
  totalOutput: number;
  calculatedDownwardFluxFraction: number;
  beamAngleC0C180: number | null;
  beamAngleC90C270: number | null;
  fieldAngleC0C180: number | null;
  fieldAngleC90C270: number | null;
  cPlaneCount: number;
  gammaCount: number;
}

export interface DocResponse {
  doc: EulumdatDoc;
  warnings: Warning[];
  photometry: Photometry;
  path: string | null;
  dirty: boolean;
  /** Whether legacy strict text-length validation is currently enabled. */
  strictValidation: boolean;
}

export interface PolarOptions {
  width: number;
  height: number;
  planes: string[];
  showGrid: boolean;
  showLegend: boolean;
  showAxisLabels: boolean;
  intensityMode: 'stored' | 'converted';
  title: string | null;
}

export const TYPE_INDICATOR_LABELS: Record<number, string> = {
  1: 'Point source with symmetry',
  2: 'Linear luminaire',
  3: 'Point source without symmetry'
};

export const SYMMETRY_LABELS: Record<number, string> = {
  0: 'None',
  1: 'Rotational',
  2: 'C0 / C180',
  3: 'C90 / C270',
  4: 'C0/C180 and C90/C270'
};
