import { describe, expect, it } from 'vitest';
import type { UgrTable, UgrValues } from './types';
import { formatUgr, resolveUgrBlockerTarget, ugrTableTsv, ugrValues } from './ugr';

function values(base: number): UgrValues {
  const rows = [
    { xH: 2, yH: 2, crosswise: [base, base + 0.04, null, 1, 2], endwise: [3, 4, 5, 6, 7] }
  ];
  return { rows, dataSheetCrosswise: base, dataSheetEndwise: 3 };
}

const table: UgrTable = {
  status: 'available',
  lampFlux: 3000,
  fluxCorrection: 3.8,
  reflectances: [
    [0.7, 0.5, 0.2],
    [0.7, 0.3, 0.2],
    [0.5, 0.5, 0.2],
    [0.5, 0.3, 0.2],
    [0.3, 0.3, 0.2]
  ],
  lampFluxValues: values(22),
  normalizedValues: values(18.2)
};

describe('formatUgr', () => {
  it('rounds to one decimal and marks missing values', () => {
    expect(formatUgr(19.26)).toBe('19.3');
    expect(formatUgr(null)).toBe('—');
    expect(formatUgr(Number.NaN)).toBe('—');
  });
});

describe('ugrValues', () => {
  it('selects the values for the flux basis', () => {
    expect(ugrValues(table, 'lampFlux').dataSheetCrosswise).toBe(22);
    expect(ugrValues(table, 'normalized').dataSheetCrosswise).toBe(18.2);
  });
});

describe('ugrTableTsv', () => {
  it('writes the catalogue layout with header rows', () => {
    const lines = ugrTableTsv(table, 'normalized').trimEnd().split('\n');
    expect(lines).toHaveLength(6);
    expect(lines[0].split('\t')).toEqual([
      'UGR (1000 lm)', '', 'Crosswise', '', '', '', '', 'Endwise', '', '', '', ''
    ]);
    expect(lines[1]).toBe('Ceiling %\t\t70\t70\t50\t50\t30\t70\t70\t50\t50\t30');
    expect(lines[3]).toBe('Floor %\t\t20\t20\t20\t20\t20\t20\t20\t20\t20\t20');
    expect(lines[5]).toBe('2H\t2H\t18.2\t18.2\t—\t1.0\t2.0\t3.0\t4.0\t5.0\t6.0\t7.0');
  });

  it('labels the lamp flux basis with the file flux', () => {
    expect(ugrTableTsv(table, 'lampFlux')).toMatch(/^UGR \(3000 lm\)\t/);
  });
});

describe('resolveUgrBlockerTarget', () => {
  it('navigates to the offending field', () => {
    expect(
      resolveUgrBlockerTarget({ message: 'x', fieldKey: 'luminousAreaLength', lampIndex: null })
    ).toEqual({ section: 'geometry', fieldKey: 'luminousAreaLength' });
    expect(
      resolveUgrBlockerTarget({ message: 'x', fieldKey: 'totalLuminousFlux', lampIndex: 0 })
    ).toEqual({ section: 'lamps', fieldKey: 'lamps.0.totalLuminousFlux' });
  });

  it('sends distribution blockers to the intensity table', () => {
    expect(resolveUgrBlockerTarget({ message: 'x', fieldKey: null, lampIndex: null })).toEqual({
      section: 'intensity',
      fieldKey: null
    });
  });
});
