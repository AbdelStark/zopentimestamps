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
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--text-tertiary);
    letter-spacing: var(--tracking-wider);
    text-transform: uppercase;
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
    font-family: var(--font-sans);
    font-size: var(--text-base);
    font-weight: var(--font-normal);
    letter-spacing: var(--tracking-normal);
    height: 52px;
    transition:
      border-color var(--duration-fast) var(--ease-out),
      background var(--duration-fast) var(--ease-out),
      box-shadow var(--duration-fast) var(--ease-out);
  }

  .input::placeholder {
    color: var(--text-disabled);
  }

  .input:hover:not(:disabled):not(:focus) {
    border-color: var(--border-emphasis);
  }

  .input:focus {
    outline: none;
    border-color: var(--border-focus);
    background: var(--bg-secondary);
    box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.03);
  }

  .input:disabled {
    opacity: 0.4;
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
    box-shadow: 0 0 0 3px var(--error-muted);
  }

  .input-error {
    font-size: var(--text-xs);
    font-weight: var(--font-medium);
    color: var(--error);
    letter-spacing: var(--tracking-wide);
  }
</style>
