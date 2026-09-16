// Central document store. The Rust backend is the source of truth; this store
// holds the editable copy plus derived warnings/photometry returned by Rust.

import { ask } from '@tauri-apps/plugin-dialog';
import * as api from './api';
import type { DocResponse, EulumdatDoc, Photometry, Warning } from './types';
import { warningsByField } from './warningNavigation';

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

  /** Warning messages keyed by form field key, for inline display beside inputs. */
  fieldWarnings = $derived.by<Record<string, string[]>>(() => warningsByField(this.warnings));

  /** Field key transiently highlighted after navigating to a warning. */
  highlightedFieldKey = $state<string | null>(null);

  #commitTimer: ReturnType<typeof setTimeout> | null = null;
  #highlightTimer: ReturnType<typeof setTimeout> | null = null;

  /** Highlights a field for a short window so the user can spot it after navigation. */
  highlightField(fieldKey: string, durationMs = 2000) {
    this.highlightedFieldKey = fieldKey;
    if (this.#highlightTimer) clearTimeout(this.#highlightTimer);
    this.#highlightTimer = setTimeout(() => {
      this.highlightedFieldKey = null;
      this.#highlightTimer = null;
    }, durationMs);
  }

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
    this.#cancelPendingCommit();
    const res = await this.#run(() => api.newFromTemplate());
    if (res) this.#apply(res, true);
  }

  async open(path: string) {
    this.#cancelPendingCommit();
    const res = await this.#run(() => api.openFile(path));
    if (res) this.#apply(res, true);
  }

  /** Closes the open document and returns the UI to the welcome screen. */
  async close() {
    this.#cancelPendingCommit();
    const res = await this.#run(async () => {
      await api.closeDocument();
      return true;
    });
    if (!res) return;
    this.doc = null;
    this.warnings = [];
    this.photometry = null;
    this.path = null;
    this.dirty = false;
    this.error = null;
    this.strictValidation = false;
  }

  /** Discards in-memory edits by reloading the document from its file on disk.
   *  No-op for an unsaved (pathless) document. */
  async revert() {
    if (!this.path) return;
    await this.open(this.path);
  }

  async save() {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.save());
    if (res) this.#apply(res, false);
  }

  async saveAs(path: string) {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.saveAs(path));
    if (res) this.#apply(res, false);
  }

  async resampleGamma(step: number) {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.resampleGamma(step));
    if (res) this.#apply(res, true);
  }

  async scaleTo100() {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.scaleTo100Percent());
    if (res) this.#apply(res, true);
  }

  /** Toggles legacy strict validation and re-validates the open document. */
  async setStrictValidation(enabled: boolean) {
    if (!this.doc) {
      this.strictValidation = enabled;
      return;
    }
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.setStrictValidation(enabled));
    if (res) this.#apply(res, false);
  }

  /** Marks the document dirty and schedules a debounced validate/recompute. */
  edited() {
    this.dirty = true;
    if (this.#commitTimer) clearTimeout(this.#commitTimer);
    this.#commitTimer = setTimeout(() => this.commit(), 250);
  }

  /** Pushes the current doc to Rust for validation + photometry.
   *  Resolves false when Rust rejected the update (see `error`). */
  async commit(): Promise<boolean> {
    this.#cancelPendingCommit();
    if (!this.doc) return true;
    const snapshot = $state.snapshot(this.doc) as EulumdatDoc;
    const res = await this.#run(() => api.updateDocument(snapshot));
    if (!res) return false;
    this.#apply(res, false);
    return true;
  }

  /** Commits a pending debounced edit immediately so Rust holds the latest doc.
   *  Operations that read or replace the backend model must stop when this
   *  resolves false, or they would act on the stale model. */
  async flushEdits(): Promise<boolean> {
    if (!this.#commitTimer) return true;
    this.#cancelPendingCommit();
    return this.commit();
  }

  #cancelPendingCommit() {
    if (this.#commitTimer) clearTimeout(this.#commitTimer);
    this.#commitTimer = null;
  }
}

export const store = new DocStore();
