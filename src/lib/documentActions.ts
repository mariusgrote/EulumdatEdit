import { ask, open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import * as api from '$lib/api';
import { store } from '$lib/store.svelte';

export const EULUMDAT_FILTER = [{ name: 'EULUMDAT', extensions: ['ldt', 'LDT'] }];

export async function openFileDialog(): Promise<void> {
  const path = await openDialog({ multiple: false, filters: EULUMDAT_FILTER });
  if (typeof path === 'string') await openPath(path);
}

/** Opens `path` in this window when it is empty, otherwise in a new window. */
export async function openPath(path: string): Promise<void> {
  if (store.doc) {
    await store.openInNewWindow(path);
  } else {
    await store.open(path);
  }
}

/** Starts a new luminaire in this window when it is empty, otherwise in a new window. */
export async function newDocument(): Promise<void> {
  if (store.doc) {
    await store.openInNewWindow();
  } else {
    await store.newDoc();
  }
}

export async function closeDocument(): Promise<void> {
  if (!(await store.confirmDiscardChanges())) return;
  await store.close();
}

/** Quits the app, asking first when this or any other window has unsaved changes. */
export async function quitApplication(): Promise<void> {
  const unsaved = store.dirty || (await api.otherWindowsDirty());
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
    filters: EULUMDAT_FILTER,
    defaultPath: store.doc.fileName || 'luminaire.ldt'
  });
  if (path) await store.saveAs(path);
}
