// Typed wrappers around the Tauri commands exposed by the Rust backend.

import { invoke } from '@tauri-apps/api/core';
import type { DocResponse, EulumdatDoc, PolarOptions, WindowStateResponse } from './types';

export function newFromTemplate(): Promise<WindowStateResponse> {
  return invoke('new_from_template');
}

export function openFile(path: string): Promise<WindowStateResponse> {
  return invoke('open_file', { path });
}

export function reloadDocument(): Promise<DocResponse> {
  return invoke('reload_document');
}

/** The active document and ordered tabs for this window. */
export function currentDocument(): Promise<WindowStateResponse> {
  return invoke('current_document');
}

export function closeDocument(tabId?: string): Promise<WindowStateResponse> {
  return invoke('close_document', { tabId: tabId ?? null });
}

export function activateTab(tabId: string): Promise<WindowStateResponse> {
  return invoke('activate_tab', { tabId });
}

export function moveTab(
  tabId: string,
  targetWindow: string,
  targetIndex?: number
): Promise<WindowStateResponse> {
  return invoke('move_tab', { tabId, targetWindow, targetIndex: targetIndex ?? null });
}

export function detachTab(tabId: string, x: number, y: number): Promise<WindowStateResponse> {
  return invoke('detach_tab', { tabId, x, y });
}

export function startTabPreview(id: string, title: string): Promise<void> {
  return invoke('start_tab_preview', { id, encodedTitle: encodeURIComponent(title) });
}

export function moveTabPreview(id: string): Promise<void> {
  return invoke('move_tab_preview', { id });
}

export function endTabPreview(id: string): Promise<void> {
  return invoke('end_tab_preview', { id });
}

/** Whether any tab other than this window's active tab has unsaved changes. */
export function otherDocumentsDirty(): Promise<boolean> {
  return invoke('other_documents_dirty');
}

/** Exits the app. Call through `quitApplication`, which guards unsaved changes. */
export function quitApp(): Promise<void> {
  return invoke('quit_app');
}

/** Drains any file the OS queued for opening before the UI was ready. */
export function takePendingOpen(): Promise<string | null> {
  return invoke('take_pending_open');
}

export function updateDocument(doc: EulumdatDoc, tabId: string): Promise<DocResponse> {
  return invoke('update_document', { doc, tabId });
}

export function save(): Promise<DocResponse> {
  return invoke('save');
}

export function saveAs(path: string): Promise<DocResponse> {
  return invoke('save_as', { path });
}

export function exportIes(path: string): Promise<void> {
  return invoke('export_ies', { path });
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
