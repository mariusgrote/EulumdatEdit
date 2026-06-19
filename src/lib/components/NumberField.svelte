<script lang="ts">
  interface Props {
    label: string;
    value: number;
    step?: number;
    min?: number;
    max?: number;
    unit?: string;
    onedit: () => void;
  }

  let {
    label,
    value = $bindable(),
    step = 1,
    min,
    max,
    unit,
    onedit
  }: Props = $props();

  const uid = $props.id();
</script>

<div class="field">
  <label for={uid}>{label}{#if unit}<span class="unit"> ({unit})</span>{/if}</label>
  <input
    id={uid}
    type="number"
    bind:value
    {step}
    {min}
    {max}
    oninput={onedit}
  />
</div>

<style>
  .unit {
    color: var(--text-faint);
    font-weight: 400;
  }
</style>
