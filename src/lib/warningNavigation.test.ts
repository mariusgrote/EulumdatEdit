import { describe, expect, it } from 'vitest';
import type { EulumdatDoc, Warning } from './types';
import {
  getSectionForField,
  isDirectRatioField,
  isNavigableWarning,
  offendingLampIndices,
  resolveWarningTarget,
  resolveWarningTargets,
  warningsBySection
} from './warningNavigation';

const baseDoc: EulumdatDoc = {
  identification: 'ok',
  typeIndicator: 1,
  symmetry: 0,
  cPlaneStep: 90,
  gammaStep: 10,
  measurementReportNumber: 'ok',
  luminaireName: 'ok',
  luminaireNumber: 'ok',
  fileName: 'ok',
  dateUser: 'ok',
  luminaireLength: 100,
  luminaireWidth: 100,
  luminaireHeight: 50,
  luminousAreaLength: 80,
  luminousAreaWidth: 80,
  luminousAreaHeightC0: 0,
  luminousAreaHeightC90: 0,
  luminousAreaHeightC180: 0,
  luminousAreaHeightC270: 0,
  downwardFluxFraction: 50,
  lightOutputRatio: 100,
  conversionFactor: 1,
  tilt: 0,
  lamps: [
    {
      lampCount: 1,
      lampType: 'LED',
      totalLuminousFlux: 1000,
      colorTemperature: '4000K',
      colorRenderingIndex: '80',
      wattageIncludingBallast: 10
    },
    {
      lampCount: 0,
      lampType: 'LED',
      totalLuminousFlux: 1000,
      colorTemperature: '4000K',
      colorRenderingIndex: '80',
      wattageIncludingBallast: 10
    }
  ],
  directRatios: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
  cPlanes: [0, 90, 180],
  gammaAngles: [0, 90, 180],
  intensities: [
    [1000, 500, 10],
    [1000, 500, 10],
    [1000, 500, 10]
  ]
};

describe('getSectionForField', () => {
  it('maps general, geometry, and lamp fields', () => {
    expect(getSectionForField('Identification')).toBe('general');
    expect(getSectionForField('Width of luminaire')).toBe('geometry');
    expect(getSectionForField('Number of lamps')).toBe('lamps');
  });

  it('returns null for direct ratio fields', () => {
    expect(getSectionForField('k[1]')).toBe(null);
    expect(isDirectRatioField('k[3]')).toBe(true);
  });
});

describe('resolveWarningTarget', () => {
  it('maps a geometry warning to a doc field key', () => {
    const target = resolveWarningTarget(
      { field: 'Width of luminaire', message: 'out of range' },
      baseDoc,
      false
    );
    expect(target).toEqual({ section: 'geometry', fieldKey: 'luminaireWidth' });
  });

  it('maps lamp warnings to indexed field keys', () => {
    const target = resolveWarningTarget(
      { field: 'Number of lamps', message: 'out of range' },
      baseDoc,
      false
    );
    expect(target).toEqual({ section: 'lamps', fieldKey: 'lamps.1.lampCount' });
  });

  it('assigns successive lamp sets to duplicate warnings', () => {
    const warnings: Warning[] = [
      { field: 'Number of lamps', message: 'a' },
      { field: 'Number of lamps', message: 'b' }
    ];
    const doc: EulumdatDoc = {
      ...baseDoc,
      lamps: [
        { ...baseDoc.lamps[0], lampCount: 0 },
        { ...baseDoc.lamps[1], lampCount: 0 }
      ]
    };
    const targets = resolveWarningTargets(warnings, doc, false);
    expect(targets[0]?.fieldKey).toBe('lamps.0.lampCount');
    expect(targets[1]?.fieldKey).toBe('lamps.1.lampCount');
  });
});

describe('warningsBySection', () => {
  it('counts warnings per sidebar section', () => {
    const warnings: Warning[] = [
      { field: 'Identification', message: 'too long' },
      { field: 'Width of luminaire', message: 'out of range' },
      { field: 'Number of lamps', message: 'out of range' },
      { field: 'k[2]', message: 'out of range' }
    ];
    const counts = warningsBySection(warnings, baseDoc, false);
    expect(counts.general).toBe(1);
    expect(counts.geometry).toBe(1);
    expect(counts.lamps).toBe(1);
    expect(counts.intensity).toBe(0);
  });
});

describe('offendingLampIndices', () => {
  it('lists indices in validation order', () => {
    expect(offendingLampIndices('Number of lamps', baseDoc, false)).toEqual([1]);
  });
});

describe('isNavigableWarning', () => {
  it('is false for direct ratio warnings', () => {
    expect(isNavigableWarning({ field: 'k[1]', message: 'x' }, baseDoc, false)).toBe(false);
  });
});
