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
  <TopBar
    showValidation={rightView === 'validation'}
    toggleValidation={() =>
      (rightView = rightView === 'validation' ? 'diagram' : 'validation')}
  />

  <div class="body">
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
      <aside class="inspector">
        {#if rightView === 'validation'}
          <ValidationPanel onclose={() => (rightView = 'diagram')} />
        {:else}
          <DiagramPanel />
        {/if}
      </aside>
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
    grid-template-columns: 200px minmax(0, 1fr) clamp(380px, 34vw, 520px);
    min-height: 0;
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
</style>
