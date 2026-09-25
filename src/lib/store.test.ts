import { afterAll, afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocResponse, EulumdatDoc, Photometry, WindowStateResponse } from './types';

vi.mock('./api', () => ({
  newFromTemplate: vi.fn(),
  openFile: vi.fn(),
  reloadDocument: vi.fn(),
  currentDocument: vi.fn(),
  closeDocument: vi.fn(),
  activateTab: vi.fn(),
  moveTab: vi.fn(),
  detachTab: vi.fn(),
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
    ugr: { status: 'blocked', blockers: [] },
    path: '/tmp/test.ldt',
    dirty: false,
    strictValidation: false,
    ...overrides
  };
}

function makeWindowResponse(overrides: Partial<DocResponse> = {}): WindowStateResponse {
  const document = makeResponse(overrides);
  return {
    document,
    tabs: [
      {
        id: 'tab-1',
        title: document.path?.split('/').pop() || 'Untitled',
        path: document.path,
        dirty: document.dirty
      }
    ],
    activeTabId: 'tab-1'
  };
}

const emptyWindow = (): WindowStateResponse => ({ document: null, tabs: [], activeTabId: null });

async function openDoc() {
  vi.mocked(api.openFile).mockResolvedValue(makeWindowResponse());
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

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: string) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
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
  vi.mocked(api.newFromTemplate).mockResolvedValue(makeWindowResponse({ path: null }));
  vi.mocked(api.currentDocument).mockResolvedValue(makeWindowResponse());
  vi.mocked(api.closeDocument).mockResolvedValue(emptyWindow());
  await openDoc();
  vi.clearAllMocks();
});

afterEach(async () => {
  vi.mocked(api.closeDocument).mockResolvedValue(emptyWindow());
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
    expect(vi.mocked(api.updateDocument).mock.calls[0][1]).toBe('tab-1');
  });

  it('waits for a timer-started update before saving', async () => {
    const update = deferred<DocResponse>();
    vi.mocked(api.updateDocument).mockReturnValue(update.promise);
    editLuminaireName('Saved after update');
    vi.advanceTimersByTime(250);

    const saving = store.save();
    await Promise.resolve();
    expect(api.save).not.toHaveBeenCalled();

    update.resolve(makeResponse({ dirty: true }));
    await saving;
    expect(api.save).toHaveBeenCalledOnce();
  });

  it('stops a dependent action when a timer-started update fails', async () => {
    const update = deferred<DocResponse>();
    vi.mocked(api.updateDocument).mockReturnValue(update.promise);
    editLuminaireName('Rejected');
    vi.advanceTimersByTime(250);

    const saving = store.save();
    update.reject('invalid draft');
    await saving;

    expect(api.save).not.toHaveBeenCalled();
    expect(store.error).toBe('invalid draft');
  });

  it('does not apply a late update to another active tab', async () => {
    const update = deferred<DocResponse>();
    vi.mocked(api.updateDocument).mockReturnValue(update.promise);
    editLuminaireName('First tab edit');
    vi.advanceTimersByTime(250);

    store.tabs.push({ id: 'tab-2', title: 'second.ldt', path: '/tmp/second.ldt', dirty: false });
    store.activeTabId = 'tab-2';
    store.doc = { ...makeDoc(), luminaireName: 'Second tab' };
    store.path = '/tmp/second.ldt';
    store.dirty = false;
    update.resolve(makeResponse({ path: '/tmp/first.ldt', dirty: true }));
    await store.flushEdits();

    expect(store.doc?.luminaireName).toBe('Second tab');
    expect(store.path).toBe('/tmp/second.ldt');
    expect(store.dirty).toBe(false);
    expect(store.tabs[1]).toMatchObject({ title: 'second.ldt', dirty: false });
    await store.close('tab-1');
  });

  it('drains an edit made while a tab switch waits for an update', async () => {
    const first = deferred<DocResponse>();
    const second = deferred<DocResponse>();
    vi.mocked(api.updateDocument)
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    vi.mocked(api.activateTab).mockResolvedValue({
      document: makeResponse({ path: '/tmp/second.ldt' }),
      tabs: [
        { id: 'tab-1', title: 'test.ldt', path: '/tmp/test.ldt', dirty: true },
        { id: 'tab-2', title: 'second.ldt', path: '/tmp/second.ldt', dirty: false }
      ],
      activeTabId: 'tab-2'
    });
    store.tabs.push({ id: 'tab-2', title: 'second.ldt', path: '/tmp/second.ldt', dirty: false });
    editLuminaireName('First edit');
    vi.advanceTimersByTime(250);

    const switching = store.activateTab('tab-2');
    editLuminaireName('Second edit');
    first.resolve(makeResponse({ dirty: true }));
    try {
      for (let i = 0; i < 10; i++) await Promise.resolve();
      expect(api.activateTab).not.toHaveBeenCalled();
      expect(api.updateDocument).toHaveBeenCalledTimes(2);
      expect(vi.mocked(api.updateDocument).mock.calls[1][0].luminaireName).toBe('Second edit');
    } finally {
      second.resolve(makeResponse({ dirty: true }));
      await switching;
    }
    expect(api.activateTab).toHaveBeenCalledOnce();
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

describe('document replacement preserves the pending edit', () => {
  it('revert() reloads the active tab without committing its draft', async () => {
    vi.mocked(api.reloadDocument).mockResolvedValue(makeResponse());
    editLuminaireName('Discard me');

    await store.revert();
    await vi.advanceTimersByTimeAsync(1000);

    expect(api.reloadDocument).toHaveBeenCalledOnce();
    expect(api.openFile).not.toHaveBeenCalled();
    expect(api.updateDocument).not.toHaveBeenCalled();
    expect(store.doc?.luminaireName).toBe('Luminaire');
    expect(store.dirty).toBe(false);
  });

  it('open() commits the current tab before opening another', async () => {
    editLuminaireName('Stale');
    vi.mocked(api.openFile).mockResolvedValue(makeWindowResponse());
    await store.open('/tmp/next.ldt');

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).toHaveBeenCalledOnce();
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(order(vi.mocked(api.openFile)));
  });

  it('open() cancels the timer even while the open IPC is still running', async () => {
    editLuminaireName('Stale');
    let resolveOpen: (res: WindowStateResponse) => void = () => {};
    vi.mocked(api.openFile).mockReturnValue(
      new Promise((resolve) => {
        resolveOpen = resolve;
      })
    );
    const opening = store.open('/tmp/next.ldt');

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).toHaveBeenCalledOnce();

    resolveOpen(makeWindowResponse());
    await opening;
  });

  it('newDoc() commits the current tab first', async () => {
    editLuminaireName('Stale');
    await store.newDoc();

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.newFromTemplate).toHaveBeenCalledTimes(1);
    expect(api.updateDocument).toHaveBeenCalledOnce();
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(
      order(vi.mocked(api.newFromTemplate))
    );
  });

  it('loadCurrent() commits the previously active tab before applying a window event', async () => {
    editLuminaireName('Preserved across window event');

    await store.loadCurrent();

    expect(api.updateDocument).toHaveBeenCalledOnce();
    expect(vi.mocked(api.updateDocument).mock.calls[0][1]).toBe('tab-1');
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(
      order(vi.mocked(api.currentDocument))
    );
  });

  it('close() cancels the timer', async () => {
    editLuminaireName('Stale');
    await store.close();

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.closeDocument).toHaveBeenCalledTimes(1);
    expect(api.updateDocument).not.toHaveBeenCalled();
    expect(store.doc).toBeNull();
  });

  it('close() commits an active draft before closing another tab', async () => {
    store.tabs.push({ id: 'tab-2', title: 'other.ldt', path: '/tmp/other.ldt', dirty: false });
    vi.mocked(api.closeDocument).mockImplementation(async () =>
      makeWindowResponse({ doc: { ...makeDoc(), luminaireName: 'Keep me' }, dirty: true })
    );
    editLuminaireName('Keep me');

    await store.close('tab-2');
    await vi.advanceTimersByTimeAsync(1000);

    expect(api.updateDocument).toHaveBeenCalledOnce();
    expect(order(vi.mocked(api.updateDocument))).toBeLessThan(order(vi.mocked(api.closeDocument)));
    expect(store.doc?.luminaireName).toBe('Keep me');
    expect(store.dirty).toBe(true);
  });

  it('close() cancels the timer even while the close IPC is still running', async () => {
    editLuminaireName('Stale');
    let resolveClose: (res: WindowStateResponse) => void = () => {};
    vi.mocked(api.closeDocument).mockReturnValue(
      new Promise((resolve) => {
        resolveClose = resolve;
      })
    );
    const closing = store.close();

    await vi.advanceTimersByTimeAsync(1000);
    expect(api.updateDocument).not.toHaveBeenCalled();

    resolveClose(emptyWindow());
    await closing;
  });
});

describe('moving an inactive tab preserves the active draft', () => {
  const operations = [
    {
      name: 'moveTab',
      run: () => store.moveTab('tab-2', 'other-window', 0),
      call: () => api.moveTab
    },
    {
      name: 'detachTab',
      run: () => store.detachTab('tab-2', 100, 200),
      call: () => api.detachTab
    }
  ];

  beforeEach(() => {
    store.tabs.push({ id: 'tab-2', title: 'other.ldt', path: '/tmp/other.ldt', dirty: false });
  });

  it.each(operations)(
    '$name commits the active draft before applying the window response',
    async ({ run, call }) => {
      let backendName = 'Luminaire';
      vi.mocked(api.updateDocument).mockImplementation(async (doc) => {
        backendName = doc.luminaireName;
        return makeResponse({ doc, dirty: true });
      });
      vi.mocked(api.moveTab).mockImplementation(async () =>
        makeWindowResponse({ doc: { ...makeDoc(), luminaireName: backendName }, dirty: true })
      );
      vi.mocked(api.detachTab).mockImplementation(async () =>
        makeWindowResponse({ doc: { ...makeDoc(), luminaireName: backendName }, dirty: true })
      );
      editLuminaireName('Keep active draft');

      await run();
      await vi.advanceTimersByTimeAsync(1000);

      expect(api.updateDocument).toHaveBeenCalledOnce();
      expect(vi.mocked(api.updateDocument).mock.calls[0][1]).toBe('tab-1');
      expect(order(vi.mocked(api.updateDocument))).toBeLessThan(order(vi.mocked(call())));
      expect(store.doc?.luminaireName).toBe('Keep active draft');
      expect(store.dirty).toBe(true);
    }
  );

  it.each(operations)(
    '$name leaves the draft and tabs alone when its flush fails',
    async ({ run, call }) => {
      vi.mocked(api.updateDocument).mockRejectedValue('invalid draft');
      editLuminaireName('Uncommitted draft');

      await run();

      expect(api.updateDocument).toHaveBeenCalledOnce();
      expect(call()).not.toHaveBeenCalled();
      expect(store.doc?.luminaireName).toBe('Uncommitted draft');
      expect(store.tabs.map((tab) => tab.id)).toEqual(['tab-1', 'tab-2']);
      expect(store.activeTabId).toBe('tab-1');
      expect(store.dirty).toBe(true);
      expect(store.error).toBe('invalid draft');
    }
  );
});
