<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { renderPolarSvg } from '$lib/api';
  import type { PolarOptions } from '$lib/types';

  const ALL_PLANES = [
    { id: 'c0c180', label: 'C0/C180' },
    { id: 'c90c270', label: 'C90/C270' },
    { id: 'c45c225', label: 'C45/C225' },
    { id: 'c135c315', label: 'C135/C315' }
  ];

  let planes = $state<Record<string, boolean>>({
    c0c180: true,
    c90c270: true,
    c45c225: false,
    c135c315: false
  });
  let intensityMode = $state<'stored' | 'converted'>('stored');
  let svg = $state('');
  let renderError = $state<string | null>(null);
  let timer: ReturnType<typeof setTimeout> | null = null;

  function fmt(v: number | null | undefined, digits = 1): string {
    if (v === null || v === undefined || !Number.isFinite(v)) return '—';
    return v.toFixed(digits);
  }

  async function render() {
    if (!store.doc) {
      svg = '';
      return;
    }
    const opts: PolarOptions = {
      width: 460,
      height: 460,
      planes: ALL_PLANES.filter((p) => planes[p.id]).map((p) => p.id),
      showGrid: true,
      showLegend: true,
      showAxisLabels: true,
      intensityMode,
      title: null
    };
    try {
      svg = await renderPolarSvg(opts);
      renderError = null;
    } catch (e) {
      renderError = String(e);
    }
  }

  // Re-render after the Rust model updates (photometry is a proxy for a
  // committed change) or when the diagram options change.
  $effect(() => {
    // track dependencies
    void store.photometry;
    void planes.c0c180;
    void planes.c90c270;
    void planes.c45c225;
    void planes.c135c315;
    void intensityMode;
    if (timer) clearTimeout(timer);
    timer = setTimeout(render, 120);
  });

  const p = $derived(store.photometry);
</script>

<aside class="panel">
  <div class="diagram-wrap">
    {#if renderError}
      <div class="err">{renderError}</div>
    {:else if svg}
      <!-- eslint-disable-next-line svelte/no-at-html-tags -->
      {@html svg}
    {:else}
      <div class="placeholder">No diagram</div>
    {/if}
  </div>

  <div class="controls">
    <div class="planes">
      {#each ALL_PLANES as plane}
        <label class="chk">
          <input type="checkbox" bind:checked={planes[plane.id]} />
          {plane.label}
        </label>
      {/each}
    </div>
    <select bind:value={intensityMode} aria-label="Intensity mode">
      <option value="stored">Stored (cd/klm)</option>
      <option value="converted">Converted (× factor)</option>
    </select>
  </div>

  <div class="stats">
    <div class="stat">
      <span class="k">Total output</span>
      <span class="v">{fmt(p?.totalOutput, 0)}<small> lm</small></span>
    </div>
    <div class="stat">
      <span class="k">Calc. DFF</span>
      <span class="v">{fmt(p?.calculatedDownwardFluxFraction)}<small> %</small></span>
    </div>
    <div class="stat">
      <span class="k">Beam C0/C180</span>
      <span class="v">{fmt(p?.beamAngleC0C180)}<small> °</small></span>
    </div>
    <div class="stat">
      <span class="k">Beam C90/C270</span>
      <span class="v">{fmt(p?.beamAngleC90C270)}<small> °</small></span>
    </div>
    <div class="stat">
      <span class="k">Field C0/C180</span>
      <span class="v">{fmt(p?.fieldAngleC0C180)}<small> °</small></span>
    </div>
    <div class="stat">
      <span class="k">Field C90/C270</span>
      <span class="v">{fmt(p?.fieldAngleC90C270)}<small> °</small></span>
    </div>
  </div>
</aside>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 18px;
    overflow-y: auto;
  }
  .diagram-wrap {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    padding: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: var(--shadow);
    min-height: 300px;
  }
  .diagram-wrap :global(svg) {
    max-width: 100%;
    height: auto;
  }
  .placeholder,
  .err {
    color: var(--text-faint);
    font-size: 13px;
    padding: 40px;
    text-align: center;
  }
  .err {
    color: var(--danger);
  }
  .controls {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .planes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--text-dim);
  }
  .chk input {
    width: auto;
  }
  .stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .stat {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .stat .k {
    font-size: 11px;
    color: var(--text-faint);
    text-transform: uppercase;
    letter-spacing: 0.03em;
  }
  .stat .v {
    font-size: 18px;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }
  .stat .v small {
    font-size: 12px;
    font-weight: 400;
    color: var(--text-faint);
  }
</style>
