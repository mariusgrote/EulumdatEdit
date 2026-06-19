<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';

  interface Props {
    warningsOpen: boolean;
    toggleWarnings: () => void;
  }
  let { warningsOpen, toggleWarnings }: Props = $props();

  const filter = [{ name: 'EULUMDAT', extensions: ['ldt', 'LDT'] }];

  async function doOpen() {
    const path = await openDialog({ multiple: false, filters: filter });
    if (typeof path === 'string') await store.open(path);
  }

  async function doSave() {
    if (store.path) {
      await store.save();
    } else {
      await doSaveAs();
    }
  }

  async function doSaveAs() {
    const path = await saveDialog({
      filters: filter,
      defaultPath: store.doc?.fileName || 'luminaire.ldt'
    });
    if (path) await store.saveAs(path);
  }

  const fileName = $derived(
    store.path ? store.path.split('/').pop() : store.doc ? 'Untitled' : '—'
  );
  const warnCount = $derived(store.warnings.length);
</script>

<header class="topbar">
  <div class="brand">
    <span class="logo">◐</span>
    <span class="name">EulumdatEdit</span>
  </div>

  <div class="file">
    <span class="filename">{fileName}</span>
    {#if store.dirty}<span class="dot" title="Unsaved changes">●</span>{/if}
  </div>

  <div class="actions">
    <button class="btn ghost" onclick={() => store.newDoc()}>New</button>
    <button class="btn ghost" onclick={doOpen}>Open</button>
    <button class="btn" onclick={doSave} disabled={!store.doc}>Save</button>
    <button class="btn ghost" onclick={doSaveAs} disabled={!store.doc}>Save As</button>

    <button
      class="btn ghost badge-btn"
      class:has={warnCount > 0}
      class:active={warningsOpen}
      onclick={toggleWarnings}
      disabled={!store.doc}
      title="Validation warnings"
    >
      ⚠ {warnCount}
    </button>
  </div>
</header>

<style>
  .topbar {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 0 16px;
    height: 52px;
    background: var(--bg-elev);
    border-bottom: 1px solid var(--border);
    -webkit-app-region: drag;
  }
  .topbar button,
  .topbar .file {
    -webkit-app-region: no-drag;
  }
  .brand {
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
  .file {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-dim);
    font-size: 13px;
  }
  .dot {
    color: var(--accent);
    font-size: 10px;
  }
  .actions {
    margin-left: auto;
    display: flex;
    gap: 6px;
  }
  .badge-btn.has {
    color: var(--warn);
    border-color: var(--warn);
  }
  .badge-btn.active {
    background: var(--bg-sunken);
  }
</style>
