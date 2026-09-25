// Central document store. The Rust backend is the source of truth; this store
// holds the editable copy plus derived warnings/photometry returned by Rust.

import { ask } from '@tauri-apps/plugin-dialog';
import * as api from './api';
import type {
  DocResponse,
  EulumdatDoc,
  Photometry,
  TabSummary,
  Ugr,
  UgrFluxBasis,
  Warning,
  WindowStateResponse
} from './types';
import { warningsByField } from './warningNavigation';

class DocStore {
  doc = $state<EulumdatDoc | null>(null);
  warnings = $state<Warning[]>([]);
  photometry = $state<Photometry | null>(null);
  ugr = $state<Ugr | null>(null);
  /** Flux the UGR table and data sheet value refer to; kept across documents. */
  ugrFluxBasis = $state<UgrFluxBasis>('lampFlux');
  path = $state<string | null>(null);
  dirty = $state(false);
  busy = $state(false);
  error = $state<string | null>(null);
  /** Legacy EULUMDAT text-length limits (8.3 filename, etc.). Off by default. */
  strictValidation = $state(false);
  tabs = $state<TabSummary[]>([]);
  activeTabId = $state<string | null>(null);

  /** Warning messages keyed by form field key, for inline display beside inputs. */
  fieldWarnings = $derived.by<Record<string, string[]>>(() => warningsByField(this.warnings));

  /** Field key transiently highlighted after navigating to a warning. */
  highlightedFieldKey = $state<string | null>(null);

  #commitTimer: ReturnType<typeof setTimeout> | null = null;
  #commitInFlight: Promise<boolean> | null = null;
  #editRevision = 0;
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
    this.ugr = res.ugr;
    this.path = res.path;
    this.dirty = res.dirty;
    this.strictValidation = res.strictValidation;
    this.error = null;
    const active = this.tabs.find((tab) => tab.id === this.activeTabId);
    if (active) {
      active.path = res.path;
      active.title = res.path?.split(/[\\/]/).pop() || 'Untitled';
      active.dirty = res.dirty;
    }
  }

  #clearDocument() {
    this.doc = null;
    this.warnings = [];
    this.photometry = null;
    this.ugr = null;
    this.path = null;
    this.dirty = false;
    this.error = null;
    this.strictValidation = false;
  }

  #applyWindow(res: WindowStateResponse) {
    this.tabs = res.tabs;
    this.activeTabId = res.activeTabId;
    if (res.document) this.#apply(res.document, true);
    else this.#clearDocument();
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

  async confirmCloseWindow(): Promise<boolean> {
    const dirtyTabs = this.tabs.filter((tab) => tab.dirty);
    if (dirtyTabs.length === 0) return true;
    const names = dirtyTabs.map((tab) => tab.title).join(', ');
    return ask(`Unsaved changes in ${names} will be lost. Close this window anyway?`, {
      title: 'Unsaved changes',
      kind: 'warning'
    });
  }

  async newDoc() {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.newFromTemplate());
    if (res) this.#applyWindow(res);
  }

  async open(path: string) {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.openFile(path));
    if (res) this.#applyWindow(res);
  }

  /** Shows the document the backend prepared for this window, if any. */
  async loadCurrent() {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.currentDocument());
    if (res) this.#applyWindow(res);
  }

  async activateTab(tabId: string) {
    if (tabId === this.activeTabId) return;
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.activateTab(tabId));
    if (res) this.#applyWindow(res);
  }

  /** Closes one tab and selects the backend-chosen neighbour. */
  async close(tabId = this.activeTabId) {
    if (!tabId) return;
    if (tabId === this.activeTabId) {
      this.#cancelPendingCommit();
      await this.#commitInFlight;
    }
    else if (!(await this.flushEdits())) return;
    const res = await this.#run(async () => {
      return api.closeDocument(tabId);
    });
    if (!res) return;
    this.#applyWindow(res);
  }

  async moveTab(tabId: string, targetWindow: string, targetIndex?: number) {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.moveTab(tabId, targetWindow, targetIndex));
    if (res) this.#applyWindow(res);
  }

  async detachTab(tabId: string, x: number, y: number) {
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.detachTab(tabId, x, y));
    if (res) this.#applyWindow(res);
  }

  /** Discards in-memory edits by reloading the document from its file on disk.
   *  No-op for an unsaved (pathless) document. */
  async revert() {
    if (!this.path) return;
    this.#cancelPendingCommit();
    await this.#commitInFlight;
    const res = await this.#run(() => api.reloadDocument());
    if (res) this.#apply(res, true);
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
    this.#editRevision++;
    this.dirty = true;
    const active = this.tabs.find((tab) => tab.id === this.activeTabId);
    if (active) active.dirty = true;
    if (this.#commitTimer) clearTimeout(this.#commitTimer);
    this.#commitTimer = setTimeout(() => this.commit(), 250);
  }

  /** Pushes the current doc to Rust for validation + photometry.
   *  Resolves false when Rust rejected the update (see `error`). */
  async commit(): Promise<boolean> {
    this.#cancelPendingCommit();
    if (!this.doc || !this.activeTabId) return true;
    const snapshot = $state.snapshot(this.doc) as EulumdatDoc;
    const tabId = this.activeTabId;
    const revision = this.#editRevision;
    const previous = this.#commitInFlight;
    const pending = (async () => {
      // Preserve backend order when another edit arrives during an IPC update.
      if (previous) await previous;
      const res = await this.#run(() => api.updateDocument(snapshot, tabId));
      if (!res) return false;
      if (this.activeTabId === tabId && this.#editRevision === revision) {
        this.#apply(res, false);
      }
      return true;
    })();
    this.#commitInFlight = pending;
    try {
      return await pending;
    } finally {
      if (this.#commitInFlight === pending) this.#commitInFlight = null;
    }
  }

  /** Commits a pending debounced edit immediately so Rust holds the latest doc.
   *  Operations that read or replace the backend model must stop when this
   *  resolves false, or they would act on the stale model. */
  async flushEdits(): Promise<boolean> {
    while (this.#commitTimer || this.#commitInFlight) {
      const pending = this.#commitTimer ? this.commit() : this.#commitInFlight;
      if (pending && !(await pending)) return false;
    }
    return true;
  }

  #cancelPendingCommit() {
    if (this.#commitTimer) clearTimeout(this.#commitTimer);
    this.#commitTimer = null;
  }
}

export const store = new DocStore();
