<script lang="ts">
  import { POLAR_PLANES, type PolarState } from './registry.svelte';

  interface Props {
    state: PolarState;
  }
  let { state }: Props = $props();
</script>

<div class="controls">
  <div class="planes">
    {#each POLAR_PLANES as plane}
      <label class="chk" class:on={state.planes[plane.id]}>
        <input type="checkbox" bind:checked={state.planes[plane.id]} />
        <span
          class="swatch"
          style:background={state.planes[plane.id] ? plane.color : 'transparent'}
        ></span>
        {plane.label}
      </label>
    {/each}
  </div>
  <select bind:value={state.intensityMode} aria-label="Intensity mode">
    <option value="stored">Stored (cd/klm)</option>
    <option value="converted">Converted (× factor)</option>
  </select>
</div>

<style>
  .controls {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .planes {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 4px 14px;
  }
  .chk {
    display: flex;
    align-items: center;
    gap: 7px;
    font-size: 13px;
    color: var(--text-faint);
    padding: 3px 0;
  }
  .chk.on {
    color: var(--text);
  }
  .chk input {
    width: auto;
  }
  .swatch {
    width: 14px;
    height: 3px;
    border-radius: 2px;
    border: 1px solid var(--border-strong);
    flex: none;
  }
</style>
