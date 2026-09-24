import { describe, expect, it, vi } from 'vitest';

vi.mock('@tauri-apps/plugin-dialog', () => ({ save: vi.fn() }));
vi.mock('$lib/api', () => ({ writeBytes: vi.fn() }));

const { graphExportDefaultPath } = await import('./export');

describe('graphExportDefaultPath', () => {
  it('uses the document name and graph suffix', () => {
    expect(graphExportDefaultPath('fixture.ldt', 'polar')).toBe('fixture-polar.svg');
  });

  it('removes the LDT extension case-insensitively', () => {
    expect(graphExportDefaultPath('fixture.LDT', 'polar')).toBe('fixture-polar.svg');
  });

  it('removes the IES extension case-insensitively', () => {
    expect(graphExportDefaultPath('fixture.IES', 'polar')).toBe('fixture-polar.svg');
  });

  it('keeps document names without an LDT extension', () => {
    expect(graphExportDefaultPath('fixture', 'polar')).toBe('fixture-polar.svg');
  });

  it('falls back to a useful name when the document filename is empty', () => {
    expect(graphExportDefaultPath('  ', 'polar')).toBe('luminaire-polar.svg');
  });
});
