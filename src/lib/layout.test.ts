import { describe, expect, it } from 'vitest';
import { INSPECTOR_COLLAPSE_WIDTH, isNarrowLayout } from './layout';

describe('inspector breakpoint', () => {
  it('switches to the narrow layout just below the breakpoint', () => {
    expect(INSPECTOR_COLLAPSE_WIDTH).toBe(1120);
    expect(isNarrowLayout(1119)).toBe(true);
    expect(isNarrowLayout(1120)).toBe(false);
  });

  it('is narrow at the native minimum window width', () => {
    expect(isNarrowLayout(900)).toBe(true);
  });
});
