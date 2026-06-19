// Typed wrappers around the Tauri commands exposed by the Rust backend.

import { invoke } from '@tauri-apps/api/core';
import type { DocResponse, EulumdatDoc, PolarOptions } from './types';

export function newFromTemplate(): Promise<DocResponse> {
  return invoke('new_from_template');
}

export function openFile(path: string): Promise<DocResponse> {
  return invoke('open_file', { path });
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
