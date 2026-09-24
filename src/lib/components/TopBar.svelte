<script lang="ts">
  import { store } from '$lib/store.svelte';
  import {
    newDocument,
    openFileDialog,
    closeDocument,
    saveDocument,
    saveDocumentAs
  } from '$lib/documentActions';
  import { ask } from '@tauri-apps/plugin-dialog';
  import {
    getAllWebviewWindows,
    getCurrentWebviewWindow
  } from '@tauri-apps/api/webviewWindow';

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

  type TabDrag = {
    tabId: string;
    pointerId: number;
    startX: number;
    startY: number;
    dragging: boolean;
  };
  let tabDrag = $state<TabDrag | null>(null);

  function onTabPointerDown(event: PointerEvent, tabId: string) {
    if (event.button !== 0) return;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    tabDrag = {
      tabId,
      pointerId: event.pointerId,
      startX: event.screenX,
      startY: event.screenY,
      dragging: false
    };
  }

  function onTabPointerMove(event: PointerEvent) {
    if (!tabDrag || event.pointerId !== tabDrag.pointerId) return;
    if (Math.hypot(event.screenX - tabDrag.startX, event.screenY - tabDrag.startY) > 6) {
      tabDrag.dragging = true;
    }
  }

  async function onTabPointerUp(event: PointerEvent) {
    if (!tabDrag || event.pointerId !== tabDrag.pointerId) return;
    const drag = tabDrag;
    tabDrag = null;
    if (!drag.dragging) {
      await store.activateTab(drag.tabId);
      return;
    }

    const current = getCurrentWebviewWindow();
    const currentPosition = await current.outerPosition();
    const currentSize = await current.outerSize();
    const currentScale = await current.scaleFactor();
    const currentBounds = {
      left: currentPosition.x / currentScale,
      top: currentPosition.y / currentScale,
      right: (currentPosition.x + currentSize.width) / currentScale,
      bottom: (currentPosition.y + currentSize.height) / currentScale
    };
    const insideCurrent =
      event.screenX >= currentBounds.left &&
      event.screenX <= currentBounds.right &&
      event.screenY >= currentBounds.top &&
      event.screenY <= currentBounds.bottom;

    if (insideCurrent) {
      const tabs = [...document.querySelectorAll<HTMLElement>('[data-tab-id]')];
      const targetIndex = tabs.findIndex((tab) => {
        const rect = tab.getBoundingClientRect();
        return event.clientX < rect.left + rect.width / 2;
      });
      await store.moveTab(
        drag.tabId,
        current.label,
        targetIndex === -1 ? tabs.length : targetIndex
      );
      return;
    }

    for (const candidate of await getAllWebviewWindows()) {
      if (candidate.label === current.label) continue;
      const position = await candidate.outerPosition();
      const size = await candidate.outerSize();
      const scale = await candidate.scaleFactor();
      const left = position.x / scale;
      const top = position.y / scale;
      if (
        event.screenX >= left &&
        event.screenX <= left + size.width / scale &&
        event.screenY >= top &&
        event.screenY <= top + size.height / scale
      ) {
        await store.moveTab(drag.tabId, candidate.label);
        if (store.tabs.length === 0) await current.close();
        return;
      }
    }

    await store.detachTab(drag.tabId, event.screenX - 180, event.screenY - 18);
    if (store.tabs.length === 0) await current.close();
  }
</script>

<header class="topbar">
  <div class="brand" data-tauri-drag-region="deep">
    <span class="logo">◐</span>
    <span class="name">EulumdatEdit</span>
  </div>

  <div class="tabs" data-tauri-drag-region>
    {#each store.tabs as tab (tab.id)}
      <div
        class="tab"
        class:active={tab.id === store.activeTabId}
        class:dragging={tabDrag?.tabId === tab.id && tabDrag.dragging}
        data-tab-id={tab.id}
        title={tab.path || tab.title}
        role="tab"
        tabindex="0"
        aria-selected={tab.id === store.activeTabId}
        onpointerdown={(event) => onTabPointerDown(event, tab.id)}
        onpointermove={onTabPointerMove}
        onpointerup={onTabPointerUp}
        onpointercancel={() => (tabDrag = null)}
        onkeydown={(event) => {
          if (event.key === 'Enter' || event.key === ' ') store.activateTab(tab.id);
        }}
      >
        <span class="filename">{tab.title}</span>
        {#if tab.dirty}<span class="dot" title="Unsaved changes">●</span>{/if}
        <button
          class="tab-close"
          title={`Close ${tab.title}`}
          aria-label={`Close ${tab.title}`}
          onpointerdown={(event) => event.stopPropagation()}
          onclick={(event) => {
            event.stopPropagation();
            closeDocument(tab.id);
          }}
        >×</button>
      </div>
    {/each}
    <div class="title-drag-space" data-tauri-drag-region></div>
  </div>

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
    <button class="btn ghost" onclick={newDocument}>New</button>
    <button class="btn ghost" onclick={openFileDialog}>Open</button>
    <button class="btn" onclick={saveDocument} disabled={!store.doc}>Save</button>
    <button class="btn ghost" onclick={saveDocumentAs} disabled={!store.doc}>Save As</button>

    <button
      class="btn ghost badge-btn"
      class:has={warnCount > 0}
      class:active={showValidation}
      onclick={toggleValidation}
      disabled={!store.doc}
      title={showValidation ? 'Show diagram' : 'Show validation warnings'}
    >
      <svg class="warn-ico" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M12 4 2.5 20.5h19zM12 10v4M12 17.5v.01" />
      </svg>
      {warnCount}
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
    gap: 20px;
    /* leave room for the macOS traffic lights (titleBarStyle: Overlay) */
    padding: 0 14px 0 82px;
    height: 46px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
  }
  .brand {
    flex: none;
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .logo {
    color: var(--accent);
    font-size: 20px;
  }
  .name {
    font-weight: 600;
    letter-spacing: -0.01em;
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
  }
  .tabs::-webkit-scrollbar {
    display: none;
  }
  .tab {
    flex: 0 1 180px;
    min-width: 92px;
    max-width: 220px;
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
    cursor: default;
  }
  .tab:hover {
    background: var(--bg-sunken);
    color: var(--text);
  }
  .tab.active {
    background: var(--bg);
    border-color: var(--border);
    color: var(--text);
  }
  .tab.dragging {
    opacity: 0.55;
  }
  .tab:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: -2px;
  }
  .filename {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dot {
    color: var(--accent);
    font-size: 10px;
  }
  .tab-close {
    flex: none;
    width: 19px;
    height: 19px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: var(--text-faint);
    font-size: 16px;
    line-height: 1;
  }
  .tab-close:hover {
    background: var(--sel);
    color: var(--text);
  }
  .title-drag-space {
    flex: 1 0 18px;
    align-self: stretch;
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
  @media (max-width: 1049px) {
    .topbar {
      gap: 12px;
    }
    .name {
      display: none;
    }
  }
</style>
