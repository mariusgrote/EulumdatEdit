// Central document store. The Rust backend is the source of truth; this store
// holds the editable copy plus derived warnings/photometry returned by Rust.

import * as api from './api';
import type {
  DocResponse,
  EulumdatDoc,
  Photometry,
  TabSummary,
  Ugr,
  Warning,
  WindowStateResponse
} from './types';

export class DocumentTransitions {
  doc = $state<EulumdatDoc | null>(null);
  warnings = $state<Warning[]>([]);
  photometry = $state<Photometry | null>(null);
  ugr = $state<Ugr | null>(null);
  path = $state<string | null>(null);
  dirty = $state(false);
  busy = $state(false);
  error = $state<string | null>(null);
  /** Legacy EULUMDAT text-length limits (8.3 filename, etc.). Off by default. */
  strictValidation = $state(false);
  tabs = $state<TabSummary[]>([]);
  activeTabId = $state<string | null>(null);

  #commitTimer: ReturnType<typeof setTimeout> | null = null;
  #commitInFlight: Promise<boolean> | null = null;
  /** Serializes saves with operations that change the active backend document. */
  #saveInFlight: Promise<void> | null = null;
  #openInFlight: Promise<void> | null = null;
  #writeInFlight: Promise<DocResponse | null> | null = null;
  #editRevision = 0;
  /** A failed update must be retried before any operation uses the backend model. */
  #pendingEdit = false;

  /** Applies a backend response. `replaceDoc` is false during live edits so the
   *  user's in-progress input isn't clobbered by the round-tripped copy. */
  #apply(res: DocResponse, replaceDoc: boolean) {
    if (replaceDoc) {
      this.doc = res.doc;
      this.#pendingEdit = false;
    }
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
    this.#pendingEdit = false;
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

  async newDoc() {
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.newFromTemplate());
    if (res) this.#applyWindow(res);
  }

  async open(path: string) {
    await this.openMany([path]);
  }

  async openMany(paths: string[]) {
    if (paths.length === 0) return;
    const previous = this.#openInFlight;
    const pending = (async () => {
      if (previous) await previous;
      await this.#saveInFlight;
      if (!(await this.flushEdits())) return;
      const failures: string[] = [];
      this.busy = true;
      try {
        for (const path of paths) {
          try {
            this.#applyWindow(await api.openFile(path));
          } catch (e) {
            failures.push(paths.length === 1 ? String(e) : `${path}: ${String(e)}`);
          }
        }
      } finally {
        this.busy = false;
      }
      if (failures.length > 0) this.error = failures.join('\n');
    })();
    this.#openInFlight = pending;
    try {
      await pending;
    } finally {
      if (this.#openInFlight === pending) this.#openInFlight = null;
    }
  }

  /** Shows the document the backend prepared for this window, if any. */
  async loadCurrent() {
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.currentDocument());
    if (res) this.#applyWindow(res);
  }

  async activateTab(tabId: string) {
    if (tabId === this.activeTabId) return;
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.activateTab(tabId));
    if (res) this.#applyWindow(res);
  }

  /** Closes one tab and selects the backend-chosen neighbour. */
  async close(tabId = this.activeTabId) {
    if (!tabId) return;
    await this.#saveInFlight;
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
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return false;
    const res = await this.#run(() => api.moveTab(tabId, targetWindow, targetIndex));
    if (res) this.#applyWindow(res);
    return res !== null;
  }

  async detachTab(tabId: string, x: number, y: number) {
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return false;
    const res = await this.#run(() => api.detachTab(tabId, x, y));
    if (res) this.#applyWindow(res);
    return res !== null;
  }

  /** Discards in-memory edits by reloading the document from its file on disk.
   *  No-op for an unsaved (pathless) document. */
  async revert() {
    if (!this.path) return;
    await this.#saveInFlight;
    this.#cancelPendingCommit();
    await this.#commitInFlight;
    const res = await this.#run(() => api.reloadDocument());
    if (res) this.#apply(res, true);
  }

  async save() {
    await this.#saveDocument(() => api.save());
  }

  async saveAs(path: string) {
    await this.#saveDocument(() => api.saveAs(path));
  }

  async #saveDocument(write: () => Promise<DocResponse>) {
    const previous = this.#saveInFlight;
    const pending = (async () => {
      if (previous) await previous;
      if (!(await this.flushEdits())) return;
      const revision = this.#editRevision;
      const tabId = this.activeTabId;
      const writePending = this.#run(write);
      this.#writeInFlight = writePending;
      let res: DocResponse | null;
      try {
        res = await writePending;
      } finally {
        if (this.#writeInFlight === writePending) this.#writeInFlight = null;
      }
      if (res && this.activeTabId === tabId) this.#applySave(res, revision);
    })();
    this.#saveInFlight = pending;
    try {
      await pending;
    } finally {
      if (this.#saveInFlight === pending) this.#saveInFlight = null;
    }
  }

  /** A save response describes the document at the time the save started.
   *  Keep edits made during the write, while still showing a Save As path. */
  #applySave(res: DocResponse, revision: number) {
    if (this.#editRevision === revision) {
      this.#apply(res, true);
      return;
    }
    this.path = res.path;
    this.dirty = true;
    const active = this.tabs.find((tab) => tab.id === this.activeTabId);
    if (active) {
      active.path = res.path;
      active.title = res.path?.split(/[\\\\/]/).pop() || 'Untitled';
      active.dirty = true;
    }
  }

  async exportIes(path: string) {
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return;
    await this.#run(() => api.exportIes(path));
  }

  async resampleGamma(step: number) {
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.resampleGamma(step));
    if (res) this.#apply(res, true);
  }

  async scaleTo100() {
    await this.#saveInFlight;
    if (!(await this.flushEdits())) return;
    const res = await this.#run(() => api.scaleTo100Percent());
    if (res) this.#apply(res, true);
  }

  /** Toggles legacy strict validation and re-validates the open document. */
  async setStrictValidation(enabled: boolean) {
    await this.#saveInFlight;
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
    this.#pendingEdit = true;
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
    const write = this.#writeInFlight;
    const pending = (async () => {
      // Preserve backend order, including edits made during a save.
      if (previous) await previous;
      if (write) await write;
      const res = await this.#run(() => api.updateDocument(snapshot, tabId));
      if (!res) return false;
      if (this.activeTabId === tabId && this.#editRevision === revision) {
        this.#pendingEdit = false;
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
    while (this.#commitTimer || this.#commitInFlight || this.#pendingEdit) {
      const pending = this.#commitTimer || !this.#commitInFlight
        ? this.commit()
        : this.#commitInFlight;
      if (pending && !(await pending)) return false;
    }
    return true;
  }

  #cancelPendingCommit() {
    if (this.#commitTimer) clearTimeout(this.#commitTimer);
    this.#commitTimer = null;
  }
}
