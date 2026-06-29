// Central document store. The Rust backend is the source of truth; this store
// holds the editable copy plus derived warnings/photometry returned by Rust.

import { ask } from '@tauri-apps/plugin-dialog';
import * as api from './api';
import type { DocResponse, EulumdatDoc, Photometry, Warning } from './types';

class DocStore {
  doc = $state<EulumdatDoc | null>(null);
  warnings = $state<Warning[]>([]);
  photometry = $state<Photometry | null>(null);
  path = $state<string | null>(null);
  dirty = $state(false);
  busy = $state(false);
  error = $state<string | null>(null);
  /** Legacy EULUMDAT text-length limits (8.3 filename, etc.). Off by default. */
  strictValidation = $state(false);

  #commitTimer: ReturnType<typeof setTimeout> | null = null;

  /** Applies a backend response. `replaceDoc` is false during live edits so the
   *  user's in-progress input isn't clobbered by the round-tripped copy. */
  #apply(res: DocResponse, replaceDoc: boolean) {
    if (replaceDoc) this.doc = res.doc;
    this.warnings = res.warnings;
    this.photometry = res.photometry;
    this.path = res.path;
    this.dirty = res.dirty;
    this.strictValidation = res.strictValidation;
    this.error = null;
  }

  async #run<T>(fn: () => Promise<T>): Promise<T | null> {
    this.busy = true;
    try {
      return await fn();
    } catch (e) {
      this.error = String(e);
      return null;
    } finally {
      this.busy = false;
    }
  }

  /** Guards actions that would discard in-progress edits (New, Open, close).
   *  Returns true when it is safe to proceed: either the document is clean or
   *  the user confirmed discarding their unsaved changes. */
  async confirmDiscardChanges(): Promise<boolean> {
    if (!this.dirty) return true;
    return ask('You have unsaved changes that will be lost. Continue?', {
      title: 'Unsaved changes',
      kind: 'warning'
    });
  }

  async newDoc() {
    const res = await this.#run(() => api.newFromTemplate());
    if (res) this.#apply(res, true);
  }

  async open(path: string) {
    const res = await this.#run(() => api.openFile(path));
    if (res) this.#apply(res, true);
  }

  /** Closes the open document and returns the UI to the welcome screen. */
  async close() {
    const res = await this.#run(async () => {
      await api.closeDocument();
      return true;
    });
    if (!res) return;
    if (this.#commitTimer) {
      clearTimeout(this.#commitTimer);
      this.#commitTimer = null;
    }
    this.doc = null;
    this.warnings = [];
    this.photometry = null;
    this.path = null;
    this.dirty = false;
    this.error = null;
  }

  /** Discards in-memory edits by reloading the document from its file on disk.
   *  No-op for an unsaved (pathless) document. */
  async revert() {
    if (!this.path) return;
    await this.open(this.path);
  }

  async save() {
    const res = await this.#run(() => api.save());
    if (res) this.#apply(res, false);
  }

  async saveAs(path: string) {
    const res = await this.#run(() => api.saveAs(path));
    if (res) this.#apply(res, false);
  }

  async resampleGamma(step: number) {
    const res = await this.#run(() => api.resampleGamma(step));
    if (res) this.#apply(res, true);
  }

  async scaleTo100() {
    const res = await this.#run(() => api.scaleTo100Percent());
    if (res) this.#apply(res, true);
  }

  /** Toggles legacy strict validation and re-validates the open document. */
  async setStrictValidation(enabled: boolean) {
    if (!this.doc) {
      this.strictValidation = enabled;
      return;
    }
    const res = await this.#run(() => api.setStrictValidation(enabled));
    if (res) this.#apply(res, false);
  }

  /** Marks the document dirty and schedules a debounced validate/recompute. */
  edited() {
    this.dirty = true;
    if (this.#commitTimer) clearTimeout(this.#commitTimer);
    this.#commitTimer = setTimeout(() => this.commit(), 250);
  }

  /** Pushes the current doc to Rust for validation + photometry. */
  async commit() {
    if (!this.doc) return;
    const snapshot = $state.snapshot(this.doc) as EulumdatDoc;
    const res = await this.#run(() => api.updateDocument(snapshot));
    if (res) this.#apply(res, false);
  }
}

export const store = new DocStore();
