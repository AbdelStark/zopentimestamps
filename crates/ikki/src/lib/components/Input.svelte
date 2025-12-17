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
    <label class="input-label">{label}</label>
  {/if}
  <div class="input-container">
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
  </div>
  {#if error}
    <span class="input-error">{error}</span>
  {/if}
</div>

<style>
  .input-wrapper {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
  }

  .input-label {
    font-size: var(--text-small);
    font-weight: var(--weight-medium);
    color: var(--text-secondary);
    letter-spacing: 0.01em;
  }

  .input-container {
    position: relative;
  }

  .input {
    width: 100%;
    padding: var(--space-3) var(--space-4);
    background: var(--bg-input);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    color: var(--text-primary);
    font-family: var(--font-family);
    font-size: var(--text-body);
    transition: all var(--transition-fast);
    height: 50px;
    letter-spacing: 0.01em;
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
    cursor: default;
  }

  .has-error .input {
    border-color: var(--error);
  }

  .has-error .input:focus {
    border-color: var(--error);
  }

  .input-error {
    font-size: var(--text-caption);
    color: var(--error);
    letter-spacing: 0.01em;
  }
</style>
