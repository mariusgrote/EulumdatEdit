<script lang="ts">
  import { store } from '$lib/store.svelte';

  interface Props {
    label: string;
    value: number;
    step?: number;
    min?: number;
    max?: number;
    unit?: string;
    fieldKey?: string;
    onedit: () => void;
  }

  let {
    label,
    value = $bindable(),
    step = 1,
    min,
    max,
    unit,
    fieldKey,
    onedit
  }: Props = $props();

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
  <label for={uid}>{label}{#if unit}<span class="unit"> ({unit})</span>{/if}</label>
  <input
    id={uid}
    type="number"
    bind:value
    {step}
    {min}
    {max}
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

<style>
  .unit {
    color: var(--text-faint);
    font-weight: 400;
  }
</style>
