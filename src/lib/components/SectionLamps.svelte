<script lang="ts">
  import { ask } from '@tauri-apps/plugin-dialog';
  import {
    canAddLampSet,
    canRemoveLampSet,
    canUseAbsolutePhotometry,
    storedLampCount,
    MAX_LAMP_SETS
  } from '$lib/lampSetRules';
  import { store } from '$lib/store.svelte';
  import type { LampSet } from '$lib/types';
  import NumberField from './NumberField.svelte';
  import TextField from './TextField.svelte';

  const doc = $derived(store.doc!);
  const openedFromIes = $derived(store.path?.toLowerCase().endsWith('.ies') ?? false);
  const edit = () => store.edited();

  const absolute = $derived(doc.lamps.some((lamp) => lamp.lampCount < 0));
  const canAdd = $derived(canAddLampSet(doc.lamps.length, absolute));
  const canRemove = $derived(canRemoveLampSet(doc.lamps.length));
  const removeLabel = (i: number) =>
    canRemove ? `Remove set ${i + 1}` : 'An EULUMDAT file needs at least one lamp set';

  function addLamp() {
    if (!canAddLampSet(doc.lamps.length, absolute)) return;
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

  function changeMode(event: Event) {
    const nextAbsolute = (event.currentTarget as HTMLSelectElement).value === 'absolute';
    if (nextAbsolute && !canUseAbsolutePhotometry(doc.lamps.length)) return;
    for (const lamp of doc.lamps) {
      lamp.lampCount = storedLampCount(Math.max(1, Math.abs(lamp.lampCount)), nextAbsolute);
    }
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
  <div class="field mode-field">
    <label for="photometry-mode">Photometry mode</label>
    <select id="photometry-mode" value={absolute ? 'absolute' : 'relative'} onchange={changeMode}
      aria-describedby={!canUseAbsolutePhotometry(doc.lamps.length) ? 'absolute-set-rule' : undefined}>
      <option value="relative">Relative</option>
      <option value="absolute" disabled={!canUseAbsolutePhotometry(doc.lamps.length)}>Absolute</option>
    </select>
    {#if !canUseAbsolutePhotometry(doc.lamps.length)}
      <p id="absolute-set-rule" class="hint">Remove extra lamp sets to use absolute photometry.</p>
    {/if}
  </div>
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
          <NumberField
            fieldKey={`lamps.${i}.lampCount`}
            label={lamp.lampCount < 0 ? 'Emitter count' : 'Lamp count'}
            min={lamp.lampCount < 0 ? 1 : 0}
            max={1000}
            integer
            hardMin={lamp.lampCount < 0 ? 1 : 0}
            hardMax={lamp.lampCount < 0 ? 1000 : 2147483647}
            bind:value={() => Math.abs(lamp.lampCount), (value) => {
              lamp.lampCount = storedLampCount(value, lamp.lampCount < 0);
            }}
            onedit={edit}
          />
          <TextField fieldKey={`lamps.${i}.lampType`} label="Lamp type" sourceNote={openedFromIes ? 'Not imported from IES' : undefined} bind:value={lamp.lampType} onedit={edit} />
          <NumberField fieldKey={`lamps.${i}.totalLuminousFlux`} label={lamp.lampCount < 0 ? 'Luminaire luminous flux' : 'Total luminous flux'} unit="lm" bind:value={lamp.totalLuminousFlux} onedit={edit} />
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
        {#if absolute}Absolute photometry requires exactly one lamp set.{:else}An EULUMDAT file allows at most {MAX_LAMP_SETS} lamp sets.{/if}
      </span>
    {/if}
  </div>
</div>

<style>
  .mode-field {
    margin-bottom: 14px;
  }
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
