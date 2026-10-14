import { describe, expect, it } from 'vitest';
import { canAddLampSet, canRemoveLampSet, canUseAbsolutePhotometry, storedLampCount, MAX_LAMP_SETS, MIN_LAMP_SETS } from './lampSetRules';

describe('lamp set bounds', () => {
  it('matches the EULUMDAT 1 through 20 contract', () => {
    expect(MIN_LAMP_SETS).toBe(1);
    expect(MAX_LAMP_SETS).toBe(20);
  });

  it('allows removal only above one set', () => {
    expect(canRemoveLampSet(0)).toBe(false);
    expect(canRemoveLampSet(1)).toBe(false);
    expect(canRemoveLampSet(2)).toBe(true);
    expect(canRemoveLampSet(19)).toBe(true);
    expect(canRemoveLampSet(20)).toBe(true);
    expect(canRemoveLampSet(21)).toBe(true);
  });

  it('allows addition only below twenty sets', () => {
    expect(canAddLampSet(0)).toBe(true);
    expect(canAddLampSet(1)).toBe(true);
    expect(canAddLampSet(2)).toBe(true);
    expect(canAddLampSet(19)).toBe(true);
    expect(canAddLampSet(20)).toBe(false);
    expect(canAddLampSet(21)).toBe(false);
  });
});

describe('absolute photometry', () => {
  it('requires one set and prevents adding alternatives', () => {
    expect(canUseAbsolutePhotometry(0)).toBe(false);
    expect(canUseAbsolutePhotometry(1)).toBe(true);
    expect(canUseAbsolutePhotometry(2)).toBe(false);
    expect(canAddLampSet(1, true)).toBe(false);
    expect(canAddLampSet(1, false)).toBe(true);
  });

  it('preserves the negative marker when editing a positive emitter count', () => {
    expect(storedLampCount(42, true)).toBe(-42);
    expect(storedLampCount(1, true)).toBe(-1);
    expect(storedLampCount(42, false)).toBe(42);
    expect(storedLampCount(-42, false)).toBe(42);
  });
});
