import type { LampSet, Warning } from './types';

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

/**
 * Resolves one warning to a navigation target. Lamp warnings are attributed to
 * their set via `warning.lampIndex`, supplied by the backend validator, so the
 * frontend never re-derives which set offends.
 */
export function resolveWarningTarget(warning: Warning): WarningTarget | null {
  const section = getSectionForField(warning.field);
  if (!section) return null;

  if (section === 'lamps') {
    const prop = LAMP_FIELD_KEYS[warning.field];
    if (!prop || warning.lampIndex === null) return { section, fieldKey: null };
    return { section, fieldKey: `lamps.${warning.lampIndex}.${prop}` };
  }

  const fieldKey = DOC_FIELD_KEYS[warning.field] ?? null;
  return { section, fieldKey };
}

export function resolveWarningTargets(warnings: Warning[]): (WarningTarget | null)[] {
  return warnings.map(resolveWarningTarget);
}

export function warningsBySection(warnings: Warning[]): Record<SectionId, number> {
  const counts: Record<SectionId, number> = {
    general: 0,
    geometry: 0,
    lamps: 0,
    intensity: 0
  };

  for (const warning of warnings) {
    const target = resolveWarningTarget(warning);
    if (target) counts[target.section] += 1;
  }

  return counts;
}

/** Groups warning messages by the form field key they belong to, for inline display. */
export function warningsByField(warnings: Warning[]): Record<string, string[]> {
  const byField: Record<string, string[]> = {};

  for (const warning of warnings) {
    const target = resolveWarningTarget(warning);
    if (!target?.fieldKey) continue;
    (byField[target.fieldKey] ??= []).push(warning.message);
  }

  return byField;
}

export function isNavigableTarget(target: WarningTarget | null): boolean {
  return target !== null;
}

export function isNavigableWarning(warning: Warning): boolean {
  return resolveWarningTarget(warning) !== null;
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
