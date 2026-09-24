<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { store } from '$lib/store.svelte';
  import {
    newDocument,
    openFileDialog,
    openPath,
    closeDocument,
    saveDocument,
    saveDocumentAs,
    quitApplication
  } from '$lib/documentActions';
  import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
  import * as api from '$lib/api';
  import { isNarrowLayout } from '$lib/layout';
  import type { Warning } from '$lib/types';
  import {
    findFieldElement,
    resolveWarningTarget,
    warningsBySection,
    type SectionId,
    type WarningTarget
  } from '$lib/warningNavigation';
  import TopBar from '$lib/components/TopBar.svelte';
  import VisualizationPanel from '$lib/components/VisualizationPanel.svelte';
  import ValidationPanel from '$lib/components/ValidationPanel.svelte';
  import SectionGeneral from '$lib/components/SectionGeneral.svelte';
  import SectionGeometry from '$lib/components/SectionGeometry.svelte';
  import SectionLamps from '$lib/components/SectionLamps.svelte';
  import SectionIntensity from '$lib/components/SectionIntensity.svelte';
  import SectionUgr from '$lib/components/SectionUgr.svelte';

  const sections = [
    {
      id: 'general',
      label: 'General',
      icon: 'M5 5a2 2 0 0 1 2-2h7l5 5v11a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2zM14 3v5h5M8 13h8M8 17h6'
    },
    {
      id: 'geometry',
      label: 'Geometry',
      icon: 'M12 3l8 4.5v9L12 21l-8-4.5v-9zM4 7.5l8 4.5 8-4.5M12 12v9'
    },
    {
      id: 'lamps',
      label: 'Lamps',
      icon: 'M9.5 18h5M10.5 21h3M12 3a6 6 0 0 0-3.5 10.9c.6.5.9 1.1 1 2.1h5c.1-1 .4-1.6 1-2.1A6 6 0 0 0 12 3z'
    },
    { id: 'intensity', label: 'Intensity', icon: 'M4 4v16h16M8 15l3-4 3 3 4-6' },
    {
      id: 'ugr',
      label: 'UGR',
      icon: 'M2 12s3.5-6 10-6 10 6 10 6-3.5 6-10 6S2 12 2 12zM12 9.5a2.5 2.5 0 1 0 0 5 2.5 2.5 0 0 0 0-5z'
    }
  ] as const;

  type ActiveSectionId = SectionId | 'ugr';
  let active = $state<ActiveSectionId>('general');
  let rightView = $state<'diagram' | 'validation'>('diagram');

  // Inspector (right column) visibility. The user can collapse it; it also
  // auto-collapses on narrow windows and restores when room returns. On narrow
  // windows an open inspector overlays the editor as a drawer.
  let collapsed = $state(false);
  let narrow = $state(false);
  let wasNarrow = false;

  // Highlighted while a file is dragged over the window.
  let dragOver = $state(false);

  const sectionWarningCounts = $derived<Record<ActiveSectionId, number>>({
    ...warningsBySection(store.warnings),
    ugr: 0
  });
  const ugrBlocked = $derived(store.ugr?.status === 'blocked');

  $effect(() => {
    const n = narrow;
    // Edge-triggered: auto-collapse on entering narrow, auto-expand on leaving,
    // while leaving the user free to toggle manually in between.
    if (n && !wasNarrow) collapsed = true;
    else if (!n && wasNarrow) collapsed = false;
    wasNarrow = n;
  });

  function toggleValidation() {
    if (rightView === 'validation') {
      rightView = 'diagram';
    } else {
      rightView = 'validation';
      collapsed = false; // bring the panel back if it was hidden
    }
  }

  function navigateToWarning(warning: Warning) {
    const target = resolveWarningTarget(warning);
    if (target) navigateToTarget(target);
  }

  async function navigateToTarget(target: WarningTarget) {
    if (!store.doc) return;

    active = target.section;

    if (target.fieldKey) {
      await tick();
      const el = findFieldElement(target.fieldKey);
      if (el) {
        el.scrollIntoView({ behavior: 'smooth', block: 'center' });
        el.focus({ preventScroll: true });
        store.highlightField(target.fieldKey);
      }
    } else {
      document.querySelector('.content')?.scrollTo({ top: 0, behavior: 'smooth' });
    }
  }

  // Guard the window close button against discarding unsaved changes, and track
  // window width to auto-hide the inspector on narrow windows.
  onMount(() => {
    const appWindow = getCurrentWebviewWindow();
    const unlisten = appWindow.onCloseRequested(async (event) => {
      if (!store.tabs.some((tab) => tab.dirty)) return;
      event.preventDefault();
      if (await store.confirmCloseWindow()) await appWindow.destroy();
    });

    // macOS app menu items, sent by the backend to the focused window.
    const menuActions: Record<string, () => unknown> = {
      new: newDocument,
      open: openFileDialog,
      save: saveDocument,
      'save-as': saveDocumentAs,
      close: onCloseShortcut,
      'close-window': () => appWindow.close(),
      quit: quitApplication
    };
    const unlistenMenu = appWindow.listen<string>('menu', (e) => menuActions[e.payload]?.());

    const updateNarrow = () => (narrow = isNarrowLayout(window.innerWidth));
    updateNarrow();
    window.addEventListener('resize', updateNarrow);

    // Accept photometric files dropped onto the window.
    const unlistenDrop = appWindow.onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === 'enter') {
        dragOver = p.paths.some(isPhotometric);
      } else if (p.type === 'leave') {
        dragOver = false;
      } else if (p.type === 'drop') {
        dragOver = false;
        const file = p.paths.find(isPhotometric);
        if (file) openPath(file);
      }
    });

    // A window opened for a file or new document starts with it loaded. Then
    // files opened via the OS file association: a pending one queued before the
    // UI was ready, plus a live event for opens while the app is running. Drain
    // the queue only once the listener exists so no open falls between the two.
    const unlistenOpen = appWindow.listen<string>('open-file', (e) => openPath(e.payload));
    const unlistenWorkspace = appWindow.listen('workspace-changed', () => store.loadCurrent());
    Promise.all([unlistenOpen, unlistenWorkspace, store.loadCurrent()])
      .then(() => api.takePendingOpen())
      .then((path) => {
        if (path) openPath(path);
      });

    return () => {
      unlisten.then((fn) => fn());
      unlistenDrop.then((fn) => fn());
      unlistenOpen.then((fn) => fn());
      unlistenWorkspace.then((fn) => fn());
      unlistenMenu.then((fn) => fn());
      window.removeEventListener('resize', updateNarrow);
    };
  });

  const isPhotometric = (p: string) => /\.(ldt|ies)$/i.test(p);

  async function onCloseShortcut() {
    await closeDocument();
  }

  function onKey(e: KeyboardEvent) {
    if (e.ctrlKey && !e.altKey && !e.metaKey && e.key === 'Tab' && store.tabs.length > 1) {
      e.preventDefault();
      const current = store.tabs.findIndex((tab) => tab.id === store.activeTabId);
      const direction = e.shiftKey ? -1 : 1;
      const next = (current + direction + store.tabs.length) % store.tabs.length;
      store.activateTab(store.tabs[next].id);
      return;
    }
    const mod = e.metaKey || e.ctrlKey;
    if (!mod) return;
    const k = e.key.toLowerCase();
    if (k === 's') {
      e.preventDefault();
      if (e.shiftKey) saveDocumentAs();
      else saveDocument();
    } else if (k === 'o') {
      e.preventDefault();
      openFileDialog();
    } else if (k === 'n') {
      e.preventDefault();
      newDocument();
    } else if (k === 'w') {
      e.preventDefault();
      if (e.shiftKey) getCurrentWebviewWindow().close();
      else onCloseShortcut();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="app">
  <TopBar
    showValidation={rightView === 'validation'}
    {toggleValidation}
    panelCollapsed={collapsed}
    togglePanel={() => (collapsed = !collapsed)}
  />

  <div class="body" class:no-inspector={collapsed || !store.doc} class:narrow>
    <nav class="sidebar">
      {#each sections as s}
        <button
          class="navitem"
          class:active={active === s.id}
          class:has-warnings={sectionWarningCounts[s.id] > 0}
          onclick={() => (active = s.id)}
          disabled={!store.doc}
        >
          <svg class="ico" viewBox="0 0 24 24" aria-hidden="true">
            <path d={s.icon} />
          </svg>
          <span>{s.label}</span>
          {#if sectionWarningCounts[s.id] > 0}
            <span class="nav-badge" aria-label="{sectionWarningCounts[s.id]} warnings">
              {sectionWarningCounts[s.id]}
            </span>
          {:else if s.id === 'ugr' && store.doc && ugrBlocked}
            <span class="nav-note" title="The UGR tabular method does not apply">n/a</span>
          {/if}
        </button>
      {/each}
    </nav>

    <main class="content">
      {#if !store.doc}
        <div class="welcome">
          <div class="logo">◐</div>
          <h1>EulumdatEdit</h1>
          <p>Open an EULUMDAT <code>.ldt</code> or IES <code>.ies</code> file, or start a new luminaire.</p>
          <div class="welcome-actions">
            <button class="btn primary" onclick={openFileDialog}>Open photometric file…</button>
            <button class="btn" onclick={newDocument}>New luminaire</button>
          </div>
          <p class="welcome-hint">or drag and drop a <code>.ldt</code> or <code>.ies</code> file anywhere</p>
          {#if store.error}<p class="err">{store.error}</p>{/if}
        </div>
      {:else}
        <div class="sections">
          {#if active === 'general'}<SectionGeneral />{/if}
          {#if active === 'geometry'}<SectionGeometry />{/if}
          {#if active === 'lamps'}<SectionLamps />{/if}
          {#if active === 'intensity'}<SectionIntensity />{/if}
          {#if active === 'ugr'}<SectionUgr onnavigate={navigateToTarget} />{/if}
          {#if store.error}<div class="err card">{store.error}</div>{/if}
        </div>
      {/if}
    </main>

    {#if store.doc && !collapsed}
      <aside class="inspector">
        {#if rightView === 'validation'}
          <ValidationPanel
            onclose={() => (rightView = 'diagram')}
            onnavigate={navigateToWarning}
          />
        {:else}
          <VisualizationPanel />
        {/if}
      </aside>
    {/if}
  </div>

  {#if dragOver}
    <div class="dropzone">
      <div class="dropzone-card">
        <div class="logo">◐</div>
        <p>Drop to open <code>.ldt</code> or <code>.ies</code> file</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
  }
  .body {
    flex: 1;
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr) clamp(380px, 34vw, 520px);
    min-height: 0;
    position: relative;
  }
  .body.no-inspector,
  .body.narrow {
    grid-template-columns: 200px minmax(0, 1fr);
  }
  .sidebar {
    background: var(--bg-elev);
    border-right: 1px solid var(--border);
    padding: 12px 10px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .navitem {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--text-dim);
    padding: 8px 11px;
    border-radius: var(--radius-sm);
    font-weight: 500;
    text-align: left;
  }
  .navitem:hover:not(:disabled) {
    background: var(--sel);
    color: var(--text);
  }
  .navitem.active {
    background: var(--sel);
    color: var(--text);
  }
  /* amber leading indicator on the selected item */
  .navitem.active::before {
    content: '';
    position: absolute;
    left: 3px;
    top: 50%;
    transform: translateY(-50%);
    width: 3px;
    height: 16px;
    border-radius: 2px;
    background: var(--accent);
  }
  .navitem:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .ico {
    width: 18px;
    height: 18px;
    flex: none;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.7;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
  .navitem.active .ico {
    color: var(--accent-strong);
  }
  .navitem.has-warnings:not(.active) .ico {
    color: var(--warn);
  }
  .nav-badge {
    margin-left: auto;
    font-variant-numeric: tabular-nums;
    background: var(--warn);
    color: #1a1205;
    border-radius: 10px;
    padding: 0 6px;
    font-size: 10px;
    font-weight: 600;
    min-width: 18px;
    text-align: center;
    line-height: 18px;
  }
  .nav-note {
    margin-left: auto;
    font-size: 10px;
    font-weight: 600;
    line-height: 16px;
    padding: 0 5px;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    color: var(--text-faint);
  }
  .content {
    overflow-y: auto;
    padding: 24px 28px;
    min-width: 0;
  }
  .sections {
    display: flex;
    flex-direction: column;
    gap: 16px;
    max-width: 720px;
  }
  .inspector {
    background: var(--bg-elev);
    border-left: 1px solid var(--border);
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  /* Narrow windows: overlay the editor instead of taking a grid column. */
  .body.narrow .inspector {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    width: min(420px, calc(100vw - 200px));
    /* negative spread keeps the shadow off the top bar */
    box-shadow: -20px 0 24px -12px rgba(0, 0, 0, 0.28);
    z-index: 10;
  }
  .welcome {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 10px;
    text-align: center;
    color: var(--text-dim);
  }
  .welcome .logo {
    font-size: 64px;
    color: var(--accent);
  }
  .welcome h1 {
    font-size: 28px;
  }
  .welcome code {
    font-family: var(--mono);
    background: var(--bg-sunken);
    padding: 1px 6px;
    border-radius: 4px;
  }
  .welcome-actions {
    margin-top: 8px;
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    justify-content: center;
  }
  .welcome-hint {
    margin: 4px 0 0;
    font-size: 13px;
    color: var(--text-faint);
  }
  .err {
    color: var(--danger);
  }
  .dropzone {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    background: color-mix(in srgb, var(--bg-sunken) 75%, transparent);
    backdrop-filter: blur(2px);
    pointer-events: none;
  }
  .dropzone-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 32px 48px;
    border: 2px dashed var(--accent);
    border-radius: var(--radius);
    background: var(--bg-elev);
    color: var(--text-dim);
  }
  .dropzone-card .logo {
    font-size: 48px;
    color: var(--accent);
  }
  .dropzone code {
    font-family: var(--mono);
    background: var(--bg-sunken);
    padding: 1px 6px;
    border-radius: 4px;
  }
</style>
