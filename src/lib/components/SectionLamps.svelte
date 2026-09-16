<script lang="ts">
  import { store } from '$lib/store.svelte';
  import type { LampSet } from '$lib/types';
  import NumberField from './NumberField.svelte';
  import TextField from './TextField.svelte';

  const doc = $derived(store.doc!);
  const edit = () => store.edited();

  function addLamp() {
    const fresh: LampSet = {
      lampCount: 1,
      lampType: 'LED',
      totalLuminousFlux: 1000,
      colorTemperature: '4000K',
      colorRenderingIndex: '80',
      wattageIncludingBallast: 10
    };
    doc.lamps = [...doc.lamps, fresh];
    edit();
  }

  function removeLamp(i: number) {
    doc.lamps = doc.lamps.filter((_, idx) => idx !== i);
    edit();
  }
</script>

<div class="card">
  <h3>Lamp sets</h3>
  {#if doc.lamps.length === 0}
    <p class="empty">No lamp sets defined.</p>
  {/if}
  <div class="lamps">
    {#each doc.lamps as lamp, i (i)}
      <div class="lamp">
        <div class="lamp-head">
          <span class="badge">Set {i + 1}</span>
          <button class="btn ghost danger" onclick={() => removeLamp(i)} title="Remove set">
            Remove
          </button>
        </div>
        <div class="grid-2">
          <NumberField fieldKey={`lamps.${i}.lampCount`} label="Lamp count" min={0} integer hardMin={0} bind:value={lamp.lampCount} onedit={edit} />
          <TextField fieldKey={`lamps.${i}.lampType`} label="Lamp type" bind:value={lamp.lampType} onedit={edit} />
          <NumberField fieldKey={`lamps.${i}.totalLuminousFlux`} label="Total luminous flux" unit="lm" bind:value={lamp.totalLuminousFlux} onedit={edit} />
          <NumberField fieldKey={`lamps.${i}.wattageIncludingBallast`} label="Wattage incl. ballast" unit="W" step={0.1} bind:value={lamp.wattageIncludingBallast} onedit={edit} />
          <TextField fieldKey={`lamps.${i}.colorTemperature`} label="Color temperature" bind:value={lamp.colorTemperature} onedit={edit} />
          <TextField fieldKey={`lamps.${i}.colorRenderingIndex`} label="Color rendering index" bind:value={lamp.colorRenderingIndex} onedit={edit} />
        </div>
      </div>
    {/each}
  </div>
  <button class="btn" style="margin-top:14px" onclick={addLamp}>+ Add lamp set</button>
</div>

<style>
  .lamps {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .lamp {
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 14px;
    background: var(--bg-sunken);
  }
  .lamp-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
  }
  .badge {
    font-size: 12px;
    font-weight: 600;
    color: var(--accent-strong);
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .danger {
    color: var(--danger);
  }
  .empty {
    color: var(--text-faint);
    margin: 0 0 12px;
  }
</style>
