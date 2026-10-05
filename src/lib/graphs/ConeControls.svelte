<script lang="ts">
  import { POLAR_PLANES, type ConeState } from './registry.svelte';
  let { state }: { state: ConeState } = $props();
</script>

<div class="controls">
  <div class="planes">
    {#each POLAR_PLANES.slice(0, 2) as plane}
      <label>
        <input
          type="checkbox"
          bind:checked={state.planes[plane.id]}
          disabled={state.planes[plane.id] && Object.values(state.planes).filter(Boolean).length === 1}
        />
        <span class="swatch" style:background={plane.color}></span>
        {plane.label}
      </label>
    {/each}
  </div>
  <label class="distance">
    Maximum distance [m]
    <input type="number" min="0.1" max="100" step="0.1" bind:value={state.maxDistance} />
  </label>
  <p>Six distance levels. Lux on a horizontal plane, using the first lamp set and conversion factor.</p>
</div>

<style>
  .controls { display: flex; flex-direction: column; gap: 10px; }
  .planes { display: flex; flex-wrap: wrap; gap: 14px; }
  label { display: flex; align-items: center; gap: 7px; font-size: 13px; }
  input[type='checkbox'] { width: auto; }
  .swatch { width: 14px; height: 3px; border-radius: 2px; }
  .distance { justify-content: space-between; }
  input[type='number'] { width: 100px; }
  p { margin: 0; color: var(--text-faint); font-size: 12px; line-height: 1.5; }
</style>
