<script lang="ts">
  import { onMount } from 'svelte';
  import { flip } from 'svelte/animate';
  import { slide } from 'svelte/transition';
  import { store } from '$lib/store.svelte';
  import {
    newDocument,
    openFileDialog,
    closeDocument,
    saveDocument,
    saveDocumentAs,
    exportIes
  } from '$lib/documentActions';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { TabDragFlow, type DragState, type TabLayout } from '$lib/tabDragFlow';
  import { tabDragWindows } from '$lib/tabDragWindows';

  interface Props {
    showValidation: boolean;
    toggleValidation: () => void;
    panelCollapsed: boolean;
    togglePanel: () => void;
  }
  let { showValidation, toggleValidation, panelCollapsed, togglePanel }: Props = $props();

  async function doDiscard() {
    const ok = await ask(
      'Discard all unsaved changes and reload this file from disk? This cannot be undone.',
      { title: 'Discard changes', kind: 'warning' }
    );
    if (ok) await store.revert();
  }

  const warnCount = $derived(store.warnings.length);
  let tabsElement: HTMLDivElement;
  let tabMenuElement = $state<HTMLDetailsElement>();
  let fileMenuElement = $state<HTMLDetailsElement>();
  let isMacOS = $state(false);

  function onWindowClick(event: MouseEvent) {
    if (!(event.target instanceof Node)) return;
    if (!tabMenuElement?.contains(event.target)) tabMenuElement?.removeAttribute('open');
    if (!fileMenuElement?.contains(event.target)) fileMenuElement?.removeAttribute('open');
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key !== 'Escape') return;
    cancelTabDrag();
    tabMenuElement?.removeAttribute('open');
    fileMenuElement?.removeAttribute('open');
  }

  function showActiveTab() {
    if (!tabsElement) return;
    const active = tabsElement.querySelector<HTMLElement>('.tab.active');
    if (!active) return;
    const strip = tabsElement.getBoundingClientRect();
    const tab = active.getBoundingClientRect();
    if (tab.width > strip.width) {
      tabsElement.scrollLeft += tab.left - strip.left;
      return;
    }
    if (tab.left < strip.left) tabsElement.scrollLeft -= strip.left - tab.left;
    else if (tab.right > strip.right) tabsElement.scrollLeft += tab.right - strip.right;
  }

  $effect(() => {
    const activeTitle = store.tabs.find((tab) => tab.id === store.activeTabId)?.title;
    if (!activeTitle) return;
    const frame = requestAnimationFrame(showActiveTab);
    return () => cancelAnimationFrame(frame);
  });

  onMount(() => {
    isMacOS = /Macintosh|Mac OS X/.test(navigator.userAgent);
    const observer = new ResizeObserver(showActiveTab);
    observer.observe(tabsElement);
    return () => observer.disconnect();
  });

  const motionDuration = () =>
    window.matchMedia('(prefers-reduced-motion: reduce)').matches ? 0 : 170;

  let tabDrag = $state<DragState | null>(null);
  let suppressClick = false;
  let capturedTab: HTMLElement | null = null;
  const dragFlow = new TabDragFlow({
    title: (id) => store.tabs.find((tab) => tab.id === id)?.title ?? '',
    move: (id, target, index) => store.moveTab(id, target, index),
    detach: (id, x, y) => store.detachTab(id, x, y),
    hasTabs: () => store.tabs.length > 0,
    reportError: (message) => { store.error = message; }
  }, tabDragWindows);

  function tabLayout(): TabLayout {
    const strip = tabsElement.getBoundingClientRect();
    return {
      strip: { left: strip.left, top: strip.top, right: strip.right, bottom: strip.bottom },
      tabs: [...tabsElement.querySelectorAll<HTMLElement>('[data-tab-id]')].map((tab) => {
        const rect = tab.getBoundingClientRect();
        return { left: rect.left, right: rect.right };
      })
    };
  }

  function onTabPointerDown(event: PointerEvent, tabId: string) {
    if (event.button !== 0 || !event.isPrimary) return;
    cancelTabDrag();
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    capturedTab = target;
    dragFlow.start(tabId, event);
    tabDrag = dragFlow.state;
  }

  function cancelTabDrag() {
    const pointerId = tabDrag?.pointerId;
    dragFlow.cancel();
    tabDrag = null;
    if (pointerId !== undefined && capturedTab?.hasPointerCapture(pointerId)) {
      capturedTab.releasePointerCapture(pointerId);
    }
    capturedTab = null;
  }

  function onTabPointerMove(event: PointerEvent) {
    if (!tabDrag || event.pointerId !== tabDrag.pointerId) return;
    tabDrag = dragFlow.move(event, tabLayout());
  }

  function onTabPointerCancel(event: PointerEvent) {
    if (tabDrag?.pointerId === event.pointerId) cancelTabDrag();
  }

  async function onTabPointerUp(event: PointerEvent) {
    if (!tabDrag || event.pointerId !== tabDrag.pointerId) return;
    const layout = tabLayout();
    tabDrag = dragFlow.move(event, layout);
    const wasDragging = tabDrag?.dragging ?? false;
    const finished = dragFlow.finish(event, layout);
    suppressClick = wasDragging;
    if (wasDragging) setTimeout(() => { suppressClick = false; }, 0);
    cancelTabDrag();
    await finished;
  }

  onMount(() => () => cancelTabDrag());
</script>

<svelte:window
  onpointermove={onTabPointerMove}
  onpointerup={onTabPointerUp}
  onpointercancel={onTabPointerCancel}
  onlostpointercapture={onTabPointerCancel}
  onclick={onWindowClick}
  onkeydown={onWindowKeydown}
/>

<header class="topbar" class:macos={isMacOS} data-tauri-drag-region>
  <div class="tabs" role="tablist" aria-label="Open documents" bind:this={tabsElement}>
    {#each store.tabs as tab, index (tab.id)}
      <div
        class="tab"
        class:active={tab.id === store.activeTabId}
        class:dragging={tabDrag?.dragging && tabDrag.tabId === tab.id}
        class:drop-before={tabDrag?.dragging && tabDrag.dropIndex === index && tabDrag.tabId !== tab.id}
        data-tab-id={tab.id}
        title={tab.path || tab.title}
        role="tab"
        tabindex="0"
        aria-selected={tab.id === store.activeTabId}
        animate:flip={{ duration: motionDuration }}
        out:slide={{ axis: 'x', duration: motionDuration() }}
        onpointerdown={(event) => onTabPointerDown(event, tab.id)}
        onclick={() => { if (!suppressClick) store.activateTab(tab.id); }}
        onselectstart={(event) => event.preventDefault()}
        onkeydown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') store.activateTab(tab.id);
        }}
      >
        <span class="filename">
          <span class="filename-start">
            {tab.title.length > 32 ? tab.title.slice(0, -12) : tab.title}
          </span>
          {#if tab.title.length > 32}<span class="filename-end">{tab.title.slice(-12)}</span>{/if}
        </span>
        {#if tab.dirty}<span class="dot" title="Unsaved changes">●</span>{/if}
        <button
          class="tab-close"
          title={`Close ${tab.title}`}
          aria-label={`Close ${tab.title}`}
          draggable="false"
          onpointerdown={(event) => event.stopPropagation()}
          onclick={(event) => {
            event.stopPropagation();
            closeDocument(tab.id);
          }}
        >×</button>
      </div>
    {/each}
    <div
      class="title-drag-space"
      class:drop-end={tabDrag?.dragging && tabDrag.dropIndex === store.tabs.length}
      data-tauri-drag-region
    ></div>
  </div>

  {#if store.tabs.length > 0}
    <details class="tab-menu" bind:this={tabMenuElement}>
      <summary aria-label="Show all tabs" title="Show all tabs">
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M5 7h14M5 12h14M5 17h10" />
        </svg>
      </summary>
      <div class="tab-menu-panel">
        {#each store.tabs as tab (tab.id)}
          <button
            class:active={tab.id === store.activeTabId}
            title={tab.path || tab.title}
            onclick={(event) => {
              event.currentTarget.closest('details')?.removeAttribute('open');
              store.activateTab(tab.id);
            }}
          >
            <span>{tab.title}</span>
            {#if tab.dirty}<span class="dot" aria-label="Unsaved changes">●</span>{/if}
          </button>
        {/each}
      </div>
    </details>
  {/if}

  <details class="file-menu" bind:this={fileMenuElement}>
    <summary>File</summary>
    <div class="file-menu-panel">
      <button onclick={(event) => {
        event.currentTarget.closest('details')?.removeAttribute('open');
        newDocument();
      }}>New</button>
      <button onclick={(event) => {
        event.currentTarget.closest('details')?.removeAttribute('open');
        openFileDialog();
      }}>Open…</button>
      <button disabled={!store.doc} onclick={(event) => {
        event.currentTarget.closest('details')?.removeAttribute('open');
        saveDocumentAs();
      }}>Save As…</button>
    </div>
  </details>

  <div class="actions">
    {#if store.dirty && store.path}
      <button
        class="revert"
        onclick={doDiscard}
        title="Discard changes and reload from disk"
        aria-label="Discard changes and reload from disk"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M3 12a9 9 0 1 0 3-6.7M3 4v4h4" />
        </svg>
      </button>
    {/if}
    <button class="btn" onclick={saveDocument} disabled={!store.doc}>Save</button>
    <button class="btn ghost" onclick={exportIes} disabled={!store.doc}>Export IES</button>

    <button
      class="btn ghost badge-btn"
      class:has={warnCount > 0}
      class:active={showValidation}
      onclick={toggleValidation}
      disabled={!store.doc}
      title={showValidation ? 'Show diagram' : 'Show validation warnings'}
      aria-label={showValidation ? 'Show diagram' : `Show validation warnings, ${warnCount} warnings`}
    >
      <svg class="warn-ico" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 4 2.5 20.5h19zM12 10v4M12 17.5v.01" />
      </svg>
      {#if warnCount > 0}{warnCount}{/if}
    </button>

    <button
      class="btn ghost panel-btn"
      class:active={!panelCollapsed}
      onclick={togglePanel}
      disabled={!store.doc}
      title={panelCollapsed ? 'Show side panel' : 'Hide side panel'}
      aria-label={panelCollapsed ? 'Show side panel' : 'Hide side panel'}
    >
      <svg class="panel-ico" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M4 5h16v14H4zM15 5v14" />
      </svg>
    </button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 0 14px;
    height: 46px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
  }
  .topbar.macos {
    /* Leave room for the macOS traffic lights (titleBarStyle: Overlay). */
    padding-left: 82px;
  }
  .tabs {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: center;
    align-self: stretch;
    gap: 3px;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    user-select: none;
    -webkit-user-select: none;
  }
  .tabs::-webkit-scrollbar {
    display: none;
  }
  .tab {
    flex: 0 1 auto;
    min-width: 140px;
    align-self: end;
    height: 35px;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 8px 0 11px;
    border: 1px solid transparent;
    border-bottom: 0;
    border-radius: 7px 7px 0 0;
    color: var(--text-dim);
    font-size: 13px;
    user-select: none;
    -webkit-user-select: none;
    touch-action: none;
    cursor: grab;
    position: relative;
    --tab-close-bg: var(--bg-elev);
  }
  .tab:hover {
    background: var(--bg-sunken);
    color: var(--text);
    --tab-close-bg: var(--bg-sunken);
  }
  .tab.active {
    background: var(--bg);
    border-color: var(--border);
    color: var(--text);
    --tab-close-bg: var(--bg);
  }
  .tab.dragging {
    opacity: 0.25;
    cursor: grabbing;
  }
  .tab.drop-before::before,
  .title-drag-space.drop-end::before {
    content: '';
    position: absolute;
    left: -3px;
    top: 5px;
    bottom: 5px;
    width: 2px;
    border-radius: 2px;
    background: var(--accent);
  }
  .tab:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .filename {
    flex: 1;
    min-width: 0;
    display: flex;
    overflow: hidden;
    white-space: nowrap;
  }
  .filename-start {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .filename-end {
    flex: none;
  }
  .dot {
    color: var(--accent);
    font-size: 10px;
  }
  .tab-close {
    position: absolute;
    right: 8px;
    top: 8px;
    width: 19px;
    height: 19px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: var(--tab-close-bg);
    color: var(--text-faint);
    font-size: 16px;
    line-height: 1;
    opacity: 0;
    pointer-events: none;
  }
  .dot + .tab-close {
    right: 22px;
  }
  .tab:hover .tab-close,
  .tab:focus-within .tab-close {
    opacity: 1;
    pointer-events: auto;
  }
  .tab-close:hover {
    box-shadow: inset 0 0 0 20px var(--sel);
    color: var(--text);
  }
  .title-drag-space {
    flex: 1 0 18px;
    align-self: stretch;
    position: relative;
  }
  .tab-menu,
  .file-menu {
    flex: none;
    position: relative;
  }
  .tab-menu summary,
  .file-menu summary {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border-radius: var(--radius-sm);
    color: var(--text-dim);
    cursor: pointer;
    list-style: none;
  }
  .tab-menu summary::-webkit-details-marker {
    display: none;
  }
  .file-menu summary::-webkit-details-marker {
    display: none;
  }
  .tab-menu summary:hover,
  .tab-menu[open] summary,
  .file-menu summary:hover,
  .file-menu[open] summary {
    background: var(--bg-sunken);
    color: var(--text);
  }
  .tab-menu summary:focus-visible,
  .file-menu summary:focus-visible {
    outline: 2px solid var(--accent);
  }
  .tab-menu svg {
    width: 17px;
    height: 17px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
  }
  .tab-menu-panel {
    position: absolute;
    top: calc(100% + 7px);
    right: 0;
    z-index: 10;
    width: max-content;
    max-width: min(640px, calc(100vw - 260px));
    max-height: min(420px, 70vh);
    overflow-y: auto;
    padding: 4px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    background: var(--bg-elev);
    box-shadow: var(--shadow-pop);
  }
  .tab-menu-panel button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    text-align: left;
  }
  .tab-menu-panel button:hover,
  .tab-menu-panel button.active {
    background: var(--bg-sunken);
  }
  .tab-menu-panel button span:first-child {
    flex: 1;
    overflow-wrap: anywhere;
  }
  .file-menu summary {
    width: auto;
    padding: 0 9px;
    font-weight: 500;
  }
  .file-menu-panel {
    position: absolute;
    top: calc(100% + 7px);
    right: 0;
    z-index: 10;
    width: 220px;
    padding: 4px;
    border: 1px solid var(--border-strong);
    border-radius: var(--radius);
    background: var(--bg-elev);
    box-shadow: var(--shadow-pop);
  }
  .file-menu-panel button {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    width: 100%;
    padding: 8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--text);
    text-align: left;
  }
  .file-menu-panel button:hover:not(:disabled) {
    background: var(--bg-sunken);
  }
  .file-menu-panel button:disabled {
    color: var(--text-faint);
    cursor: default;
  }
  .revert {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 2px;
    border: none;
    background: transparent;
    color: var(--text-faint);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }
  .revert:hover {
    background: var(--sel);
    color: var(--text);
  }
  .revert svg {
    width: 14px;
    height: 14px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .actions {
    flex: none;
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .badge-btn {
    gap: 5px;
    font-variant-numeric: tabular-nums;
  }
  .warn-ico {
    width: 15px;
    height: 15px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .badge-btn.has {
    color: var(--warn);
    border-color: var(--warn);
  }
  .badge-btn.active {
    background: var(--accent-soft);
    border-color: var(--accent);
    color: var(--accent-strong);
  }
  .panel-btn {
    padding: 0 8px;
  }
  .panel-ico {
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .panel-btn.active {
    color: var(--text);
  }
</style>
