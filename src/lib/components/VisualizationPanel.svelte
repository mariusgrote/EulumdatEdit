<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { GRAPHS } from '$lib/graphs/registry.svelte';
  import GraphView from '$lib/graphs/GraphView.svelte';
  import { downloadGraph } from '$lib/graphs/export';

  // One persistent state instance per graph type, so switching tabs (or opening
  // the maximize modal, which shares the instance) preserves each graph's
  // controls.
  const states: Record<string, unknown> = {};
  for (const g of GRAPHS) states[g.id] = g.createState();

  let activeId = $state(GRAPHS[0].id);
  const active = $derived(GRAPHS.find((g) => g.id === activeId) ?? GRAPHS[0]);
  const activeState = $derived(states[active.id]);

  let maximized = $state(false);
  let downloading = $state(false);
  let downloadError = $state<string | null>(null);

  async function download() {
    downloading = true;
    downloadError = null;
    try {
      await downloadGraph(active, activeState);
    } catch (e) {
      downloadError = String(e);
    } finally {
      downloading = false;
    }
  }

  function onModalKey(e: KeyboardEvent) {
    if (e.key === 'Escape') maximized = false;
  }

  function fmt(v: number | null | undefined, digits = 1): string {
    if (v === null || v === undefined || !Number.isFinite(v)) return '—';
    return v.toFixed(digits);
  }

  const p = $derived(store.photometry);
  const ActiveControls = $derived(active.Controls);
</script>

<div class="panel">
  <div class="head">
    {#if GRAPHS.length > 1}
      <div class="tabs" role="tablist">
        {#each GRAPHS as g}
          <button
            class="tab"
            class:active={g.id === activeId}
            role="tab"
            aria-selected={g.id === activeId}
            onclick={() => (activeId = g.id)}
          >
            {g.label}
          </button>
        {/each}
      </div>
    {:else}
      <span class="title">{active.label}</span>
    {/if}

    <div class="tools">
      <button
        class="tool"
        onclick={download}
        disabled={downloading}
        title="Download graph (SVG or PNG)"
        aria-label="Download graph"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M12 3v12M7 11l5 5 5-5M5 20h14" />
        </svg>
      </button>
      <button
        class="tool"
        onclick={() => (maximized = true)}
        title="Enlarge graph"
        aria-label="Enlarge graph"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M9 4H4v5M4 4l6 6M15 4h5v5M20 4l-6 6M9 20H4v-5M4 20l6-6M15 20h5v-5M20 20l-6-6" />
        </svg>
      </button>
    </div>
  </div>

  <GraphView graph={active} graphState={activeState} size={520} />

  <ActiveControls state={activeState} />

  {#if downloadError}
    <div class="err">{downloadError}</div>
  {/if}

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
</div>

{#if maximized}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="backdrop" onclick={() => (maximized = false)}>
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="modal-head">
        <span class="modal-title">{active.label}</span>
        <div class="tools">
          <button
            class="tool"
            onclick={download}
            disabled={downloading}
            title="Download graph (SVG or PNG)"
            aria-label="Download graph"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M12 3v12M7 11l5 5 5-5M5 20h14" />
            </svg>
          </button>
          <button
            class="tool"
            onclick={() => (maximized = false)}
            title="Close"
            aria-label="Close"
          >
            <svg viewBox="0 0 24 24" aria-hidden="true">
              <path d="M6 6l12 12M18 6L6 18" />
            </svg>
          </button>
        </div>
      </div>
      <div class="modal-body">
        <div class="modal-graph">
          <GraphView graph={active} graphState={activeState} size={900} />
        </div>
        <div class="modal-controls">
          <ActiveControls state={activeState} />
        </div>
      </div>
    </div>
  </div>
{/if}

<svelte:window onkeydown={maximized ? onModalKey : undefined} />

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    overflow-y: auto;
    min-height: 0;
  }
  .head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .tabs {
    display: flex;
    gap: 2px;
    background: var(--bg-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 2px;
  }
  .tab {
    border: none;
    background: transparent;
    color: var(--text-dim);
    padding: 4px 12px;
    border-radius: calc(var(--radius-sm) - 2px);
    font-size: 13px;
    font-weight: 500;
  }
  .tab:hover {
    color: var(--text);
  }
  .tab.active {
    background: var(--sel);
    color: var(--text);
  }
  .title {
    font-weight: 600;
    font-size: 14px;
  }
  .tools {
    margin-left: auto;
    display: flex;
    gap: 4px;
  }
  .tool {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border: 1px solid var(--border);
    background: transparent;
    color: var(--text-dim);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .tool:hover:not(:disabled) {
    background: var(--sel);
    color: var(--text);
  }
  .tool:disabled {
    opacity: 0.5;
    cursor: progress;
  }
  .tool svg {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .err {
    color: var(--danger);
    font-size: 13px;
  }
  .stats {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .stat {
    background: var(--bg-sunken);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    padding: 9px 12px;
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

  /* Maximize modal */
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 32px;
    z-index: 50;
  }
  .modal {
    background: var(--bg-elev);
    border: 1px solid var(--border);
    border-radius: var(--radius);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px;
    max-width: min(92vw, 1100px);
    max-height: 92vh;
    overflow: auto;
  }
  .modal-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .modal-title {
    font-weight: 600;
    font-size: 15px;
  }
  .modal-body {
    display: flex;
    gap: 20px;
    align-items: flex-start;
  }
  .modal-graph {
    width: min(70vh, 760px);
    max-width: 100%;
    flex: none;
  }
  .modal-controls {
    flex: 1;
    min-width: 200px;
    padding-top: 8px;
  }
  @media (max-width: 720px) {
    .modal-body {
      flex-direction: column;
    }
    .modal-graph {
      width: 100%;
    }
  }
</style>
