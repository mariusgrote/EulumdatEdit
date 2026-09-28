import { ask, open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import * as api from '$lib/api';
import { store } from '$lib/store.svelte';

export const PHOTOMETRIC_FILTERS = [
  { name: 'EULUMDAT', extensions: ['ldt'] },
  { name: 'IES photometry', extensions: ['ies'] }
];

export async function openFileDialog(): Promise<void> {
  const path = await openDialog({ multiple: false, filters: PHOTOMETRIC_FILTERS });
  if (typeof path === 'string') await openPath(path);
}

/** Opens `path` in a new tab, or focuses its existing tab in any window. */
export async function openPath(path: string): Promise<void> {
  await store.open(path);
}

/** Starts a new luminaire in a new tab. */
export async function newDocument(): Promise<void> {
  await store.newDoc();
}

export async function closeDocument(tabId = store.activeTabId): Promise<void> {
  if (!tabId) {
    await getCurrentWebviewWindow().close();
    return;
  }
  const tab = store.tabs.find((candidate) => candidate.id === tabId);
  if (tab?.dirty) {
    const ok = await ask(`Unsaved changes in "${tab.title}" will be lost. Close it anyway?`, {
      title: 'Unsaved changes',
      kind: 'warning'
    });
    if (!ok) return;
  }
  await store.close(tabId);
  if (store.tabs.length === 0) await getCurrentWebviewWindow().close();
}

/** Quits the app, asking first when any tab has unsaved changes. */
export async function quitApplication(): Promise<void> {
  const unsaved = store.dirty || (await api.otherDocumentsDirty());
  if (unsaved) {
    const ok = await ask('Unsaved changes in open windows will be lost. Quit anyway?', {
      title: 'Unsaved changes',
      kind: 'warning'
    });
    if (!ok) return;
  }
  await api.quitApp();
}

/** Saves to the document's path, or falls back to a Save As dialog when the
 *  document has never been saved. */
export async function saveDocument(): Promise<void> {
  if (!store.doc) return;
  if (store.path) {
    await store.save();
  } else {
    await saveDocumentAs();
  }
}

export async function saveDocumentAs(): Promise<void> {
  if (!store.doc) return;
  const path = await saveDialog({
    filters: PHOTOMETRIC_FILTERS,
    defaultPath: store.doc.fileName || 'luminaire.ldt'
  });
  if (path) await store.saveAs(path);
}

export async function exportIes(): Promise<void> {
  if (!store.doc) return;
  const name = (store.path?.split(/[\\/]/).pop() || store.doc.fileName || 'luminaire')
    .replace(/\.(ldt|ies)$/i, '');
  const path = await saveDialog({
    filters: [{ name: 'IES photometry', extensions: ['ies'] }],
    defaultPath: `${name}.ies`
  });
  if (!path) return;
  await store.exportIes(path);
}
