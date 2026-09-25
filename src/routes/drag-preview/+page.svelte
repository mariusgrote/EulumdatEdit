<script lang="ts">
  const title = new URLSearchParams(window.location.search).get('title') ?? '';
</script>

<svelte:head><title>Tab preview</title></svelte:head>

<div class="preview" title={title}>{title}</div>

<style>
  :global(html), :global(body) {
    margin: 0;
    width: 100%;
    height: 100%;
    overflow: hidden;
  }
  .preview {
    box-sizing: border-box;
    width: 100vw;
    height: 100vh;
    overflow: hidden;
    padding: 8px 11px;
    border: 1px solid var(--border-strong);
    background: var(--bg-elev);
    color: var(--text);
    font: 13px/20px system-ui, sans-serif;
    white-space: nowrap;
    text-overflow: ellipsis;
  }
</style>
