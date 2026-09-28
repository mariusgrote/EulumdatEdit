import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocResponse, EulumdatDoc, Photometry, WindowStateResponse } from './types';

const windowApi = vi.hoisted(() => ({ close: vi.fn() }));

vi.mock('./api', () => ({
  newFromTemplate: vi.fn(),
  openFile: vi.fn(),
  closeDocument: vi.fn(),
  otherDocumentsDirty: vi.fn(),
  save: vi.fn(),
  saveAs: vi.fn(),
  exportIes: vi.fn(),
  quitApp: vi.fn()
}));

vi.mock('@tauri-apps/plugin-dialog', () => ({
  ask: vi.fn(),
  open: vi.fn(),
  save: vi.fn()
}));

vi.mock('@tauri-apps/api/webviewWindow', () => ({
  getCurrentWebviewWindow: () => windowApi
}));

const api = await import('./api');
const dialog = await import('@tauri-apps/plugin-dialog');
const { store } = await import('./store.svelte');
const { closeDocument, newDocument, openPath, quitApplication, saveDocument, exportIes } = await import(
  './documentActions'
);

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

async function seedDoc(path: string | null) {
  vi.mocked(api.newFromTemplate).mockResolvedValue(makeWindowResponse({ path }));
  await store.newDoc();
  vi.clearAllMocks();
}

beforeEach(() => {
  vi.mocked(api.save).mockResolvedValue(makeResponse({ path: '/tmp/test.ldt', dirty: false }));
  vi.mocked(api.saveAs).mockResolvedValue(makeResponse({ path: '/tmp/new.ldt', dirty: false }));
  vi.mocked(api.exportIes).mockResolvedValue(undefined);
  vi.mocked(api.closeDocument).mockResolvedValue(emptyWindow());
  vi.mocked(api.otherDocumentsDirty).mockResolvedValue(false);
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

it('exports IES as a separate file', async () => {
  await seedDoc('/tmp/test.ldt');
  vi.mocked(dialog.save).mockResolvedValue('/tmp/test.ies');

  await exportIes();

  expect(dialog.save).toHaveBeenCalledWith(expect.objectContaining({ defaultPath: 'test.ies' }));
  expect(api.exportIes).toHaveBeenCalledWith('/tmp/test.ies');
  expect(store.path).toBe('/tmp/test.ldt');
});

describe('opening documents', () => {
  it('opens a file in this window when it is empty', async () => {
    vi.mocked(api.openFile).mockResolvedValue(makeWindowResponse({ path: '/tmp/a.ldt' }));

    await openPath('/tmp/a.ldt');

    expect(api.openFile).toHaveBeenCalledWith('/tmp/a.ldt');
    expect(store.path).toBe('/tmp/a.ldt');
  });

  it('opens a file in a new tab when this window has a document', async () => {
    await seedDoc('/tmp/test.ldt');
    vi.mocked(api.openFile).mockResolvedValue(makeWindowResponse({ path: '/tmp/a.ldt' }));

    await openPath('/tmp/a.ldt');

    expect(api.openFile).toHaveBeenCalledWith('/tmp/a.ldt');
    expect(dialog.ask).not.toHaveBeenCalled();
    expect(store.path).toBe('/tmp/a.ldt');
  });

  it('shows a new-window open error in this window', async () => {
    await seedDoc('/tmp/test.ldt');
    vi.mocked(api.openFile).mockRejectedValue('bad file');

    await openPath('/tmp/bad.ldt');

    expect(store.error).toBe('bad file');
  });

  it('starts a new document in this window when it is empty', async () => {
    vi.mocked(api.newFromTemplate).mockResolvedValue(makeWindowResponse());

    await newDocument();

    expect(api.newFromTemplate).toHaveBeenCalledOnce();
  });

  it('starts a new document in a new tab when this window has a document', async () => {
    await seedDoc(null);
    vi.mocked(api.newFromTemplate).mockResolvedValue(makeWindowResponse());

    await newDocument();

    expect(api.newFromTemplate).toHaveBeenCalledOnce();
  });
});

describe('closing tabs', () => {
  it('closes the native window after its last clean tab', async () => {
    vi.mocked(api.newFromTemplate).mockResolvedValue(makeWindowResponse({ dirty: false }));
    await store.newDoc();
    vi.clearAllMocks();

    await closeDocument();

    expect(api.closeDocument).toHaveBeenCalledWith('tab-1');
    expect(windowApi.close).toHaveBeenCalledOnce();
  });

  it('keeps a dirty tab open when discarding is declined', async () => {
    await seedDoc(null);
    vi.mocked(dialog.ask).mockResolvedValue(false);

    await closeDocument();

    expect(api.closeDocument).not.toHaveBeenCalled();
    expect(windowApi.close).not.toHaveBeenCalled();
  });
});

describe('quitApplication', () => {
  it('quits without asking when no document is open', async () => {
    await quitApplication();

    expect(dialog.ask).not.toHaveBeenCalled();
    expect(api.quitApp).toHaveBeenCalledOnce();
  });

  it('quits without asking when the document is clean', async () => {
    vi.mocked(api.newFromTemplate).mockResolvedValue(makeWindowResponse({ dirty: false }));
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

  it('asks before quitting when another tab has unsaved changes', async () => {
    vi.mocked(api.otherDocumentsDirty).mockResolvedValue(true);
    vi.mocked(dialog.ask).mockResolvedValue(false);

    await quitApplication();

    expect(dialog.ask).toHaveBeenCalledOnce();
    expect(api.quitApp).not.toHaveBeenCalled();
  });
});
