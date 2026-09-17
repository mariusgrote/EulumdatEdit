// Presentation helpers for the UGR table computed by eulumdat-core.

import type { UgrBlocker, UgrFluxBasis, UgrTable, UgrValues } from './types';
import { resolveWarningTarget, type WarningTarget } from './warningNavigation';

/** Index of the data sheet room 4H × 8H in the table rows. */
export const DATA_SHEET_ROOM = 10;
/** Index of the data sheet reflectances 70/50/20 in the table columns. */
export const DATA_SHEET_REFLECTANCE = 0;

export function ugrValues(table: UgrTable, basis: UgrFluxBasis): UgrValues {
  return basis === 'lampFlux' ? table.lampFluxValues : table.normalizedValues;
}

/** Formats a UGR value to one decimal, as in catalogue tables. */
export function formatUgr(value: number | null | undefined): string {
  if (value === null || value === undefined || !Number.isFinite(value)) return '—';
  return value.toFixed(1);
}

export function formatReflectance(fraction: number): string {
  return String(Math.round(fraction * 100));
}

export function fluxBasisLabel(table: UgrTable, basis: UgrFluxBasis): string {
  return basis === 'lampFlux' ? `${Math.round(table.lampFlux)} lm` : '1000 lm';
}

/** The table as tab-separated text in catalogue layout, for pasting into spreadsheets. */
export function ugrTableTsv(table: UgrTable, basis: UgrFluxBasis): string {
  const reflectanceRow = (label: string, index: number) => {
    const cells = table.reflectances.map((r) => formatReflectance(r[index]));
    return [label, '', ...cells, ...cells];
  };
  const pad = (n: number) => Array(n).fill('');
  const lines = [
    [`UGR (${fluxBasisLabel(table, basis)})`, '', 'Crosswise', ...pad(4), 'Endwise', ...pad(4)],
    reflectanceRow('Ceiling %', 0),
    reflectanceRow('Walls %', 1),
    reflectanceRow('Floor %', 2),
    ['X', 'Y', ...pad(10)],
    ...ugrValues(table, basis).rows.map((row) => [
      `${row.xH}H`,
      `${row.yH}H`,
      ...row.crosswise.map(formatUgr),
      ...row.endwise.map(formatUgr)
    ])
  ];
  return lines.map((cells) => cells.join('\t')).join('\n') + '\n';
}

/** Where to go to fix a blocker: its form field, or the intensity table for distribution problems. */
export function resolveUgrBlockerTarget(blocker: UgrBlocker): WarningTarget {
  if (blocker.fieldKey === null) return { section: 'intensity', fieldKey: null };
  return (
    resolveWarningTarget({ field: '', ...blocker }) ?? {
      section: 'intensity',
      fieldKey: null
    }
  );
}
