import { open as openDialog } from '@tauri-apps/plugin-dialog';
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
