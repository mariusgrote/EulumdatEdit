<script lang="ts">
  import { store } from '$lib/store.svelte';
  import type { Warning } from '$lib/types';
  import { isNavigableTarget, resolveWarningTargets } from '$lib/warningNavigation';

  interface Props {
    onclose: () => void;
    onnavigate?: (warning: Warning, index: number) => void;
  }
  let { onclose, onnavigate }: Props = $props();

  const targets = $derived(
    store.doc
      ? resolveWarningTargets(store.warnings, store.doc, store.strictValidation)
      : []
  );

  function handleNavigate(w: Warning, index: number) {
    if (!isNavigableTarget(targets[index])) return;
    onnavigate?.(w, index);
  }

  function handleKeydown(e: KeyboardEvent, w: Warning, index: number) {
    if (e.key !== 'Enter' && e.key !== ' ') return;
    e.preventDefault();
    handleNavigate(w, index);
  }
</script>

<div class="vpanel">
  <div class="vhead">
    <h3>
      Validation
      {#if store.warnings.length > 0}<span class="count">{store.warnings.length}</span>{/if}
    </h3>
    <button class="btn ghost icon" onclick={onclose} title="Back to diagram" aria-label="Back to diagram">
      ✕
    </button>
  </div>

  <label class="strict">
    <input
      type="checkbox"
      checked={store.strictValidation}
      onchange={(e) => store.setStrictValidation(e.currentTarget.checked)}
    />
    <span>
      <span class="strict-title">Legacy strict limits</span>
      <span class="strict-hint">Enforce the original EULUMDAT 8.3-era field lengths (8-char file name, etc.).</span>
    </span>
  </label>

  <div class="vbody">
    {#if store.warnings.length === 0}
      <div class="ok">
        <span class="ok-mark">✓</span>
        <span>No validation warnings.</span>
      </div>
    {:else}
      <ul>
        {#each store.warnings as w, i}
          {@const target = targets[i]}
          {@const navigable = isNavigableTarget(target)}
          <li
            class:navigable
            class:orphan={!navigable}
            role={navigable ? 'button' : undefined}
            tabindex={navigable ? 0 : undefined}
            title={navigable ? 'Go to field' : 'No editor for this field'}
            onclick={() => handleNavigate(w, i)}
            onkeydown={(e) => handleKeydown(e, w, i)}
          >
            <span class="wfield">{w.field}</span>
            <span class="wmsg">{w.message}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .vpanel {
    height: 100%;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
  .vhead {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    border-bottom: 1px solid var(--border);
  }
  .vhead h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-faint);
    font-weight: 600;
  }
  .count {
    font-variant-numeric: tabular-nums;
    background: var(--warn);
    color: #1a1205;
    border-radius: 10px;
    padding: 1px 7px;
    font-size: 11px;
    letter-spacing: 0;
  }
  .icon {
    padding: 4px 8px;
  }
  .strict {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    padding: 12px 16px;
    border-bottom: 1px solid var(--border);
    cursor: pointer;
  }
  .strict input[type='checkbox'] {
    width: auto;
    margin: 2px 0 0;
    padding: 0;
    border: none;
    background: none;
    flex-shrink: 0;
  }
  .strict span {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .strict-title {
    font-size: 13px;
    color: var(--text);
  }
  .strict-hint {
    font-size: 11px;
    line-height: 1.4;
    color: var(--text-faint);
  }
  .vbody {
    overflow-y: auto;
    padding: 14px 16px;
    min-height: 0;
  }
  .ok {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-dim);
    font-size: 13px;
  }
  .ok-mark {
    color: #2e9e5b;
    font-weight: 700;
  }
  ul {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  li {
    border: 1px solid var(--border);
    border-left: 3px solid var(--warn);
    background: var(--bg-elev);
    border-radius: var(--radius-sm);
    padding: 9px 12px;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  li.navigable {
    cursor: pointer;
  }
  li.navigable:hover {
    background: var(--sel);
  }
  li.orphan {
    opacity: 0.75;
    cursor: default;
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
