<script lang="ts">
  import { store } from '$lib/store.svelte';
  import type { UgrBlocker } from '$lib/types';
  import {
    DATA_SHEET_REFLECTANCE,
    DATA_SHEET_ROOM,
    fluxBasisLabel,
    formatReflectance,
    formatUgr,
    resolveUgrBlockerTarget,
    ugrTableTsv,
    ugrValues
  } from '$lib/ugr';
  import type { WarningTarget } from '$lib/warningNavigation';

  interface Props {
    onnavigate: (target: WarningTarget) => void;
  }
  let { onnavigate }: Props = $props();

  const ugr = $derived(store.ugr);
  const table = $derived(ugr?.status === 'available' ? ugr : null);
  const values = $derived(table ? ugrValues(table, store.ugrFluxBasis) : null);

  const views = ['crosswise', 'endwise'] as const;
  const reflectanceRows = [
    { label: 'Ceiling %', index: 0 },
    { label: 'Walls %', index: 1 },
    { label: 'Floor %', index: 2 }
  ] as const;

  let copied = $state(false);
  let copyError = $state<string | null>(null);
  let copiedTimer: ReturnType<typeof setTimeout> | null = null;

  async function copy() {
    if (!table) return;
    copyError = null;
    try {
      await navigator.clipboard.writeText(ugrTableTsv(table, store.ugrFluxBasis));
      copied = true;
      if (copiedTimer) clearTimeout(copiedTimer);
      copiedTimer = setTimeout(() => (copied = false), 1500);
    } catch (e) {
      copyError = String(e);
    }
  }

  function blockerSection(blocker: UgrBlocker): string {
    const { section } = resolveUgrBlockerTarget(blocker);
    return section.charAt(0).toUpperCase() + section.slice(1);
  }

  const isDataSheetCell = (room: number, reflectance: number) =>
    room === DATA_SHEET_ROOM && reflectance === DATA_SHEET_REFLECTANCE;
</script>

{#if table && values}
  <div class="card">
    <div class="bar">
      <h3 style="margin:0">UGR table</h3>
      <div class="tools">
        <div class="segmented" role="radiogroup" aria-label="Luminous flux">
          <button
            role="radio"
            aria-checked={store.ugrFluxBasis === 'lampFlux'}
            class:active={store.ugrFluxBasis === 'lampFlux'}
            onclick={() => (store.ugrFluxBasis = 'lampFlux')}
            title="Values for the lamp flux of the file"
          >
            {fluxBasisLabel(table, 'lampFlux')}
          </button>
          <button
            role="radio"
            aria-checked={store.ugrFluxBasis === 'normalized'}
            class:active={store.ugrFluxBasis === 'normalized'}
            onclick={() => (store.ugrFluxBasis = 'normalized')}
            title="Values for 1000 lm, as in CIE 190 catalogue tables"
          >
            1000 lm
          </button>
        </div>
        <button class="btn" onclick={copy} title="Copy as tab-separated text">
          {copied ? 'Copied' : 'Copy'}
        </button>
      </div>
    </div>

    <p class="caption">
      CIE 117 / CIE 190 tabular method, luminaire spacing 0.25 H.
      {#if store.ugrFluxBasis === 'normalized'}
        Correction to the file flux: {table.fluxCorrection >= 0 ? '+' : ''}{table.fluxCorrection.toFixed(1)}.
      {/if}
    </p>
    {#if copyError}<p class="err">{copyError}</p>{/if}

    <div class="scroll">
      <table>
        <thead>
          <tr>
            <th class="corner" colspan="2"></th>
            <th class="view" colspan="5">Viewed crosswise</th>
            <th class="view" colspan="5">Viewed endwise</th>
          </tr>
          {#each reflectanceRows as r}
            <tr>
              <th class="corner rlabel" colspan="2">{r.label}</th>
              {#each views as view}
                {#each table.reflectances as refl, i}
                  <th class:group-start={i === 0 && view === 'endwise'}>
                    {formatReflectance(refl[r.index])}
                  </th>
                {/each}
              {/each}
            </tr>
          {/each}
          <tr>
            <th class="dim">X</th>
            <th class="dim">Y</th>
            <th class="spacer" colspan="10"></th>
          </tr>
        </thead>
        <tbody>
          {#each values.rows as row, r}
            <tr class:group-top={r > 0 && row.xH !== values.rows[r - 1].xH}>
              <th class="dim">{row.xH}H</th>
              <th class="dim">{row.yH}H</th>
              {#each views as view}
                {#each row[view] as value, i}
                  <td
                    class:group-start={i === 0 && view === 'endwise'}
                    class:data-sheet={isDataSheetCell(r, i)}
                    title={isDataSheetCell(r, i) ? 'Data sheet value (4H × 8H, 70/50/20)' : undefined}
                  >
                    {formatUgr(value)}
                  </td>
                {/each}
              {/each}
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </div>
{:else if ugr?.status === 'blocked'}
  <div class="card">
    <h3>UGR table</h3>
    <p class="blocked-intro">
      The CIE tabular method does not apply to this luminaire, so no UGR values are shown.
    </p>
    <ul class="blockers">
      {#each ugr.blockers as blocker}
        <li>
          <button type="button" onclick={() => onnavigate(resolveUgrBlockerTarget(blocker))}>
            <span class="bsection">{blockerSection(blocker)}</span>
            <span class="bmsg">{blocker.message}</span>
          </button>
        </li>
      {/each}
    </ul>
  </div>
{/if}

<style>
  .bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    margin-bottom: 8px;
    flex-wrap: wrap;
  }
  .tools {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .segmented {
    display: flex;
    gap: 2px;
    background: var(--bg-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px;
  }
  .segmented button {
    border: none;
    background: transparent;
    color: var(--text-dim);
    padding: 4px 12px;
    border-radius: calc(var(--radius-sm) - 2px);
    font-size: 13px;
    font-weight: 500;
    font-variant-numeric: tabular-nums;
  }
  .segmented button:hover {
    color: var(--text);
  }
  .segmented button.active {
    background: var(--sel);
    color: var(--text);
  }
  .caption {
    margin: 0 0 14px;
    font-size: 12px;
    color: var(--text-faint);
  }
  .err {
    margin: -6px 0 12px;
    font-size: 12px;
    color: var(--danger);
  }
  .scroll {
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
  }
  table {
    width: 100%;
    border-collapse: separate;
    border-spacing: 0;
    font-variant-numeric: tabular-nums;
    font-size: 13px;
  }
  th,
  td {
    border-bottom: 1px solid var(--border);
    padding: 5px 6px;
    text-align: right;
    white-space: nowrap;
  }
  thead th {
    background: var(--bg-sunken);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-dim);
  }
  thead th.view {
    text-align: center;
    color: var(--text);
  }
  .corner {
    text-align: left;
  }
  .rlabel {
    color: var(--text-faint);
  }
  .dim {
    padding-left: 4px;
    background: var(--bg-sunken);
    font-size: 12px;
    font-weight: 500;
    color: var(--text-dim);
  }
  .group-start {
    border-left: 1px solid var(--border-strong);
  }
  thead th.view + th.view {
    border-left: 1px solid var(--border-strong);
  }
  tbody tr:last-child > * {
    border-bottom: none;
  }
  tr.group-top > * {
    border-top: 1px solid var(--border-strong);
  }
  td.data-sheet {
    background: var(--accent-soft);
    box-shadow: inset 0 0 0 1px var(--accent);
    font-weight: 600;
  }
  .blocked-intro {
    margin: 0 0 12px;
    color: var(--text-dim);
    font-size: 13px;
  }
  .blockers {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .blockers li {
    border: 1px solid var(--border);
    border-left: 3px solid var(--text-faint);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .blockers button {
    width: 100%;
    border: none;
    background: transparent;
    padding: 9px 12px;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
    color: inherit;
    font: inherit;
  }
  .blockers button:hover {
    background: var(--sel);
  }
  .bsection {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--text-faint);
    font-weight: 600;
  }
  .bmsg {
    font-size: 13px;
    line-height: 1.4;
  }
</style>
