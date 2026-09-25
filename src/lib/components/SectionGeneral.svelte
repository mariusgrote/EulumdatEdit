<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { TYPE_INDICATOR_LABELS, SYMMETRY_LABELS } from '$lib/types';
  import TextField from './TextField.svelte';

  const doc = $derived(store.doc!);
  const openedFromIes = $derived(store.path?.toLowerCase().endsWith('.ies') ?? false);
  const edit = () => store.edited();
  const typeIndicatorLabel = $derived(
    TYPE_INDICATOR_LABELS[doc.typeIndicator] ?? `Unknown (${doc.typeIndicator})`
  );
  const symmetryLabel = $derived(SYMMETRY_LABELS[doc.symmetry] ?? `Unknown (${doc.symmetry})`);
</script>

<div class="card">
  <h3>Identification</h3>
  <div class="stack">
    <TextField fieldKey="identification" label="Identification" bind:value={doc.identification} onedit={edit} />
    <div class="grid-2">
      <TextField fieldKey="luminaireName" label="Luminaire name" bind:value={doc.luminaireName} onedit={edit} />
      <TextField fieldKey="luminaireNumber" label="Luminaire number" sourceNote={openedFromIes ? 'Not imported from IES' : undefined} bind:value={doc.luminaireNumber} onedit={edit} />
    </div>
    <div class="grid-2">
      <TextField
        fieldKey="measurementReportNumber"
        label="Measurement report number"
        sourceNote={openedFromIes ? 'Not imported from IES' : undefined}
        bind:value={doc.measurementReportNumber}
        onedit={edit}
      />
      <TextField fieldKey="fileName" label="File name" bind:value={doc.fileName} onedit={edit} />
    </div>
    <TextField fieldKey="dateUser" label="Date / user" sourceNote={openedFromIes ? 'Not imported from IES' : undefined} bind:value={doc.dateUser} onedit={edit} />
  </div>
</div>

<div class="card">
  <h3>Classification</h3>
  <div class="grid-2">
    <div class="field" data-field-key="typeIndicator">
      <span class="field-label">Type indicator</span>
      <span class="readonly-value">{typeIndicatorLabel}</span>
    </div>
    <div class="field" data-field-key="symmetry">
      <span class="field-label">Symmetry</span>
      <span class="readonly-value">{symmetryLabel}</span>
    </div>
  </div>
  <p class="hint">
    Classification follows the photometric distribution in this file. Changing it
    safely requires converting the C-plane data, which this version does not support.
  </p>
</div>

<style>
  .stack {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .field-label {
    font-size: 12px;
    color: var(--text-dim);
    font-weight: 500;
  }
  .readonly-value {
    font-size: 14px;
    color: var(--text);
    background: var(--field-bg);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 7px 9px;
  }
  .hint {
    margin: 14px 0 0;
    font-size: 12px;
    color: var(--text-faint);
    line-height: 1.5;
  }
</style>
