import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocResponse, EulumdatDoc, Photometry } from './types';

vi.mock('./api', () => ({
  newFromTemplate: vi.fn(),
  openFile: vi.fn(),
  openWindow: vi.fn(),
  closeDocument: vi.fn(),
  otherWindowsDirty: vi.fn(),
  save: vi.fn(),
  saveAs: vi.fn(),
  quitApp: vi.fn()
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  ask: vi.fn(),
  open: vi.fn(),
  save: vi.fn()
}));

const api = await import('./api');
const dialog = await import('@tauri-apps/plugin-dialog');
const { store } = await import('./store.svelte');
const { newDocument, openPath, quitApplication, saveDocument } = await import('./documentActions');

function makeResponse(overrides: Partial<DocResponse> = {}): DocResponse {
  return {
    doc: { fileName: 'test.ldt' } as EulumdatDoc,
    warnings: [],
    photometry: {} as Photometry,
    ugr: { status: 'blocked', blockers: [] },
    path: null,
    dirty: true,
    strictValidation: false,
    ...overrides
  };
}

async function seedDoc(path: string | null) {
  vi.mocked(api.newFromTemplate).mockResolvedValue(makeResponse({ path }));
  await store.newDoc();
  vi.clearAllMocks();
}

beforeEach(() => {
  vi.mocked(api.save).mockResolvedValue(makeResponse({ path: '/tmp/test.ldt', dirty: false }));
  vi.mocked(api.saveAs).mockResolvedValue(makeResponse({ path: '/tmp/new.ldt', dirty: false }));
  vi.mocked(api.closeDocument).mockResolvedValue(undefined);
  vi.mocked(api.openWindow).mockResolvedValue(undefined);
  vi.mocked(api.otherWindowsDirty).mockResolvedValue(false);
});

afterEach(async () => {
  await store.close();
  vi.resetAllMocks();
});

describe('saveDocument', () => {
  it('opens Save As for a document without a path', async () => {
    await seedDoc(null);
    vi.mocked(dialog.save).mockResolvedValue('/tmp/new.ldt');

    await saveDocument();

    expect(dialog.save).toHaveBeenCalledOnce();
    expect(api.saveAs).toHaveBeenCalledWith('/tmp/new.ldt');
    expect(api.save).not.toHaveBeenCalled();
  });

  it('does not save when the dialog is cancelled', async () => {
    await seedDoc(null);
    vi.mocked(dialog.save).mockResolvedValue(null);

    await saveDocument();

    expect(dialog.save).toHaveBeenCalledOnce();
    expect(api.saveAs).not.toHaveBeenCalled();
    expect(api.save).not.toHaveBeenCalled();
  });

  it('saves in place when the document has a path', async () => {
    await seedDoc('/tmp/test.ldt');

    await saveDocument();

    expect(api.save).toHaveBeenCalledOnce();
    expect(dialog.save).not.toHaveBeenCalled();
  });

  it('does nothing without a document', async () => {
    await saveDocument();

    expect(dialog.save).not.toHaveBeenCalled();
    expect(api.save).not.toHaveBeenCalled();
    expect(api.saveAs).not.toHaveBeenCalled();
  });
});

describe('opening documents', () => {
  it('opens a file in this window when it is empty', async () => {
    vi.mocked(api.openFile).mockResolvedValue(makeResponse({ path: '/tmp/a.ldt' }));

    await openPath('/tmp/a.ldt');

    expect(api.openFile).toHaveBeenCalledWith('/tmp/a.ldt');
    expect(api.openWindow).not.toHaveBeenCalled();
    expect(store.path).toBe('/tmp/a.ldt');
  });

  it('opens a file in a new window when this one has a document', async () => {
    await seedDoc('/tmp/test.ldt');
    vi.mocked(api.openWindow).mockResolvedValue(undefined);

    await openPath('/tmp/a.ldt');

    expect(api.openWindow).toHaveBeenCalledWith('/tmp/a.ldt');
    expect(api.openFile).not.toHaveBeenCalled();
    expect(dialog.ask).not.toHaveBeenCalled();
    expect(store.path).toBe('/tmp/test.ldt');
  });

  it('shows a new-window open error in this window', async () => {
    await seedDoc('/tmp/test.ldt');
    vi.mocked(api.openWindow).mockRejectedValue('bad file');

    await openPath('/tmp/bad.ldt');

    expect(store.error).toBe('bad file');
  });

  it('starts a new document in this window when it is empty', async () => {
    vi.mocked(api.newFromTemplate).mockResolvedValue(makeResponse());

    await newDocument();

    expect(api.newFromTemplate).toHaveBeenCalledOnce();
    expect(api.openWindow).not.toHaveBeenCalled();
  });

  it('starts a new document in a new window when this one has a document', async () => {
    await seedDoc(null);
    vi.mocked(api.openWindow).mockResolvedValue(undefined);

    await newDocument();

    expect(api.openWindow).toHaveBeenCalledWith(undefined);
    expect(api.newFromTemplate).not.toHaveBeenCalled();
  });
});

describe('quitApplication', () => {
  it('quits without asking when no document is open', async () => {
    await quitApplication();

    expect(dialog.ask).not.toHaveBeenCalled();
    expect(api.quitApp).toHaveBeenCalledOnce();
  });

  it('quits without asking when the document is clean', async () => {
    vi.mocked(api.newFromTemplate).mockResolvedValue(makeResponse({ dirty: false }));
    await store.newDoc();
    vi.clearAllMocks();

    await quitApplication();

    expect(dialog.ask).not.toHaveBeenCalled();
    expect(api.quitApp).toHaveBeenCalledOnce();
  });

  it('stays open when discarding a dirty document is declined', async () => {
    await seedDoc(null);
    vi.mocked(dialog.ask).mockResolvedValue(false);

    await quitApplication();

    expect(dialog.ask).toHaveBeenCalledOnce();
    expect(api.quitApp).not.toHaveBeenCalled();
  });

  it('quits once discarding a dirty document is confirmed', async () => {
    await seedDoc(null);
    vi.mocked(dialog.ask).mockResolvedValue(true);

    await quitApplication();

    expect(dialog.ask).toHaveBeenCalledOnce();
    expect(api.quitApp).toHaveBeenCalledOnce();
  });

  it('asks before quitting when another window has unsaved changes', async () => {
    vi.mocked(api.otherWindowsDirty).mockResolvedValue(true);
    vi.mocked(dialog.ask).mockResolvedValue(false);

    await quitApplication();

    expect(dialog.ask).toHaveBeenCalledOnce();
    expect(api.quitApp).not.toHaveBeenCalled();
  });
});
