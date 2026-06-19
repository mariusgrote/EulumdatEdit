<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { open as openDialog, save as saveDialog, ask } from '@tauri-apps/plugin-dialog';

  interface Props {
    showValidation: boolean;
    toggleValidation: () => void;
  }
  let { showValidation, toggleValidation }: Props = $props();

  const filter = [{ name: 'EULUMDAT', extensions: ['ldt', 'LDT'] }];

  async function doNew() {
    if (!(await store.confirmDiscardChanges())) return;
    await store.newDoc();
  }

  async function doOpen() {
    if (!(await store.confirmDiscardChanges())) return;
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

  async function doDiscard() {
    const ok = await ask(
      'Discard all unsaved changes and reload this file from disk? This cannot be undone.',
      { title: 'Discard changes', kind: 'warning' }
    );
    if (ok) await store.revert();
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
    <button class="btn ghost" onclick={doNew}>New</button>
    <button class="btn ghost" onclick={doOpen}>Open</button>
    <button class="btn" onclick={doSave} disabled={!store.doc}>Save</button>
    <button class="btn ghost" onclick={doSaveAs} disabled={!store.doc}>Save As</button>
    {#if store.dirty && store.path}
      <button class="btn ghost" onclick={doDiscard} title="Discard changes and reload from disk">
        Discard
      </button>
    {/if}

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
</style>
