<script lang="ts">
  import { store } from '$lib/store.svelte';

  interface Props {
    label: string;
    value: string;
    placeholder?: string;
    fieldKey?: string;
    onedit: () => void;
  }

  let { label, value = $bindable(), placeholder, fieldKey, onedit }: Props = $props();

  const uid = $props.id();
  const warnings = $derived(fieldKey ? (store.fieldWarnings[fieldKey] ?? []) : []);
  const highlighted = $derived(!!fieldKey && store.highlightedFieldKey === fieldKey);
</script>

<div
  class="field"
  data-field-key={fieldKey}
  class:has-warning={warnings.length > 0}
  class:warn-highlight={highlighted}
>
  <label for={uid}>{label}</label>
  <input
    id={uid}
    type="text"
    bind:value
    {placeholder}
    oninput={onedit}
    aria-invalid={warnings.length > 0}
    aria-describedby={warnings.length > 0 ? `${uid}-warn` : undefined}
  />
  {#if warnings.length > 0}
    <ul class="field-warning" id="{uid}-warn">
      {#each warnings as msg}<li>{msg}</li>{/each}
    </ul>
  {/if}
</div>
