import { describe, expect, it } from 'vitest';
import type { Warning } from './types';
import {
  isNavigableWarning,
  resolveWarningTarget,
  resolveWarningTargets,
  warningsByField,
  warningsBySection
} from './warningNavigation';

function w(fieldKey: string | null, message: string, lampIndex: number | null = null): Warning {
  return { field: `label for ${fieldKey}`, fieldKey, message, lampIndex };
}

describe('resolveWarningTarget', () => {
  it('maps general, geometry, and lamp keys to their sections', () => {
    expect(resolveWarningTarget(w('identification', 'x'))?.section).toBe('general');
    expect(resolveWarningTarget(w('luminaireWidth', 'x'))?.section).toBe('geometry');
    expect(resolveWarningTarget(w('lampCount', 'x', 0))?.section).toBe('lamps');
  });

  it('maps a geometry warning to a doc field key', () => {
    expect(resolveWarningTarget(w('luminaireWidth', 'out of range'))).toEqual({
      section: 'geometry',
      fieldKey: 'luminaireWidth'
    });
  });

  it('maps a lamp warning to its indexed field key via lampIndex', () => {
    expect(resolveWarningTarget(w('lampCount', 'out of range', 1))).toEqual({
      section: 'lamps',
      fieldKey: 'lamps.1.lampCount'
    });
  });

  it('leaves a lamp warning unmapped when it carries no lampIndex', () => {
    expect(resolveWarningTarget(w('lampCount', 'out of range', null))).toEqual({
      section: 'lamps',
      fieldKey: null
    });
  });

  it('assigns each lamp warning to the set named by its lampIndex', () => {
    const targets = resolveWarningTargets([w('lampCount', 'a', 0), w('lampCount', 'b', 1)]);
    expect(targets[0]?.fieldKey).toBe('lamps.0.lampCount');
    expect(targets[1]?.fieldKey).toBe('lamps.1.lampCount');
  });

  it('navigates by fieldKey regardless of the display label', () => {
    const warning: Warning = {
      field: 'Reworded upstream label',
      fieldKey: 'luminaireLength',
      message: 'out of range',
      lampIndex: null
    };
    expect(resolveWarningTarget(warning)).toEqual({
      section: 'geometry',
      fieldKey: 'luminaireLength'
    });
  });

  it('returns null for a warning without a fieldKey', () => {
    const warning: Warning = {
      field: 'Width of luminaire',
      fieldKey: null,
      message: 'out of range',
      lampIndex: null
    };
    expect(resolveWarningTarget(warning)).toBe(null);
  });
});

describe('warningsBySection', () => {
  it('counts warnings per sidebar section', () => {
    const counts = warningsBySection([
      w('identification', 'too long'),
      w('luminaireWidth', 'out of range'),
      w('lampCount', 'out of range', 0),
      w(null, 'out of range')
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
      w('identification', 'too long'),
      w('luminaireWidth', 'out of range'),
      w('lampCount', 'out of range', 1)
    ]);
    expect(byField.identification).toEqual(['too long']);
    expect(byField.luminaireWidth).toEqual(['out of range']);
    expect(byField['lamps.1.lampCount']).toEqual(['out of range']);
  });

  it('collects multiple messages under the same field key', () => {
    const byField = warningsByField([w('identification', 'first'), w('identification', 'second')]);
    expect(byField.identification).toEqual(['first', 'second']);
  });

  it('omits warnings with no editable field', () => {
    expect(warningsByField([w(null, 'out of range')])).toEqual({});
  });
});

describe('isNavigableWarning', () => {
  it('is false for warnings without a field key', () => {
    expect(isNavigableWarning(w(null, 'x'))).toBe(false);
  });

  it('is true for a lamp warning with a lampIndex', () => {
    expect(isNavigableWarning(w('lampCount', 'x', 0))).toBe(true);
  });
});
