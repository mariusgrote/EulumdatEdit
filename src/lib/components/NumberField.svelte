<script lang="ts">
  import { store } from '$lib/store.svelte';
  import { committableNumber } from '$lib/numberInput';

  interface Props {
    label: string;
    value: number;
    step?: number;
    min?: number;
    max?: number;
    /** Hard limits for values that can cross the DTO boundary; `min`/`max` are only hints. */
    integer?: boolean;
    hardMin?: number;
    hardMax?: number;
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
    integer = false,
    hardMin,
    hardMax,
    unit,
    fieldKey,
    onedit
  }: Props = $props();

  const uid = $props.id();
  const warnings = $derived(fieldKey ? (store.fieldWarnings[fieldKey] ?? []) : []);
  const highlighted = $derived(!!fieldKey && store.highlightedFieldKey === fieldKey);

  function parse(input: HTMLInputElement): number | null {
    return committableNumber(input.valueAsNumber, { integer, hardMin, hardMax });
  }

  // Empty or partial drafts stay in the DOM; only committable numbers reach the document.
  function onInput(event: Event) {
    const next = parse(event.currentTarget as HTMLInputElement);
    if (next === null || next === value) return;
    value = next;
    onedit();
  }

  function onBlur(event: FocusEvent) {
    const input = event.currentTarget as HTMLInputElement;
    if (parse(input) === null) input.value = String(value);
  }
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
    {value}
    {step}
    {min}
    {max}
    oninput={onInput}
    onblur={onBlur}
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
