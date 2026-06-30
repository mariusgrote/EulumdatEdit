import { describe, expect, it } from 'vitest';
import type { Warning } from './types';
import {
  getSectionForField,
  isDirectRatioField,
  isNavigableWarning,
  resolveWarningTarget,
  resolveWarningTargets,
  warningsByField,
  warningsBySection
} from './warningNavigation';

function w(field: string, message: string, lampIndex: number | null = null): Warning {
  return { field, message, lampIndex };
}

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
    expect(resolveWarningTarget(w('Width of luminaire', 'out of range'))).toEqual({
      section: 'geometry',
      fieldKey: 'luminaireWidth'
    });
  });

  it('maps a lamp warning to its indexed field key via lampIndex', () => {
    expect(resolveWarningTarget(w('Number of lamps', 'out of range', 1))).toEqual({
      section: 'lamps',
      fieldKey: 'lamps.1.lampCount'
    });
  });

  it('leaves a lamp warning unmapped when it carries no lampIndex', () => {
    expect(resolveWarningTarget(w('Number of lamps', 'out of range', null))).toEqual({
      section: 'lamps',
      fieldKey: null
    });
  });

  it('assigns each lamp warning to the set named by its lampIndex', () => {
    const targets = resolveWarningTargets([
      w('Number of lamps', 'a', 0),
      w('Number of lamps', 'b', 1)
    ]);
    expect(targets[0]?.fieldKey).toBe('lamps.0.lampCount');
    expect(targets[1]?.fieldKey).toBe('lamps.1.lampCount');
  });
});

describe('warningsBySection', () => {
  it('counts warnings per sidebar section', () => {
    const counts = warningsBySection([
      w('Identification', 'too long'),
      w('Width of luminaire', 'out of range'),
      w('Number of lamps', 'out of range', 0),
      w('k[2]', 'out of range')
    ]);
    expect(counts.general).toBe(1);
    expect(counts.geometry).toBe(1);
    expect(counts.lamps).toBe(1);
    expect(counts.intensity).toBe(0);
  });
});

describe('warningsByField', () => {
  it('groups messages by resolved field key', () => {
    const byField = warningsByField([
      w('Identification', 'too long'),
      w('Width of luminaire', 'out of range'),
      w('Number of lamps', 'out of range', 1)
    ]);
    expect(byField.identification).toEqual(['too long']);
    expect(byField.luminaireWidth).toEqual(['out of range']);
    expect(byField['lamps.1.lampCount']).toEqual(['out of range']);
  });

  it('collects multiple messages under the same field key', () => {
    const byField = warningsByField([
      w('Identification', 'first'),
      w('Identification', 'second')
    ]);
    expect(byField.identification).toEqual(['first', 'second']);
  });

  it('omits warnings with no editable field', () => {
    expect(warningsByField([w('k[2]', 'out of range')])).toEqual({});
  });
});

describe('isNavigableWarning', () => {
  it('is false for direct ratio warnings', () => {
    expect(isNavigableWarning(w('k[1]', 'x'))).toBe(false);
  });

  it('is true for a lamp warning with a lampIndex', () => {
    expect(isNavigableWarning(w('Number of lamps', 'x', 0))).toBe(true);
  });
});
