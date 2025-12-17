<script lang="ts">
  export let type: "text" | "number" | "password" = "text";
  export let value: string = "";
  export let placeholder: string = "";
  export let label: string = "";
  export let error: string = "";
  export let disabled: boolean = false;
  export let readonly: boolean = false;
  export let inputmode: "text" | "decimal" | "numeric" = "text";
  export let oninput: (e: Event) => void = () => {};
</script>

<div class="input-wrapper" class:has-error={error}>
  {#if label}
    <span class="input-label">{label}</span>
  {/if}
  <input
    class="input"
    {type}
    {value}
    {placeholder}
    {disabled}
    {readonly}
    {inputmode}
    {oninput}
  />
  {#if error}
    <span class="input-error">{error}</span>
  {/if}
</div>

<style>
  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--space-sm);
  }

  .input-label {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
  }

  .input {
    width: 100%;
    padding: var(--space-md);
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-family: var(--font-family);
    font-size: var(--text-body);
    transition: all var(--transition-fast);
    height: 48px;
  }

  .input::placeholder {
    color: var(--text-tertiary);
  }

  .input:focus {
    outline: none;
    border-color: var(--border-focus);
    background: var(--bg-secondary);
  }

  .input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .input:read-only {
    background: var(--bg-card);
  }

  .has-error .input {
    border-color: var(--error);
  }

  .input-error {
    font-size: var(--text-small);
    color: var(--error);
  }
</style>
