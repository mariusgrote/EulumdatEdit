import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { DocResponse, EulumdatDoc, Photometry } from './types';

vi.mock('./api', () => ({
  newFromTemplate: vi.fn(),
  closeDocument: vi.fn(),
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
const { quitApplication, saveDocument } = await import('./documentActions');

function makeResponse(overrides: Partial<DocResponse> = {}): DocResponse {
  return {
    doc: { fileName: 'test.ldt' } as EulumdatDoc,
    warnings: [],
    photometry: {} as Photometry,
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
});
