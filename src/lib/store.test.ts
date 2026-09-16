import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocResponse, EulumdatDoc, Photometry } from './types';

vi.mock('./api', () => ({
  newFromTemplate: vi.fn(),
  openFile: vi.fn(),
  closeDocument: vi.fn(),
  updateDocument: vi.fn(),
  save: vi.fn(),
  saveAs: vi.fn(),
  resampleGamma: vi.fn(),
  scaleTo100Percent: vi.fn(),
  setStrictValidation: vi.fn()
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  ask: vi.fn()
}));

const api = await import('./api');
const { store } = await import('./store.svelte');

function makeDoc(): EulumdatDoc {
  return {
    identification: 'Test',
    typeIndicator: 1,
    symmetry: 1,
    cPlaneStep: 0,
    gammaStep: 90,
    measurementReportNumber: '',
    luminaireName: 'Luminaire',
    luminaireNumber: '',
    fileName: 'test.ldt',
    dateUser: '',
    luminaireLength: 100,
    luminaireWidth: 100,
    luminaireHeight: 50,
    luminousAreaLength: 90,
    luminousAreaWidth: 90,
    luminousAreaHeightC0: 0,
    luminousAreaHeightC90: 0,
    luminousAreaHeightC180: 0,
    luminousAreaHeightC270: 0,
    downwardFluxFraction: 100,
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
      }
    ],
    directRatios: Array.from({ length: 10 }, () => 0),
    cPlanes: [0],
    gammaAngles: [0, 90, 180],
    intensities: [[100, 50, 0]]
  };
}

function makePhotometry(): Photometry {
  return {
    totalOutput: 1000,
    calculatedDownwardFluxFraction: 100,
    beamAngleC0C180: 120,
    beamAngleC90C270: 120,
    fieldAngleC0C180: null,
    fieldAngleC90C270: null,
    cPlaneCount: 1,
    gammaCount: 3
  };
}

function makeResponse(overrides: Partial<DocResponse> = {}): DocResponse {
  return {
    doc: makeDoc(),
    warnings: [],
    photometry: makePhotometry(),
    path: '/tmp/test.ldt',
    dirty: false,
    strictValidation: false,
    ...overrides
  };
}

async function openDoc() {
  vi.mocked(api.openFile).mockResolvedValue(makeResponse());
  await store.open('/tmp/test.ldt');
}

/** Simulates a user edit that has not yet been committed to Rust. */
function editLuminaireName(name: string) {
  store.doc!.luminaireName = name;
  store.edited();
}

function order(fn: { mock: { invocationCallOrder: number[] } }): number {
  return fn.mock.invocationCallOrder[0];
}

beforeEach(async () => {
  vi.useFakeTimers();
  vi.mocked(api.updateDocument).mockImplementation(async (doc) =>
    makeResponse({ doc, dirty: true })
  );
  vi.mocked(api.save).mockResolvedValue(makeResponse());
  vi.mocked(api.saveAs).mockResolvedValue(makeResponse({ path: '/tmp/other.ldt' }));
  vi.mocked(api.resampleGamma).mockResolvedValue(makeResponse({ dirty: true }));
  vi.mocked(api.scaleTo100Percent).mockResolvedValue(makeResponse({ dirty: true }));
  vi.mocked(api.setStrictValidation).mockResolvedValue(makeResponse({ strictValidation: true }));
  vi.mocked(api.newFromTemplate).mockResolvedValue(makeResponse({ path: null }));
  vi.mocked(api.closeDocument).mockResolvedValue(undefined);
  await openDoc();
  vi.clearAllMocks();
});

afterEach(async () => {
  vi.mocked(api.closeDocument).mockResolvedValue(undefined);
  await store.close();
  vi.clearAllTimers();
  vi.resetAllMocks();
});

afterAll(() => {
  vi.useRealTimers();
});

describe('debounced commit', () => {
  it('waits 250 ms after an edit, then updates once', async () => {
    editLuminaireName('A');
    editLuminaireName('AB');

    await vi.advanceTimersByTimeAsync(249);
    expect(api.updateDocument).not.toHaveBeenCalled();

    await vi.advanceTimersByTimeAsync(1);
    expect(api.updateDocument).toHaveBeenCalledTimes(1);
    expect(vi.mocked(api.updateDocument).mock.calls[0][0].luminaireName).toBe('AB');
  });
});

describe('flush before backend model operations', () => {
  it('save() commits the pending edit first and the old timer never fires', async () => {
    editLuminaireName('Saved name');
    await store.save();

    expect(api.updateDocument).toHaveBeenCalledTimes(1);
    expect(vi.mocked(api.updateDocument).mock.calls[0][0].luminaireName).toBe('Saved name');
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(order(vi.mocked(api.save)));

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).toHaveBeenCalledTimes(1);
  });

  it('saveAs() commits the pending edit first', async () => {
    editLuminaireName('Saved as');
    await store.saveAs('/tmp/other.ldt');

    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(order(vi.mocked(api.saveAs)));
    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).toHaveBeenCalledTimes(1);
  });

  it('resampleGamma() and scaleTo100() commit the pending edit first', async () => {
    editLuminaireName('Resample');
    await store.resampleGamma(5);
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(
      order(vi.mocked(api.resampleGamma))
    );

    vi.clearAllMocks();
    editLuminaireName('Scale');
    await store.scaleTo100();
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(
      order(vi.mocked(api.scaleTo100Percent))
    );

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).toHaveBeenCalledTimes(1);
  });

  it('setStrictValidation() commits the pending edit first', async () => {
    editLuminaireName('Strict');
    await store.setStrictValidation(true);

    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(
      order(vi.mocked(api.setStrictValidation))
    );
    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).toHaveBeenCalledTimes(1);
  });

  it('does not issue a redundant update when no edit is pending', async () => {
    await store.save();
    await store.saveAs('/tmp/other.ldt');
    await store.resampleGamma(5);
    await store.scaleTo100();
    await store.setStrictValidation(true);

    expect(api.updateDocument).not.toHaveBeenCalled();
    expect(api.save).toHaveBeenCalledTimes(1);
    expect(api.saveAs).toHaveBeenCalledTimes(1);
    expect(api.resampleGamma).toHaveBeenCalledTimes(1);
    expect(api.scaleTo100Percent).toHaveBeenCalledTimes(1);
    expect(api.setStrictValidation).toHaveBeenCalledTimes(1);
  });
});

describe('failed flush', () => {
  const operations = [
    { name: 'save', run: () => store.save(), call: () => api.save },
    { name: 'saveAs', run: () => store.saveAs('/tmp/other.ldt'), call: () => api.saveAs },
    { name: 'resampleGamma', run: () => store.resampleGamma(5), call: () => api.resampleGamma },
    { name: 'scaleTo100', run: () => store.scaleTo100(), call: () => api.scaleTo100Percent },
    {
      name: 'setStrictValidation',
      run: () => store.setStrictValidation(true),
      call: () => api.setStrictValidation
    }
  ];

  it.each(operations)('prevents $name when updateDocument rejects', async ({ run, call }) => {
    vi.mocked(api.updateDocument).mockRejectedValue('invalid draft');
    editLuminaireName('Invalid');

    await run();

    expect(api.updateDocument).toHaveBeenCalledTimes(1);
    expect(call()).not.toHaveBeenCalled();
    expect(store.error).toBe('invalid draft');
  });
});

describe('document replacement cancels the pending commit', () => {
  it('open() cancels the timer before opening', async () => {
    editLuminaireName('Stale');
    vi.mocked(api.openFile).mockResolvedValue(makeResponse());
    await store.open('/tmp/next.ldt');

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).not.toHaveBeenCalled();
  });

  it('open() cancels the timer even while the open IPC is still running', async () => {
    editLuminaireName('Stale');
    let resolveOpen: (res: DocResponse) => void = () => {};
    vi.mocked(api.openFile).mockReturnValue(
      new Promise((resolve) => {
        resolveOpen = resolve;
      })
    );
    const opening = store.open('/tmp/next.ldt');

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).not.toHaveBeenCalled();

    resolveOpen(makeResponse());
    await opening;
  });

  it('newDoc() cancels the timer', async () => {
    editLuminaireName('Stale');
    await store.newDoc();

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.newFromTemplate).toHaveBeenCalledTimes(1);
    expect(api.updateDocument).not.toHaveBeenCalled();
  });

  it('close() cancels the timer', async () => {
    editLuminaireName('Stale');
    await store.close();

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.closeDocument).toHaveBeenCalledTimes(1);
    expect(api.updateDocument).not.toHaveBeenCalled();
    expect(store.doc).toBeNull();
  });

  it('close() cancels the timer even while the close IPC is still running', async () => {
    editLuminaireName('Stale');
    let resolveClose: () => void = () => {};
    vi.mocked(api.closeDocument).mockReturnValue(
      new Promise((resolve) => {
        resolveClose = resolve;
      })
    );
    const closing = store.close();

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).not.toHaveBeenCalled();

    resolveClose();
    await closing;
  });
});
