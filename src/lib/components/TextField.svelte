<script lang="ts">
  interface Props {
    label: string;
    value: string;
    placeholder?: string;
    fieldKey?: string;
    onedit: () => void;
  }

  let { label, value = $bindable(), placeholder, fieldKey, onedit }: Props = $props();

  const uid = $props.id();
</script>

<div class="field" data-field-key={fieldKey}>
  <label for={uid}>{label}</label>
  <input id={uid} type="text" bind:value {placeholder} data-field-key={fieldKey} oninput={onedit} />
</div>
