export interface NumberInputConstraints {
  integer?: boolean;
  hardMin?: number;
  hardMax?: number;
}

/**
 * Returns `raw` when it can safely cross the command boundary, otherwise `null`.
 * Invalid input is rejected, never rounded or clamped, so the backend still
 * sees (and warns about) out-of-domain values within the hard limits.
 */
export function committableNumber(
  raw: number | null | undefined,
  constraints: NumberInputConstraints = {}
): number | null {
  if (raw === null || raw === undefined || !Number.isFinite(raw)) return null;
  if (constraints.integer && !Number.isInteger(raw)) return null;
  if (constraints.hardMin !== undefined && raw < constraints.hardMin) return null;
  if (constraints.hardMax !== undefined && raw > constraints.hardMax) return null;
  return raw;
}

export function isValidResampleStep(value: number | null | undefined): value is number {
  return committableNumber(value, { integer: true, hardMin: 1, hardMax: 90 }) !== null;
}
