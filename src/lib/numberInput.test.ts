import { describe, expect, it } from 'vitest';
import { committableNumber, isValidResampleStep } from './numberInput';

describe('committableNumber', () => {
  it('rejects missing and non-finite values', () => {
    expect(committableNumber(null)).toBeNull();
    expect(committableNumber(undefined)).toBeNull();
    expect(committableNumber(NaN)).toBeNull();
    expect(committableNumber(Infinity)).toBeNull();
    expect(committableNumber(-Infinity)).toBeNull();
  });

  it('passes finite floating-point values through unchanged by default', () => {
    expect(committableNumber(1.2345)).toBe(1.2345);
    expect(committableNumber(-7.5)).toBe(-7.5);
    expect(committableNumber(0)).toBe(0);
  });

  it('accepts only whole numbers in integer mode', () => {
    expect(committableNumber(0, { integer: true })).toBe(0);
    expect(committableNumber(2, { integer: true })).toBe(2);
    expect(committableNumber(1.5, { integer: true })).toBeNull();
  });

  it('rejects values outside hard limits instead of clamping', () => {
    expect(committableNumber(-1, { hardMin: 0 })).toBeNull();
    expect(committableNumber(0, { hardMin: 0 })).toBe(0);
    expect(committableNumber(91, { hardMax: 90 })).toBeNull();
    expect(committableNumber(90, { hardMax: 90 })).toBe(90);
  });
});

describe('isValidResampleStep', () => {
  it('accepts whole steps from 1 to 90', () => {
    expect(isValidResampleStep(1)).toBe(true);
    expect(isValidResampleStep(90)).toBe(true);
  });

  it('rejects out-of-range, fractional, and empty steps', () => {
    expect(isValidResampleStep(0)).toBe(false);
    expect(isValidResampleStep(1.5)).toBe(false);
    expect(isValidResampleStep(91)).toBe(false);
    expect(isValidResampleStep(null)).toBe(false);
  });
});
