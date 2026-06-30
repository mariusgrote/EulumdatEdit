import type { EulumdatDoc, LampSet, Warning } from './types';

export type SectionId = 'general' | 'geometry' | 'lamps' | 'intensity';

export interface WarningTarget {
  section: SectionId;
  /** Stable DOM key, e.g. "identification" or "lamps.0.lampCount". */
  fieldKey: string | null;
}

/** EULUMDAT validation field name → doc property key (top-level fields). */
const DOC_FIELD_KEYS: Record<string, string> = {
  Identification: 'identification',
  'Measurement report number': 'measurementReportNumber',
  'Luminaire name': 'luminaireName',
  'Luminaire number': 'luminaireNumber',
  'File name': 'fileName',
  'Date/user': 'dateUser',
  'Length/diameter of luminaire': 'luminaireLength',
  'Width of luminaire': 'luminaireWidth',
  'Height of luminaire': 'luminaireHeight',
  'Length/diameter of luminous area': 'luminousAreaLength',
  'Width of luminous area': 'luminousAreaWidth',
  'Height of luminous area C0-plane': 'luminousAreaHeightC0',
  'Height of luminous area C90-plane': 'luminousAreaHeightC90',
  'Height of luminous area C180-plane': 'luminousAreaHeightC180',
  'Height of luminous area C270-plane': 'luminousAreaHeightC270',
  'Downward flux fraction': 'downwardFluxFraction',
  'Light output ratio of luminaire': 'lightOutputRatio',
  'Conversion factor for luminous intensities': 'conversionFactor',
  'Tilt of luminaire during measurement': 'tilt'
};

/** Lamp validation field name → lamp set property key. */
const LAMP_FIELD_KEYS: Record<string, keyof LampSet> = {
  'Number of lamps': 'lampCount',
  'Type of lamps': 'lampType',
  'Total luminous flux of lamps': 'totalLuminousFlux',
  'Color temperature of lamps': 'colorTemperature',
  'Color rendering index': 'colorRenderingIndex',
  'Wattage including ballast': 'wattageIncludingBallast'
};

const GENERAL_FIELDS = new Set([
  'Identification',
  'Measurement report number',
  'Luminaire name',
  'Luminaire number',
  'File name',
  'Date/user'
]);

const GEOMETRY_FIELDS = new Set(Object.keys(DOC_FIELD_KEYS).filter((f) => !GENERAL_FIELDS.has(f)));

const LAMP_FIELDS = new Set(Object.keys(LAMP_FIELD_KEYS));

const DIRECT_RATIO_RE = /^k\[\d+\]$/;

interface ValidationLimits {
  maxIdentificationLen: number | null;
  maxFileNameLen: number | null;
  maxLampTypeLen: number | null;
  maxColorTemperatureLen: number | null;
  maxColorRenderingIndexLen: number | null;
}

function validationLimits(strictValidation: boolean): ValidationLimits {
  if (!strictValidation) {
    return {
      maxIdentificationLen: null,
      maxFileNameLen: null,
      maxLampTypeLen: null,
      maxColorTemperatureLen: null,
      maxColorRenderingIndexLen: null
    };
  }
  return {
    maxIdentificationLen: 78,
    maxFileNameLen: 8,
    maxLampTypeLen: 24,
    maxColorTemperatureLen: 16,
    maxColorRenderingIndexLen: 6
  };
}

export function getSectionForField(field: string): SectionId | null {
  if (GENERAL_FIELDS.has(field)) return 'general';
  if (GEOMETRY_FIELDS.has(field)) return 'geometry';
  if (LAMP_FIELDS.has(field)) return 'lamps';
  // Direct-ratio (k[n]) and intensity warnings have no editable field control.
  return null;
}

export function isDirectRatioField(field: string): boolean {
  return DIRECT_RATIO_RE.test(field);
}

function warnLen(len: number, max: number | null): boolean {
  return max !== null && len > max;
}

function warnRange(value: number, min: number, max: number): boolean {
  return value < min || value > max;
}

const utf8Encoder = new TextEncoder();

/** UTF-8 byte length, matching Rust's `String::len()` used by the backend validator. */
function byteLen(value: string): number {
  return utf8Encoder.encode(value).length;
}

/** Returns lamp set indices that would emit a warning for `field`, in validation order. */
export function offendingLampIndices(
  field: string,
  doc: EulumdatDoc,
  strictValidation: boolean
): number[] {
  const limits = validationLimits(strictValidation);
  const indices: number[] = [];

  doc.lamps.forEach((lamp, index) => {
    if (lampWouldWarn(field, lamp, limits)) indices.push(index);
  });

  return indices;
}

function lampWouldWarn(field: string, lamp: LampSet, limits: ValidationLimits): boolean {
  switch (field) {
    case 'Number of lamps':
      return warnRange(lamp.lampCount, 1, 1000);
    case 'Type of lamps':
      return warnLen(byteLen(lamp.lampType), limits.maxLampTypeLen);
    case 'Total luminous flux of lamps':
      return warnRange(lamp.totalLuminousFlux, 1, 9_999_999);
    case 'Color temperature of lamps':
      return warnLen(byteLen(lamp.colorTemperature), limits.maxColorTemperatureLen);
    case 'Color rendering index':
      return warnLen(byteLen(lamp.colorRenderingIndex), limits.maxColorRenderingIndexLen);
    case 'Wattage including ballast':
      return warnRange(lamp.wattageIncludingBallast, 0.1, 10_000);
    default:
      return false;
  }
}

/** Resolves one warning to a navigation target. Pass `occurrence` when multiple warnings share the same field. */
export function resolveWarningTarget(
  warning: Warning,
  doc: EulumdatDoc,
  strictValidation: boolean,
  occurrence = 0
): WarningTarget | null {
  const section = getSectionForField(warning.field);
  if (!section) return null;

  if (section === 'lamps') {
    const prop = LAMP_FIELD_KEYS[warning.field];
    if (!prop) return { section, fieldKey: null };
    const indices = offendingLampIndices(warning.field, doc, strictValidation);
    // No `?? indices[0]` fallback: if our re-derived predicate disagrees with the
    // backend on which sets offend, leave the warning unmapped rather than point
    // at the wrong lamp.
    const lampIndex = indices[occurrence];
    if (lampIndex === undefined) return { section, fieldKey: null };
    return { section, fieldKey: `lamps.${lampIndex}.${prop}` };
  }

  const fieldKey = DOC_FIELD_KEYS[warning.field] ?? null;
  return { section, fieldKey };
}

/** Maps each warning to a target, matching duplicate lamp warnings to successive offending sets. */
export function resolveWarningTargets(
  warnings: Warning[],
  doc: EulumdatDoc,
  strictValidation: boolean
): (WarningTarget | null)[] {
  const counters: Record<string, number> = {};
  // Cache offending-lamp lookups so a field warning that repeats per lamp set
  // doesn't re-scan every set on each occurrence.
  const lampIndexCache = new Map<string, number[]>();

  return warnings.map((warning) => {
    const occurrence = counters[warning.field] ?? 0;
    counters[warning.field] = occurrence + 1;

    const section = getSectionForField(warning.field);
    if (section !== 'lamps') {
      return resolveWarningTarget(warning, doc, strictValidation, occurrence);
    }

    const prop = LAMP_FIELD_KEYS[warning.field];
    if (!prop) return { section, fieldKey: null };

    let indices = lampIndexCache.get(warning.field);
    if (!indices) {
      indices = offendingLampIndices(warning.field, doc, strictValidation);
      lampIndexCache.set(warning.field, indices);
    }
    const lampIndex = indices[occurrence];
    if (lampIndex === undefined) return { section, fieldKey: null };
    return { section, fieldKey: `lamps.${lampIndex}.${prop}` };
  });
}

export function warningsBySection(
  warnings: Warning[],
  doc: EulumdatDoc,
  strictValidation: boolean
): Record<SectionId, number> {
  const counts: Record<SectionId, number> = {
    general: 0,
    geometry: 0,
    lamps: 0,
    intensity: 0
  };

  for (const target of resolveWarningTargets(warnings, doc, strictValidation)) {
    if (target) counts[target.section] += 1;
  }

  return counts;
}

/** Groups warning messages by the form field key they belong to, for inline display. */
export function warningsByField(
  warnings: Warning[],
  doc: EulumdatDoc,
  strictValidation: boolean
): Record<string, string[]> {
  const byField: Record<string, string[]> = {};

  const targets = resolveWarningTargets(warnings, doc, strictValidation);
  targets.forEach((target, i) => {
    if (!target?.fieldKey) return;
    (byField[target.fieldKey] ??= []).push(warnings[i].message);
  });

  return byField;
}

export function isNavigableTarget(target: WarningTarget | null): boolean {
  return target !== null;
}

export function isNavigableWarning(
  warning: Warning,
  doc: EulumdatDoc,
  strictValidation: boolean,
  occurrence = 0
): boolean {
  return resolveWarningTarget(warning, doc, strictValidation, occurrence) !== null;
}

/** CSS selector for a field key used in the form DOM. */
export function fieldSelector(fieldKey: string): string {
  return `[data-field-key="${CSS.escape(fieldKey)}"]`;
}

/** Finds the focusable control for a field key in the mounted section DOM. */
export function findFieldElement(fieldKey: string): HTMLElement | null {
  const nodes = document.querySelectorAll<HTMLElement>(fieldSelector(fieldKey));
  for (const node of nodes) {
    if (node.matches('input, select, textarea')) return node;
  }
  for (const node of nodes) {
    const input = node.querySelector<HTMLElement>('input, select, textarea');
    if (input) return input;
  }
  return null;
}
