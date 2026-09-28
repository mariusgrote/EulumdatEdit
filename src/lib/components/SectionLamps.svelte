<script lang="ts">
  import { ask } from '@tauri-apps/plugin-dialog';
  import { canAddLampSet, canRemoveLampSet, MAX_LAMP_SETS } from '$lib/lampSetRules';
  import { store } from '$lib/store.svelte';
  import type { LampSet } from '$lib/types';
  import NumberField from './NumberField.svelte';
  import TextField from './TextField.svelte';

  const doc = $derived(store.doc!);
  const openedFromIes = $derived(store.path?.toLowerCase().endsWith('.ies') ?? false);
  const edit = () => store.edited();

  const canAdd = $derived(canAddLampSet(doc.lamps.length));
  const canRemove = $derived(canRemoveLampSet(doc.lamps.length));
  const removeLabel = (i: number) =>
    canRemove ? `Remove set ${i + 1}` : 'An EULUMDAT file needs at least one lamp set';

  function addLamp() {
    if (!canAddLampSet(doc.lamps.length)) return;
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

  async function removeLamp(i: number) {
    if (!canRemoveLampSet(doc.lamps.length) || i >= doc.lamps.length) return;
    const ok = await ask(`Set ${i + 1} and all of its values will be removed. Continue?`, {
      title: 'Remove lamp set',
      kind: 'warning'
    });
    // The document may have changed while the dialog was open.
    if (!ok || !canRemoveLampSet(doc.lamps.length) || i >= doc.lamps.length) return;
    doc.lamps = doc.lamps.filter((_, idx) => idx !== i);
    edit();
  }
</script>

<div class="card">
  <h3>Lamp sets</h3>
  <div class="lamps">
    {#each doc.lamps as lamp, i (i)}
      <div class="lamp">
        <div class="lamp-head">
          <span class="badge">Set {i + 1}</span>
          <button
            class="btn ghost danger"
            onclick={() => removeLamp(i)}
            disabled={!canRemove}
            title={removeLabel(i)}
            aria-label={removeLabel(i)}
          >
            Remove
          </button>
        </div>
        <div class="grid-2">
          <NumberField fieldKey={`lamps.${i}.lampCount`} label="Lamp count" min={0} integer hardMin={0} bind:value={lamp.lampCount} onedit={edit} />
          <TextField fieldKey={`lamps.${i}.lampType`} label="Lamp type" sourceNote={openedFromIes ? 'Not imported from IES' : undefined} bind:value={lamp.lampType} onedit={edit} />
          <NumberField fieldKey={`lamps.${i}.totalLuminousFlux`} label="Total luminous flux" unit="lm" bind:value={lamp.totalLuminousFlux} onedit={edit} />
          <NumberField fieldKey={`lamps.${i}.wattageIncludingBallast`} label="Wattage incl. ballast" unit="W" step={0.1} bind:value={lamp.wattageIncludingBallast} onedit={edit} />
          <TextField fieldKey={`lamps.${i}.colorTemperature`} label="Color temperature" sourceNote={openedFromIes ? 'Not imported from IES' : undefined} bind:value={lamp.colorTemperature} onedit={edit} />
          <TextField fieldKey={`lamps.${i}.colorRenderingIndex`} label="Color rendering index" sourceNote={openedFromIes ? 'Not imported from IES' : undefined} bind:value={lamp.colorRenderingIndex} onedit={edit} />
        </div>
      </div>
    {/each}
  </div>
  <div class="add-row">
    <button
      class="btn"
      onclick={addLamp}
      disabled={!canAdd}
      aria-describedby={canAdd ? undefined : 'lamp-set-limit'}
    >
      + Add lamp set
    </button>
    {#if !canAdd}
      <span id="lamp-set-limit" class="hint">
        An EULUMDAT file allows at most {MAX_LAMP_SETS} lamp sets.
      </span>
    {/if}
  </div>
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
  .add-row {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-top: 14px;
  }
  .hint {
    font-size: 12px;
    color: var(--text-faint);
  }
</style>
