<script lang="ts">
  interface Props {
    label: string;
    value: string;
    placeholder?: string;
    onedit: () => void;
  }

  let { label, value = $bindable(), placeholder, onedit }: Props = $props();

  const uid = $props.id();
</script>

<div class="field">
  <label for={uid}>{label}</label>
  <input id={uid} type="text" bind:value {placeholder} oninput={onedit} />
</div>
