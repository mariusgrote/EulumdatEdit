<script lang="ts">
  import { store } from '$lib/store.svelte';
  interface Props {
    onclose: () => void;
  }
  let { onclose }: Props = $props();
</script>

<div class="vpanel">
  <div class="vhead">
    <h3>Validation</h3>
    <button class="btn ghost" onclick={onclose}>✕</button>
  </div>
  {#if store.warnings.length === 0}
    <p class="ok">✓ No validation warnings.</p>
  {:else}
    <ul>
      {#each store.warnings as w}
        <li>
          <span class="wfield">{w.field}</span>
          <span class="wmsg">{w.message}</span>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .vpanel {
    background: var(--bg-elev);
    border-left: 1px solid var(--border);
    height: 100%;
    overflow-y: auto;
    padding: 16px;
    width: 320px;
  }
  .vhead {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 14px;
  }
  .vhead h3 {
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.04em;
    color: var(--text-dim);
  }
  .ok {
    color: var(--text-dim);
    font-size: 13px;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  li {
    border-left: 3px solid var(--warn);
    background: var(--bg-sunken);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .wfield {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--warn);
    font-weight: 600;
  }
  .wmsg {
    font-size: 13px;
    line-height: 1.4;
  }
</style>
