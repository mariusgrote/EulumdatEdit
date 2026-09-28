import { ask } from '@tauri-apps/plugin-dialog';
import { DocumentTransitions } from './documentTransitions.svelte';
import type { UgrFluxBasis } from './types';
import { warningsByField } from './warningNavigation';

class DocStore extends DocumentTransitions {
  /** Flux the UGR table and data sheet value refer to; kept across documents. */
  ugrFluxBasis = $state<UgrFluxBasis>('lampFlux');

  /** Warning messages keyed by form field key, for inline display beside inputs. */
  fieldWarnings = $derived.by<Record<string, string[]>>(() => warningsByField(this.warnings));

  /** Field key transiently highlighted after navigating to a warning. */
  highlightedFieldKey = $state<string | null>(null);
  #highlightTimer: ReturnType<typeof setTimeout> | null = null;

  highlightField(fieldKey: string, durationMs = 2000) {
    this.highlightedFieldKey = fieldKey;
    if (this.#highlightTimer) clearTimeout(this.#highlightTimer);
    this.#highlightTimer = setTimeout(() => {
      this.highlightedFieldKey = null;
      this.#highlightTimer = null;
    }, durationMs);
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
}

export const store = new DocStore();
