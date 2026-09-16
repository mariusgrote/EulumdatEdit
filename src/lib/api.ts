// Typed wrappers around the Tauri commands exposed by the Rust backend.

import { invoke } from '@tauri-apps/api/core';
import type { DocResponse, EulumdatDoc, PolarOptions } from './types';

export function newFromTemplate(): Promise<DocResponse> {
  return invoke('new_from_template');
}

export function openFile(path: string): Promise<DocResponse> {
  return invoke('open_file', { path });
}

/** Opens `path`, or a new template document when omitted, in a new window. */
export function openWindow(path?: string): Promise<void> {
  return invoke('open_window', { path: path ?? null });
}

/** The document already loaded for this window (set up by `openWindow`). */
export function currentDocument(): Promise<DocResponse | null> {
  return invoke('current_document');
}

export function closeDocument(): Promise<void> {
  return invoke('close_document');
}

/** Whether another window holds unsaved changes. */
export function otherWindowsDirty(): Promise<boolean> {
  return invoke('other_windows_dirty');
}

/** Exits the app. Call through `quitApplication`, which guards unsaved changes. */
export function quitApp(): Promise<void> {
  return invoke('quit_app');
}

/** Drains any file the OS queued for opening before the UI was ready. */
export function takePendingOpen(): Promise<string | null> {
  return invoke('take_pending_open');
}

export function updateDocument(doc: EulumdatDoc): Promise<DocResponse> {
  return invoke('update_document', { doc });
}

export function save(): Promise<DocResponse> {
  return invoke('save');
}

export function saveAs(path: string): Promise<DocResponse> {
  return invoke('save_as', { path });
}

export function resampleGamma(step: number): Promise<DocResponse> {
  return invoke('resample_gamma', { step });
}

export function scaleTo100Percent(): Promise<DocResponse> {
  return invoke('scale_to_100_percent');
}

export function setStrictValidation(enabled: boolean): Promise<DocResponse> {
  return invoke('set_strict_validation', { enabled });
}

export function renderPolarSvg(options: PolarOptions): Promise<string> {
  return invoke('render_polar_svg', { options });
}

export function writeBytes(path: string, contents: Uint8Array): Promise<void> {
  return invoke('write_bytes', { path: path, contents: Array.from(contents) });
}
