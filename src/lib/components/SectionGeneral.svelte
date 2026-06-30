<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { TYPE_INDICATOR_LABELS, SYMMETRY_LABELS } from '$lib/types';
  import TextField from './TextField.svelte';

  const doc = $derived(store.doc!);
  const edit = () => store.edited();
</script>

<div class="card">
  <h3>Identification</h3>
  <div class="stack">
    <TextField fieldKey="identification" label="Identification" bind:value={doc.identification} onedit={edit} />
    <div class="grid-2">
      <TextField fieldKey="luminaireName" label="Luminaire name" bind:value={doc.luminaireName} onedit={edit} />
      <TextField fieldKey="luminaireNumber" label="Luminaire number" bind:value={doc.luminaireNumber} onedit={edit} />
    </div>
    <div class="grid-2">
      <TextField
        fieldKey="measurementReportNumber"
        label="Measurement report number"
        bind:value={doc.measurementReportNumber}
        onedit={edit}
      />
      <TextField fieldKey="fileName" label="File name" bind:value={doc.fileName} onedit={edit} />
    </div>
    <TextField fieldKey="dateUser" label="Date / user" bind:value={doc.dateUser} onedit={edit} />
  </div>
</div>

<div class="card">
  <h3>Classification</h3>
  <div class="grid-2">
    <div class="field" data-field-key="typeIndicator">
      <label for="type-indicator">Type indicator</label>
      <select id="type-indicator" bind:value={doc.typeIndicator} onchange={edit}>
        {#each Object.entries(TYPE_INDICATOR_LABELS) as [val, label]}
          <option value={Number(val)}>{label}</option>
        {/each}
      </select>
    </div>
    <div class="field" data-field-key="symmetry">
      <label for="symmetry">Symmetry</label>
      <select id="symmetry" bind:value={doc.symmetry} onchange={edit}>
        {#each Object.entries(SYMMETRY_LABELS) as [val, label]}
          <option value={Number(val)}>{label}</option>
        {/each}
      </select>
    </div>
  </div>
  <p class="hint">
    Changing symmetry may require the intensity table to be re-entered to match
    the new stored C-plane count.
  </p>
</div>

<style>
  .stack {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }
  .hint {
    margin: 14px 0 0;
    font-size: 12px;
    color: var(--text-faint);
    line-height: 1.5;
  }
</style>
