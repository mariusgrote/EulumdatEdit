import { describe, expect, it } from 'vitest';
import { canAddLampSet, canRemoveLampSet, MAX_LAMP_SETS, MIN_LAMP_SETS } from './lampSetRules';

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
