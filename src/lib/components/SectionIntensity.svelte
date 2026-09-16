<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { isValidResampleStep } from '$lib/numberInput';

  const doc = $derived(store.doc!);
  // intensities[cPlaneRow][gammaIndex]
  const rows = $derived(doc.intensities.length);
  const cols = $derived(doc.gammaAngles.length);

  // Svelte's numeric binding yields null while the input is empty.
  let resampleStep = $state<number | null>(1);
  const validResampleStep = $derived(isValidResampleStep(resampleStep));
  const resampleHintId = $props.id();

  function planeLabel(i: number): string {
    const angle = doc.cPlanes[i];
    return angle === undefined ? `#${i + 1}` : `C${angle}°`;
  }

  function onCell(c: number, g: number, e: Event) {
    const v = Number((e.target as HTMLInputElement).value);
    doc.intensities[c][g] = Number.isFinite(v) ? v : 0;
    store.edited();
  }

  async function doScaleTo100() {
    const ok = await ask(
      'This rescales every intensity value so the peak reaches 100%. The original values cannot be recovered. Continue?',
      { title: 'Scale to 100%', kind: 'warning' }
    );
    if (ok) await store.scaleTo100();
  }

  async function doResample() {
    const step = resampleStep;
    if (!isValidResampleStep(step)) return;
    const ok = await ask(
      `This rebuilds the gamma table at a ${step}° step by interpolation. ` +
        'The current gamma angles and their values will be replaced. Continue?',
      { title: 'Resample γ', kind: 'warning' }
    );
    if (ok) await store.resampleGamma(step);
  }
</script>

<div class="card">
  <div class="bar">
    <h3 style="margin:0">Luminous intensity table</h3>
    <div class="tools">
      <span class="dims">{rows} × {cols} (cd/klm)</span>
      <button class="btn ghost" onclick={doScaleTo100}>Scale to 100%</button>
      <div class="resample">
        <input
          type="number"
          min="1"
          max="90"
          bind:value={resampleStep}
          aria-label="Gamma step"
          aria-invalid={!validResampleStep}
          aria-describedby={validResampleStep ? undefined : resampleHintId}
        />
        <button class="btn" onclick={doResample} disabled={!validResampleStep}>
          Resample γ
        </button>
      </div>
    </div>
  </div>
  {#if !validResampleStep}
    <p class="resample-hint" id={resampleHintId}>Enter a whole number from 1 to 90.</p>
  {/if}

  <div class="scroll">
    <table>
      <thead>
        <tr>
          <th class="corner">γ \ C</th>
          {#each doc.intensities as _, c}
            <th>{planeLabel(c)}</th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each doc.gammaAngles as gamma, g}
          <tr>
            <th class="rowhead">{gamma}°</th>
            {#each doc.intensities as row, c}
              <td>
                <input
                  type="number"
                  step="1"
                  value={row[g]}
                  oninput={(e) => onCell(c, g, e)}
                />
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 14px;
    flex-wrap: wrap;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .dims {
    font-size: 12px;
    color: var(--text-faint);
    font-variant-numeric: tabular-nums;
  }
  .resample {
    display: flex;
    gap: 6px;
  }
  .resample input {
    width: 64px;
  }
  .resample-hint {
    margin: -6px 0 12px;
    font-size: 12px;
    color: var(--danger);
    text-align: right;
  }
  .scroll {
    overflow: auto;
    max-height: calc(100vh - 220px);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  table {
    border-collapse: separate;
    border-spacing: 0;
    font-variant-numeric: tabular-nums;
  }
  th,
  td {
    border-right: 1px solid var(--border);
    border-bottom: 1px solid var(--border);
    padding: 0;
  }
  thead th {
    position: sticky;
    top: 0;
    z-index: 2;
    background: var(--bg-sunken);
    padding: 6px 10px;
    font-size: 12px;
    color: var(--text-dim);
    white-space: nowrap;
  }
  .rowhead {
    position: sticky;
    left: 0;
    z-index: 1;
    background: var(--bg-sunken);
    padding: 4px 10px;
    font-size: 12px;
    color: var(--text-dim);
    text-align: right;
    white-space: nowrap;
  }
  .corner {
    left: 0;
    z-index: 3;
  }
  td input {
    width: 78px;
    border: none;
    border-radius: 0;
    background: transparent;
    padding: 5px 8px;
    text-align: right;
  }
  td input:focus {
    box-shadow: inset 0 0 0 2px var(--accent);
    background: var(--field-bg);
  }
</style>
