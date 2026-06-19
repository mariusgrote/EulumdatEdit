<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import TopBar from '$lib/components/TopBar.svelte';
  import DiagramPanel from '$lib/components/DiagramPanel.svelte';
  import ValidationPanel from '$lib/components/ValidationPanel.svelte';
  import SectionGeneral from '$lib/components/SectionGeneral.svelte';
  import SectionGeometry from '$lib/components/SectionGeometry.svelte';
  import SectionLamps from '$lib/components/SectionLamps.svelte';
  import SectionIntensity from '$lib/components/SectionIntensity.svelte';

  const sections = [
    { id: 'general', label: 'General', icon: '◷' },
    { id: 'geometry', label: 'Geometry', icon: '▦' },
    { id: 'lamps', label: 'Lamps', icon: '◍' },
    { id: 'intensity', label: 'Intensity', icon: '⊹' }
  ] as const;

  type SectionId = (typeof sections)[number]['id'];
  let active = $state<SectionId>('general');
  let warningsOpen = $state(false);

  async function openFile() {
    const path = await openDialog({
      multiple: false,
      filters: [{ name: 'EULUMDAT', extensions: ['ldt', 'LDT'] }]
    });
    if (typeof path === 'string') await store.open(path);
  }

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
      store.newDoc();
    }
  }
</script>

<svelte:window onkeydown={onKey} />

<div class="app">
  <TopBar {warningsOpen} toggleWarnings={() => (warningsOpen = !warningsOpen)} />

  <div class="body">
    <nav class="sidebar">
      {#each sections as s}
        <button
          class="navitem"
          class:active={active === s.id}
          onclick={() => (active = s.id)}
          disabled={!store.doc}
        >
          <span class="ico">{s.icon}</span>
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
            <button class="btn primary" onclick={() => store.newDoc()}>New luminaire</button>
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

    {#if store.doc}
      <DiagramPanel />
    {/if}

    {#if warningsOpen && store.doc}
      <ValidationPanel onclose={() => (warningsOpen = false)} />
    {/if}
  </div>
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
    grid-template-columns: 168px minmax(0, 1fr) 520px auto;
    min-height: 0;
  }
  .sidebar {
    background: var(--bg-elev);
    border-right: 1px solid var(--border);
    padding: 14px 10px;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .navitem {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
    border: none;
    background: transparent;
    color: var(--text-dim);
    padding: 9px 12px;
    border-radius: var(--radius-sm);
    font-weight: 500;
    text-align: left;
  }
  .navitem:hover:not(:disabled) {
    background: var(--bg-sunken);
    color: var(--text);
  }
  .navitem.active {
    background: var(--accent);
    color: var(--accent-contrast);
  }
  .navitem:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .ico {
    font-size: 16px;
    width: 18px;
    text-align: center;
  }
  .content {
    overflow-y: auto;
    padding: 22px;
    min-width: 0;
  }
  .sections {
    display: flex;
    flex-direction: column;
    gap: 18px;
    max-width: 760px;
    margin: 0 auto;
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
</style>
