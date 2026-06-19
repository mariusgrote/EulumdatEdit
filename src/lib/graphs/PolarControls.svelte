<script lang="ts">
  import { POLAR_PLANES, POLAR_PALETTE, type PolarState } from './registry.svelte';

  interface Props {
    state: PolarState;
  }
  let { state }: Props = $props();

  // plane id -> curve color, indexed by its position among the enabled planes.
  const colorOf = $derived.by(() => {
    const map: Record<string, string> = {};
    let i = 0;
    for (const p of POLAR_PLANES) {
      if (state.planes[p.id]) map[p.id] = POLAR_PALETTE[i++ % POLAR_PALETTE.length];
    }
    return map;
  });
</script>

<div class="controls">
  <div class="planes">
    {#each POLAR_PLANES as plane}
      <label class="chk" class:on={state.planes[plane.id]}>
        <input type="checkbox" bind:checked={state.planes[plane.id]} />
        <span
          class="swatch"
          style:background={state.planes[plane.id] ? colorOf[plane.id] : 'transparent'}
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
