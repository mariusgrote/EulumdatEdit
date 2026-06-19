<script lang="ts">
  import { store } from '$lib/store.svelte';
  import NumberField from './NumberField.svelte';

  const doc = $derived(store.doc!);
  const edit = () => store.edited();
</script>

<div class="card">
  <h3>Luminaire dimensions</h3>
  <div class="grid-3">
    <NumberField label="Length / diameter" unit="mm" bind:value={doc.luminaireLength} onedit={edit} />
    <NumberField label="Width" unit="mm" bind:value={doc.luminaireWidth} onedit={edit} />
    <NumberField label="Height" unit="mm" bind:value={doc.luminaireHeight} onedit={edit} />
  </div>
</div>

<div class="card">
  <h3>Luminous area</h3>
  <div class="grid-2">
    <NumberField label="Length / diameter" unit="mm" bind:value={doc.luminousAreaLength} onedit={edit} />
    <NumberField label="Width" unit="mm" bind:value={doc.luminousAreaWidth} onedit={edit} />
  </div>
  <div class="grid-2" style="margin-top:14px">
    <NumberField label="Height C0" unit="mm" bind:value={doc.luminousAreaHeightC0} onedit={edit} />
    <NumberField label="Height C90" unit="mm" bind:value={doc.luminousAreaHeightC90} onedit={edit} />
    <NumberField label="Height C180" unit="mm" bind:value={doc.luminousAreaHeightC180} onedit={edit} />
    <NumberField label="Height C270" unit="mm" bind:value={doc.luminousAreaHeightC270} onedit={edit} />
  </div>
</div>

<div class="card">
  <h3>Photometric metadata</h3>
  <div class="grid-3">
    <NumberField label="Downward flux fraction" unit="%" step={0.1} bind:value={doc.downwardFluxFraction} onedit={edit} />
    <NumberField label="Light output ratio" unit="%" step={0.1} bind:value={doc.lightOutputRatio} onedit={edit} />
    <NumberField label="Conversion factor" step={0.001} bind:value={doc.conversionFactor} onedit={edit} />
    <NumberField label="Tilt" unit="°" step={0.1} bind:value={doc.tilt} onedit={edit} />
  </div>
</div>
