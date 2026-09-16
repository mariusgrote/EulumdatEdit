import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import * as api from '$lib/api';
import { store } from '$lib/store.svelte';

export const EULUMDAT_FILTER = [{ name: 'EULUMDAT', extensions: ['ldt', 'LDT'] }];

export async function openFileDialog(): Promise<void> {
  if (!(await store.confirmDiscardChanges())) return;
  const path = await openDialog({ multiple: false, filters: EULUMDAT_FILTER });
  if (typeof path === 'string') await store.open(path);
}

export async function newDocument(): Promise<void> {
  if (!(await store.confirmDiscardChanges())) return;
  await store.newDoc();
}

export async function closeDocument(): Promise<void> {
  if (!(await store.confirmDiscardChanges())) return;
  await store.close();
}

/** Quits the app after the same unsaved-changes guard as closing a document. */
export async function quitApplication(): Promise<void> {
  if (!(await store.confirmDiscardChanges())) return;
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
