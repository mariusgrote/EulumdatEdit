<script lang="ts">
  import { onMount } from 'svelte';
  import { store } from '$lib/store.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { listen } from '@tauri-apps/api/event';
  import * as api from '$lib/api';
  import TopBar from '$lib/components/TopBar.svelte';
  import VisualizationPanel from '$lib/components/VisualizationPanel.svelte';
  import ValidationPanel from '$lib/components/ValidationPanel.svelte';
  import SectionGeneral from '$lib/components/SectionGeneral.svelte';
  import SectionGeometry from '$lib/components/SectionGeometry.svelte';
  import SectionLamps from '$lib/components/SectionLamps.svelte';
  import SectionIntensity from '$lib/components/SectionIntensity.svelte';

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
    { id: 'intensity', label: 'Intensity', icon: 'M4 4v16h16M8 15l3-4 3 3 4-6' }
  ] as const;

  type SectionId = (typeof sections)[number]['id'];
  let active = $state<SectionId>('general');
  let rightView = $state<'diagram' | 'validation'>('diagram');

  // Inspector (right column) visibility. The user can collapse it; it also
  // auto-collapses on narrow windows and restores when room returns.
  let collapsed = $state(false);
  let narrow = $state(false);
  let wasNarrow = false;
  const PANEL_MIN_WIDTH = 900;

  // Highlighted while a file is dragged over the window.
  let dragOver = $state(false);

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

  async function openFile() {
    if (!(await store.confirmDiscardChanges())) return;
    const path = await openDialog({
      multiple: false,
      filters: [{ name: 'EULUMDAT', extensions: ['ldt', 'LDT'] }]
    });
    if (typeof path === 'string') await store.open(path);
  }

  async function newDoc() {
    if (!(await store.confirmDiscardChanges())) return;
    await store.newDoc();
  }

  // Opens a known path (drag-and-drop, file association) behind the same
  // unsaved-changes guard the Open button uses.
  async function openPath(path: string) {
    if (!(await store.confirmDiscardChanges())) return;
    await store.open(path);
  }

  // Guard the window close button against discarding unsaved changes, and track
  // window width to auto-hide the inspector on narrow windows.
  onMount(() => {
    const appWindow = getCurrentWindow();
    const unlisten = appWindow.onCloseRequested(async (event) => {
      if (await store.confirmDiscardChanges()) return;
      event.preventDefault();
    });

    const updateNarrow = () => (narrow = window.innerWidth < PANEL_MIN_WIDTH);
    updateNarrow();
    window.addEventListener('resize', updateNarrow);

    // Accept .ldt files dropped onto the window.
    const unlistenDrop = appWindow.onDragDropEvent((event) => {
      const p = event.payload;
      if (p.type === 'enter') {
        dragOver = p.paths.some(isLdt);
      } else if (p.type === 'leave') {
        dragOver = false;
      } else if (p.type === 'drop') {
        dragOver = false;
        const file = p.paths.find(isLdt);
        if (file) openPath(file);
      }
    });

    // Files opened via the OS file association: a pending one queued before the
    // UI was ready, plus a live event for opens while the app is running.
    const unlistenOpen = listen<string>('open-file', (e) => openPath(e.payload));
    api.takePendingOpen().then((path) => {
      if (path) openPath(path);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenDrop.then((fn) => fn());
      unlistenOpen.then((fn) => fn());
      window.removeEventListener('resize', updateNarrow);
    };
  });

  const isLdt = (p: string) => p.toLowerCase().endsWith('.ldt');

  function onKey(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;
    if (!mod) return;
    const k = e.key.toLowerCase();
    if (k === 's') {
      e.preventDefault();
      if (store.doc) store.save();
    } else if (k === 'o') {
      e.preventDefault();
      openFile();
    } else if (k === 'n') {
      e.preventDefault();
      newDoc();
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

  <div class="body" class:no-inspector={collapsed || !store.doc}>
    <nav class="sidebar">
      {#each sections as s}
        <button
          class="navitem"
          class:active={active === s.id}
          onclick={() => (active = s.id)}
          disabled={!store.doc}
        >
          <svg class="ico" viewBox="0 0 24 24" aria-hidden="true">
            <path d={s.icon} />
          </svg>
          <span>{s.label}</span>
        </button>
      {/each}
    </nav>

    <main class="content">
      {#if !store.doc}
        <div class="welcome">
          <div class="logo">◐</div>
          <h1>EulumdatEdit</h1>
          <p>Open a EULUMDAT <code>.ldt</code> file or start a new luminaire.</p>
          <div class="welcome-actions">
            <button class="btn primary" onclick={newDoc}>New luminaire</button>
          </div>
          {#if store.error}<p class="err">{store.error}</p>{/if}
        </div>
      {:else}
        <div class="sections">
          {#if active === 'general'}<SectionGeneral />{/if}
          {#if active === 'geometry'}<SectionGeometry />{/if}
          {#if active === 'lamps'}<SectionLamps />{/if}
          {#if active === 'intensity'}<SectionIntensity />{/if}
          {#if store.error}<div class="err card">{store.error}</div>{/if}
        </div>
      {/if}
    </main>

    {#if store.doc && !collapsed}
      <aside class="inspector">
        {#if rightView === 'validation'}
          <ValidationPanel onclose={() => (rightView = 'diagram')} />
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
        <p>Drop to open <code>.ldt</code> file</p>
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
  }
  .body.no-inspector {
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
