import type { LampSet, Warning } from './types';

export type SectionId = 'general' | 'geometry' | 'lamps' | 'intensity';

export interface WarningTarget {
  section: SectionId;
  /** Stable DOM key, e.g. "identification" or "lamps.0.lampCount". */
  fieldKey: string | null;
}

const GENERAL_FIELDS = new Set([
  'identification',
  'measurementReportNumber',
  'luminaireName',
  'luminaireNumber',
  'fileName',
  'dateUser',
  'typeIndicator',
  'symmetry'
]);

const GEOMETRY_FIELDS = new Set([
  'luminaireLength',
  'luminaireWidth',
  'luminaireHeight',
  'luminousAreaLength',
  'luminousAreaWidth',
  'luminousAreaHeightC0',
  'luminousAreaHeightC90',
  'luminousAreaHeightC180',
  'luminousAreaHeightC270',
  'downwardFluxFraction',
  'lightOutputRatio',
  'conversionFactor',
  'tilt'
]);

const LAMP_FIELDS = new Set<string>([
  'lampCount',
  'lampType',
  'totalLuminousFlux',
  'colorTemperature',
  'colorRenderingIndex',
  'wattageIncludingBallast'
] satisfies (keyof LampSet)[]);

/**
 * Resolves one warning to a navigation target from its backend-supplied
 * `fieldKey`; the display `field` label plays no part. Lamp warnings are
 * attributed to their set via `warning.lampIndex`, so the frontend never
 * re-derives which set offends.
 */
export function resolveWarningTarget(warning: Warning): WarningTarget | null {
  const key = warning.fieldKey;
  if (key === null) return null;

  if (LAMP_FIELDS.has(key)) {
    if (warning.lampIndex === null) return { section: 'lamps', fieldKey: null };
    return { section: 'lamps', fieldKey: `lamps.${warning.lampIndex}.${key}` };
  }
  if (GENERAL_FIELDS.has(key)) return { section: 'general', fieldKey: key };
  if (GEOMETRY_FIELDS.has(key)) return { section: 'geometry', fieldKey: key };
  return null;
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
