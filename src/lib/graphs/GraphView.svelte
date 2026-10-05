<script lang="ts">
  import { store } from '$lib/store.svelte';
  import type { GraphType } from './registry.svelte';

  interface Props {
    graph: GraphType;
    graphState: unknown;
    /** Square render resolution in px; CSS scales the result to fit. */
    size: number;
    /** Called with the latest rendered SVG (empty string when none). */
    onsvg?: (svg: string) => void;
  }
  let { graph, graphState, size, onsvg }: Props = $props();

  let svg = $state('');
  let renderError = $state<string | null>(null);

  // Re-render after the Rust model updates (photometry is a proxy for a
  // committed change), when the controls change, or when the size changes.
  $effect(() => {
    void store.photometry;
    const opts = graph.toOptions(graphState); // synchronous reactive read
    const sz = size;
    let cancelled = false;
    const descriptor = graph;
    const timer = setTimeout(async () => {
      if (!store.doc) {
        svg = '';
        renderError = null;
        onsvg?.('');
        return;
      }
      try {
        const result = await descriptor.render(opts, sz);
        if (cancelled) return;
        svg = result;
        renderError = null;
        onsvg?.(svg);
      } catch (e) {
        if (cancelled) return;
        svg = '';
        renderError = String(e);
      }
    }, 120);
    return () => {
      cancelled = true;
      clearTimeout(timer);
    };
  });
</script>

<div class="graph">
  {#if renderError}
    <div class="err">{renderError}</div>
  {:else if svg}
    <!-- eslint-disable-next-line svelte/no-at-html-tags -->
    {@html svg}
  {:else}
    <div class="placeholder">No diagram</div>
  {/if}
</div>

<style>
  .graph {
    width: 100%;
    aspect-ratio: 1 / 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .graph :global(svg) {
    width: 100%;
    height: 100%;
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
</style>
